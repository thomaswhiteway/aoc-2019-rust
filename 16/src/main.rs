use std::io::stdin;
use std::env;
use std::iter::repeat;

fn get_num_phases() -> usize {
    let args: Vec<_> = env::args().collect();
    args[1].parse().unwrap()
}

fn read_sequence() -> Box<[i32]> {
    let mut sequence = String::new();
    stdin().read_line(&mut sequence).unwrap();
    sequence.trim().chars().map(|c| c.to_digit(10).unwrap() as i32).collect::<Vec<_>>().into_boxed_slice()
}

fn step(pattern: &[i32], sequence: &[i32]) -> Box<[i32]> {
    let mut result: Vec<i32> = vec![];

    for index in 0..sequence.len() {
        let pattern = pattern.iter().flat_map(|d| repeat(d).take(index+1)).cycle().skip(1);
        let value: i32 = sequence.iter().zip(pattern).map(|(x, y)| x * y).sum();
        result.push(value.abs() % 10);
    }

    result.into_boxed_slice()
}

fn sequence_string(sequence: &[i32]) -> String {
    sequence.iter().map(|c| c.to_string()).collect()
}

fn value(digits: &[i32]) -> usize {
    Iterator::zip(digits.iter(), (0..digits.len()).rev()).map(|(d, e)| *d as usize * (10 as usize).pow(e as u32)).sum::<usize>()
}

fn get_offset(sequence: &[i32], offset: usize, phases: usize) -> Box<[i32]> {
    let mut full_sequence: Vec<i32> = sequence.iter().cycle().take(10_000 * sequence.len()).skip(offset).cloned().collect();

    assert!(offset > 5_000 * sequence.len());

    for _ in 0..phases {
        let mut total = 0;
        for index in (0..full_sequence.len()).rev() {
            total += full_sequence[index];
            full_sequence[index] = total % 10;
        }
    }

    full_sequence.truncate(8);
    full_sequence.into_boxed_slice()
}

fn main() {
    let mut sequence = read_sequence();
    let offset: usize = value(&sequence[..7]);
    let phases = get_num_phases();

    let result = get_offset(&sequence, offset, phases);

    println!("{}", &sequence_string(&result));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn short() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let output = step(&[0, 1, 0, -1], &input);

        assert_eq!(&*output, &[4, 8, 2, 2, 6, 1, 5, 8]);

        let output = step(&[0, 1, 0, -1], &output);

        assert_eq!(&*output, &[3, 4, 0, 4, 0, 4, 3, 8]);
    }
}