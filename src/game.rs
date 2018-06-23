use components::*;
use specs::{Entity, World, Dispatcher};
use state_machine::{State, Transition, Event};
use tcod;
use tcod::Console;

pub struct Game<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    player: Entity,
}

impl<'a, 'b> Game<'a, 'b> {
    pub fn new(screen_width: i32, screen_height: i32) -> Game<'a, 'b> {
        let mut con = tcod::console::Offscreen::new(screen_width, screen_height);
        con.set_default_foreground(tcod::colors::WHITE);

        let (mut world, dispatcher) = create_world(con);

        let player = create_player(&mut world, 5, 5);
        create_npc(&mut world, 4, 4);
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
        match key {
            Key { code: KeyCode::Up, .. } => event_storage.insert(self.player, MoveEvent(0, -1)).unwrap(),
            Key { code: KeyCode::Down, .. } => event_storage.insert(self.player, MoveEvent(0, 1)).unwrap(),
            Key { code: KeyCode::Left, .. } => event_storage.insert(self.player, MoveEvent(-1, 0)).unwrap(),
            Key { code: KeyCode::Right, .. } => event_storage.insert(self.player, MoveEvent(1, 0)).unwrap(),
            Key { code: KeyCode::Escape, .. } => return Transition::Quit,
            _ => return Transition::None,
        };
        Transition::None
    }
}

impl<'a, 'b> State for Game<'a, 'b> {
    fn render(&self, screen: &mut tcod::Console) {
        let console = self.world.read_resource::<DisplayConsole>();
        console.get().print(0, 0, format!("turns: {}", self.world.read_resource::<Turns>().0));
        tcod::console::blit(&*console.get(), (0, 0), (0, 0), screen, (0, 0), 1.0, 1.0);
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