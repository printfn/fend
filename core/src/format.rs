use crate::error::{IntErr, Interrupt};
use crate::num::Exact;
use std::fmt;

pub(crate) trait Format {
    type Params: Default;
    type Out: fmt::Display + fmt::Debug;
    type Error;

    fn format<I: Interrupt>(
        &self,
        params: &Self::Params,
        int: &I,
    ) -> Result<Exact<Self::Out>, IntErr<Self::Error, I>>;

    /// Simpler alternative to calling format
    fn fm<I: Interrupt>(&self, int: &I) -> Result<Self::Out, IntErr<Self::Error, I>> {
        Ok(self.format(&Default::default(), int)?.value)
    }
}

pub(crate) trait DisplayDebug: fmt::Display + fmt::Debug {}

impl<T: fmt::Display + fmt::Debug> DisplayDebug for T {}
