use std::time::Instant;

pub struct ScopedTimer<'a> {
    name: &'a str,
    start: Instant,
}

impl<'a> ScopedTimer<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        println!(
            "[BENCH] {:<20} {:?}",
            self.name,
            self.start.elapsed()
        );
    }
}