use super::program::Program;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Complete,
    Blocked,
}

#[derive(Debug)]
enum Mode {
    Position,
    Immediate,
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
            Immediate => self.value,
        }
    }
}

struct Parameters<'a> {
    data: &'a [i64],
    modes: Modes,
}

impl<'a> Parameters<'a> {
    fn new(data: &'a [i64], modes: i64) -> Self {
        Parameters {
            data,
            modes: Modes(modes),
        }
    }

    fn get(&self, index: usize) -> Parameter {
        Parameter {
            mode: self.modes.mode(index).unwrap(),
            value: self.data[index],
        }
    }

    fn get_address(&self, index: usize) -> usize {
        self.data[index] as usize
    }
}

#[derive(Debug)]
enum Instruction {
    Add {
        x: Parameter,
        y: Parameter,
        output: usize,
    },
    Mul {
        x: Parameter,
        y: Parameter,
        output: usize,
    },
    Input {
        output: usize,
    },
    Output {
        input: Parameter,
    },
    JumpIfTrue {
        input: Parameter,
        address: Parameter,
    },
    JumpIfFalse {
        input: Parameter,
        address: Parameter,
    },
    LessThan {
        x: Parameter,
        y: Parameter,
        output: usize,
    },
    Equals {
        x: Parameter,
        y: Parameter,
        output: usize,
    },
    Exit,
}

impl Instruction {
    fn parse(data: &[i64]) -> Result<Self, String> {
        use Instruction::*;
        let opcode = data[0] % 100;
        let parameters = Parameters::new(&data[1..], data[0] / 100);
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
                output: parameters.get_address(0),
            }),
            4 => Ok(Output {
                input: parameters.get(0),
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
            Add { .. } | Mul { .. } | LessThan { .. } | Equals { .. } => 4,
            JumpIfTrue { .. } | JumpIfFalse { .. } => 3,
            Input { .. } | Output { .. } => 2,
            Exit => 1,
        }
    }
}

pub trait Input<T> {
    fn get(&self) -> Option<T>;
}

pub trait Output<T> {
    fn put(&self, value: T);
}

pub struct Channel<T> {
    buffer: RefCell<Vec<T>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Channel {
            buffer: RefCell::new(vec![]),
        }
    }
}

impl<T, I: Input<T>> Input<T> for &I {
    fn get(&self) -> Option<T> {
        (*self).get()
    }
}

impl<T, O: Output<T>> Output<T> for &O {
    fn put(&self, value: T) {
        (*self).put(value)
    }
}

impl<T> Input<T> for Channel<T> {
    fn get(&self) -> Option<T> {
        let mut buffer = self.buffer.borrow_mut();
        if !buffer.is_empty() {
            Some(buffer.remove(0))
        } else {
            None
        }
    }
}

impl<T> Output<T> for Channel<T> {
    fn put(&self, value: T) {
        self.buffer.borrow_mut().push(value)
    }
}

pub struct Process<I, O> {
    #[allow(dead_code)]
    name: String,
    memory: Box<[i64]>,
    instruction_pointer: usize,
    input: I,
    output: O,
}

impl<I: Input<i64>, O: Output<i64>> Process<I, O> {
    pub fn new(name: String, program: &Program, input: I, output: O) -> Self {
        Process {
            name,
            memory: program.data.clone(),
            instruction_pointer: 0,
            input,
            output,
        }
    }

    fn next_instruction(&mut self) -> Instruction {
        let instruction = Instruction::parse(&self.memory[self.instruction_pointer..]).unwrap();
        self.instruction_pointer += instruction.size();
        instruction
    }

    pub fn execute(&mut self) -> State {
        loop {
            let instruction = self.next_instruction();
            match instruction {
                Instruction::Add { x, y, output } => {
                    self.memory[output] = x.resolve(&self.memory) + y.resolve(&self.memory)
                }
                Instruction::Mul { x, y, output } => {
                    self.memory[output] = x.resolve(&self.memory) * y.resolve(&self.memory)
                }
                Instruction::Input { output } => {
                    if let Some(input) = self.input.get() {
                        self.memory[output] = input
                    } else {
                        self.instruction_pointer -= instruction.size();
                        return State::Blocked;
                    }
                }
                Instruction::Output { input } => self.output.put(input.resolve(&self.memory)),
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
                Instruction::Exit => return State::Complete,
            }
        }
    }
}

pub fn run_to_completion<I, O>(mut processes: Vec<&mut Process<I, O>>)
where
    I: Input<i64>,
    O: Output<i64>,
{
    while !processes.is_empty() {
        let mut remaining_processes = vec![];
        for process in processes {
            if process.execute() != State::Complete {
                remaining_processes.push(process);
            }
        }
        processes = remaining_processes;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn jump_position_zero() {
        let program = Program {
            data: vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9].into_boxed_slice(),
        };

        let input = Channel::new();
        let output = Channel::new();

        input.put(0);

        Process::new("TEST".to_string(), &program, &input, &output).execute();

        assert_eq!(output.get(), Some(0));
    }

    #[test]
    fn jump_position_nonzero() {
        let program = Program {
            data: vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9].into_boxed_slice(),
        };

        let input = Channel::new();
        let output = Channel::new();

        input.put(1);

        Process::new("TEST".to_string(), &program, &input, &output).execute();

        assert_eq!(output.get(), Some(1));
    }

    #[test]
    fn jump_immediate_zero() {
        let program = Program {
            data: vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1].into_boxed_slice(),
        };

        let input = Channel::new();
        let output = Channel::new();

        input.put(0);

        Process::new("TEST".to_string(), &program, &input, &output).execute();

        assert_eq!(output.get(), Some(0));
    }

    #[test]
    fn jump_immediate_nonzero() {
        let program = Program {
            data: vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1].into_boxed_slice(),
        };

        let input = Channel::new();
        let output = Channel::new();

        input.put(1);

        Process::new("TEST".to_string(), &program, &input, &output).execute();

        assert_eq!(output.get(), Some(1));
    }
}
