use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::convert::TryFrom;
use std::io::{stdin, BufRead};
use std::iter::repeat;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Key(char);

enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn all() -> impl Iterator<Item = Direction> {
        (0..4).map(Direction::try_from).map(Result::unwrap)
    }
}

impl TryFrom<u32> for Direction {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Position {
    x: isize,
    y: isize,
}

#[allow(dead_code)]
impl Position {
    fn new(x: isize, y: isize) -> Self {
        Position { x, y }
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Empty,
    Wall,
    Door(Key),
}

struct Map {
    tiles: HashMap<Position, Tile>,
    keys: HashMap<Position, Key>,
    start: Vec<Position>,
}

#[allow(dead_code)]
impl Map {
    fn distance(&self, from: Position, to: Position, keys: &HashSet<Key>) -> Option<(usize, Vec<Key>, bool)> {
        #[derive(PartialEq, Eq)]
        struct Entry {
            position: Position,
            destination: Position,
            distance: usize,
            used_keys: Vec<Key>,
            passed_key: bool
        }

        impl Entry {
            fn min_distance(&self) -> usize {
                self.distance + self.position.distance(self.destination)
            }
        }

        impl Ord for Entry {
            fn cmp(&self, other: &Self) -> Ordering {
                self.min_distance().cmp(&other.min_distance()).reverse()
            }
        }

        impl PartialOrd for Entry {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();

        heap.push(Entry {
            position: from,
            destination: to,
            distance: 0,
            used_keys: vec![],
            passed_key: false,
        });

        while let Some(Entry {
            position,
            destination,
            distance,
            mut used_keys,
            passed_key,
        }) = heap.pop()
        {
            visited.insert(position);

            if position == destination {
                return Some((distance, used_keys, passed_key));
            }

            if let Tile::Door(key) = self.tiles.get(&position).unwrap() {
                used_keys.push(*key);
            } 

            let passed_key = passed_key || (distance > 0 && self.keys.contains_key(&position));

            for position in position.adjacent() {
                if !visited.contains(&position) && self.can_pass(position, keys) {
                    heap.push(Entry {
                        position,
                        destination,
                        distance: distance + 1,
                        used_keys: used_keys.clone(),
                        passed_key,
                    });
                }
            }
        }

        None
    }

    fn read(input: impl BufRead) -> Self {
        let mut tiles = HashMap::new();
        let mut keys = HashMap::new();
        let mut start = vec![];

        for (y, line) in input.lines().enumerate() {
            for (x, c) in line.unwrap().chars().enumerate() {
                let position = Position::new(x as isize, y as isize);

                tiles.insert(
                    position,
                    if c == '#' {
                        Tile::Wall
                    } else if c.is_ascii_uppercase() {
                        Tile::Door(Key(c.to_ascii_lowercase()))
                    } else {
                        Tile::Empty
                    },
                );

                if c.is_ascii_lowercase() {
                    keys.insert(position, Key(c));
                } 

                if c == '@' {
                    start.push(position);
                }
            }
        }

        Map {
            tiles,
            keys,
            start,
        }
    }

    fn can_pass(&self, position: Position, keys: &HashSet<Key>) -> bool {
        use Tile::*;
        match self.tiles.get(&position).cloned().unwrap_or(Wall) {
            Wall => false,
            Empty => true,
            Door(key) => keys.contains(&key),
        }
    }

    fn reachable(&self, position: Position) -> Vec<(Position, usize, Tile)> {
        use Tile::*;

        let mut positions = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![(position, 0, Empty)];

        while let Some((position, distance, tile)) = stack.pop() {
            positions.push((position, distance, tile));
            visited.insert(position);

            for position in position.adjacent() {
                if !visited.contains(&position) {
                    let tile = self.tiles.get(&position).cloned().unwrap_or(Wall);
                    match tile {
                        Empty | Door(_) => stack.push((position, distance + 1, tile)),
                        _ => {}
                    }
                }
            }
        }

        positions
    }

    fn reachable_objects<'a>(
        &'a self,
        position: Position,
    ) -> impl Iterator<Item = (Object, usize)> + 'a {
        use Tile::*;
        self.reachable(position).into_iter().filter_map(
            move |(position, distance, tile)| match tile {
                Empty => self
                    .keys
                    .get(&position)
                    .cloned()
                    .map(|key| (Object::Key(key), distance)),
                Door(key) => Some((Object::Door(key), distance)),
                _ => None,
            },
        )
    }

    fn reachable_keys<'a>(
        &'a self,
        position: Position,
    ) -> impl Iterator<Item = (Position, Key)> + 'a {
        use Tile::*;
        self.reachable(position).into_iter().filter_map(
            move |(position, _, tile)| match tile {
                Empty => self
                    .keys
                    .get(&position)
                    .cloned()
                    .map(|key| (position, key)),
                _ => None,
            },
        )
    }

    fn routes_to(&self, from: Position, to: Position) -> Routes {
        let all_keys: HashSet<_> = self.keys.values().cloned().collect();
        let (distance, required_keys, passed_key) = self.distance(from, to, &all_keys).unwrap();

        let mut routes = vec![];
        
        if !passed_key {
            routes.push(Route { length: distance, keys_required: required_keys.iter().cloned().collect() });
        } 

        let mut sets_to_check: Vec<_> = required_keys.iter().map(|key| {
            let mut set = all_keys.clone();
            set.remove(key);
            set
        }).collect();

        while let Some(key_set) = sets_to_check.pop() {
            let (distance, required_keys, passed_key) = if let Some(result) = self.distance(from, to, &key_set) {
                result
            } else {
                continue;
            };

            let route = Route {
                length: distance,
                keys_required: required_keys.iter().cloned().collect()
            };

            let mut index = 0;
            while index < routes.len() && routes[index].length < route.length {
                index += 1;
            }

            if !routes[index..].contains(&route) {
                if !passed_key {
                    routes.insert(index, route);
                }

                for key in required_keys {
                    let mut new_set = key_set.clone();
                    new_set.remove(&key);
                    sets_to_check.push(new_set);
                }
            }
        }

        Routes(routes)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Object {
    Key(Key),
    Door(Key),
    Start,
}

#[derive(PartialEq, Eq, Debug)]
struct Route {
    length: usize,
    keys_required: HashSet<Key>,
}

#[derive(Debug)]
struct Routes(Vec<Route>);

impl Routes {
    fn best_route<'a>(&'a self, missing_keys: &HashSet<Key>) -> Option<&'a Route> {
        for route in self.0.iter() {
            if route.keys_required.is_disjoint(&missing_keys) {
                return Some(route)
            }
        }
        None
    }
}

struct Node {
    routes: HashMap<Object, Routes>
}

struct Nodes(HashMap<Object, Node>);

impl Nodes {
    fn new(map: &Map, start: Position) -> Self {
        let mut nodes = HashMap::new();

        let reachable_keys: Vec<_> = map.reachable_keys(start).collect();
        assert!(reachable_keys.len() > 0);
        let routes: HashMap<_, _> = reachable_keys.iter().map(|&(position, key)| 
            (Object::Key(key), map.routes_to(start, position))
        ).collect();
        assert!(routes.len() > 0);
        nodes.insert(Object::Start, Node { routes });

        for &(position, key) in reachable_keys.iter() {
            println!("Getting routes from {:?}", key);
            let routes: HashMap<_, _> = reachable_keys.iter().map(|&(other_position, other_key)| 
                (Object::Key(other_key), map.routes_to(position, other_position))
            ).collect();
            assert!(routes.len() > 0);
            nodes.insert(Object::Key(key), Node { routes });
        }

        Nodes(nodes)
    }

    fn reachable_keys(&self, object: Object, missing_keys: &HashSet<Key>) -> Vec<(Key, usize)> {
        let mut reachable = vec![];

        #[derive(PartialEq, Eq)]
        struct Entry {
            object: Object,
            distance: usize,
        }

        impl Ord for Entry {
            fn cmp(&self, other: &Self) -> Ordering {
                self.distance.cmp(&other.distance).reverse()
            }
        }

        impl PartialOrd for Entry {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(&other))
            }
        }

        let mut visited = HashSet::new();
        let mut heap = BinaryHeap::new();

        for (destination, routes) in self.0.get(&object).unwrap().routes.iter() {
            if let Some(route) = routes.best_route(missing_keys) {
                heap.push(Entry {
                    object: *destination,
                    distance: route.length,
                });
            }
        }

        while let Some(Entry { object, distance }) = heap.pop() {
            if visited.contains(&object) {
                continue;
            }
            
            visited.insert(object);

            match object {
                Object::Key(key) => reachable.push((key, distance)),
                Object::Door(key) => {
                    if missing_keys.contains(&key) {
                        continue;
                    }
                }
                _ => {}
            }

            for (destination, routes) in self.0.get(&object).unwrap().routes.iter() {
                if let Some(route) = routes.best_route(missing_keys) {
                    heap.push(Entry {
                        object: *destination,
                        distance: distance + route.length,
                    })
                }
            }
        }

        reachable
    }
}


fn get_all_keys(map: &Map) -> Option<usize> {
    let nodes: Vec<_> = map.start.iter().cloned().map(|start| Nodes::new(map, start)).collect();
    let objects: Vec<_> = repeat(Object::Start).take(nodes.len()).collect();
    println!("Computed nodes");
    get_keys(&nodes, map.keys.values().cloned().collect(), &objects, usize::max_value(), &mut HashMap::new())
}

enum CacheEntry {
    Found(usize),
    AtLeast(usize),
}

fn get_keys(nodes: &[Nodes], keys: HashSet<Key>, start: &[Object], max_distance: usize, cache: &mut HashMap<(Vec<Key>, Vec<Object>), CacheEntry>) -> Option<usize> {
    if keys.len() == 0 {
        return Some(0);
    }

    let mut cache_key = (keys.iter().cloned().collect::<Vec<_>>(), start.iter().cloned().collect::<Vec<_>>());
    cache_key.0.sort();
    if let Some(entry) = cache.get(&cache_key) {
        match *entry {
            CacheEntry::Found(distance) => if distance < max_distance {
                return Some(distance)
            } else {
                return None
            },
            CacheEntry::AtLeast(distance) => if distance >= max_distance {
                return None
            }
        }
    }

    let mut best = None;

    for index in 0..nodes.len() {
        for (key, key_distance) in nodes[index].reachable_keys(start[index], &keys) {
            if keys.contains(&key) && key_distance < best.unwrap_or(max_distance) {
                let mut keys = keys.clone();
                keys.remove(&key);
                
                let mut positions = start.to_vec();
                positions[index] = Object::Key(key);

                if let Some(distance) = get_keys(&nodes, keys, &positions, best.unwrap_or(max_distance) - key_distance, cache) {
                    best = Some(key_distance + distance);
                }
            }
        }
    }

    cache.insert(cache_key, match best {
        Some(distance) => CacheEntry::Found(distance),
        None => CacheEntry::AtLeast(max_distance),
    });

    best
}

fn main() {
    let map = Map::read(stdin().lock());

    if let Some(distance) = get_all_keys(&map) {
        println!("Distance: {}", distance);
    } else {
        println!("No solution possible");
    }
}
