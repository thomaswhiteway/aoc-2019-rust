use std::io::{stdin, Read};
use std::str::FromStr;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Debug)]
enum Mode {
    Position,
    Immediate
}

struct Modes(i64);

impl Modes {
    fn mode(&self, index: usize) -> Result<Mode, String> {
        let mode = (self.0 / (10 as i64).pow(index as u32)) % (10 as i64).pow(index as u32 + 1);
        match mode {
            0 => Ok(Mode::Position),
            1 => Ok(Mode::Immediate),
            _ => Err(format!("Unknown mode {}", mode)),
        }
    }
}

#[derive(Debug)]
struct Parameter {
    mode: Mode,
    value: i64,
}

impl Parameter {
    fn resolve(&self, memory: &[i64]) -> i64 {
        use Mode::*;
        match self.mode {
            Position => memory[self.value as usize],
            Immediate => self.value
        }
    }
}

struct Parameters<'a> {
    data: &'a [i64],
    modes: Modes
}

impl<'a> Parameters<'a> {
    fn new(data: &'a [i64], modes: i64) -> Self {
        Parameters {
            data,
            modes: Modes(modes)
        }
    }

    fn get(&self, index: usize) -> Parameter {
        Parameter {
            mode: self.modes.mode(index).unwrap(),
            value: self.data[index]
        }
    }

    fn get_address(&self, index: usize) -> usize {
        self.data[index] as usize
    }
}

#[derive(Debug)]
enum Instruction {
    Add { x: Parameter, y: Parameter, output: usize },
    Mul { x: Parameter, y: Parameter, output: usize },
    Input { output: usize },
    Output { input: Parameter },
    JumpIfTrue { input: Parameter, address: Parameter },
    JumpIfFalse { input: Parameter, address: Parameter },
    LessThan { x: Parameter, y: Parameter, output: usize },
    Equals { x: Parameter, y: Parameter, output: usize },
    Exit,
}

impl Instruction {
    fn parse(data: &[i64]) -> Result<Self, String> {
        use Instruction::*;
        let opcode = data[0] % 100;
        let parameters = Parameters::new(&data[1..],  data[0] / 100);
        match opcode {
            1 => Ok(Add {
                x: parameters.get(0),
                y: parameters.get(1),
                output: parameters.get_address(2),
            }),
            2 => Ok(Mul {
                x: parameters.get(0),
                y: parameters.get(1),
                output: parameters.get_address(2),
            }),
            3 => Ok(Input { 
                output: parameters.get_address(0)
            }),
            4 => Ok(Output {
                input: parameters.get(0)
            }),
            5 => Ok(JumpIfTrue {
                input: parameters.get(0),
                address: parameters.get(1),
            }),
            6 => Ok(JumpIfFalse {
                input: parameters.get(0),
                address: parameters.get(1),
            }),
            7 => Ok(LessThan {
                x: parameters.get(0),
                y: parameters.get(1),
                output: parameters.get_address(2),
            }), 
            8 => Ok(Equals {
                x: parameters.get(0),
                y: parameters.get(1),
                output: parameters.get_address(2),
            }),
            99 => Ok(Exit),
            _ => Err(format!("Unknown opcode {}", opcode)),
        }
    }

    fn size(&self) -> usize {
        use Instruction::*;
        match self {
            Add {..} | Mul {..} | LessThan {..} | Equals {..} => 4,
            JumpIfTrue {..} | JumpIfFalse {..} => 3,
            Input {..} | Output {..} => 2,
            Exit => 1,
        }
    }
}

struct Program {
    data: Box<[i64]>,
}

impl Program {
    fn parse(mut input: impl Read) -> Result<Self, Error> {
        let mut data_string = String::new();
        input.read_to_string(&mut data_string)?;
        let data = data_string
            .split(',')
            .map(str::trim)
            .map(i64::from_str)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(Program { data })
    }
}

trait Input {
    fn get(&mut self) -> i64;
}

trait Output {
    fn put(&mut self, value: i64);
}

impl Input for &mut i64 {
    fn get(&mut self) -> i64 {
        **self
    }
}

impl Output for &mut i64 {
    fn put(&mut self, value: i64) {
        **self = value
    }
}

struct Process<I, O> {
    memory: Box<[i64]>,
    instruction_pointer: usize,
    input: I,
    output: O,
}

impl<I: Input, O: Output> Process<I, O> {
    fn new(program: &Program, input: I, output: O) -> Self {
        Process {
            memory: program.data.clone(),
            instruction_pointer: 0,
            input,
            output
        }
    }

    fn next_instruction(&mut self) -> Instruction {
        let instruction = Instruction::parse(&self.memory[self.instruction_pointer..]).unwrap();
        self.instruction_pointer += instruction.size();
        instruction
    }

    fn execute(mut self) {
        loop {
            match self.next_instruction() {
                Instruction::Add { x, y, output } => {
                    self.memory[output] = x.resolve(&self.memory) + y.resolve(&self.memory)
                }
                Instruction::Mul { x, y, output } => {
                    self.memory[output] = x.resolve(&self.memory) * y.resolve(&self.memory)
                }
                Instruction::Input { output } => {
                    self.memory[output] = self.input.get()
                }
                Instruction::Output { input } => {
                    self.output.put(input.resolve(&self.memory))
                }
                Instruction::JumpIfTrue { input, address } => {
                    if input.resolve(&self.memory) != 0 {
                        self.instruction_pointer = address.resolve(&self.memory) as usize
                    }
                }
                Instruction::JumpIfFalse { input, address } => {
                    if input.resolve(&self.memory) == 0 {
                        self.instruction_pointer = address.resolve(&self.memory) as usize
                    }
                }
                Instruction::LessThan { x, y, output } => {
                    self.memory[output] = if x.resolve(&self.memory) < y.resolve(&self.memory) {
                        1
                    } else {
                        0
                    }
                }
                Instruction::Equals { x, y, output } => {
                    self.memory[output] = if x.resolve(&self.memory) == y.resolve(&self.memory) {
                        1
                    } else {
                        0
                    }
                }
                Instruction::Exit => break,
            }
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let program = Program::parse(stdin()).unwrap();

    let mut input = args[1].trim().parse().unwrap();
    let mut output = 0;

    Process::new(&program, &mut input, &mut output).execute();

    println!("{}", output);
}

#[test]
fn jump_position_zero() {
    let program = Program {
        data : vec![
            3,12,
            6,12,15,
            1,13,14,13,
            4,13,
            99,
            -1,0,1,9
        ].into_boxed_slice()
    };

    let mut input = 0;
    let mut output = 0;

    Process::new(&program, &mut input, &mut output).execute();

    assert_eq!(output, 0);
}

#[test]
fn jump_position_nonzero() {
    let program = Program {
        data : vec![
            3,12,
            6,12,15,
            1,13,14,13,
            4,13,
            99,
            -1,0,1,9
        ].into_boxed_slice()
    };

    let mut input = 1;
    let mut output = 0;

    Process::new(&program, &mut input, &mut output).execute();

    assert_eq!(output, 1);
}

#[test]
fn jump_immediate_zero() {
    let program = Program {
        data : vec![
            3,3,
            1105,-1,9,
            1101,0,0,12,
            4,12,
            99,
            1
        ].into_boxed_slice()
    };

    let mut input = 0;
    let mut output = 0;

    Process::new(&program, &mut input, &mut output).execute();

    assert_eq!(output, 0);
}

#[test]
fn jump_immediate_nonzero() {
    let program = Program {
        data : vec![
            3,3,
            1105,-1,9,
            1101,0,0,12,
            4,12,
            99,
            1
        ].into_boxed_slice()
    };

    let mut input = 1;
    let mut output = 0;

    Process::new(&program, &mut input, &mut output).execute();

    assert_eq!(output, 1);
}