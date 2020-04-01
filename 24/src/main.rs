use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::io::{BufRead, stdin};
use std::iter::once;
use itertools::Either;
use std::fmt;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Tile {
    Bug,
    Empty
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        use Tile::*;
        match c {
            '#' => Ok(Bug),
            '.' | '?' => Ok(Empty),
            _ => Err(format!("Unknown tile {}", c).into())
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Tile::*;
        match *self {
            Bug => write!(f, "#"),
            Empty => write!(f, "."),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Position {
    x: isize,
    y: isize,
    level : isize
}

impl Position {
    fn adjacent(self) -> impl Iterator<Item=Position> {
        [(0, 1), (-1, 0), (0, -1), (1, 0)].into_iter().map(move |(x, y)| Position {
            x: self.x + x,
            y: self.y + y,
            level: self.level,
        }).flat_map(move |Position { x, y, level }| {
            if x < 0 {
                Either::Left(once(Position { x: 1, y: 2, level: level - 1 }))
            } else if x > 4 {
                Either::Left(once(Position { x: 3, y: 2, level: level - 1 }))
            } else if y < 0 {
                Either::Left(once(Position { x: 2, y: 1, level: level - 1 }))
            } else if y > 4 {
                Either::Left(once(Position { x: 2, y: 3, level: level - 1 }))
            } else if (x, y) == (2, 2) {
                Either::Right((0..5).map(move |index| 
                    if self.x < 2 {
                        Position { x: 0, y: index, level: level + 1 }
                    } else if self.x > 2 {
                        Position { x: 4, y: index, level: level + 1 }
                    } else if self.y < 2 {
                        Position { x: index, y: 0, level: level + 1 }
                    } else if self.y > 2 {
                        Position { x: index, y: 4, level: level + 1 }
                    } else {
                        unreachable!();
                    }
                ))
            } else {
                Either::Left(once(Position { x, y, level }))
            }

        })
    }
}

struct Map {
    tiles: HashMap<Position, Tile>
}

impl Map {
    fn read(input: impl BufRead) -> Self {
        let mut tiles = HashMap::new();
        for (y, line) in input.lines().enumerate() {
            for (x, c) in line.unwrap().chars().enumerate() {
                let position = Position { x: x as isize, y: y as isize, level: 0 };
                tiles.insert(position, c.try_into().unwrap());
            }
        }

        Map { tiles }
    }

    fn rating(&self) -> usize {
        self.tiles.iter().map(|(&Position { x, y, .. }, &tile)| if tile == Tile::Bug {
            (2 as usize).pow(x as u32 + 5 * y as u32)
        } else {
            0
        }).sum()
    }

    fn tile(&self, position: Position) -> Tile {
        self.tiles.get(&position).cloned().unwrap_or(Tile::Empty)
    } 

    fn adjacent_bugs(&self, position: Position) -> usize {
        position.adjacent().filter(|p| self.tile(*p) == Tile::Bug).count()
    }

    fn num_bugs(&self) -> usize {
        self.tiles.values().filter(|&&tile| tile == Tile::Bug).count()
    }

    fn display(&self) {
        let levels: HashSet<_> = self.tiles.keys().map(|&Position{ level, .. }| level).collect();
        let mut levels: Vec<_> = levels.into_iter().collect();
        levels.sort();

        for level in levels {
            println!("Depth {}:", level);
            for y in 0..4 {
                for x in 0..4 {
                    let position = Position { x, y, level };
                    let tile = self.tiles.get(&position).cloned().unwrap_or(Tile::Empty);
                    print!("{}", tile);
                }
                println!("");
            }
        }

    }

    fn next(&self) -> Self {
        let mut positions: HashSet<_> = self.tiles.keys().cloned().collect();
        for position in self.tiles.keys() {
            for p in position.adjacent() {
                positions.insert(p);
            }
        }

        let mut tiles = HashMap::new();

        for position in positions {
            let tile = self.tiles.get(&position).cloned().unwrap_or(Tile::Empty);
            
            let new_tile = match tile {
                Tile::Bug => {
                    if self.adjacent_bugs(position) == 1 {
                        Tile::Bug
                    } else {
                        Tile::Empty
                    }
                },
                Tile::Empty => {
                    let bugs = self.adjacent_bugs(position);
                    if bugs == 1 || bugs == 2 {
                        Tile::Bug
                    } else {
                        Tile::Empty
                    }
                }
            };
            
            if new_tile == Tile::Bug {
                tiles.insert(position, new_tile);
            }
        }

        Map { tiles }
    }
}

fn main() {
    let mut map = Map::read(stdin().lock());

    for _ in 0..200 {
        map = map.next();
    }
    map.display();
    
    println!("{}", map.num_bugs());
}
