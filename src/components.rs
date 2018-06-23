use specs;
use tcod;
use std;

use specs::{Component, VecStorage};
use specs::World;
use specs::{WriteStorage, WriteExpect, ReadStorage, System};
use specs::{Dispatcher, DispatcherBuilder};

use std::sync::{Arc, Mutex};

#[derive(Component)]
#[storage(VecStorage)]
struct Position {
    x: i32,
    y: i32,
    old_x: i32,
    old_y: i32,
    z: i8
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
struct Displayable {
    char: char,
    color: tcod::colors::Color,
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

struct Print;
impl<'a> System<'a> for Print {
    type SystemData = (WriteExpect<'a, DisplayConsole>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, Displayable>);

    fn run(&mut self, (mut console, position, displayable): Self::SystemData) {
        use specs::Join;
        use tcod::Console;

        let mut con = console.get_mut();

        let mut data = (&position, &displayable).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| a.0.z.cmp(&b.0.z));

        for (position, _) in data.iter() {
            (*con).put_char(position.old_x, position.old_y, ' ', tcod::BackgroundFlag::None);
        }

        for (position, displayable) in data {
            (*con).set_default_foreground(displayable.color);
            (*con).put_char(position.x, position.y, displayable.char, tcod::BackgroundFlag::None);
        }
    }
}

struct HandleMoveEvents;
impl<'a> System<'a> for HandleMoveEvents {
    type SystemData = (specs::Entities<'a>, WriteStorage<'a, Position>, WriteStorage<'a, MoveEvent>);

    fn run(&mut self, (entities, mut pos, mut event_storage): Self::SystemData) {
        use specs::Join;

        let mut to_remove = Vec::new();

        for (ent, pos, event) in (&*entities, &mut pos, &mut event_storage).join() {
            pos.old_x = pos.x;
            pos.old_y = pos.y;
            pos.x += event.0;
            pos.y += event.1;
            to_remove.push(ent);
        }
        for e in to_remove {
            event_storage.remove(e);
        }
    }
}

pub struct Turns(pub i64);

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
        .build()
}

pub fn create_npc(world: &mut World, x: i32, y: i32) {
    world.create_entity()
        .with(Position::new(x, y, 0))
        .with(Displayable::new('o', tcod::colors::YELLOW))
        .build();
}

pub fn create_world<'a, 'b>(con: tcod::console::Offscreen) -> (World, Dispatcher<'a, 'b>) {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Displayable>();
    world.register::<MoveEvent>();
    world.add_resource(DisplayConsole(Arc::new(Mutex::new(con))));
    world.add_resource(Turns(0));
    let mut dispatcher = DispatcherBuilder::new()
        .with(HandleMoveEvents, "move_event", &[])
        .with_thread_local(Print).build();
    dispatcher.setup(&mut world.res);
    (world, dispatcher)
}