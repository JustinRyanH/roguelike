use components::*;
use specs::{Entity, World, Dispatcher};
use state_machine::{State, Transition, Event};
use tcod;
use tcod::Console;

use map;
use rand;

pub struct Game<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    player: Entity
}

impl<'a, 'b> Game<'a, 'b> {
    pub fn new(seed: u32, screen_width: i32, screen_height: i32) -> Game<'a, 'b> {
        let mut con = tcod::console::Offscreen::new(screen_width, screen_height);
        con.set_default_foreground(tcod::colors::WHITE);

        let map = map::Map::new(screen_width, screen_height - 15);

        let (mut world, dispatcher) = create_world(con, map, tcod::random::Rng::new_with_seed(tcod::random::Algo::CMWC, seed));

        let player = create_player(&mut world, 25, 23);
        Game {
            world: world,
            dispatcher: dispatcher,
            player: player
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
            Key { code: KeyCode::Escape, .. } => return Transition::Quit,
            _ => (0, 0),
        };
        event_storage.insert(self.player, MoveEvent(dx, dy)).unwrap();
        Transition::None
    }
}

impl<'a, 'b> State for Game<'a, 'b> {
    fn render(&self, screen: &mut tcod::Console) {
        let console = self.world.read_resource::<DisplayConsole>();
        console.get().print(0, 35, format!("turns: {}", self.world.read_resource::<Turns>().0));
        tcod::console::blit(&*console.get(), (0, 0), (0, 0), screen, (0, 0), 1.0, 1.0);
    }

    fn on_start(&mut self) {
        let storage = self.world.write_resource::<Rng>();
        let mut rng = storage.0.lock().unwrap();
        let player = self.world.write_resource::<map::Map>().generate_map(&mut rng);
    }

    fn update(&mut self) -> Transition {
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