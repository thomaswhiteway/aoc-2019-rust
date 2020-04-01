use std::char;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::stdin;

mod process;
mod program;

use process::{Channel, Input, Output, Process, State};
use program::Program;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl TryFrom<isize> for Direction {
    type Error = String;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        use Direction::*;
        match value {
            0 => Ok(North),
            1 => Ok(East),
            2 => Ok(South),
            3 => Ok(West),
            _ => Err(format!("Invalid direction {}", value)),
        }
    }
}

impl Direction {
    fn turn(&mut self, turn: Turn) {
        use Turn::*;
        let increment = match turn {
            Left => 3,
            Right => 1,
        };
        *self = ((*self as isize + increment) % 4).try_into().unwrap();
    }

    fn move_from(&self, (x, y): (isize, isize)) -> (isize, isize) {
        use Direction::*;
        match *self {
            North => (x, y - 1),
            East => (x + 1, y),
            South => (x, y + 1),
            West => (x - 1, y),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Turn {
    Left,
    Right,
}

impl TryFrom<i64> for Turn {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Turn::Left),
            1 => Ok(Turn::Right),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Colour {
    Black,
    White,
}

impl Colour {
    fn as_char(self) -> char {
        use Colour::*;
        match self {
            White => char::from_u32(0x2588).unwrap(),
            Black => ' ',
        }
    }
}

impl Default for Colour {
    fn default() -> Self {
        Colour::Black
    }
}

impl TryFrom<i64> for Colour {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Colour::Black),
            1 => Ok(Colour::White),
            _ => Err(()),
        }
    }
}

struct Robot {
    position: (isize, isize),
    direction: Direction,
}

impl Robot {
    fn new() -> Self {
        Robot {
            position: (0, 0),
            direction: Direction::North,
        }
    }

    fn turn(&mut self, turn: Turn) {
        self.direction.turn(turn)
    }

    fn move_forwards(&mut self) {
        self.position = self.direction.move_from(self.position)
    }
}

enum Signal {
    Paint,
    Turn,
}

impl Signal {
    fn flip(&mut self) {
        use Signal::*;
        *self = match *self {
            Paint => Turn,
            Turn => Paint,
        }
    }
}

fn paint(program: &Program) -> HashMap<(isize, isize), Colour> {
    let mut robot = Robot::new();

    let input = Channel::new();
    let output = Channel::new();
    let mut process = Process::new("Robot".to_string(), &program, &input, &output);
    let mut signal = Signal::Paint;

    let mut cells: HashMap<(isize, isize), Colour> = HashMap::new();
    cells.insert((0, 0), Colour::White);

    while process.execute() != State::Complete {
        while let Some(value) = output.get() {
            match signal {
                Signal::Paint => {
                    cells.insert(robot.position, value.try_into().unwrap());
                }
                Signal::Turn => {
                    robot.turn(value.try_into().unwrap());
                    robot.move_forwards();
                }
            }
            signal.flip()
        }

        input.put(cells.get(&robot.position).cloned().unwrap_or_default() as i64);
    }

    cells
}

fn display_cells(cells: &HashMap<(isize, isize), Colour>) {
    let min_x = cells.keys().map(|(x, _)| x).min().cloned().unwrap();
    let max_x = cells.keys().map(|(x, _)| x).max().cloned().unwrap();
    let min_y = cells.keys().map(|(_, y)| y).min().cloned().unwrap();
    let max_y = cells.keys().map(|(_, y)| y).max().cloned().unwrap();
    for y in min_y..=max_y {
        let row = (min_x..=max_x).map(|x| cells.get(&(x, y)).cloned().unwrap_or_default());
        println!("{}", row.map(Colour::as_char).collect::<String>())
    }
}

fn main() {
    let program = Program::parse(stdin()).unwrap();

    let cells = paint(&program);
    display_cells(&cells);
}
