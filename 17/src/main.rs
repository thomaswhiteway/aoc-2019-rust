use std::char;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::io::{stdin, stdout};
use std::iter::once;
use structopt::StructOpt;
use termion::raw::IntoRawMode;

mod display;
mod process;
mod program;
mod utils;

use display::{Screen, ScreenBuffer};
use process::{Channel, Output, Process, State};
use program::Program;
use termion::cursor;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Movement {
    Left,
    Forward,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Left,
    Forward(usize),
    Right,
}

impl From<Movement> for Instruction {
    fn from(movement: Movement) -> Self {
        match movement {
            Movement::Left => Instruction::Left,
            Movement::Forward => Instruction::Forward(1),
            Movement::Right => Instruction::Right,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Instruction::*;
        match self {
            Left => write!(f, "L"),
            Forward(num) => write!(f, "{}", num),
            Right => write!(f, "R"),
        }
    }
}

#[derive(Clone)]
struct Instructions<T>(Vec<T>);

impl Instructions<Instruction> {
    fn new(moves: impl IntoIterator<Item = Movement>) -> Self {
        let mut instructions = vec![];
        let mut current_forward = None;

        for movement in moves {
            if movement == Movement::Forward {
                current_forward = Some(current_forward.unwrap_or_default() + 1)
            } else {
                if let Some(num) = current_forward {
                    instructions.push(Instruction::Forward(num));
                    current_forward = None;
                }
                instructions.push(movement.into());
            }
        }

        if let Some(num) = current_forward {
            instructions.push(Instruction::Forward(num));
        }

        Instructions(instructions)
    }
}

impl<T: fmt::Display> fmt::Display for Instructions<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for instruction in self.0.iter() {
            if !first {
                write!(f, ",")?;
            } else {
                first = false;
            }
            write!(f, "{}", instruction)?;
        }

        Ok(())
    }
}

impl TryFrom<i64> for Direction {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        use Direction::*;
        match value {
            0 => Ok(North),
            1 => Ok(East),
            2 => Ok(South),
            3 => Ok(West),
            _ => Err(format!("Unknown direction {}", value).into()),
        }
    }
}

impl Direction {
    fn all() -> impl Iterator<Item = Direction> {
        (0..4).map(Direction::try_from).map(Result::unwrap)
    }

    fn turn(self, movement: Movement) -> Self {
        let offset = match movement {
            Movement::Left => 3,
            Movement::Forward => 0,
            Movement::Right => 1,
        };

        (((self as i64) + offset) % 4).try_into().unwrap()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Tile {
    Empty,
    Scaffolding,
    Robot(Direction),
}

impl Default for Tile {
    fn default() -> Tile {
        Tile::Empty
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Direction::*;
        use Tile::*;
        match self {
            Empty => write!(f, " "),
            Scaffolding => write!(f, "#"),
            Robot(North) => write!(f, "^"),
            Robot(East) => write!(f, ">"),
            Robot(South) => write!(f, "v"),
            Robot(West) => write!(f, "<"),
        }
    }
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Direction::*;
        use Tile::*;
        match value {
            '#' => Ok(Scaffolding),
            '.' => Ok(Empty),
            '^' => Ok(Robot(North)),
            '>' => Ok(Robot(East)),
            'v' => Ok(Robot(South)),
            '<' => Ok(Robot(West)),
            _ => Err(format!("Unknown tile {}", value).into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Position {
    x: i64,
    y: i64,
}

#[allow(dead_code)]
impl Position {
    fn origin() -> Self {
        Position { x: 0, y: 0 }
    }

    fn moved(&self, direction: Direction) -> Position {
        use Direction::*;
        let mut position = self.clone();
        match direction {
            North => position.y -= 1,
            East => position.x += 1,
            South => position.y += 1,
            West => position.x -= 1,
        }
        position
    }

    fn adjacent(self) -> impl Iterator<Item = Position> {
        Direction::all().map(move |direction| self.moved(direction))
    }

    fn length(self) -> usize {
        self.x.abs() as usize + self.y.abs() as usize
    }

    fn offset(self, other: Position) -> Position {
        Position {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }

    fn distance(self, other: Position) -> usize {
        self.offset(other).length()
    }
}

impl From<Position> for [u16; 2] {
    fn from(position: Position) -> Self {
        [position.x as u16, position.y as u16]
    }
}

#[derive(Clone)]
struct Robot {
    position: Position,
    direction: Direction,
}

struct Map {
    occupied: HashMap<Position, bool>,
    robot: Robot,
}

#[allow(dead_code)]
impl Map {
    fn route(&self) -> Vec<Movement> {
        let mut robot = self.robot.clone();
        let mut route = vec![];

        let next_move = |robot: &mut Robot| {
            for movement in [Movement::Forward, Movement::Left, Movement::Right].iter() {
                let direction = robot.direction.turn(*movement);
                let position = robot.position.moved(direction);

                if self.has_scaffolding(position) {
                    match movement {
                        Movement::Forward => robot.position = position,
                        _ => robot.direction = direction,
                    }
                    return Some(*movement);
                }
            }
            None
        };

        while let Some(next_move) = next_move(&mut robot) {
            route.push(next_move);
        }

        route
    }

    fn has_scaffolding(&self, position: Position) -> bool {
        !self.occupied.get(&position).cloned().unwrap_or(true)
    }

    fn intersections<'a>(&'a self) -> impl Iterator<Item = Position> + 'a {
        let positions: Vec<_> = self.occupied.keys().collect();
        positions.into_iter().cloned().filter(move |position| {
            self.has_scaffolding(*position)
                && Direction::all().all(|direction| self.has_scaffolding(position.moved(direction)))
        })
    }
}

fn build_map(output: &[i64]) -> Map {
    let mut occupied = HashMap::new();
    let mut robot = None;

    let mut position = Position::origin();

    for item in output
        .iter()
        .map(|v| *v as u32)
        .map(char::from_u32)
        .map(Option::unwrap)
    {
        if item == '\n' {
            position.x = 0;
            position.y += 1;
        } else {
            let tile: Tile = item.try_into().unwrap();
            occupied.insert(position, tile == Tile::Empty);

            if let Tile::Robot(direction) = tile {
                robot = Some(Robot {
                    position,
                    direction,
                });
            }

            position.x += 1;
        }
    }

    Map {
        occupied,
        robot: robot.unwrap(),
    }
}

#[derive(Debug, Clone, Copy)]
enum Function {
    A,
    B,
    C,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Function::*;
        match self {
            A => write!(f, "A"),
            B => write!(f, "B"),
            C => write!(f, "C"),
        }
    }
}

#[derive(Clone)]
struct RobotProgram {
    functions: Vec<Instructions<Instruction>>,
    calls: Instructions<Function>,
}

impl RobotProgram {
    fn encode<'a>(&'a self) -> impl Iterator<Item = u8> + 'a {
        once(self.calls.to_string())
            .chain(self.functions.iter().map(|f| f.to_string()))
            .flat_map(|seq| seq.as_bytes().to_vec().into_iter().chain(once('\n' as u8)))
    }
}

fn collapse(sequence: &[Movement]) -> Option<RobotProgram> {
    for a_length in (2..sequence.len()).rev() {
        let a_moves = &sequence[0..a_length];
        let a = Instructions::new(a_moves.iter().cloned());

        if a.to_string().len() > 20 {
            continue;
        }

        let mut b_start = a_length;
        let mut calls = vec![Function::A];

        loop {
            if sequence[b_start..].starts_with(a_moves) {
                b_start += a_moves.len();
                calls.push(Function::A)
            } else {
                break;
            }
        }

        for b_end in (b_start + 2..sequence.len()).rev() {
            let b_moves = &sequence[b_start..b_end];
            let b = Instructions::new(b_moves.iter().cloned());

            if b.to_string().len() > 20 {
                continue;
            }

            let mut c_start = b_end;
            let mut calls = calls.clone();
            calls.push(Function::B);

            loop {
                if sequence[c_start..].starts_with(a_moves) {
                    c_start += a_moves.len();
                    calls.push(Function::A);
                } else if sequence[c_start..].starts_with(b_moves) {
                    c_start += b_moves.len();
                    calls.push(Function::B);
                } else {
                    break;
                }
            }

            for c_end in c_start + 2..sequence.len() {
                let c_moves = &sequence[c_start..c_end];
                let c = Instructions::new(c_moves.iter().cloned());

                if c.to_string().len() > 20 {
                    continue;
                }

                let mut index = c_end;
                let mut calls = calls.clone();
                calls.push(Function::C);

                loop {
                    if sequence[index..].starts_with(a_moves) {
                        index += a_moves.len();
                        calls.push(Function::A);
                    } else if sequence[index..].starts_with(b_moves) {
                        index += b_moves.len();
                        calls.push(Function::B);
                    } else if sequence[index..].starts_with(c_moves) {
                        index += c_moves.len();
                        calls.push(Function::C);
                    } else {
                        break;
                    }
                }

                if calls.len() > 10 {
                    continue;
                }

                if index != sequence.len() {
                    continue;
                }

                return Some(RobotProgram {
                    functions: vec![a, b, c],
                    calls: Instructions(calls),
                });
            }
        }
    }

    None
}

fn read_map(program: &Program) -> Map {
    let input = Channel::new();
    let output = Channel::new();

    let mut process = Process::new("Camera", program, &input, &output);
    let state = process.execute();
    assert_eq!(state, State::Complete);

    let result: Vec<_> = output.into();

    build_map(&result)
}

fn run_program(program: &Program, robot_program: &RobotProgram) -> i64 {
    let input = Channel::new();
    let output = Channel::new();

    for b in robot_program.encode() {
        input.put(b as i64);
    }

    input.put('n' as i64);
    input.put('\n' as i64);

    let mut process = Process::new("Robot", program, &input, &output);
    process.set(0, 2);

    let state = process.execute();
    assert_eq!(state, State::Complete);

    let result: Vec<_> = output.into();
    result[result.len()-1]
}

#[allow(dead_code)]
fn display_map(map: &Map, screen: &mut impl Screen) {
    for (position, occupied) in map.occupied.iter() {
        screen.set_tile(
            (*position).into(),
            if *occupied {
                Tile::Empty
            } else {
                Tile::Scaffolding
            },
        )
    }
    screen.set_tile(map.robot.position.into(), Tile::Robot(map.robot.direction));

    let max_y = map.occupied.keys().map(|p| p.y).max().unwrap();
    screen.goto([0, max_y as u16 + 1]);
}

fn run(program: &Program, mut screen: impl Screen, _speed: Option<u64>) {
    screen.clear();
    let map = read_map(program);
    display_map(&map, &mut screen);

    let alignment: i64 = map.intersections().map(|Position { x, y }| x * y).sum();
    screen.print(format!("Alignment: {}", alignment));

    let route = map.route();
    let instructions = Instructions::new(route.iter().cloned());
    screen.print(format!("{}", instructions));

    let robot_program = collapse(&route).unwrap();
    screen.print(format!("A: {}", robot_program.functions[0]));
    screen.print(format!("B: {}", robot_program.functions[1]));
    screen.print(format!("C: {}", robot_program.functions[2]));
    screen.print(format!("Calls: {}", robot_program.calls));

    let dust = run_program(program, &robot_program);
    screen.print(format!("Dust collected: {}", dust));
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

fn main() {
    let program = Program::parse(stdin()).unwrap();

    let opts = Opts::from_args();

    if !opts.debug {
        let screen = cursor::HideCursor::from(stdout().into_raw_mode().unwrap());
        run(&program, screen, opts.speed);
    } else {
        let screen = ScreenBuffer {};
        run(&program, screen, opts.speed);
    };
}
