use std::cell::{RefCell};
use std::convert::TryFrom;
use std::fmt;
use std::io::{stdin, stdout, Write};
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::cmp::Ordering;
use structopt::StructOpt;

mod process;
mod program;

use process::{Input, Output, Process, State};
use program::Program;

struct Ticker {
    interval: Duration,
    next_tick: Instant,
}

impl Ticker {
    fn new(interval: Duration) -> Self {
        Ticker {
            interval,
            next_tick: Instant::now()
        }
    }

    fn wait(&mut self) {
        let tick = self.next().unwrap();
        while Instant::now() < tick {
        }
    }
}

impl Iterator for Ticker {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        let tick = self.next_tick;
        self.next_tick += self.interval;
        Some(tick)
    }
}

trait Screen {
    fn clear(&mut self);
    fn set_tile(&mut self, position: [u16; 2], tile: Tile);
    fn display_score(&mut self, score: i64);
}

impl<T: Write> Screen for T {
    fn clear(&mut self) {
        let _ = write!(self, "{}", clear::All);
    }

    fn set_tile(&mut self, [x, y]: [u16; 2], tile: Tile) {
        let _ = write!(self, "{}{}", cursor::Goto(x + 1, y + 1), tile);
        let _ = self.flush();
    }

    fn display_score(&mut self, score: i64) {
        let _ = write!(self, "{}{}{}Score: {}{}", cursor::Goto(1, 24), clear::CurrentLine, color::Fg(color::Blue), score, color::Fg(color::Reset));
        let _ = self.flush();
    }
}

struct ScreenBuffer {}

impl Screen for ScreenBuffer {
    fn clear(&mut self) {}

    fn set_tile(&mut self, [x, y]: [u16; 2], tile: Tile) {
        if tile == Tile::Ball || tile == Tile::Paddle {
            println!("Output: ({}, {}): {:?}", x, y, tile);
        }
    }

    fn display_score(&mut self, score: i64) {
        println!("Score: {}", score);
    }
}

#[derive(Clone)]
struct GameState {
    score: i64,
    ball_position: [u16; 2],
    paddle_position: [u16; 2],
    ball_velocity: [i16; 2],
    cells: HashMap<[u16; 2], Tile>
}

impl GameState {
    fn new() -> Self {
        GameState {
            score: 0,
            ball_position: [19, 17],
            paddle_position: [0, 0],
            ball_velocity: [1, 1],
            cells: HashMap::new(),
        }
    }

    fn print_position(&self, position: &[u16; 2]) {
        for y in position[1]-1..position[1]+2 {
            for x in position[0]-1..position[0]+2 {
                let tile = self.cells.get(&[x, y]).cloned().unwrap_or_default();
                print!("{}", tile);
            }
            print!("\n");
        }
    }
}

struct Display<'a, T> {
    screen: RefCell<T>,
    state: &'a RefCell<GameState>,
    buffer: RefCell<Vec<i64>>,
}

impl<'a, T: Screen> Display<'a, T> {
    fn new(mut screen: T, state: &'a RefCell<GameState>) -> Self {
        screen.clear();
        Display {
            screen: RefCell::new(screen),
            state,
            buffer: RefCell::new(vec![]),
        }
    }
}

impl<'a, T: Screen> Output<i64> for Display<'a, T> {
    fn put(&self, value: i64) {
        let mut buffer = self.buffer.borrow_mut();
        let mut state = self.state.borrow_mut();

        buffer.push(value);

        let clear = match &**buffer {
            [x, y, value] => {
                if (*x, *y) == (-1, 0) {
                    state.score = *value;
                    self.screen.borrow_mut().display_score(*value);
                } else {
                    let tile = Tile::try_from(*value).unwrap();
                    let position = [*x as u16, *y as u16];
                    self.screen
                        .borrow_mut()
                        .set_tile(position, tile);
                    match tile {
                        Tile::Ball => {
                            if position != state.ball_position {
                                state.ball_velocity[0] = position[0] as i16 - state.ball_position[0] as i16;
                                state.ball_velocity[1] = position[1] as i16 - state.ball_position[1] as i16;
                                state.ball_position = position;
                            }
                        }
                        Tile::Paddle => state.paddle_position = position,
                        _ => {}
                    }
                    state.cells.insert(position, tile);
                }
                true
            }
            _ => { false }
        };

        if clear {
            buffer.clear();
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl Default for Tile {
    fn default() -> Tile {
        Tile::Empty
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Tile::*;
        match self {
            Empty => write!(f, " "),
            Wall => write!(f, "\u{2588}"),
            Block => write!(f, "{}X{}", color::Fg(color::Red), color::Fg(color::Reset)),
            Paddle => write!(f, "\u{2588}"),
            Ball => write!(f, "O"),
        }
    }
}

impl TryFrom<i64> for Tile {
    type Error = i64;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        use Tile::*;
        match value {
            0 => Ok(Empty),
            1 => Ok(Wall),
            2 => Ok(Block),
            3 => Ok(Paddle),
            4 => Ok(Ball),
            _ => Err(value),
        }
    }
}

struct Joystick<'a> {
    last_state: RefCell<Option<GameState>>,
    state: &'a RefCell<GameState>,
    ticker: Option<RefCell<Ticker>>
}

impl<'a> Joystick<'a> {
    fn new(update_rate: Option<u64>, state: &'a RefCell<GameState>) -> Self {
        Joystick {
            last_state: RefCell::new(None),
            state,
            ticker: update_rate.map(|rate| RefCell::new(Ticker::new(Duration::from_nanos(1_000_000_000 / rate))))
        }
    }
}

fn offset_from(position: &[u16;2], offset: &[i16;2]) -> [u16;2] {
    [
        (position[0] as i16 + offset[0]) as u16,
        (position[1] as i16 + offset[1]) as u16
    ]
}

fn step(state: &mut GameState) {
    loop {
        let mut collision = false;

        for axis in 0..2 {
            let mut offset = [0;2];
            offset[axis] = state.ball_velocity[axis];
            let position = offset_from(&state.ball_position, &offset);
            
            let next_tile = state.cells.get(&position).cloned().unwrap_or_default();
            
            if next_tile != Tile::Empty {
                collision = true;
                state.ball_velocity[axis] *= -1;
                if next_tile == Tile::Block {
                    state.cells.insert(position, Tile::Empty);
                }
            }
        }
        
        let position = offset_from(&state.ball_position, &state.ball_velocity);
        let next_tile = state.cells.get(&position).cloned().unwrap_or_default();
        if next_tile != Tile::Empty {
            collision = true;
            for axis in 0..2 {
                state.ball_velocity[axis] *= -1;
            }
            if next_tile == Tile::Block {
                state.cells.insert(position, Tile::Empty);
            }
        }

        if !collision {
            break
        }
    }
        
    state.cells.insert(state.ball_position, Tile::Empty);
    for axis in 0..2 {
        state.ball_position[axis] = (state.ball_position[axis] as i16 + state.ball_velocity[axis]) as u16;
    }
    state.cells.insert(state.ball_position, Tile::Ball);
}

fn calculate_intersect(mut state: GameState) -> u16 {
    while state.ball_position[1] < state.paddle_position[1] - 1 {
        step(&mut state);
    }
    state.ball_position[0]
}

impl<'a> Input<i64> for Joystick<'a> {
    fn get(&self) -> Option<i64> {
        let state = self.state.borrow();
        let intersect = calculate_intersect(state.clone());
        let input = match intersect.cmp(&state.paddle_position[0]) {
            Ordering::Greater => 1,
            Ordering::Less => -1,
            Ordering::Equal => 0
        };

        let mut last_state = self.last_state.borrow_mut();
        if let Some(ref last_state) = *last_state {
            let mut expected_state = last_state.clone();
            step(&mut expected_state);

            if last_state.ball_position[1] < last_state.paddle_position[1] - 1 && expected_state.ball_position != state.ball_position {
                println!("Was ({}, {}):", last_state.ball_velocity[0], last_state.ball_velocity[1]);
                last_state.print_position(&last_state.ball_position);
                println!("Expected:");
                expected_state.print_position(&last_state.ball_position);
                println!("Got:");
                state.print_position(&last_state.ball_position);
                panic!("Unexpected state change");
            }
        }
        
        *last_state = Some(state.clone());

        if let Some(ref ticker) = self.ticker {
            ticker.borrow_mut().wait();
        }
        Some(input as i64)
    }
}

#[derive(Debug, StructOpt)]
struct Opts {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Set speed
    #[structopt(short, long)]
    speed: Option<u64>,
}

fn run<T: Screen>(program: &Program, screen: T, speed: Option<u64>) {
    let state = RefCell::new(GameState::new());

    let input = Joystick::new(speed, &state);
    let output = Display::new(screen, &state);

    {
        let mut process = Process::new("Game".to_string(), &program, &input, &output);
        process.set(0, 2);

        let state = process.execute();
        assert_eq!(state, State::Complete);
    }
}

fn main() {
    let program = Program::parse(stdin()).unwrap();

    let opts = Opts::from_args();

    if !opts.debug {
        let screen = cursor::HideCursor::from(stdout().into_raw_mode().unwrap());
        run(&program, screen, opts.speed);
        println!("{}", cursor::Goto(1, 25));
    } else {
        let screen = ScreenBuffer {};
        run(&program, screen, opts.speed);
    };
}
