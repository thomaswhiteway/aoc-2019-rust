use std::fmt;
use std::io::Write;

use termion::{clear, cursor};

pub trait Screen {
    fn clear(&mut self);
    fn set_tile<T: fmt::Display + fmt::Debug>(&mut self, position: [u16; 2], tile: T);
}

impl<W: Write> Screen for W {
    fn clear(&mut self) {
        let _ = write!(self, "{}", clear::All);
    }

    fn set_tile<T: fmt::Display + fmt::Debug>(&mut self, [x, y]: [u16; 2], tile: T) {
        let _ = write!(self, "{}{}", cursor::Goto(x + 1, y + 1), tile);
        let _ = self.flush();
    }
}

pub struct ScreenBuffer {}

impl Screen for ScreenBuffer {
    fn clear(&mut self) {}

    fn set_tile<T: fmt::Display + fmt::Debug>(&mut self, [x, y]: [u16; 2], tile: T) {
        println!("Output: ({}, {}): {:?}", x, y, tile);
    }
}
