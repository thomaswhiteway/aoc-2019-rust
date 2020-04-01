use std::collections::{HashMap, HashSet};
use std::io::{stdin, BufRead};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Wall,
    Empty,
}

enum PortalType {
    Up,
    Down,
}

struct Portal {
    exit: Position,
    portal_type: PortalType,
}

impl Portal {
    fn traverse(&self, level: usize) -> Option<(Position, usize)> {
        match self.portal_type {
            PortalType::Down => Some((self.exit, level + 1)),
            PortalType::Up if level > 0 => Some((self.exit, level - 1)),
            _ => None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn adjacent(self) -> impl Iterator<Item = Position> {
        [(0, -1), (1, 0), (0, 1), (-1, 0)]
            .into_iter()
            .map(move |(x, y)| Position {
                x: self.x + x,
                y: self.y + y,
            })
    }
}

struct Map {
    tiles: HashMap<Position, Tile>,
    portals: HashMap<Position, Portal>,
    start: (Position, usize),
    end: (Position, usize),
}

impl Map {

    fn read(input: impl BufRead) -> Map {
        let mut label_fragments = HashMap::new();
        let mut tiles = HashMap::new();

        for (y, line) in input.lines().enumerate() {
            for (x, c) in line.unwrap().chars().enumerate() {
                let position = Position {
                    x: x as isize,
                    y: y as isize,
                };
                if c == '#' {
                    tiles.insert(position, Tile::Wall);
                } else if c == '.' {
                    tiles.insert(position, Tile::Empty);
                } else if c.is_ascii_uppercase() {
                    label_fragments.insert(position, c);
                }
            }
        }

        let min_x = tiles.keys().cloned().map(|p| p.x).min().unwrap();
        let min_y = tiles.keys().cloned().map(|p| p.y).min().unwrap();
        let max_x = tiles.keys().cloned().map(|p| p.x).max().unwrap();
        let max_y = tiles.keys().cloned().map(|p| p.y).max().unwrap();

        let mut start = None;
        let mut end = None;
        let mut labels: HashMap<String, Vec<Position>> = HashMap::new();

        while let Some(position_a) = label_fragments.keys().cloned().next() {
            let position_b = position_a
                .adjacent()
                .filter(|pos| label_fragments.contains_key(pos))
                .next()
                .unwrap();
            let a = label_fragments.remove(&position_a).unwrap();
            let b = label_fragments.remove(&position_b).unwrap();

            let mut order = vec![(position_a, a), (position_b, b)];
            order.sort();

            let label: String = order.into_iter().map(|(_, c)| c).collect();
            let position = [position_a, position_b]
                .into_iter()
                .filter_map(|pos| {
                    pos.adjacent()
                        .filter(|p| tiles.get(&p).cloned() == Some(Tile::Empty))
                        .next()
                })
                .next()
                .unwrap();

            match label.as_str() {
                "AA" => start = Some(position),
                "ZZ" => end = Some(position),
                _ => labels.entry(label).or_default().push(position),
            }
        }

        let mut portals = HashMap::new();
        let portal_type = |p: Position| {
            if p.x == min_x || p.x == max_x || p.y == min_y || p.y == max_y {
                PortalType::Up
            } else {
                PortalType::Down
            }
        };

        for positions in labels.values() {
            portals.insert(positions[0], Portal {
                exit: positions[1],
                portal_type: portal_type(positions[0]),
            });
            portals.insert(positions[1],  Portal {
                exit: positions[0],
                portal_type: portal_type(positions[1]),
            });
        }

        Map {
            tiles,
            portals,
            start: (start.unwrap(), 0),
            end: (end.unwrap(), 0),
        }
    }

    fn can_visit(&self, position: Position) -> bool {
        self.tiles.get(&position).cloned().unwrap_or(Tile::Wall) == Tile::Empty
    }

    fn shortest_distance(&self, from: (Position, usize), to: (Position, usize)) -> Option<usize> {
        let mut visited = HashSet::new();
        let mut distance = 0;
        let mut layer = vec![from];
        
        while !layer.is_empty() {
            let mut next_layer: Vec<(Position, usize)> = vec![];

            for (position, level) in layer {
                if (position, level) == to {
                    return Some(distance);
                }
                    
                visited.insert((position, level));

                for position in position.adjacent() {
                    let next_position = (position, level);
                    if !visited.contains(&next_position) && !next_layer.contains(&next_position) && self.can_visit(position) {
                        next_layer.push(next_position);
   
                    }
                }

                if let Some(portal) = self.portals.get(&position) {
                    if let Some(exit) = portal.traverse(level) {
                        if !visited.contains(&exit) && !next_layer.contains(&exit) {
                            next_layer.push(exit)
                        }
                    }
                }
            }

            distance += 1;
            layer = next_layer;
        }

        None
    }
}

fn main() {
    let map = Map::read(stdin().lock());
    let distance = map.shortest_distance(map.start, map.end).unwrap();
    println!("{}", distance);
}
