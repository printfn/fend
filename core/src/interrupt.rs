use crate::err::IntErr;
use std::time::{Duration, Instant};

pub trait Interrupt {
    fn should_interrupt(&self) -> bool;
}

impl<T: Interrupt> crate::err::Interrupt for T {
    type Int = ();
    fn test(&self) -> Result<(), Self::Int> {
        if self.should_interrupt() {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub fn test_int<I: crate::err::Interrupt>(int: &I) -> Result<(), IntErr<crate::err::Never, I>> {
    if let Err(i) = int.test() {
        Err(IntErr::Interrupt(i))
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
