use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        use Direction::*;
        match c {
            'U' => Ok(Up),
            'D' => Ok(Down),
            'L' => Ok(Left),
            'R' => Ok(Right),
            _ => Err(format!("Invalid direction '{}'", c)),
        }
    }
}

#[derive(Clone, Copy)]
struct Movement {
    direction: Direction,
    distance: usize,
}

impl FromStr for Movement {
    type Err = String;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let direction = data
            .chars()
            .nth(0)
            .ok_or_else(|| "Empty movement".to_string())
            .and_then(Direction::try_from)?;
        let distance = data[1..]
            .trim()
            .parse()
            .map_err(|err| format!("Invalid distance: {}", err))?;
        Ok(Movement {
            direction,
            distance,
        })
    }
}

impl Movement {
    fn flatten(self) -> impl Iterator<Item = Direction> {
        (0..self.distance).map(move |_| self.direction)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Position {
    fn origin() -> Self {
        Position { x: 0, y: 0 }
    }

    fn shift(&mut self, direction: Direction) {
        use Direction::*;
        match direction {
            Up => self.y += 1,
            Down => self.y -= 1,
            Left => self.x -= 1,
            Right => self.x += 1,
        }
    }

    fn distance(self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

struct Wire {
    route: Box<[Position]>,
}

impl FromStr for Wire {
    type Err = String;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let movements = data
            .split(',')
            .map(Movement::from_str)
            .map(Result::unwrap)
            .flat_map(Movement::flatten);

        let mut route = vec![];
        let mut position = Position::origin();
        for direction in movements {
            position.shift(direction);
            route.push(position)
        }

        Ok(Wire {
            route: route.into_boxed_slice(),
        })
    }
}
impl Wire {
    fn read(mut input: impl BufRead) -> Result<Self, Error> {
        let mut buffer = String::new();
        input.read_line(&mut buffer)?;
        Ok(buffer.parse()?)
    }

    fn intersection(&self, other: &Wire) -> Vec<(Position, usize, usize)> {
        let left: HashMap<_, _> = self.route.iter().zip(1..self.route.len() + 1).rev().collect();
        let right: HashMap<_, _> = other.route.iter().zip(1..other.route.len() + 1).rev().collect();

        let mut intersection = vec![];
        for position in left.keys() {
            if right.contains_key(position) {
                intersection.push((**position, left[position], right[position]))
            }
        }

        intersection
    }
}

fn main() {
    let input = stdin();
    let wire_1 = Wire::read(input.lock()).unwrap();
    let wire_2 = Wire::read(input.lock()).unwrap();

    let intersection = wire_1.intersection(&wire_2);

    let closest = intersection
        .into_iter()
        .min_by_key(|(_, step_1, step_2)| step_1 + step_2);

    if let Some((position, step_1, step_2)) = closest {
        println!("Closest point is at {}, distance {}, total steps = {}", position, position.distance(), step_1 + step_2);
    } else {
        println!("Lines do not intersect");
    }
}
