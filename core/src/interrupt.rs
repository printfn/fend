use crate::{error::FendError, result::FendCoreResult};

pub trait Interrupt {
	fn should_interrupt(&self) -> bool;
}

pub(crate) fn test_int<I: crate::error::Interrupt>(int: &I) -> FendCoreResult<()> {
	if int.should_interrupt() {
		Err(FendError::Interrupted)
	} else {
		Ok(())
	}
}

#[derive(Default)]
pub(crate) struct Never;
impl Interrupt for Never {
	fn should_interrupt(&self) -> bool {
		false
	}
}
