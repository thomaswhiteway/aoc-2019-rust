use std::cmp::Ordering;
use std::io::{stdin, BufRead};
use std::str::FromStr;
use std::collections::HashSet;
use std::hash::Hash;

struct Vector([i64; 3]);

impl Vector {
    fn zero() -> Self {
        Vector([0, 0, 0])
    }

    fn sum(&self) -> i64 {
        self.0.iter().cloned().map(i64::abs).sum()
    }
}

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

impl FromStr for Vector {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if !s.starts_with('<') || !s.ends_with('>') {
            return Err(Error("Missing <, or >".to_string()));
        }

        let mut result = [0; 3];
        for part in s[1..s.len() - 1].split(',') {
            let mut parts = part.splitn(2, '=');
            let index = match parts.next().unwrap().trim() {
                "x" => 0,
                "y" => 1,
                "z" => 2,
                c => return Err(Error(format!("Unknown component {:?}", c))),
            };

            if let Some(value) = parts.next() {
                result[index] = value.trim().parse()?;
            } else {
                return Err(Error("No value for component".to_string()));
            }
        }
        result[0] = 0;
        result[1] = 0;
        Ok(Vector(result))
    }
}

struct Moon {
    position: Vector,
    velocity: Vector,
}

impl Moon {
    fn new(position: Vector) -> Self {
        Moon {
            position,
            velocity: Vector::zero(),
        }
    }

    fn potential_energy(&self) -> i64 {
        self.position.sum()
    }

    fn kinetic_energy(&self) -> i64 {
        self.velocity.sum()
    }

    fn total_energy(&self) -> i64 {
        self.potential_energy() * self.kinetic_energy()
    }

    fn step(&mut self) {
        for axis in 0..3 {
            self.position.0[axis] += self.velocity.0[axis];
        }
    }
}

fn update_velocity(moon_a: &mut Moon, moon_b: &mut Moon) {
    for axis in 0..3 {
        match moon_a.position.0[axis].cmp(&moon_b.position.0[axis]) {
            Ordering::Less => {
                moon_a.velocity.0[axis] += 1;
                moon_b.velocity.0[axis] -= 1;
            }
            Ordering::Greater => {
                moon_a.velocity.0[axis] -= 1;
                moon_b.velocity.0[axis] += 1;
            }
            _ => {}
        }
    }
}

fn apply_gravity(moons: &mut [Moon]) {
    for i in 0..moons.len() {
        for j in i + 1..moons.len() {
            let (left, right) = moons.split_at_mut(j);
            update_velocity(&mut left[i], &mut right[0])
        }
    }
}

fn update_position(moons: &mut [Moon]) {
    for moon in moons {
        moon.step()
    }
}

fn step(moons: &mut [Moon]) {
    apply_gravity(moons);
    update_position(moons);
}

fn parse_moons(input: impl BufRead) -> Box<[Moon]> {
    input
        .lines()
        .map(Result::unwrap)
        .map(|line| line.parse().unwrap())
        .map(Moon::new)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn total_energy(moons: &[Moon]) -> i64 {
    moons.iter().map(Moon::total_energy).sum()
}

fn key(moons: &[Moon]) -> impl Hash + Eq {
    let mut result: Vec<i64> = Vec::new();
    for moon in moons {
        result.extend(&moon.position.0);
        result.extend(&moon.velocity.0);
    }
    result
}

fn find_cycle(moons: &mut [Moon]) -> usize {
    let mut seen = HashSet::new();
    let mut num_steps = 0;

    while !seen.contains(&key(moons)) {
        seen.insert(key(moons));
        step(moons);
        num_steps += 1;
    }

    num_steps
}

fn main() {
    let mut moons = parse_moons(stdin().lock());
    let cycle_len = find_cycle(&mut moons);
    println!("{}", cycle_len);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_step() {
        let mut moons = vec![
            Moon::new(Vector([-1, 0, 2])),
            Moon::new(Vector([2, -10, -7])),
            Moon::new(Vector([4, -8, 8])),
            Moon::new(Vector([3, 5, -1])),
        ];

        step(&mut moons);

        assert_eq!(moons[0].position.0, [2, -1, 1]);
        assert_eq!(moons[1].position.0, [3, -7, -4]);
        assert_eq!(moons[2].position.0, [1, -7, 5]);
        assert_eq!(moons[3].position.0, [2, 2, 0]);

        step(&mut moons);

        assert_eq!(moons[0].position.0, [5, -3, -1]);
        assert_eq!(moons[1].position.0, [1, -2, 2]);
        assert_eq!(moons[2].position.0, [1, -4, -1]);
        assert_eq!(moons[3].position.0, [1, -4, 2]);
    }
}
