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
}
