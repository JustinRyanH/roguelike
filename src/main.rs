extern crate tcod;
extern crate specs;
#[macro_use]
extern crate specs_derive;

use tcod::console::*;
use tcod::colors;
use tcod::input::Key;
use tcod::input::KeyCode::*;

use specs::{Component, VecStorage};
use specs::World;
use specs::{WriteStorage, WriteExpect, ReadStorage, System};
use specs::DispatcherBuilder;

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
    color: colors::Color,
}

impl Displayable {
    fn new(c: char, color: colors::Color) -> Displayable {
        Displayable {
            char: c,
            color: color
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
struct MoveEvent(i32, i32);

struct Print;
impl<'a> System<'a> for Print {
    type SystemData = (WriteExpect<'a, DisplayConsole>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, Displayable>);

    fn run(&mut self, (mut console, position, displayable): Self::SystemData) {
        use specs::Join;

        let mut con = console.get_mut();

        let mut data = (&position, &displayable).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| a.0.z.cmp(&b.0.z));

        for (position, displayable) in data.iter() {
            (*con).put_char(position.old_x, position.old_y, ' ', BackgroundFlag::None);
        }

        for (position, displayable) in data {
            (*con).set_default_foreground(displayable.color);
            (*con).put_char(position.x, position.y, displayable.char, BackgroundFlag::None);
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

struct DisplayConsole(Arc<Mutex<Offscreen>>);
impl DisplayConsole {
    fn get<'ret, 'me:'ret>(&'me self) -> std::sync::MutexGuard<'ret, Offscreen> {
        self.0.lock().unwrap()
    }

    fn get_mut<'ret, 'me:'ret>(&'me mut self) -> std::sync::MutexGuard<'ret, Offscreen> {
        self.0.lock().unwrap()
    }
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

fn handle_keys(root: &mut Root, world: &mut World, player: specs::Entity) -> bool {
    let mut event_storage = world.write_storage::<MoveEvent>();
    match root.wait_for_keypress(true) {
        Key { code: Up, .. } => event_storage.insert(player, MoveEvent(0, -1)).unwrap(),
        Key { code: Down, .. } => event_storage.insert(player, MoveEvent(0, 1)).unwrap(),
        Key { code: Left, .. } => event_storage.insert(player, MoveEvent(-1, 0)).unwrap(),
        Key { code: Right, .. } => event_storage.insert(player, MoveEvent(1, 0)).unwrap(),
        Key { code: Escape, .. } => return true,
        _ => return false,
    };
    false
}

fn create_player(world: &mut World) -> specs::Entity {
    let x = SCREEN_WIDTH / 2;
    let y = SCREEN_HEIGHT / 2;
    world.create_entity()
        .with(Position::new(x, y, 1))
        .with(Displayable::new('@', colors::WHITE))
        .build()
}

fn create_npc(world: &mut World) {
    let x = SCREEN_WIDTH / 2 - 5;
    let y = SCREEN_HEIGHT / 2;
    world.create_entity()
        .with(Position::new(x, y, 0))
        .with(Displayable::new('o', colors::YELLOW))
        .build();
}

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Displayable>();
    world.register::<MoveEvent>();

    let player = create_player(&mut world);
    create_npc(&mut world);

    let mut con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    con.set_default_foreground(colors::WHITE);

    world.add_resource(DisplayConsole(Arc::new(Mutex::new(con))));
    let mut dispatcher = DispatcherBuilder::new()
        .with(HandleMoveEvents, "move_event", &[])
        .with_thread_local(Print).build();
    dispatcher.setup(&mut world.res);

    tcod::system::set_fps(LIMIT_FPS);
    while !root.window_closed() {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
        {
            let console = world.read_resource::<DisplayConsole>();
            blit(&*console.get(), (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut root, (0, 0), 1.0, 1.0);
        }
        root.flush();
        let exit = handle_keys(&mut root, &mut world, player);
        if exit {
            break;
        }
    }
}
