use std::str::FromStr;
use std::io::{BufRead, stdin};
use std::fmt;

const DECK_SIZE: u128 = 119315717514047;


fn inverse(x: u128, m: u128) -> Option<u128> {
    let mut t = 0 as i128;
    let mut r = m as i128;
    let mut new_t = 1 as i128;
    let mut new_r = x as i128;

    while new_r != 0 {
        let quotient = r / new_r;
        let tmp_t = t - quotient * new_t;
        t = new_t;
        new_t = tmp_t;
        let tmp_r = r - quotient * new_r;
        r = new_r;
        new_r = tmp_r;
    }

    if r > 1 {
        None
    } else if t < 0 {
        Some((t + m as i128) as u128)
    } else {
        Some(t as u128)
    }
}

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Invert,
    Sub(u128),
    Add(u128),
    Mul(u128),
}

impl Operation {
    fn to_term(self, input: Box<Term>) -> Box<Term> {
        Box::new(match self {
            Operation::Invert => Term::Mul(input, Box::new(Term::Value(DECK_SIZE - 1))),
            Operation::Sub(x) => Term::Add(input, Box::new(Term::Mul(Box::new(Term::Value(x)), Box::new(Term::Value(DECK_SIZE - 1))))),
            Operation::Add(x) => Term::Add(input, Box::new(Term::Value(x))),
            Operation::Mul(x) => Term::Mul(input, Box::new(Term::Value(x))),
        })
    }
}

#[derive(Debug, Clone)]
enum Term {
    Value(u128),
    Variable(&'static str),
    Add(Box<Term>, Box<Term>),
    Mul(Box<Term>, Box<Term>),
}

impl Term {
    fn normalize(self) -> Box<Self> {
        use Term::*;
        Box::new(match self {
            Value(x) => Value(x),
            Variable(x) => Variable(x),
            Add(x, y) => {
                let x = x.normalize();
                let y = y.normalize();
                
                if let (Value(a), Value(b)) = (&*x, &*y) {
                    Value((a + b) % DECK_SIZE)
                } else if let (Value(a), Value(b)) = (&*x, &*y) {
                    Value((a + b) % DECK_SIZE)
                } else if let (Add(a, b), Value(c)) = (&*x, &*y) {
                    if let Value(d) = &**b {
                        Add(a.clone(), Box::new(Value((c + d) % DECK_SIZE)))
                    } else {
                        Add(x, y)
                    }
                } else {
                    Add(x, y)
                }
            },
            Mul(x, y) => {
                let x = x.normalize();
                let y = y.normalize();

                if let Add(a, b) = *x {
                    Add(Mul(a, y.clone()).normalize(), Mul(b, y).normalize())
                } else if let (Value(a), Value(b)) = (&*x, &*y) {
                    Value((a * b) % DECK_SIZE)
                } else if let (Mul(a, b), Value(c)) = (&*x, &*y) {
                    if let Value(d) = &**b {
                        Mul(a.clone(), Box::new(Value((c * d) % DECK_SIZE)))
                    } else {
                        Mul(x, y)
                    }
                } else {
                    Mul(x, y)
                }
            }
        })
    }

    fn set(self, variable: &str, value: &Term) -> Box<Term> {
        use Term::*;
        match self {
            Value(x) => Value(x),
            Variable(x) if x == variable => value.clone(),
            Variable(x) => Variable(x),
            Add(x, y) => Add(x.set(variable, value), y.set(variable, value)),
            Mul(x, y) => Mul(x.set(variable, value), y.set(variable, value)),
        }.normalize()
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Term::*;
        match self {
            Value(x) => write!(f, "{}", x),
            Variable(x) => write!(f, "{}", x),
            Add(x, y) =>  write!(f, "({} + {})", x, y),
            Mul(x, y) => write!(f, "({} * {})", x, y),
        }
    }
}

fn new_stack_invert() -> Vec<Operation> {
    vec![Operation::Invert, Operation::Sub(1)]
}

fn cut_invert(split: u128) -> Vec<Operation> {
    vec![Operation::Add(split)]
}

fn deal_with_increment_inverse(increment: u128) -> Vec<Operation> {
    vec![Operation::Mul(increment)]
}

enum Technique {
    NewStack,
    Cut(u128),
    DealWithIncrement(u128)
}

impl FromStr for Technique {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "deal into new stack" {
            Ok(Technique::NewStack)
        } else if s.starts_with("cut ") {
            let depth: i128 = s[4..].parse()?;
            let split = if depth > 0 {
                depth as u128
            } else {
                DECK_SIZE - depth.abs() as u128
            };  

            Ok(Technique::Cut(split))
        } else if s.starts_with("deal with increment ") {
            Ok(Technique::DealWithIncrement(inverse(s[20..].parse()?, DECK_SIZE).unwrap()))
        } else {
            Err(format!("Unrecognised technique {}", s).into())
        }
    }
}

impl Technique {
    fn operation(&self) -> Vec<Operation> {
        use Technique::*;
        match *self {
            NewStack => new_stack_invert(),
            Cut(depth) => cut_invert(depth),
            DealWithIncrement(increment) => deal_with_increment_inverse(increment),
        }
    }
}

fn techniques<T: BufRead>(input: T) -> impl Iterator<Item = Technique> {
    input.lines().map(|line| line.unwrap().parse().unwrap())
}

fn main() {
    let mut techniques: Vec<_> =  techniques(stdin().lock()).collect();
    techniques.reverse();

    let mut term = Box::new(Term::Variable("x"));
    for technique in techniques.iter() {
        for operation in technique.operation() {
            term = operation.to_term(term);
        }
    }

    term = term.normalize();

    let mut num_iterations = 101741582076661;

    let mut powers = vec![];

    for index in 0.. {
        powers.push(term.clone());
        
        if (2 as u128).pow(index) > num_iterations {
            break;
        }

        let term_2 = term.clone();
        term = term.set("x", &term_2);
    }

    let mut full_term = Box::new(Term::Variable("x"));

    while num_iterations > 0 {
        let mut exponent = 0;
        while (2 as u128).pow(exponent + 1) < num_iterations {
            exponent += 1;
        } 

        full_term = powers[exponent as usize].clone().set("x", &full_term);
        num_iterations -= (2 as u128).pow(exponent);
    }

    let result = full_term.set("x", &Term::Value(2020));

    println!("{}", result);
}
