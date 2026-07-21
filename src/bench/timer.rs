use std::borrow::Cow;
use std::time::Instant;

pub struct ScopedTimer {
    name: Cow<'static, str>,
    start: Instant,
}

impl ScopedTimer{
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        println!("[BENCH] {:<20} {:?}", self.name, self.start.elapsed());
    }
}
