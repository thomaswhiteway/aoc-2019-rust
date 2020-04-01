use std::io::{stdin, stdout, Write};
use std::env;
use std::fs;
use std::char;
use std::cell::RefCell;

mod display;
mod process;
mod program;
mod utils;

use process::{Input, Output, Process, State};
use program::Program;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}


impl<T: Write> Output<i64> for RefCell<T> {
    fn put(&self, value: i64) {
        if let Some(c) = char::from_u32(value as u32) {
            write!(self.borrow_mut(), "{}", c).unwrap();
        } else {
            write!(self.borrow_mut(), "Damage: {}\n", value).unwrap();
        }
    }
}

impl Input<i64> for RefCell<String> {
    fn get(&self) -> Option<i64> {
        if self.borrow().len() > 0 {
            Some(self.borrow_mut().remove(0) as i64)
        } else {
            None
        }
    }
}

fn run(program: &Program, code: String) {
    let mut process = Process::new("springdroid", program, RefCell::new(code), RefCell::new(stdout()));

    let state = process.execute();
    assert_eq!(state, State::Complete);
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let code = fs::read_to_string(args[1].clone()).unwrap();

    let program = Program::parse(stdin()).unwrap();

    run(&program, code);
}
