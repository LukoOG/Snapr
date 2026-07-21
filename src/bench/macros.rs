#[cfg(feature = "benchmark")]
#[macro_export]
macro_rules! benchmark {
    ($name:expr, $body:block) => {{
        let start = std::time::Instant::now();
        let result = { $body };
        println!("{}: {:.2?}", $name, start.elapsed());
        result
    }};
}

#[cfg(not(feature = "benchmark"))]
#[macro_export]
macro_rules! benchmark {
    ($name:expr, $body:block) => {{
        $body
    }};
}

#[cfg(feature = "benchmark")]
#[macro_export]
macro_rules! scoped_timer {
    ($name:literal) => {
        let _timer = $crate::bench::ScopedTimer::new($name);
    };

    ($($arg:tt)*) => {
        let _timer = $crate::bench::ScopedTimer::new(format!($($arg)*));
    };
}

#[cfg(not(feature = "benchmark"))]
#[macro_export]
macro_rules! scoped_timer {
    ($name:expr) => {};
    ($($arg:tt)*) => {};
}