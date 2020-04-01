use std::str::FromStr;
use std::io::{BufRead, stdin};
use std::collections::HashMap;

struct Orbit {
    object: String,
    centre: String
}

impl FromStr for Orbit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().splitn(2, ')');
        let centre = parts.next().unwrap().to_string();
        let object = parts.next().ok_or_else(|| "No centre".to_string())?.to_string();
        Ok(Orbit { object, centre })
    }
}

fn get_orbiters<'a>(orbits: impl Iterator<Item = &'a Orbit>) -> HashMap<String, Vec<String>> {
    let mut orbiters: HashMap<String, Vec<String>> = HashMap::new();

    for orbit in orbits {
        orbiters.entry(orbit.centre.clone()).or_default().push(orbit.object.clone());
    }

    orbiters
}

#[allow(dead_code)]
fn total_orbits<'a>(orbits: impl Iterator<Item = &'a Orbit>) -> usize {
    let orbiters = get_orbiters(orbits);
    
    let mut indirect_counts = 0;
    let mut stack = vec![("COM".to_string(), 0)];
    while let Some((object, count)) = stack.pop() {
        indirect_counts += count;
        if let Some(objects) = orbiters.get(&object) {
            for orbiter in objects {
                stack.push((orbiter.clone(), count + 1));
            }
        }
    }

    indirect_counts
}

fn get_chain(object: &str, orbits: &HashMap<String, String>) -> Vec<String> {
    let mut position = object.to_string();
    let mut chain = vec![];
    while let Some(object) = orbits.get(&position) {
        chain.push(object.clone());
        position = object.clone();
    }
    chain.reverse();
    chain
}

fn common_prefix<T: Eq>(left: impl Iterator<Item=T>, right: impl Iterator<Item=T>) -> impl Iterator<Item=T> {
    left.zip(right).take_while(|(x, y)| x == y).map(|(x, _)| x)
}

fn orbit_transfers(orbits: Vec<Orbit>, from: String, to: String) -> usize {
    let orbits: HashMap<_, _> = orbits.into_iter().map(|orbit| (orbit.object, orbit.centre)).collect();
    let from_chain = get_chain(&from, &orbits);
    let to_chain = get_chain(&to, &orbits);
    from_chain.len() + to_chain.len() - 2 * common_prefix(from_chain.iter(), to_chain.iter()).count()
}

fn main() {
    let orbits: Vec<Orbit> = stdin().lock().lines().map(|line| line.unwrap().parse().unwrap()).collect();
    println!("{}", orbit_transfers(orbits, "YOU".to_string(), "SAN".to_string()));
}

#[test]
fn count_orbits() {
    let orbits = vec![
        Orbit { centre: "COM".to_string(), object: "B".to_string() },
        Orbit { centre: "B".to_string(), object: "C".to_string() },
        Orbit { centre: "C".to_string(), object: "D".to_string() },
        Orbit { centre: "D".to_string(), object: "E".to_string() },
        Orbit { centre: "E".to_string(), object: "F".to_string() },
        Orbit { centre: "B".to_string(), object: "G".to_string() },
        Orbit { centre: "G".to_string(), object: "H".to_string() },
        Orbit { centre: "D".to_string(), object: "I".to_string() },
        Orbit { centre: "E".to_string(), object: "J".to_string() },
        Orbit { centre: "J".to_string(), object: "K".to_string() },
        Orbit { centre: "K".to_string(), object: "L".to_string() },
    ];
    assert_eq!(total_orbits(orbits.iter()), 42);
}

#[test]
fn transfers() {
    let orbits = vec![
        Orbit { centre: "COM".to_string(), object: "B".to_string() },
        Orbit { centre: "B".to_string(), object: "C".to_string() },
        Orbit { centre: "C".to_string(), object: "D".to_string() },
        Orbit { centre: "D".to_string(), object: "E".to_string() },
        Orbit { centre: "E".to_string(), object: "F".to_string() },
        Orbit { centre: "B".to_string(), object: "G".to_string() },
        Orbit { centre: "G".to_string(), object: "H".to_string() },
        Orbit { centre: "D".to_string(), object: "I".to_string() },
        Orbit { centre: "E".to_string(), object: "J".to_string() },
        Orbit { centre: "J".to_string(), object: "K".to_string() },
        Orbit { centre: "K".to_string(), object: "L".to_string() },
        Orbit { centre: "K".to_string(), object: "YOU".to_string() },
        Orbit { centre: "I".to_string(), object: "SAN".to_string() },
    ];
    assert_eq!(orbit_transfers(orbits, "YOU".to_string(), "SAN".to_string()), 4);
}