mod grid;

use termion::{async_stdin, clear, color, cursor, style};
use termion::raw::{IntoRawMode, RawTerminal};

use std::io::{self, Read, Write, Result};
use std::{time, thread};

const LAYOUT_QUIZ_WIDTH: u8 = 32;

fn main() {
    let stdout = io::stdout();
    let stdin = async_stdin();

    // get console info
    let (term_width, term_height) = termion::terminal_size().ok().unwrap();

    let scale = Scale::new(2, 1);

    let mut game = Game::new(stdin, stdout.lock(), scale);
    if let Err(err) = game.run() {
        write!(io::stderr(), "{}", err).unwrap();
    }
}

struct Scale {
    pub x: u8,
    pub y: u8,
}

impl Scale {
    pub fn new(x: u8, y: u8) -> Scale {
        Scale {x, y}
    }
}

struct Game<R, W: Write> {
    grid: grid::Grid,
    stdin: R,
    stdout: W,
    scale: Scale,
}

impl<R: Read, W: Write> Game<R, W> {
    fn new(stdin: R, stdout: W, scale: Scale) -> Game<R, RawTerminal<W>> {
        let grid = grid::Grid::new(
            time::Duration::from_nanos(1_200_000_000)
        );

        Game {
            grid,
            stdin,
            stdout: stdout.into_raw_mode().unwrap(),
            scale,
        }
    }

    fn run(&mut self) -> Result<()> {
        write!(self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Hide)?;
        self.draw_layout()?;
        self.stdout.flush()?;

        let mut b: [u8; 1] = [0];
        let tick = time::Duration::from_millis(50);
        'main: loop {
            thread::sleep(tick);
            if b[0] != 0 {
                write!(self.stdout, "{}{}", cursor::Goto(1, 1), b[0])?;
            }

            // process input
            if self.stdin.read(&mut b).is_ok() {
                match b[0] {
                    b'\x1b' | b'q' => break 'main,
                    b'h' => self.grid.horizontal_move(-1),
                    b'l' => self.grid.horizontal_move(1),
                    b'k' => self.grid.rotate(),
                    b'j' => self.grid.fall(false),
                    _ => (),
                }

                b[0] = 0;
            }

            // update grid
            self.grid.tick(tick);

            // draw
            self.draw_grid()?;
            self.draw_status()?;

            self.stdout.flush()?;
        }

        Ok(())
    }

    // quiz | tetris grid | status
    fn draw_layout(&mut self) -> Result<()> {
        for y in 1..(grid::HEIGHT * self.scale.y) + 2 {
            let grid_width = grid::WIDTH * self.scale.x;
            write!(self.stdout, "{}<!", cursor::Goto(LAYOUT_QUIZ_WIDTH as u16, y as u16))?;
            write!(self.stdout, "{}!>", cursor::Goto((LAYOUT_QUIZ_WIDTH + grid_width + 2) as u16, y as u16))?;
        }

        for x in 0..grid::WIDTH * self.scale.x {
            let offset_x = LAYOUT_QUIZ_WIDTH + x + 2;
            let offset_y = grid::HEIGHT * self.scale.y + 1;
            write!(self.stdout, "{}*", cursor::Goto(offset_x as u16, offset_y as u16))?;

            let offset_y = grid::HEIGHT * self.scale.y + 1;
            let c = if x % 2 == 0 { '\\' } else { '/' };
            write!(self.stdout, "{}{}", cursor::Goto(offset_x as u16, (offset_y + 1) as u16), c)?;
        }

        Ok(())
    }

    fn draw_grid(&mut self) -> Result<()> {
        let offset_x = LAYOUT_QUIZ_WIDTH + 2;

        for i in 0..self.grid.cells.len() {
            let y = i as u8 / grid::WIDTH + 1;
            let x = i as u8 % grid::WIDTH;

            let x = offset_x + x * self.scale.x;
            let c = if self.grid.cells[i] { "[]" } else { " ." };
            write!(self.stdout, "{}{}", cursor::Goto(x as u16, y as u16), c)?;
        }

        Ok(())
    }

    fn draw_status(&mut self) -> Result<()> {
        let grid_width = grid::WIDTH * self.scale.x;
        let offset_x = LAYOUT_QUIZ_WIDTH + grid_width + 6;
        let offset_y = grid::HEIGHT * self.scale.y;

        write!(self.stdout, "{}{}{}", cursor::Goto(offset_x as u16, (offset_y - 4) as u16), "SCORE: ", self.grid.score)?;
        write!(self.stdout, "{}{}{}", cursor::Goto(offset_x as u16, (offset_y - 2) as u16), "LEVEL: ", self.grid.level)?;
        write!(self.stdout, "{}{}{}", cursor::Goto(offset_x as u16, (offset_y - 0) as u16), "LINES: ", self.grid.cleared)?;

        Ok(())
    }
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        write!(self.stdout, "{}{}", style::Reset, cursor::Show).unwrap();
    }
}
