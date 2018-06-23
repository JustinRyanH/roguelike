use tcod;
use rect;
use std;

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

pub struct Map {
    rooms: Vec<Room>,
    width: i32,
    height: i32
}

impl Map {
    pub fn new(width: i32, height: i32) -> Map {
        Map {
            rooms: Vec::new(),
            width: width,
            height: height
        }
    }

    pub fn can_walk(&self, x: i32, y: i32) -> bool {
        true
    }

    pub fn render(&self, con: &mut tcod::Console) {
        for room in &self.rooms {
            room.render(con);
        }
    }
}