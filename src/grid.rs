use std::time::Duration;
use rand::{self, rngs::ThreadRng, Rng, thread_rng, seq::SliceRandom};

pub const WIDTH: u8 = 10;
pub const HEIGHT: u8 = 20;

pub const INIT_INTERVAL: Duration = Duration::from_nanos(1_000_000_000);

#[derive(Clone)]
pub struct Tetromino {
    rotations: [[i16; 4]; 4],
}

impl Tetromino {
    pub fn get_cells(&self, position: u8, rotation: u8) -> [i16; 4] {
        let mut cells: [i16; 4] = [0; 4];
        for i in 0..4 {
            cells[i] = self.rotations[rotation as usize][i] + position as i16;
        }

        return cells;
    }
}

const TETROMINO_VARIANT: usize = 7;
const TETROMINOES: [Tetromino; TETROMINO_VARIANT] = [
    //           []
    // []()[][]  ()
    //           []
    //           []
    Tetromino {
        rotations: [
            [-1, 0, 1, 2],
            [0 - WIDTH as i16, 0, WIDTH as i16, (WIDTH * 2) as i16],
            [-1, 0, 1, 2],
            [0 - WIDTH as i16, 0, WIDTH as i16, (WIDTH * 2) as i16],
        ]
    },

    // ()[]
    // [][]
    Tetromino {
        rotations: [
            [0, 1, WIDTH as i16, (WIDTH + 1) as i16],
            [0, 1, WIDTH as i16, (WIDTH + 1) as i16],
            [0, 1, WIDTH as i16, (WIDTH + 1) as i16],
            [0, 1, WIDTH as i16, (WIDTH + 1) as i16],
        ]
    },

    // []()      []
    //   [][]  []()
    //         []
    Tetromino {
        rotations: [
            [-1, 0, WIDTH as i16, WIDTH as i16 + 1],
            [0 - WIDTH as i16, -1, 0, WIDTH as i16 - 1],
            [-1, 0, WIDTH as i16, WIDTH as i16 + 1],
            [0 - WIDTH as i16, -1, 0, WIDTH as i16 - 1],
        ]
    },

    //   ()[]  []
    // [][]    ()[]
    //           []
    Tetromino {
        rotations: [
            [0, 1, WIDTH as i16 - 1, WIDTH as i16],
            [0 - WIDTH as i16, 0, 1, WIDTH as i16 + 1],
            [0, 1, WIDTH as i16 - 1, WIDTH as i16],
            [0 - WIDTH as i16, 0, 1, WIDTH as i16 + 1],
        ]
    },

    //         [][]      []  []
    // []()[]    ()  []()[]  ()
    // []        []          [][]
    Tetromino {
        rotations: [
            [WIDTH as i16 - 1, -1, 0, 1],
            [0 - WIDTH as i16 - 1, 0 - WIDTH as i16, 0, WIDTH as i16],
            [-1, 0, 1, 0 - WIDTH as i16 + 1],
            [0 - WIDTH as i16, 0, WIDTH as i16, WIDTH as i16 + 1],
        ]
    },

    //           []  []      [][]
    // []()[]    ()  []()[]  ()
    //     []  [][]          []
    Tetromino {
        rotations: [
            [-1, 0, 1, WIDTH as i16 + 1],
            [0 - WIDTH as i16, 0, WIDTH as i16, WIDTH as i16 - 1],
            [0 - WIDTH as i16 - 1, -1, 0, 1],
            [0 - WIDTH as i16 + 1, 0 - WIDTH as i16, 0, WIDTH as i16],
        ]
    },

    //          []    []    []
    // []()[] []()  []()[]  ()[]
    //   []     []          []
    Tetromino {
        rotations: [
            [-1, 0, 1, WIDTH as i16],
            [0 - WIDTH as i16, -1, 0, WIDTH as i16],
            [0 - WIDTH as i16, -1, 0, 1],
            [0 - WIDTH as i16, 0, 1, WIDTH as i16],
        ]
    },
];

const SCORE_MAP:[u32; 5] = [0, 4, 10, 30, 120];

pub struct Grid {
    pub cells: [bool; (WIDTH * HEIGHT) as usize],

    pub tetromino_id: Option<usize>,
    pub on_new_tetromino: bool,
    pub position: u8,
    pub rotation: u8,
    pub interval: Duration,
    pub timer: Duration,
    pub gravity_bonus: u8,
    pub score: u32,
    pub cleared: u32,
    pub level: u8,
    pub rng: ThreadRng,
    pub game_over: bool,

}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            cells: [false; (WIDTH * HEIGHT) as usize],
            tetromino_id: None,
            on_new_tetromino: false,
            position: 0,
            rotation: 0,
            interval: INIT_INTERVAL,
            timer: Duration::ZERO,
            gravity_bonus: HEIGHT - 1,
            score: 0,
            cleared: 0,
            level: 0,
            rng: rand::thread_rng(),
            game_over: false,
        }
    }

    pub fn tick(&mut self, interval: Duration) {
        if self.game_over {
            return;
        }

        self.timer = self.timer + interval;

        if self.timer >= self.interval {
            self.timer = self.timer - self.interval;

            match self.tetromino_id {
                Some(_) => self.fall(true),
                None => self.next_tetromino(),
            }
        }
    }

    pub fn fall(&mut self, due_to_gravity: bool) {
        if self.game_over {
            return;
        }

        if let Some(_) = self.tetromino_id {
            if !self.move_if_can(self.position + WIDTH, self.rotation) {
                self.clear();
                self.next_tetromino();
            } else if due_to_gravity {
                self.gravity_bonus = self.gravity_bonus - 1;
            }
        }
    }

    pub fn horizontal_move(&mut self, offset: i16) {
        if self.game_over {
            return;
        }

        if let Some(_) = self.tetromino_id {
            self.move_if_can((self.position as i16 + offset) as u8, self.rotation);
        }
    }

    pub fn rotate(&mut self) {
        if self.game_over {
            return;
        }

        if let Some(_) = self.tetromino_id {
            let next_rotation = (self.rotation + 1) % 4;
            if self.move_if_can(self.position, next_rotation) {
                self.rotation = next_rotation;
            }
        }
    }

    fn clear(&mut self) {
        let mut finished: [bool; HEIGHT as usize] = [true; HEIGHT as usize];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if !self.cells[(y * WIDTH + x) as usize] {
                    finished[y as usize] = false;
                    break;
                }
            }
        }


        let mut y = 0;
        let mut cleared = 0;
        loop {
            if finished[y as usize] {
                let mut yy = y;
                loop {
                    for x in 0..WIDTH {
                        if yy == 0 {
                            self.cells[(yy * WIDTH + x) as usize] = false;
                        } else {
                            self.cells[(yy * WIDTH + x) as usize] = self.cells[((yy - 1) * WIDTH + x) as usize];
                        }
                    }

                    if yy == 0 {
                        break;
                    }

                    yy = yy - 1;
                }

                cleared = cleared + 1;
            }

            if y == HEIGHT - 1 {
                break;
            }

            y = y + 1;
        }

        if cleared > 0 {
            self.update_score(cleared);
        }
    }

    fn update_score(&mut self, cleared: u8) {
        self.cleared  = self.cleared + cleared as u32;
        let score = SCORE_MAP[cleared as usize] * self.gravity_bonus as u32 * (self.level as u32 + 1);
        self.score = self.score + score;
    }

    fn move_if_can(&mut self, new_position: u8, new_rotation: u8) -> bool {
        let tetromino_id = self.tetromino_id.unwrap();
        let current = TETROMINOES[tetromino_id].get_cells(self.position, self.rotation);
        let after = TETROMINOES[tetromino_id].get_cells(new_position, new_rotation);

        let mut can_move = true;

        for i in 0..after.len() {
            if after[i] < 0 {
                continue;
            }

            if after[i] as usize >= self.cells.len() {
                can_move = false;
                break;
            }

            if self.cells[after[i] as usize] && !current.contains(&after[i]) {
                can_move = false;
                break;
            }

            if after[i] as u8 % WIDTH == 0 && self.position % WIDTH >= WIDTH / 2 {
                can_move = false;
                break;
            }

            if after[i] as u8 % WIDTH == WIDTH - 1 && self.position % WIDTH < WIDTH / 2 {
                can_move = false;
                break;
            }
        }

        if can_move {
            for i in 0..current.len() {
                if current[i] >= 0 {
                    self.cells[current[i] as usize] = false;
                }
            }
            for i in 0..after.len() {
                if after[i] >= 0 {
                    self.cells[after[i] as usize] = true;
                }
            }
            self.position = new_position;
            return true;
        }

        return false;
    }

    fn next_tetromino(&mut self) {
        let n = self.rng.gen_range(0..TETROMINO_VARIANT);
        self.tetromino_id = Some(n);
        self.rotation = 0;
        self.reset_position();
        self.gravity_bonus = 0;

        let placement = TETROMINOES[n].get_cells(self.position, self.rotation);

        for i in 0..placement.len() {
            if self.cells[placement[i as usize] as usize] {
                // cannot place new tetromino - GAME OVER!
                self.game_over = true;
            }
        }

        self.on_new_tetromino = !self.game_over;
    }

    fn reset_position(&mut self) {
        self.position = WIDTH / 2 - 1;
    }

    pub fn punish(&mut self) {
        if self.game_over {
            return;
        }

        let current = self.tetromino_id.map(|id| {
            TETROMINOES[id].get_cells(self.position, self.rotation)
        });

        // move all cells 1 row up
        for i in 0..self.cells.len() - WIDTH as usize {
            let is_current = current.map_or(false, |tetromino| {
                tetromino.contains(&(i as i16)) || tetromino.contains(&((i + WIDTH as usize) as i16))
            });

            if !is_current {
                self.cells[i] = self.cells[i + WIDTH as usize];
            }
        }

        // populate random cells in last row - make sure it's not complete
        let mut last_row: [bool; WIDTH as usize] = [false; WIDTH as usize];
        let mut ratio = 100;
        for i in 0..last_row.len() {
            if thread_rng().gen_ratio(ratio, 100) {
                last_row[i] = true;
                ratio = ratio - 100 / WIDTH as u32;
            }
        }

        last_row.shuffle(&mut thread_rng());

        let n = self.cells.len() - WIDTH as usize;
        for i in 0..last_row.len() {
            self.cells[n + i] = last_row[i];
        }
    }

    pub fn reset(&mut self) {
        self.tetromino_id = None;
        self.on_new_tetromino = false;
        self.interval = INIT_INTERVAL;
        self.timer = Duration::ZERO;
        self.gravity_bonus =  HEIGHT - 1;
        self.score = 0;
        self.cleared = 0;
        self.level = 0;
        self.game_over = false;

        self.cells = [false; (WIDTH * HEIGHT) as usize];
    }

    pub fn reset_on_new_tetromino(&mut self) {
        self.on_new_tetromino = false;
    }
}
