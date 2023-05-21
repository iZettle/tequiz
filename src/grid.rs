use std::time::Duration;
use rand::{self, rngs::ThreadRng, Rng};
use termion::cursor;

pub const WIDTH: u8 = 10;
pub const HEIGHT: u8 = 21;

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

const TETROMINO_VARIANT: usize = 2;
const TETROMINOS: [Tetromino; TETROMINO_VARIANT] = [
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
];

pub struct Grid {
    pub cells: [bool; (WIDTH * HEIGHT) as usize],

    tetromino_id: Option<usize>,
    position: u8,
    rotation: u8,
    rate: Duration,
    timer: Duration,

    rng: ThreadRng,
}

impl Grid {
    pub fn new(rate: Duration) -> Grid {
        Grid {
            cells: [false; (WIDTH * HEIGHT) as usize],
            tetromino_id: None,
            position: 0,
            rotation: 0,
            rate,
            timer: Duration::ZERO,
            rng: rand::thread_rng(),
        }
    }

    pub fn tick(&mut self, interval: Duration) {
        self.timer = self.timer + interval;

        if self.timer >= self.rate {
            self.timer = self.timer - self.rate;

            match self.tetromino_id {
                Some(_) => self.fall(),
                None => self.next_tetromino(),
            }
        }
    }

    fn fall(&mut self) {
        if !self.move_if_can(self.position + WIDTH) {
            self.clear();
            self.next_tetromino();
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
        loop {
            if finished[y as usize] {
                print!("{}{}", cursor::Goto(1, 1), y);
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
            }

            if y == HEIGHT - 1 {
                break;
            }

            y = y + 1;
        }
    }

    pub fn horizontal_move(&mut self, offset: i8) {
        if let Some(tetromino_id) = self.tetromino_id {
            let current = TETROMINOS[tetromino_id].get_cells(self.position, self.rotation);
            let mut can_move = true;
            for i in 0..current.len() {
                if current[i] < 0 {
                    continue;
                }

                if offset < 0 && current[i] as u8 % WIDTH == 0 {
                    can_move = false;
                }

                if offset > 0 && current[i] as u8 % WIDTH == WIDTH - 1 {
                    can_move = false;
                }
            }

            if can_move {
                self.move_if_can((self.position as i8 + offset) as u8);
            }
        }
    }

    fn move_if_can(&mut self, new_position: u8) -> bool {
        let tetromino_id = self.tetromino_id.unwrap();
        let current = TETROMINOS[tetromino_id].get_cells(self.position, self.rotation);
        let after = TETROMINOS[tetromino_id].get_cells(new_position, self.rotation);

        let mut can_move = true;

        for i in 0..after.len() {
            if after[i] < 0 {
                continue;
            }

            if after[i] as usize >= self.cells.len() {
                can_move = false;
                break;
            }

            if self.cells[after[i] as usize] {
                // check if this cell is in the tetromino itself
                let mut is_self = false;
                for j in 0..current.len() {
                    if after[i] == current[j] {
                        is_self = true;
                        break;
                    }
                }

                if !is_self {
                    can_move = false;
                    break;
                }
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
    }

    pub fn reset_position(&mut self) {
        self.position = WIDTH / 2 - 1;
    }
}
