use std::io::stdin;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

mod display;
mod process;
mod program;
mod utils;

use process::{Input, Output, Process, run_to_completion};
use program::Program;
use std::rc::Rc;

#[derive(Debug)]
struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

struct Nat {
    buffer: Cell<Option<(i64, i64)>>,   
    last_sent: Cell<Option<(i64, i64)>>, 
}

impl Nat {
    fn new() -> Self {
        Nat {
            buffer: Cell::new(None),
            last_sent: Cell::new(None)
        }
    }

    fn receive(&self, x: i64, y: i64) {
        self.buffer.set(Some((x, y)));
    }

    fn poll(&self, router: &Router) {
        if router.network_idle() {
            if let Some((x, y)) = self.buffer.get() {
                router.send(0, x, y);
                
                if let Some(last_packet) = self.last_sent.get() {
                    if last_packet == (x, y) {
                        println!("Duplicate Packet: ({}, {})", x, y)
                    }
                }
                self.last_sent.set(Some((x, y)));
            }
        }
    }
}

struct Router {
    interfaces: RefCell<Vec<Rc<Nic>>>,
    nat: Nat
}

impl Router {
    fn new() -> Rc<Self> {
        Rc::new(Router {
            interfaces: RefCell::new(vec![]),
            nat: Nat::new()
        })
    }

    fn new_interface(self: Rc<Self>) -> Rc<Nic> {
        let mut interfaces = self.interfaces.borrow_mut();
        let interface = Rc::new(Nic::new(self.clone(), interfaces.len() as i64));
        interfaces.push(interface.clone());
        interface
    }

    fn send(&self, destination: usize, x: i64, y: i64) {
        let interfaces = self.interfaces.borrow();
        if destination == 255 {
            self.nat.receive(x, y);
        } else {
            interfaces[destination].receive(x, y);
        }
    }

    fn network_idle(&self) -> bool {
        self.interfaces.borrow().iter().all(|interface| interface.is_idle())
    }

    fn poll(&self) {
        self.nat.poll(&self)
    }
}

struct Nic {
    index: i64,
    got_index: Cell<bool>,
    got_input: Cell<bool>,
    router: Rc<Router>,
    input_buffer: RefCell<VecDeque<i64>>,
    output_buffer: RefCell<Vec<i64>>,
}

impl Nic {
    fn new(router: Rc<Router>, index: i64) -> Self {
        Nic {
            index,
            got_index: Cell::new(false),
            got_input: Cell::new(true),
            router,
            input_buffer: RefCell::new(VecDeque::new()),
            output_buffer: RefCell::new(vec![]),
        }
    }

    fn is_idle(&self) -> bool {
        self.input_buffer.borrow().is_empty() && self.output_buffer.borrow().is_empty() && !self.got_input.get()
    }

    fn receive(&self, x: i64, y: i64) {
        let mut buffer = self.input_buffer.borrow_mut();
        buffer.push_back(x);
        buffer.push_back(y);
    }
}

impl Input<i64> for Rc<Nic> {
    fn get(&self) -> Option<i64> {
        let mut buffer = self.input_buffer.borrow_mut();
        if !self.got_index.get() {
            self.got_index.set(true);
            Some(self.index)
        } else if let Some(value) = buffer.pop_front() {
            self.got_input.set(true);
            Some(value)
        } else {
            self.got_input.set(false);
            Some(-1)
        }
    }
}

impl Output<i64> for Rc<Nic> {
    fn put(&self, value: i64) {
        let mut buffer = self.output_buffer.borrow_mut();
        buffer.push(value);

        if buffer.len() == 3 {
            let destination = buffer[0] as usize;
            let x = buffer[1];
            let y = buffer[2];
            self.router.send(destination, x, y);

            buffer.clear();
        }
    }
}

fn run(program: &Program) {
    let router = Router::new();
    let mut processes: Vec<_> = (0..50).map(|index| {
        let nic = router.clone().new_interface();
        Process::new(format!("Computer {}", index), program, nic.clone(), nic)
    }).collect();

    run_to_completion(processes.iter_mut().collect(), || router.poll());
}

fn main() {
    let program = Program::parse(stdin()).unwrap();
    run(&program);
}
