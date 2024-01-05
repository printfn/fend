use crate::error::Interrupt;
use crate::num::Exact;
use crate::result::FendCoreResult;
use std::fmt;

pub(crate) trait Format {
	type Params: Default;
	type Out: fmt::Display + fmt::Debug;

	fn format<I: Interrupt>(
		&self,
		params: &Self::Params,
		int: &I,
	) -> FendCoreResult<Exact<Self::Out>>;

	/// Simpler alternative to calling format
	fn fm<I: Interrupt>(&self, int: &I) -> FendCoreResult<Self::Out> {
		Ok(self.format(&Default::default(), int)?.value)
	}
}

pub(crate) trait DisplayDebug: fmt::Display + fmt::Debug {}

impl<T: fmt::Display + fmt::Debug> DisplayDebug for T {}
