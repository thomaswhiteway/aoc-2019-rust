use gcd::Gcd;
use itertools::iproduct;
use std::io::{stdin, Read};
use std::str::FromStr;
use std::cmp::Ordering;
use std::f64::consts::PI;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Clone)]
struct Map {
    asteroids: Box<[Box<[bool]>]>,
}

type Position = (usize, usize);

fn angle_from((dest_x, dest_y): Position, (source_x, source_y): Position) -> f64 {
    let (x, y) = (dest_x as f64 - source_x as f64, dest_y as f64 - source_y as f64);
    let angle = (x.abs() / y.abs()).atan();
    if x >= 0.0 {
        if y < 0.0 {
            angle
        } else {
            PI - angle
        }
    } else {
        if y >= 0.0 {
            PI + angle
        } else {
            2.0 * PI - angle
        }
    }
}

impl Map {
    fn width(&self) -> usize {
        self.asteroids[0].len()
    }

    fn height(&self) -> usize {
        self.asteroids.len()
    }

    fn destroy_asteroid(&mut self, position: Position) {
        self.asteroids[position.1][position.0] = false;
    }

    fn positions(&self) -> impl Iterator<Item = Position> {
        iproduct!(0..self.width(), 0..self.height())
    }

    fn asteroids<'a>(&'a self) -> impl Iterator<Item = Position> + 'a {
        self.positions()
            .filter(move |position| self.asteroid_at(*position))
    }

    fn viewable_from<'a>(&'a self, position: Position) -> impl Iterator<Item = Position> + 'a {
        self.asteroids()
            .filter(move |other| *other != position)
            .filter(move |other| self.can_see(position, *other))
    }

    fn can_see(&self, viewer: Position, asteroid: Position) -> bool {
        let offset = (asteroid.0 as isize - viewer.0 as isize, asteroid.1 as isize - viewer.1 as isize);
        let divisor = if offset.0 == 0 {
            offset.1.abs()
        } else if offset.1 == 0 {
            offset.0.abs()
        } else {
             (offset.0.abs() as usize).gcd(offset.1.abs() as usize) as isize
        };

        Iterator::zip(
            (1..divisor).map(|index| index * offset.0 / divisor),
            (1..divisor).map(|index| index * offset.1 / divisor),
        )
        .map(|(x, y)| ((viewer.0 as isize + x) as usize, (viewer.1 as isize + y) as usize))
        .all(|position| !self.asteroid_at(position))
    }

    fn asteroid_at(&self, (x, y): Position) -> bool {
        self.asteroids[y][x]
    }

    fn read(mut input: impl Read) -> Result<Self, Error> {
        let mut buffer = String::new();
        input.read_to_string(&mut buffer)?;
        let map = buffer.parse()?;
        Ok(map)
    }
}

impl FromStr for Map {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let asteroids = s
            .lines()
            .map(|line| {
                line.trim()
                    .chars()
                    .map(|c| c == '#')
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(Map { asteroids })
    }
}

fn base_location(map: &Map) -> Position {
    map.asteroids()
        .max_by_key(|position| map.viewable_from(*position).count())
        .unwrap()
}

fn nth_destroyed(map: &mut Map, mut index: usize) -> Position {
    let location = base_location(map);
    let mut asteroids: Vec<_> = map.viewable_from(location).collect();
    while index > asteroids.len() {
        for asteroid in asteroids {
            map.destroy_asteroid(asteroid);
            index -= 1;
        }
        asteroids = map.viewable_from(location).collect();
    }
    asteroids.sort_by(|left, right| {
        f64::partial_cmp(&angle_from(*left, location), &angle_from(*right, location)).unwrap_or(Ordering::Equal)
    });
    asteroids[index-1]
}

fn main() {
    let mut map = Map::read(stdin().lock()).unwrap();
    let position = nth_destroyed(&mut map, 200);
    println!("{}", position.0 * 100 + position.1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn destroyed() {
        let data = r#".#..##.###...#######
        ##.############..##.
        .#.######.########.#
        .###.#######.####.#.
        #####.##.#.##.###.##
        ..#####..#.#########
        ####################
        #.####....###.#.#.##
        ##.#################
        #####.##.###..####..
        ..######..##.#######
        ####.##.####...##..#
        .#####..#.######.###
        ##...#.##########...
        #.##########.#######
        .####.#.###.###.#.##
        ....##.##.###..#####
        .#.#.###########.###
        #.#.#.#####.####.###
        ###.##.####.##.#..##"#;
        let map: Map = data.parse().unwrap();
        assert_eq!(angle_from((11, 12), (11, 13)), 0.0);
        assert!(angle_from((19, 12), (11, 13)) > PI / 4.0);
        assert!(angle_from((19, 12), (11, 13)) < PI / 2.0);
        assert_eq!(base_location(&map), (11, 13));
        assert_eq!(nth_destroyed(&mut map.clone(), 1), (11, 12));
        assert_eq!(nth_destroyed(&mut map.clone(), 2), (12, 1));
    }
}