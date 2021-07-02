use crate::error::FendError;

pub trait Interrupt {
    fn should_interrupt(&self) -> bool;
}

impl<T: Interrupt> crate::error::Interrupt for T {
    fn test(&self) -> Result<(), ()> {
        if self.should_interrupt() {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn test_int<I: crate::error::Interrupt>(int: &I) -> Result<(), FendError> {
    if let Err(()) = int.test() {
        Err(FendError::Interrupted)
    } else {
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct Never {}
impl Interrupt for Never {
    fn should_interrupt(&self) -> bool {
        false
    }
}
