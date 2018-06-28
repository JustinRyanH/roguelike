use tcod;
use rect;
use std;

use std::sync::{Arc, Mutex};

const COLOR_DARK_WALL: tcod::Color = tcod::Color{r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: tcod::Color = tcod::Color{r: 50, g: 50, b: 150};

#[derive(Clone, Copy)]
struct Tile {
    blocked: bool,
    block_sight: bool,
    color: tcod::Color
}

impl Tile {
    fn empty(color: tcod::Color) -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            color: color
        }
    }

    fn wall(color: tcod::Color) -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            color: color
        }
    }
}

struct Room {
    rect: rect::Rect,
    wall: Tile,
    ground: Tile,
}

impl Room {
    fn new(rect: rect::Rect) -> Room {
        Room{rect: rect, wall: Tile::wall(COLOR_DARK_WALL), ground: Tile::empty(COLOR_DARK_GROUND)}
    }
    
    fn render(&self, con: &mut tcod::Console) {
        for y in self.rect.y1..self.rect.y2 {
            for x in self.rect.x1..self.rect.x2 {
                let tile = if y == self.rect.y1 || y == self.rect.y2 - 1
                            || x == self.rect.x1 || x == self.rect.x2 - 1 {
                                self.wall
                            } else {
                                self.ground
                            };
                con.set_char_background(x, y, tile.color, tcod::BackgroundFlag::Set);
            }
        }
    }
}

pub struct Map<'a> {
    rooms: std::collections::HashMap<(i32, i32), Room>,
    bsp: Arc<Mutex<tcod::bsp::Bsp<'a>>>
}

impl<'a> Map<'a> {
    pub fn new(width: i32, height: i32) -> Map<'a> {
        Map {
            rooms: std::collections::HashMap::new(),
            bsp: Arc::new(Mutex::new(tcod::bsp::Bsp::new_with_size(0, 0, width, height)))
        }
    }

    pub fn generate_map(&mut self, rng: &mut tcod::random::Rng) -> (i32, i32) {
        let mut bsp = self.bsp.lock().unwrap();
        bsp.split_recursive(Some(rng), 5, 15, 15, 0.7, 0.6);
        let mut rooms = &mut self.rooms;
        let mut player = (0, 0);
        bsp.traverse(tcod::bsp::TraverseOrder::PreOrder, |node| {
            if !node.is_leaf() {
                return true;
            }
            let x1 = rng.get_int(node.x, node.x + node.w / 2);
            let y1 = rng.get_int(node.y, node.y + node.h / 2);
            let mut x2 = rng.get_int(std::cmp::max(node.x, x1), node.x + node.w);
            while x2 - x1 <= 2 {
                x2 = rng.get_int(std::cmp::max(node.x, x1), node.x + node.w);
            }
            let mut y2 = rng.get_int(std::cmp::max(node.y, y1), node.y + node.h);
            while y2 - y1 <= 2 {
                y2 = rng.get_int(std::cmp::max(node.y, y1), node.y + node.h);
            }
            rooms.insert((node.x, node.y), Room::new(rect::Rect::new(x1, y1, x2 - x1, y2 - y1)));
            if player == (0, 0) {
                player = (x1 + 1, y1 + 1);
            }
            true
        });
        player
    }

    pub fn can_walk(&self, x: i32, y: i32) -> bool {
        true
    }

    pub fn render(&self, con: &mut tcod::Console) {
        for room in self.rooms.values() {
            room.render(con);
        }
    }
}