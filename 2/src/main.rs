use std::io::{stdin, Read};
use std::str::FromStr;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

enum Instruction {
    Add { x: usize, y: usize, result: usize },
    Mul { x: usize, y: usize, result: usize },
    Exit,
}

impl Instruction {
    fn parse(data: &[usize]) -> Option<Self> {
        match data[0] {
            1 => Some(Instruction::Add {
                x: data[1],
                y: data[2],
                result: data[3],
            }),
            2 => Some(Instruction::Mul {
                x: data[1],
                y: data[2],
                result: data[3],
            }),
            99 => Some(Instruction::Exit),
            _ => None,
        }
    }

    fn size(&self) -> usize {
        4
    }
}

struct Program {
    data: Box<[usize]>,
}

impl Program {
    fn parse(mut input: impl Read) -> Result<Self, Error> {
        let mut data_string = String::new();
        input.read_to_string(&mut data_string)?;
        let data = data_string
            .split(',')
            .map(str::trim)
            .map(usize::from_str)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(Program { data })
    }
}

struct Process {
    memory: Box<[usize]>,
    instruction_pointer: usize,
}

impl Process {
    fn new(program: &Program) -> Self {
        Process {
            memory: program.data.clone(),
            instruction_pointer: 0,
        }
    }

    fn next_instruction(&mut self) -> Option<Instruction> {
        let result = Instruction::parse(&self.memory[self.instruction_pointer..]);
        if let Some(ref instruction) = result {
            self.instruction_pointer += instruction.size();
        }
        result
    }

    fn execute(mut self, noun: usize, verb: usize) -> usize {
        self.memory[1] = noun;
        self.memory[2] = verb;

        loop {
            match self.next_instruction() {
                Some(Instruction::Add { x, y, result }) => {
                    self.memory[result] = self.memory[x] + self.memory[y]
                }
                Some(Instruction::Mul { x, y, result }) => {
                    self.memory[result] = self.memory[x] * self.memory[y]
                }
                _ => break,
            }
        }

        self.memory[0]
    }
}

fn find_result(program: &Program, expected_result: usize) -> Option<(usize, usize)> {
    for noun in 0..=99 {
        for verb in 0..=99 {
            let process = Process::new(program);
            let result = process.execute(noun, verb);
            if result == expected_result {
                return Some((noun, verb));
            }
        }
    }
    None
}

fn main() {
    let program = Program::parse(stdin()).unwrap();

    if let Some((noun, verb)) = find_result(&program, 19_690_720) {
        println!("{}", 100 * noun + verb);
    } else {
        println!("Not possible")
    }
}
