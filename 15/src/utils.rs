use std::time::{Duration, Instant};

pub struct Ticker {
    interval: Duration,
    next_tick: Instant,
}

impl Ticker {
    pub fn new(interval: Duration) -> Self {
        Ticker {
            interval,
            next_tick: Instant::now()
        }
    }

    pub fn with_rate(rate: u64) -> Self {
        Self::new(Duration::from_nanos(1_000_000_000 / rate))
    }

    pub fn wait(&mut self) {
        let tick = self.next().unwrap();
        while Instant::now() < tick {
        }
    }
}

impl Iterator for Ticker {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        let tick = self.next_tick;
        self.next_tick += self.interval;
        Some(tick)
    }
}
