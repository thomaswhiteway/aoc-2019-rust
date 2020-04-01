use std::io::stdin;

mod process;
mod program;

use process::{Channel, Output, Process, State};
use program::Program;

fn run_test_program(program: &Program, value: i64) -> Vec<i64> {
    let input = Channel::new();
    let output = Channel::new();

    input.put(value);

    let state = Process::new("test".to_string(), program, &input, &output).execute();

    assert_eq!(state, State::Complete);

    output.into()
}

fn main() {
    let program = Program::parse(stdin()).unwrap();

    let output = run_test_program(&program, 2);

    for value in output {
        println!("{}", value);
    }
}
