use components::*;
use specs::{Entity, World, Dispatcher};
use state_machine::{State, Transition, Event};
use tcod;
use tcod::Console;

use map;

use observer;
use observer::*;

use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn
}

struct Log;

impl Listener for Log {
    fn notify(&mut self, event: &observer::Event) {
        println!("{:?}", event);
    }
}

use std;
unsafe impl std::marker::Send for Log {}

pub struct Game<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    player: Entity,
    action: PlayerAction
}

impl<'a, 'b> Game<'a, 'b> {
    pub fn new(seed: u32, screen_width: i32, screen_height: i32) -> Game<'a, 'b> {
        let mut con = tcod::console::Offscreen::new(screen_width, screen_height);
        con.set_default_foreground(tcod::colors::WHITE);

        let mut map = map::Map::new(screen_width, screen_height - 15);

        let mut rng = tcod::random::Rng::new_with_seed(tcod::random::Algo::CMWC, seed);

        let (x, y) = map.generate_map(&mut rng);

        let (mut world, dispatcher) = create_world(con, map, rng);

        let player = create_player(&mut world, x, y);

        let logger = Arc::new(Mutex::new(Log));
        world.write_resource::<observer::Dispatcher>().register_listener(logger);
        world.add_resource(Player(player.clone()));

        Game {
            world: world,
            dispatcher: dispatcher,
            player: player,
            action: PlayerAction::TookTurn
        }
    }

    fn handle_key(&mut self, key: tcod::input::Key) -> Transition {
        use tcod::input::Key;
        use tcod::input::KeyCode;
        let mut event_storage = self.world.write_storage::<MoveEvent>();
        let (dx, dy) = match key {
            Key { code: KeyCode::Up, .. } => (0, -1),
            Key { code: KeyCode::Down, .. } => (0, 1),
            Key { code: KeyCode::Left, .. } => (-1, 0),
            Key { code: KeyCode::Right, .. } => (1, 0),
            Key { code: KeyCode::Escape, .. } => return Transition::Pop,
            _ => (0, 0),
        };
        self.action = if dx == 0 && dy == 0 {
            PlayerAction::DidntTakeTurn
        } else {
            event_storage.insert(self.player, MoveEvent(dx, dy)).unwrap();
            PlayerAction::TookTurn

        };
        Transition::None
    }
}

impl<'a, 'b> State for Game<'a, 'b> {
    fn render(&self, screen: &mut tcod::Console) {
        let console = self.world.read_resource::<DisplayConsole>();
        console.get().print(0, 35, format!("turns: {}", self.world.read_resource::<Turns>().0));
        tcod::console::blit(&*console.get(), (0, 0), (0, 0), screen, (0, 0), 1.0, 1.0);
    }

    fn update(&mut self) -> Transition {
        if self.action == PlayerAction::DidntTakeTurn {
            return Transition::None
        }
        self.world.write_resource::<Turns>().0 += 1;
        self.dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
        Transition::None
    }

    fn handle_event(&mut self, event: Event) -> Transition {
        match event {
            Event::Key(key) => self.handle_key(key)
        }
    }
}