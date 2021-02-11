use crate::error::IntErr;

pub trait Interrupt {
    fn should_interrupt(&self) -> bool;
}

impl<T: Interrupt> crate::error::Interrupt for T {
    type Int = ();
    fn test(&self) -> Result<(), Self::Int> {
        if self.should_interrupt() {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn test_int<I: crate::error::Interrupt>(
    int: &I,
) -> Result<(), IntErr<crate::error::Never, I>> {
    if let Err(i) = int.test() {
        Err(IntErr::Interrupt(i))
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
