use itertools::iproduct;
use std::io::stdin;

mod process;
mod program;

use process::{run_to_completion, Channel, Input, Output, Process};
use program::Program;

fn get_output_signal(program: &Program, phase_settings: &[i64]) -> i64 {
    let channels: Vec<_> = phase_settings
        .iter()
        .map(|_| Channel::new())
        .collect();
    let mut processes: Vec<_> = channels
        .iter()
        .zip(channels.iter().cycle().skip(1))
        .enumerate()
        .map(|(index, (input, output))| Process::new(format!("Amplifier {}", index), program, input, output))
        .collect();

    for (channel, setting) in channels.iter().zip(phase_settings) {
        channel.put(*setting)
    }

    channels[0].put(0);

    run_to_completion(processes.iter_mut().collect());

    channels[0].get().unwrap()
}

fn find_max_output_signal(program: &Program) -> i64 {
    iproduct!(5..10, 5..10, 5..10, 5..10, 5..10)
        .map(|(a, b, c, d, e)| vec![a, b, c, d, e])
        .filter(|settings| (5..10).all(|x| settings.contains(&x)))
        .map(|settings| get_output_signal(program, &settings))
        .max()
        .unwrap()
}

fn main() {
    let program = Program::parse(stdin()).unwrap();

    let max_signal = find_max_output_signal(&program);

    println!("{}", max_signal);
}

#[test]
fn output_signal_1() {
    let program = Program {
        data: vec![
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ]
        .into_boxed_slice(),
    };

    assert_eq!(get_output_signal(&program, &vec![9, 8, 7, 6, 5]), 139629729);
    assert_eq!(find_max_output_signal(&program), 139629729);
}

#[test]
fn output_signal_2() {
    let program = Program {
        data: vec![
            3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
            -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
            53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
        ]
        .into_boxed_slice(),
    };

    assert_eq!(get_output_signal(&program, &vec![9, 7, 8, 5, 6]), 18216);
    assert_eq!(find_max_output_signal(&program), 18216);
}
