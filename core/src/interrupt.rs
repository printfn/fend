use crate::err::IntErr;

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
