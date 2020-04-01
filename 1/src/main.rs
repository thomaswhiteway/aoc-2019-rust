use std::io::{stdin, BufRead};
use std::str::FromStr;

struct Component {
    weight: u32,
}

impl Component {
    fn new(weight: u32) -> Self {
        Component { weight }
    }

    fn fuel_required(&self) -> u32 {
        let mut fuel_added: u32 = fuel_for_weight(self.weight);

        let mut fuel_total = 0;
        while fuel_added > 0 {
            fuel_total += fuel_added;
            fuel_added = fuel_for_weight(fuel_added);
        }

        fuel_total
    }
}

fn get_components(input: impl BufRead) -> impl Iterator<Item = Component> {
    input
        .lines()
        .map(|result| u32::from_str(&result.unwrap()).unwrap())
        .map(Component::new)
}

fn fuel_for_weight(weight: u32) -> u32 {
    (weight / 3).saturating_sub(2)
}

fn main() {
    let fuel: u32 = get_components(stdin().lock())
        .map(|component| component.fuel_required())
        .sum();

    println!("{}", fuel)
}
