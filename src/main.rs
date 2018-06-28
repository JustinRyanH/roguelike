extern crate tcod;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate rand;

use tcod::console::{FontType, FontLayout, Root};

mod state_machine;
mod components;
mod game;
mod map;
mod rect;
use state_machine::Event;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Nameless")
        .init();
    
    let mut machine = state_machine::StateMachine::new(Box::new(game::Game::new(42, SCREEN_WIDTH, SCREEN_HEIGHT)));

    machine.start();

    while !root.window_closed() && machine.is_running() {
        machine.update();
        machine.render(&mut root);
        root.flush();
        machine.handle_event(Event::Key(root.wait_for_keypress(true)));
    }
}
