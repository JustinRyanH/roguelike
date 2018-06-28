use tcod;
use rect;
use std;

const COLOR_DARK_WALL: tcod::Color = tcod::Color{r: 0, g: 0, b: 100};
const COLOR_LIGHT_WALL: tcod::Color = tcod::Color { r: 130, g: 110, b: 50 };
const COLOR_DARK_GROUND: tcod::Color = tcod::Color{r: 50, g: 50, b: 150};
const COLOR_LIGHT_GROUND: tcod::Color = tcod::Color { r: 200, g: 180, b: 50 };


const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const FOV_ALGO: tcod::map::FovAlgorithm = tcod::map::FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

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
    height: i32,
    fov: std::sync::Arc<std::sync::Mutex<tcod::map::Map>>
}

impl Map {
    pub fn new(width: i32, height: i32) -> Map {
        Map {
            map: vec![Tile::wall(); (width * height) as usize],
            width: width,
            height: height,
            fov: std::sync::Arc::new(std::sync::Mutex::new(tcod::map::Map::new(width, height)))
        }
    }

    fn get(&self, x: i32, y: i32) -> &Tile {
        &self.map[(x + y * self.width) as usize]
    }

    fn get_mut(&mut self, x: i32, y: i32) -> &mut Tile {
        &mut self.map[(x + y * self.width) as usize]
    }

    fn create_room(&mut self, room: rect::Rect) {
        for x in (room.x1 + 1)..room.x2 {
            for y in (room.y1 + 1)..room.y2 {
                std::mem::swap(self.get_mut(x, y), &mut Tile::empty());
            }
        }
    }

    fn create_h_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in std::cmp::min(x1, x2)..=std::cmp::max(x1, x2) {
            std::mem::swap(self.get_mut(x, y), &mut Tile::empty());
        }
    }

    fn create_v_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in std::cmp::min(y1, y2)..=std::cmp::max(y1, y2) {
            std::mem::swap(self.get_mut(x, y), &mut Tile::empty());
        }
    }

    pub fn recompute_fov(&mut self, x: i32, y: i32) {
        self.fov.lock().unwrap().compute_fov(x, y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    pub fn generate_map(&mut self, rng: &mut tcod::random::Rng) -> (i32, i32) {
        let mut rooms = vec![];

        let mut start = (0, 0);

        for _ in 0..MAX_ROOMS {
            // random width and height
            let w = rng.get_int(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
            let h = rng.get_int(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
            // random position without going out of the boundaries of the map
            let x = rng.get_int(0, self.width - w);
            let y = rng.get_int(0, self.height - h);

            let new_room = rect::Rect::new(x, y, w, h);
            // run through the other rooms and see if they intersect with this one
            let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));
            if !failed {
                // "paint" it to the map's tiles
                self.create_room(new_room);
                let (new_x, new_y) = new_room.center();
                if rooms.is_empty() {
                    start = (new_x, new_y);
                } else {
                    // center coordinates of the previous room
                    let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                    // draw a coin (random bool value -- either true or false)
                    if rng.get_int(0, 2) == 1 {
                        // first move horizontally, then vertically
                        self.create_h_tunnel(prev_x, new_x, prev_y);
                        self.create_v_tunnel(prev_y, new_y, new_x);
                    } else {
                        // first move vertically, then horizontally
                        self.create_v_tunnel(prev_y, new_y, prev_x);
                        self.create_h_tunnel(prev_x, new_x, new_y);
                    }
                }
                rooms.push(new_room);
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                self.fov.lock().unwrap().set(x, y, !self.get(x, y).block_sight, !self.get(x, y).blocked);
            }
        }
        self.recompute_fov(start.0, start.1);
        start
    }

    pub fn can_walk(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            false
        } else {
            !self.map[(x + y * self.width) as usize].blocked
        }
    }

    pub fn render(&self, con: &mut tcod::Console) {
        for y in 0..self.height {
            for x in 0..self.width {
                let visible = self.fov.lock().unwrap().is_in_fov(x, y);
                let wall = self.get(x, y).block_sight;
                let color = match (visible, wall) {
                    (false, false) => COLOR_DARK_GROUND,
                    (false, true) => COLOR_DARK_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                    (true, true) => COLOR_LIGHT_WALL
                };
                con.set_char_background(x, y, color, tcod::BackgroundFlag::Set);
            }
        }
    }
}