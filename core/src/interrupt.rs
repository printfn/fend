use crate::err::IntErr;
use std::time::{Duration, Instant};

pub trait Interrupt {
    fn should_interrupt(&self) -> bool;
}

pub fn test_int(int: &impl Interrupt) -> Result<(), IntErr<crate::err::Never>> {
    if int.should_interrupt() {
        Err(crate::err::Interrupt::default().into())
    } else {
        Ok(())
    }
}

#[derive(Default)]
pub struct Never {}
impl Interrupt for Never {
    fn should_interrupt(&self) -> bool {
        false
    }
}

// A simple way to interrupt computations after a fixed amount of time.
pub struct Timeout {
    start: Instant,
    duration: Duration,
}

impl Interrupt for Timeout {
    fn should_interrupt(&self) -> bool {
        Instant::now().duration_since(self.start) >= self.duration
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    #[test]
    fn test_timeout() {
        let int = crate::interrupt::Timeout {
            start: Instant::now(),
            duration: Duration::from_millis(10),
        };
        let ctx = crate::Context::new();
        let res = crate::eval::evaluate_to_value("10^1000000", &ctx.scope, &int);
        // we must have an interrupt and not an error
        res.unwrap_err().unwrap();
    }
}
