use std::collections::{HashMap};
use std::fmt;
use std::io::{stdin, BufRead};
use std::iter::FromIterator;
use std::str::FromStr;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::env;

lazy_static! {
    static ref CHEMICAL_BOOK: ChemicalBook = ChemicalBook::new();
}


struct ChemicalBookData {
    chemicals: HashMap<String, Chemical>,
    names: HashMap<Chemical, String>,
    next_index: usize,
}

struct ChemicalBook {
    data: Mutex<ChemicalBookData>
}

impl ChemicalBook {
    fn new() -> Self {
        ChemicalBook {
            data: Mutex::new(ChemicalBookData {
                chemicals: HashMap::new(),
                names: HashMap::new(),
                next_index: 0,
            })
        }
    }

    fn lookup(&self, name: &str) -> Chemical {
        let mut data = self.data.lock().unwrap();
        if let Some(chemical) = data.chemicals.get(name).cloned() {
            chemical
        } else {
            let chemical = Chemical::new(data.next_index);
            data.next_index += 1;
            data.chemicals.insert(name.to_string(), chemical.clone());
            data.names.insert(chemical.clone(), name.to_string());
            println!("Mapping {} to {}", name, chemical.0);
            chemical
        }
    }

    fn get_name(&self, chemical: &Chemical) -> String {
        let data = self.data.lock().unwrap();
        data.names.get(chemical).cloned().unwrap_or_default()
    }
}

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Chemical(usize);

impl Chemical {
    fn new(index: usize) -> Self {
        assert!(index < 64);
        Chemical(index)
    }
}

impl fmt::Display for Chemical {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", CHEMICAL_BOOK.get_name(self))
    }
}

#[derive(Clone)]
struct Quantity {
    chemical: Chemical,
    quantity: usize,
}

impl Quantity {
    fn new(chemical: Chemical, quantity: usize) -> Self {
        Quantity { chemical, quantity }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.quantity, self.chemical)
    }
}

impl FromStr for Quantity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ' ');
        let quantity = parts.next().unwrap().parse()?;
        let chemical = parts
            .next()
            .map(|name| CHEMICAL_BOOK.lookup(name))
            .ok_or(Error("No chemical".to_string()))?;
        Ok(Quantity { chemical, quantity })
    }
}

#[derive(Clone)]
struct Quantities([usize;64]);

impl FromIterator<Quantity> for Quantities {
    fn from_iter<I: IntoIterator<Item = Quantity>>(quantities: I) -> Self {
        let mut entries = [0;64];

        for quantity in quantities {
            entries[quantity.chemical.0] += quantity.quantity;
        }

        Quantities(entries)
    }
}

impl From<Quantity> for Quantities {
    fn from(quantity: Quantity) -> Self {
        let mut entries = [0;64];
        entries[quantity.chemical.0] = quantity.quantity;
        Quantities(entries)
    }
}

impl fmt::Display for Quantities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for quantity in self.iter() {
            if !first {
                write!(f, ", ")?;
            } else {
                first = false;
            }

            write!(f, "{}", quantity)?;
        }

        Ok(())
    }
}

impl Quantities {
    fn get(&self, chemical: &Chemical) -> usize {
        self.0[chemical.0]
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = Quantity> + 'a {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, quantity)| **quantity > 0)
            .map(|(index, quantity)| Quantity::new(Chemical::new(index), *quantity))
    }

    fn only_contains(&self, chemical: &Chemical) -> bool {
        self.0[chemical.0] > 0 && (0..self.0.len()).all(|index| index == chemical.0 || self.0[index] == 0)
    }

    fn before(&self, reaction: &Reaction) -> (Self, usize) {
        let mut entries = self.0.clone();

        let num_needed = entries[reaction.output.chemical.0];
        entries[reaction.output.chemical.0] = 0;
        let num_reactions = reaction.num_required(num_needed);
        let extra = num_reactions * reaction.output.quantity - num_needed;

        for quantity in reaction.input.iter() {
            entries[quantity.chemical.0] += quantity.quantity * num_reactions;
        }

        (Quantities(entries), extra)
    }

    fn apply(&mut self, reaction: &Reaction) {
        for quantity in reaction.input.iter() {
            self.remove(&quantity)
        }
        self.add(&reaction.output);
    }

    fn add(&mut self, quantity: &Quantity) {
        self.0[quantity.chemical.0] += quantity.quantity;
    }

    fn remove(&mut self, quantity: &Quantity) {
        self.0[quantity.chemical.0] -= quantity.quantity;
    }
}

#[derive(Clone)]
struct Reaction {
    input: Quantities,
    output: Quantity,
}

impl Reaction {
    fn num_required(&self, quantity: usize) -> usize {
        let mut num_reactions = quantity / self.output.quantity;
        if quantity % self.output.quantity > 0 {
            num_reactions += 1;
        }
        num_reactions
    }
    
    fn can_apply(&self, chemicals: &Quantities) -> bool {
        self.input.iter().all(|quantity| chemicals.get(&quantity.chemical) >= quantity.quantity)
    }
}

impl fmt::Display for Reaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} => {}", self.input, self.output)
    }
}

impl FromStr for Reaction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, "=>");
        let input: Quantities = parts
            .next()
            .unwrap()
            .split(",")
            .map(str::trim)
            .map(Quantity::from_str)
            .collect::<Result<Quantities, _>>()?;
        let output = parts
            .next()
            .ok_or(Error("No output".to_string()))
            .map(str::trim)
            .and_then(Quantity::from_str)?;
        Ok(Reaction { input, output })
    }
}

struct Reactions(HashMap<Chemical, Reaction>);

impl FromIterator<Reaction> for Reactions {
    fn from_iter<I: IntoIterator<Item = Reaction>>(reactions: I) -> Self {
        let mut map: HashMap<Chemical, Reaction> = HashMap::new();

        for reaction in reactions {
            map.insert(reaction.output.chemical.clone(), reaction);
        }

        Reactions(map)
    }
}

impl Reactions {

    fn next_reaction(&self, chemicals: &Quantities, desired: &Chemical) -> Option<&Reaction> {
        let mut next_chemical = *desired;
        while let Some(reaction) = self.0.get(&next_chemical) {
            let input_needed = reaction.input.iter().filter(|quantity| {
                chemicals.get(&quantity.chemical) < quantity.quantity
            }).next();

            if let Some(quantity) = input_needed {
                next_chemical = quantity.chemical; 
            } else {
                return Some(reaction);
            }
        }

        None
    }

    fn to_get(&self, from: Chemical, to: Chemical, amount: usize) -> Option<usize> {

        let mut reactions = self.0.clone();

        let mut chemicals: Quantities = Quantity::new(to.clone(), amount).into();
        let mut output = chemicals.clone();

        while !chemicals.only_contains(&from) {
            let chemical = reactions
                .keys()
                .filter(|chemical| {
                    !reactions
                        .values()
                        .any(|reaction| reaction.input.get(&chemical) > 0)
                })
                .next()
                .cloned();

            let chemical = if let Some(chemical) = chemical {
                chemical
            } else {
                return None;
            };

            let reaction = reactions.remove(&chemical).unwrap();
            let (before, extra) = chemicals.before(&reaction);
            chemicals = before;
            output.add(&Quantity::new(chemical, extra));
        }

        Some(chemicals.get(&from))
    }

    fn can_get(&self, from: &str, amount: usize, to: &str) -> usize {
        let from = CHEMICAL_BOOK.lookup(from);
        let to = CHEMICAL_BOOK.lookup(to);
        
        let mut output = 1;
        while self.to_get(from, to, output).unwrap() < amount {
            output *= 2;
        }

        let mut lower = output / 2;
        let mut higher = output;

        while higher > lower + 1 {
            let middle = (higher + lower) / 2;
            if self.to_get(from, to, middle).unwrap() <= amount {
                lower = middle;
            } else {
                higher = middle;
            }
        }

        lower
    }

    fn num_obtained(&self, from: &str, amount: usize, to: &str) -> usize {
        let from = CHEMICAL_BOOK.lookup(from);
        let to = CHEMICAL_BOOK.lookup(to);

        let mut chemicals: Quantities = Quantity::new(from.clone(), amount).into();
        
        let mut index = 0;
        println!("Applying reactions to remainder");
        while let Some(reaction) = self.next_reaction(&chemicals, &to) {
            chemicals.apply(reaction);
            index += 1;
            if index % 10_000_000 == 0 {
                println!("{}: {} -> {}", index, chemicals.get(&from), chemicals.get(&to));
            }
        }
    
        chemicals.get(&to)
    }
}

fn read_reactions(input: &mut impl BufRead) -> Result<Reactions, Error> {
    input
        .lines()
        .map(Result::unwrap)
        .map(|line| line.trim().parse())
        .collect()
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let reactions = read_reactions(&mut stdin().lock()).unwrap();

    let num_obtained = reactions.can_get("ORE", args[1].parse().unwrap(), "FUEL");
    println!("{}", num_obtained);
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_1() {
        let input = r#"157 ORE => 5 NZVS
        165 ORE => 6 DCFZ
        44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
        12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
        179 ORE => 7 PSHF
        177 ORE => 5 HKGWZ
        7 DCFZ, 7 PSHF => 2 XJWVT
        165 ORE => 2 GPVTF
        3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT"#;

        let mut reader = BufReader::new(input.as_bytes());
        let reactions = read_reactions(&mut reader).unwrap();

        let num_obtained = reactions.num_obtained("ORE", 1000000000000, "FUEL");   
        assert_eq!(num_obtained, 82892753);
    }
}