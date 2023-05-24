mod grid;
mod quiz;

use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use termion::{async_stdin, clear, color, cursor, style};
use termion::raw::{IntoRawMode, RawTerminal};

use std::io::{self, Read, Write, Result};
use std::{time, thread};

use quiz::Quiz;

const LAYOUT_QUIZ_WIDTH: u16 = 32;

const ARROW_UP: (u8, u8, u8) = (27, 91, 65);
const ARROW_DOWN: (u8, u8, u8) = (27, 91, 66);
const ARROW_LEFT: (u8, u8, u8) = (27, 91, 68);
const ARROW_RIGHT: (u8, u8, u8) = (27, 91, 67);

fn main() {
    let stdout = io::stdout();
    let stdin = async_stdin();

    // get console info
    let (term_width, term_height) = termion::terminal_size().ok().unwrap();

    let scale = Scale::new(2, 1);

    let mut game = Game::new(
        stdin,
        stdout.lock(),
        term_width,
        term_height,
        scale,
    );

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

struct CurrentQuiz {
    pub question: String,
    pub answers: Vec<String>,
    pub correct_answer_id: u8,
}

impl CurrentQuiz {
    pub fn new(
        question: String,
        answers: Vec<String>,
        correct_answer_id: u8,
    ) -> CurrentQuiz {
        CurrentQuiz {
            question,
            answers,
            correct_answer_id,
        }
    }
}

struct Game<R, W: Write> {
    grid: grid::Grid,
    stdin: R,
    stdout: W,
    offset_x: u16,
    offset_y: u16,
    scale: Scale,

    quizzes: Vec<Quiz>,
    quiz: Option<CurrentQuiz>,
}

impl<R: Read, W: Write> Game<R, W> {
    fn new(stdin: R, stdout: W, term_width: u16, term_height: u16, scale: Scale) -> Game<R, RawTerminal<W>> {
        let grid = grid::Grid::new();

        let f = std::fs::File::open("quizzes.yaml").unwrap();
        let quizzes: Vec<Quiz> = serde_yaml::from_reader(f).unwrap();

        Game {
            grid,
            stdin,
            stdout: stdout.into_raw_mode().unwrap(),
            offset_x: (term_width - grid::WIDTH as u16 * scale.x as u16) / 2 - LAYOUT_QUIZ_WIDTH,
            offset_y: term_height - grid::HEIGHT as u16 - 8,
            scale,
            quizzes,
            quiz: None,
        }
    }

    fn run(&mut self) -> Result<()> {
        write!(self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Hide)?;

        self.draw_layout()?;
        self.stdout.flush()?;

        let mut b: [u8; 3] = [0; 3];
        let interval = time::Duration::from_millis(50);
        'main: loop {
            thread::sleep(interval);

            if self.grid.position < grid::WIDTH {
                //self.quiz_rng();
            }

            // process input
            if let Ok(_) = self.stdin.read(&mut b) {

                match (b[0], b[1], b[2]) {
                    // quit
                    (b'\x1b', 0, 0) | (b'q', _, _) => break 'main,

                    // answer quiz
                    (n @ b'1'..=b'4', 0, 0) => self.answer(n),

                    // play tetris
                    (b'h', _, _) | ARROW_LEFT  if self.quiz.is_none() => self.grid.horizontal_move(-1),
                    (b'l', _, _) | ARROW_RIGHT if self.quiz.is_none() => self.grid.horizontal_move(1),
                    (b'k', _, _) | ARROW_UP    if self.quiz.is_none() => self.grid.rotate(),
                    (b'j', _, _) | ARROW_DOWN  if self.quiz.is_none() => self.grid.fall(false),

                    // reset game
                    (b'r', _, _) if self.grid.game_over => self.grid.reset(),

                    _ => (),
                }

                b[0] = 0;
                b[1] = 0;
                b[2] = 0;
            }

            // update grid
            self.grid.tick(interval);

            // draw
            self.draw_quiz()?;
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
            let x = self.offset_x + LAYOUT_QUIZ_WIDTH;
            let y = self.offset_y + y as u16;
            write!(self.stdout, "{}<!", cursor::Goto(x, y))?;
            write!(self.stdout, "{}!>", cursor::Goto(x + grid_width as u16 + 2, y as u16))?;
        }

        for x in 0..grid::WIDTH * self.scale.x {
            let offset_x = self.offset_x + LAYOUT_QUIZ_WIDTH + x as u16 + 2;
            let offset_y = self.offset_y + grid::HEIGHT as u16 * self.scale.y as u16 + 1;
            write!(self.stdout, "{}*", cursor::Goto(offset_x as u16, offset_y as u16))?;

            let offset_y = self.offset_y + grid::HEIGHT as u16 * self.scale.y as u16 + 1;
            let c = if x % 2 == 0 { '\\' } else { '/' };
            write!(self.stdout, "{}{}", cursor::Goto(offset_x as u16, (offset_y + 1) as u16), c)?;
        }

        Ok(())
    }

    fn draw_grid(&mut self) -> Result<()> {
        if self.grid.game_over {
            return self.draw_game_over();
        }

        let offset_x = self.offset_x + LAYOUT_QUIZ_WIDTH + 2;

        for i in 0..self.grid.cells.len() {
            let y = i as u8 / grid::WIDTH + 1;
            let x = i as u8 % grid::WIDTH;

            let x = offset_x + x as u16 * self.scale.x as u16;
            let y = self.offset_y + y as u16;
            let c = if self.grid.cells[i] { "[]" } else { " ." };

            write!(self.stdout, "{}{}", cursor::Goto(x as u16, y as u16), c)?;
        }

        Ok(())
    }

    fn draw_game_over(&mut self) -> Result<()> {
        let x = LAYOUT_QUIZ_WIDTH + 2;
        let y = 1;
        self.clear_area(x, y, grid::WIDTH as u16 * 2, grid::HEIGHT as u16)?;

        write!(self.stdout, "{}{}", cursor::Goto(x + self.offset_x + 4, y + self.offset_y + 3), "╭──────────╮")?;
        write!(self.stdout, "{}{}", cursor::Goto(x + self.offset_x + 4, y + self.offset_y + 4), "  G A M E")?;
        write!(self.stdout, "{}{}", cursor::Goto(x + self.offset_x + 4, y + self.offset_y + 6), "   O V E R")?;
        write!(self.stdout, "{}{}", cursor::Goto(x + self.offset_x + 4, y + self.offset_y + 7), "╰──────────╯")?;
        write!(self.stdout, "{}{}", cursor::Goto(x + self.offset_x + 4, y + self.offset_y + 10), "To try again")?;
        write!(self.stdout, "{}{}", cursor::Goto(x + self.offset_x + 5, y + self.offset_y + 12), "Press 'r'")?;

        Ok(())
    }

    fn draw_status(&mut self) -> Result<()> {
        let grid_width = grid::WIDTH * self.scale.x;
        let offset_x = self.offset_x + LAYOUT_QUIZ_WIDTH + grid_width as u16 + 6;
        let offset_y = self.offset_y + grid::HEIGHT as u16 * self.scale.y as u16;

        write!(self.stdout, "{}[{}{: >8}]", cursor::Goto(offset_x as u16, (offset_y - 4) as u16), "SCORE: ", self.grid.score)?;
        write!(self.stdout, "{}[{}{: >8}]", cursor::Goto(offset_x as u16, (offset_y - 2) as u16), "LEVEL: ", self.grid.level)?;
        write!(self.stdout, "{}[{}{: >8}]", cursor::Goto(offset_x as u16, (offset_y - 0) as u16), "LINES: ", self.grid.cleared)?;

        Ok(())
    }

    fn quiz_rng(&mut self) {
        if self.quiz.is_some() {
            return;
        }

        let id = rand::thread_rng().gen_range(0..self.quizzes.len());
        let quiz = &self.quizzes[id];

        let mut answers = quiz.wrong_answers.clone();
        answers.shuffle(&mut thread_rng());

        let correct_answer_id = thread_rng().gen_range(0..=answers.len());
        answers.insert(correct_answer_id, quiz.answer.clone());

        self.quiz = Some(CurrentQuiz::new(
            quiz.question.clone(),
            answers,
            correct_answer_id as u8,
        ));
    }

    fn answer(&mut self, n: u8) {
        if let Some(quiz) = &self.quiz {
            if n - 48 != quiz.correct_answer_id + 1 {
                self.grid.punish();
            }
        }

        self.quiz = None;
    }

    fn draw_quiz(&mut self) -> Result<()> {
        if let Some(quiz) = &self.quiz {

            // draw questoin
            let line_width = LAYOUT_QUIZ_WIDTH - 4;
            let offset_x = self.offset_x + 2;
            let offset_y = self.offset_y;

            let lines = split_into_lines(&quiz.question, line_width);
            for i in 0..lines.len() {
                write!(self.stdout, "{}{}", cursor::Goto(offset_x, offset_y + i as u16), lines[i])?;
            }

            // draw answer
            let line_width = LAYOUT_QUIZ_WIDTH - 7;
            let mut offset_y = self.offset_y + lines.len() as u16 + 1;

            for i in 0..quiz.answers.len() {
                let lines = split_into_lines(&quiz.answers[i], line_width);
                write!(self.stdout, "{}{}. ", cursor::Goto(offset_x, offset_y as u16), i + 1)?;

                for j in 0..lines.len() {
                    write!(self.stdout, "{}{}", cursor::Goto(offset_x + 3, offset_y as u16), lines[j])?;
                    offset_y = offset_y + 1;
                }

                offset_y = offset_y + 1;
            }

            Ok(())
        } else {
            self.clear_area(0, 0, LAYOUT_QUIZ_WIDTH, grid::HEIGHT as u16 + 10)
        }
    }

    fn clear_area(&mut self, from_x: u16, from_y: u16, width: u16, height: u16) -> Result<()> {
        let offset_x = self.offset_x + from_x;
        let offset_y = self.offset_y + from_y;

        let spaces = " ".repeat(width as usize);

        for y in offset_y..offset_y + height {
            write!(self.stdout, "{}{}", cursor::Goto(offset_x, y), spaces)?;
        }

        Ok(())
    }
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        write!(self.stdout, "{}{}", style::Reset, cursor::Show).unwrap();
    }
}

fn split_into_lines(text: &String, width: u16) -> Vec<String> {
    let mut lines = Vec::with_capacity(text.len() / width as usize + 1);
    let mut iter = text.split_whitespace();

    let mut line = String::with_capacity(width as usize);
    loop {
        if let Some(word) = iter.next() {
            if line.len() + word.len() + 1 > width as usize {
                lines.push(line.to_string());
                line = String::with_capacity(width as usize);
            }

            if line.len() > 0 {
                line = line + " " + word;
            } else {
                line = line + word;
            }
        } else {
            if line.len() > 0 {
                lines.push(line.to_string());
            }
            break;
        }
    }

    return lines;
}
