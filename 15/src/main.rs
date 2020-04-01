use std::convert::{TryInto, TryFrom};
use std::fmt;
use std::io::{stdin, stdout, Write};

use std::time::Duration;
use std::collections::{HashMap, VecDeque, HashSet, BinaryHeap};
use structopt::StructOpt;
use termion::raw::IntoRawMode;
use std::cmp::{Ord, Ordering};

mod process;
mod program;
mod utils;
mod display;

use display::{Screen ,ScreenBuffer};
use process::{Input, Output, Process, State, Channel};
use program::Program;
use termion::{color, cursor};
use utils::Ticker;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Direction {
    North = 1,
    South,
    West,
    East
}

impl Direction {
    fn all() -> impl Iterator<Item = Direction> {
        (1..5).map(Direction::try_from).map(Result::unwrap)
    }
}

impl TryFrom<i64> for Direction {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        use Direction::*;
        match value {
            1 => Ok(North),
            2 => Ok(South),
            3 => Ok(West),
            4 => Ok(East),
            _ => Err(format!("Unknown direction {}", value).into())
        }
    }
}

#[derive(PartialEq, Eq)]
enum Status {
    HitWall,
    Moved,
    FoundOxygenMachine,
}

impl TryFrom<i64> for Status {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        use Status::*;
        match value {
            0 => Ok(HitWall),
            1 => Ok(Moved),
            2 => Ok(FoundOxygenMachine),
            _ => Err(format!("Unknown status {}", value).into())
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Tile {
    Empty,
    Wall,
    OxygenMachine,
    Oxygen,
    Robot,
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
            Empty => write!(f, "{} {}", color::Bg(color::LightGreen), color::Bg(color::Reset)),
            Wall => write!(f, "\u{2588}"),
            Robot => write!(f, "{}X{}", color::Fg(color::Red), color::Fg(color::Reset)),
            OxygenMachine => write!(f, "{}O{}", color::Fg(color::Blue), color::Fg(color::Reset)),
            Oxygen => write!(f, "{} {}", color::Bg(color::LightBlue), color::Bg(color::Reset)),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Position {
    x: i64,
    y: i64
}

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
        [(position.x + 25) as u16, (position.y + 25) as u16]
    }
}

struct Map {
    occupied: HashMap<Position, bool>,
    robot: Position,
    oxygen_machine: Position,
}

impl Map {
    fn route(&self, from: Position, to: Position) -> Vec<Direction> {
        #[derive(Eq, PartialEq)]
        struct Entry {
            position: Position,
            route: Vec<Direction>,
            to: Position,
        };

        impl Entry {
            fn value(&self) -> usize {
                self.position.distance(self.to) + self.route.len()
            }
        }

        impl Ord for Entry {
            fn cmp(&self, other: &Entry) -> Ordering {
                self.value().cmp(&other.value()).reverse()
            }
        }

        impl PartialOrd for Entry {
            fn partial_cmp(&self, other: &Entry) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut heap = Vec::new();
        heap.push(Entry{ position: from, route: vec![], to });
        
        let mut visited = HashSet::new();
        
        while let Some(Entry { position, route, .. }) = heap.pop() {
            if position == to {
                return route;
            }

            visited.insert(position);

            for direction in Direction::all() {
                let position = position.moved(direction);
                if !visited.contains(&position) && !self.occupied.get(&position).cloned().unwrap_or(true) {
                    let mut route = route.clone();
                    route.push(direction);
                    heap.push(Entry { position, route, to });
                }
            }
        }

        vec![]
    }
}

fn display_map(map: &Map, screen: &mut impl Screen) {
    for (position, occupied) in map.occupied.iter() {
        screen.set_tile((*position).into(), if *occupied { Tile::Wall } else { Tile::Empty })
    }
}

fn fill_map(map: &Map, screen: &mut impl Screen, speed: Option<u64>) -> usize {
    display_map(map, screen);

    let mut ticker = speed.map(Ticker::with_rate);
    let mut oxgenated = HashSet::new();
    let mut next = vec![map.oxygen_machine];
    let mut turn = 0;

    while !next.is_empty() {
        turn += 1;

        let mut next_turn: Vec<_> = vec![];
        for position in next {
            oxgenated.insert(position);
            screen.set_tile(position.into(), Tile::Oxygen);

            for direction in Direction::all() {
                let position = position.moved(direction);
                if !map.occupied.get(&position).cloned().unwrap_or_default() && !oxgenated.contains(&position) {
                    next_turn.push(position);
                }
            }
        }

        next = next_turn;

        if let Some(ref mut ticker) = ticker {
            ticker.wait();
        }
    }

    turn - 1
}

fn pick_route(robot: Position, occupied: &HashMap<Position, bool>) -> Vec<Direction> {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    for direction in Direction::all() {
        queue.push_back((robot.moved(direction), vec![direction]));
    }

    while let Some((position, route)) = queue.pop_front() {
        if visited.contains(&position) {
            continue
        }

        visited.insert(position);

        if occupied.get(&position).cloned().unwrap_or(false) {
            continue
        }

        if !occupied.contains_key(&position) {
            return route;
        }

        for direction in Direction::all() {
            let mut route = route.clone();
            route.push(direction);
            queue.push_back((position.moved(direction), route));
        }
    }

    vec![]
}

fn explore(program: &Program, screen: &mut impl Screen, speed: Option<u64>) -> Map {
    let input = Channel::new();
    let output = Channel::new();

    let mut ticker = speed.map(Ticker::with_rate);

    let mut process = Process::new("ROBOT", program, &input, &output);
    let mut robot = Position::origin();
    let mut oxygen_machine = None;
    let mut occupied = HashMap::new();
    occupied.insert(robot, false);

    let mut route = vec![];

    loop {
        if route.is_empty() {
            route = pick_route(robot, &occupied);
        }

        if route.is_empty() {
            break
        }

        let direction = route.remove(0);
        input.put(direction as i64);

        let state = process.execute();
        assert_eq!(state, State::Blocked);

        let status: Status = output.get().unwrap().try_into().unwrap();
        let position = robot.moved(direction);
        occupied.insert(position, status == Status::HitWall);

        if status != Status::HitWall {
            if Some(robot) == oxygen_machine {
                screen.set_tile(robot.into(), Tile::OxygenMachine);
            } else {
                screen.set_tile(robot.into(), Tile::Empty);
            }

            robot = position;

            screen.set_tile(robot.into(), Tile::Robot);
        } else {
            screen.set_tile(position.into(), Tile::Wall);
        }

        if status == Status::FoundOxygenMachine {
            oxygen_machine = Some(position);
        }

        if let Some(ref mut ticker) = ticker {
            ticker.wait();
        }
    }

    Map {
        occupied,
        robot: Position::origin(),
        oxygen_machine: oxygen_machine.unwrap()
    }
}


fn run(program: &Program, mut screen: impl Screen, speed: Option<u64>) {
    screen.clear();
    let map = explore(program, &mut screen, speed);
    let distance = map.route(map.robot, map.oxygen_machine).len();
    screen.clear();
    let num_turns = fill_map(&map, &mut screen, speed);
    print!("{}", cursor::Goto(1, 50));
    println!("{}: {}", distance, num_turns);
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
