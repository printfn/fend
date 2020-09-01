use std::time::{Duration, Instant};

pub trait Interrupt {
    fn should_interrupt(&self) -> bool;
    fn test(&self) -> Result<(), crate::err::Interrupt>;
}

#[derive(Default)]
pub struct Never {}
impl Interrupt for Never {
    fn should_interrupt(&self) -> bool {
        false
    }
    fn test(&self) -> Result<(), crate::err::Interrupt> {
        return Ok(())
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
    fn test(&self) -> Result<(), crate::err::Interrupt> {
        if self.should_interrupt() {
            crate::err::err()
        } else {
            Ok(())
        }
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
        let res = crate::evaluate_to_value("10^1000000", &ctx.scope, &int);
        assert_eq!(res.unwrap_err(), "Interrupted".to_string());
    }
}
