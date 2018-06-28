use tcod;
use rect;
use std;

use std::sync::{Arc, Mutex};

const COLOR_DARK_WALL: tcod::Color = tcod::Color{r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: tcod::Color = tcod::Color{r: 50, g: 50, b: 150};

#[derive(Clone, Copy)]
struct Tile {
    blocked: bool,
    block_sight: bool
}

impl Tile {
    fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false
        }
    }

    fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true
        }
    }
}

pub struct Map {
    map: Vec<Tile>,
    width: i32,
    height: i32
}

impl Map {
    pub fn new(width: i32, height: i32) -> Map {
        Map {
            map: vec![Tile::empty(); (width * height) as usize],
            width: width,
            height: height
        }
    }

    pub fn can_walk(&self, x: i32, y: i32) -> bool {
        !self.map[(x + y * self.width) as usize].blocked
    }

    pub fn render(&self, con: &mut tcod::Console) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.map[(x + y * self.width) as usize].blocked {
                    con.set_char_background(x, y, COLOR_DARK_WALL, tcod::BackgroundFlag::Set);
                } else {
                    con.set_char_background(x, y, COLOR_DARK_GROUND, tcod::BackgroundFlag::Set);
                }
            }
        }
    }
}