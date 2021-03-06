use specs;
use tcod;
use std;
use map;

use specs::VecStorage;
use specs::World;
use specs::{WriteStorage, WriteExpect, ReadStorage, ReadExpect, System};
use specs::{Dispatcher, DispatcherBuilder};

use std::sync::{Arc, Mutex};

use observer;

#[derive(Component)]
#[storage(VecStorage)]
struct Position {
    x: i32,
    y: i32,
    old_x: i32,
    old_y: i32,
    z: i8
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Properties {
    name: String,
    pub blocks: bool,
    pub alive: bool,
    max_hp: i32,
    hp: i32,
    death_callback: Option<Box<Mutex<Fn(&mut Properties, &mut Displayable) + std::marker::Send + 'static>>>
}

impl Properties {
    fn new<FN>(name: &str, blocks: bool, alive: bool, max_hp: i32, hp: i32, f: FN) -> Self 
        where FN: Fn(&mut Properties, &mut Displayable) + std::marker::Send + 'static
    {
        Properties {
            name: name.into(),
            blocks: blocks,
            alive: alive,
            max_hp: max_hp,
            hp: hp,
            death_callback: Some(Box::new(Mutex::new(f)))
        }
    }
}

impl Position {
    fn new(x: i32, y: i32, z: i8) -> Position {
        Position{
            x: x,
            y: y,
            old_x: x,
            old_y: y,
            z: z
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Displayable {
    pub char: char,
    pub color: tcod::colors::Color,
}

impl Displayable {
    fn new(c: char, color: tcod::colors::Color) -> Displayable {
        Displayable {
            char: c,
            color: color
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct MoveEvent(pub i32, pub i32);

#[derive(Component)]
#[storage(VecStorage)]
struct MeleeEvent(specs::Entity);

#[derive(Component)]
#[storage(VecStorage)]
struct HpChange(i32);

#[derive(Component)]
#[storage(VecStorage)]
pub struct Fighter {
    defense: i32,
    attack: i32
}

impl Fighter {
    pub fn new(attack: i32, defense: i32) -> Self {
        Fighter {
            attack: attack,
            defense: defense
        }
    }
}

struct Print;
impl<'a> System<'a> for Print {
    type SystemData = (WriteExpect<'a, DisplayConsole>,
                       ReadExpect<'a, map::Map>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, Displayable>);

    fn run(&mut self, (mut console, map, position, displayable): Self::SystemData) {
        use specs::Join;
        use tcod::Console;

        let mut con = console.get_mut();

        let mut data = (&position, &displayable).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| a.0.z.cmp(&b.0.z));

        for (position, _) in data.iter() {
            (*con).put_char(position.old_x, position.old_y, ' ', tcod::BackgroundFlag::None);
        }

        map.render(&mut *con);

        for (position, displayable) in data {
            if map.is_in_fov(position.x, position.y) {
                (*con).set_default_foreground(displayable.color);
                (*con).put_char(position.x, position.y, displayable.char, tcod::BackgroundFlag::None);
            }
        }
    }
}

struct HandleHpChange;
impl <'a> System<'a> for HandleHpChange {
    type SystemData = (specs::Entities<'a>, WriteStorage<'a, HpChange>, WriteStorage<'a, Properties>, WriteExpect<'a, observer::Dispatcher<'static>>, WriteStorage<'a, Displayable>);

    fn run(&mut self, (entities, mut hp_change, mut properties, mut dispatcher, mut displayable): Self::SystemData) {
        use specs::Join;

        let mut to_remove = Vec::new();

        for (ent, hp, prop, disp) in (&*entities, &mut hp_change, &mut properties, &mut displayable).join() {
            prop.hp += hp.0;
                if prop.hp <= 0 {
                    prop.hp = 0;
                    prop.alive = false;
                    dispatcher.dispatch(observer::Event::Log(ent, format!("{} died", prop.name)));
                    if let Some(callback) = prop.death_callback.take() {
                        (callback.lock().unwrap())(prop, disp);
                    }
                }
            to_remove.push(ent);
        }

        for e in to_remove {
            hp_change.remove(e);
        }
    }
}

struct HandleMelee;
impl<'a> System<'a> for HandleMelee {
    type SystemData = (specs::Entities<'a>, WriteExpect<'a, observer::Dispatcher<'static>>, WriteStorage<'a, MeleeEvent>, ReadStorage<'a, Properties>, ReadStorage<'a, Fighter>, WriteStorage<'a, HpChange>);

    fn run(&mut self, (entities, mut dispatcher, mut melee_storage, properties, fighter_storage, mut hp_change): Self::SystemData) {
        use specs::Join;

        let mut to_remove = Vec::new();

        for (ent, melee, prop) in (&*entities, &mut melee_storage, &properties).join() {
            if let Some(ref atk) = fighter_storage.get(melee.0) {
                // if the one attacking cannot attack, we don't attack
                let def = if let Some(ref def) = fighter_storage.get(ent) {
                    def.defense
                } else {
                    0 // if it's not a fighter, then it doesn't have any defense!
                };
                let p = properties.get(melee.0).unwrap();
                let mut damage = atk.attack - def;
                if damage > 0 {
                    dispatcher.dispatch(observer::Event::Log(ent, format!("{} attacked the {} for {} damage", p.name, prop.name, damage)));
                    if let Some(change) = hp_change.get(ent) {
                        damage -= change.0;
                    }
                    hp_change.insert(ent, HpChange(-damage)).unwrap();
                } else {
                    dispatcher.dispatch(observer::Event::Log(ent, format!("{} attacked the {} but there was no effect!", p.name, prop.name)));
                }
            }
            to_remove.push(ent);
        }
        for e in to_remove {
            melee_storage.remove(e);
        }
    }
}

struct HandleMoveEvents;
impl<'a> System<'a> for HandleMoveEvents {
    type SystemData = (specs::Entities<'a>, WriteExpect<'a, map::Map>, WriteStorage<'a, Position>, WriteStorage<'a, MoveEvent>, ReadExpect<'a, Player>, ReadStorage<'a, Properties>, WriteStorage<'a, MeleeEvent>);

    fn run(&mut self, (entities, mut map, mut pos, mut event_storage, player, properties, mut melee_storage): Self::SystemData) {
        use specs::Join;

        let mut to_remove = Vec::new();

        let positions: Vec<(specs::Entity, i32,i32)> = (&*entities, &pos, &properties).join().filter_map(|e| {
            if e.2.blocks && e.0 != player.0 {
                Some((e.0, e.1.x, e.1.y))
            } else {
                None
            }
        }).collect();

        for (ent, pos, event) in (&*entities, &mut pos, &mut event_storage).join() {
            let other = positions.iter().find(|(_, x, y)| { (*x, *y) == (pos.x + event.0, pos.y + event.1) });
            if map.can_walk(pos.x + event.0, pos.y + event.1) && other == None {
                pos.old_x = pos.x;
                pos.old_y = pos.y;
                pos.x += event.0;
                pos.y += event.1;
                if ent == player.0 {
                    map.recompute_fov(pos.x, pos.y);
                }
            } else if let Some(other) = other {
                // this is considered as a melee attack
                melee_storage.insert(other.0, MeleeEvent(ent)).unwrap();
            }
            to_remove.push(ent);
        }
        for e in to_remove {
            event_storage.remove(e);
        }
    }
}

pub struct Turns(pub i64);

pub struct Rng(pub Arc<Mutex<tcod::random::Rng>>);

pub struct Player(pub specs::Entity);

pub struct DisplayConsole(Arc<Mutex<tcod::console::Offscreen>>);
impl DisplayConsole {
    pub fn get<'ret, 'me:'ret>(&'me self) -> std::sync::MutexGuard<'ret, tcod::console::Offscreen> {
        self.0.lock().unwrap()
    }

    pub fn get_mut<'ret, 'me:'ret>(&'me mut self) -> std::sync::MutexGuard<'ret, tcod::console::Offscreen> {
        self.0.lock().unwrap()
    }
}

pub fn create_player(world: &mut World, x: i32, y: i32) -> specs::Entity {
    world.create_entity()
        .with(Position::new(x, y, 1))
        .with(Displayable::new('@', tcod::colors::WHITE))
        .with(Properties::new("You", true, true, 30, 30, |props, display| {
            props.alive = false;
            display.char = '%';
            display.color = tcod::colors::DARK_RED;
        }))
        .with(Fighter::new(5, 2))
        .build()
}

pub fn create_npc<FN>(world: &mut World, x: i32, y: i32, c: char, name: &str, max_hp: i32, hp: i32, fighter: Option<Fighter>, f: FN, color: tcod::colors::Color)
    where FN: Fn(&mut Properties, &mut Displayable) + std::marker::Send + 'static
    {
    let e = world.create_entity()
        .with(Position::new(x, y, 0))
        .with(Displayable::new(c, color))
        .with(Properties::new(name, true, true, max_hp, hp, f))
        .build();
    if let Some(f) = fighter {
        world.write_storage::<Fighter>().insert(e, f).unwrap();
    }
}

pub fn create_world<'a, 'b>(con: tcod::console::Offscreen) -> (World, Dispatcher<'a, 'b>) {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Displayable>();
    world.register::<MoveEvent>();
    world.add_resource(DisplayConsole(Arc::new(Mutex::new(con))));
    world.add_resource(Turns(0));
    world.add_resource(observer::Dispatcher::new());
    let mut dispatcher = DispatcherBuilder::new()
        .with(HandleMoveEvents, "move_event", &[])
        .with(HandleMelee, "melee_event", &["move_event"])
        .with(HandleHpChange, "hp_change", &["melee_event"])
        .with_thread_local(Print).build();
    dispatcher.setup(&mut world.res);
    (world, dispatcher)
}