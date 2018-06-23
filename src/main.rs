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
use specs::{WriteExpect, ReadStorage, System};
use specs::DispatcherBuilder;

#[derive(Component)]
#[storage(VecStorage)]
struct Position {
    x: i32,
    y: i32
}

#[derive(Component)]
#[storage(VecStorage)]
struct Displayable(char);

struct Print;
impl<'a> System<'a> for Print {
    type SystemData = (WriteExpect<'a, Console>, ReadStorage<'a, Position>, ReadStorage<'a, Displayable>);

    fn run(&mut self, (mut console, position, displayable): Self::SystemData) {
        use specs::Join;

        for (position, displayable) in (&position, &displayable).join() {
            console.0.put_char(position.x, position.y, displayable.0, BackgroundFlag::None);
        }
    }
}

struct Console(Offscreen);

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

fn handle_keys(root: &mut Root) -> bool {
    match root.wait_for_keypress(true) {
        Key { code: Escape, .. } => return true,
        _ => {}
    }
    false
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
    let player = world.create_entity().with(Position{x: SCREEN_WIDTH / 2, y: SCREEN_HEIGHT / 2})
        .with(Displayable('@')).build();

    let mut con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    con.set_default_foreground(colors::WHITE);

    world.add_resource(Console(con));
    let mut dispatcher = DispatcherBuilder::new().with_thread_local(Print).build();

    tcod::system::set_fps(LIMIT_FPS);
    while !root.window_closed() {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
        let console = world.read_resource::<Console>();
        blit(&console.0, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut root, (0, 0), 1.0, 1.0);
        root.flush();
        let exit = handle_keys(&mut root);
        if exit {
            break;
        }
    }
}
