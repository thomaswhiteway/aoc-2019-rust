use std::io::stdin;
use std::ops::Range;

mod display;
mod process;
mod program;
mod utils;

use process::{Channel, Input, Output, Process, State};
use program::Program;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

fn within_beam(program: &Program, x: usize, y: usize) -> bool {
    let input = Channel::new();
    let output = Channel::new();

    let mut process = Process::new("Probe", program, &input, &output);

    input.put(x as i64);
    input.put(y as i64);

    let state = process.execute();
    assert_eq!(state, State::Complete);

    output.get().unwrap() == 1
}

fn first_pulled(program: &Program, y: usize) -> usize {
    for x in 0.. {
        if within_beam(program, x, y) {
            return x
        }
    }
    unreachable!();
}

fn can_fit(program: &Program, y: usize, side: usize) -> bool {
    let x = first_pulled(program, y);
    within_beam(program, x + side - 1, y + 1 - side)
}

fn closest_fit(program: &Program, side: usize) -> (usize, usize) {
    let mut y = side - 1;
    while !can_fit(program, y, side) {
        y *= 2;
    }

    let mut lower = y / 2;
    let mut upper = y;

    while upper - lower > 1 {
        let middle = (upper + lower) / 2;

        if can_fit(program, middle, side) {
            upper = middle;
        } else {
            lower = middle;
        }
    }

    (first_pulled(program, upper), upper - side + 1)
}

fn display_area(program: &Program, ship_x_range: Range<usize>, ship_y_range: Range<usize>, x_range: Range<usize>, y_range: Range<usize>) {
    for y in y_range {
        for x in x_range.clone() {
            if ship_x_range.contains(&x) && ship_y_range.contains(&y) {
                print!("O");
            } else if within_beam(program, x, y) {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!("");
    }
}

fn main() {
    let program = Program::parse(stdin()).unwrap();

    let (x, y) = closest_fit(&program, 100);

    if !within_beam(&program, x, y) {
        panic!("{}, {} outside beam", x, y);
    }

    if !within_beam(&program, x+99, y) {
        panic!("{}, {} outside beam", x, y);
    }

    if !within_beam(&program, x+99, y+99) {
        panic!("{}, {} outside beam", x, y);
    }

    if !within_beam(&program, x, y+99) {
        panic!("{}, {} outside beam", x, y);
    }

    display_area(&program, x..x+100, y..y+100, x-2..x+102, y-2..y+102);

    println!("{}", x*10_000 + y);
}
