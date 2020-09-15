use crate::err::{IntErr, Interrupt, Never};
use std::{
    cell::Cell,
    fmt::{Display, Error, Formatter},
};

mod base;
mod bigrat;
mod biguint;
mod complex;
mod exact_base;
mod formatting_style;
mod unit;

pub use formatting_style::FormattingStyle;

pub type Number = unit::UnitValue;
pub type Base = base::Base;
pub type BaseOutOfRangeError = base::BaseOutOfRangeError;
pub type InvalidBasePrefixError = base::InvalidBasePrefixError;

// Small formatter helper
pub fn to_string<I: Interrupt, F: Fn(&mut Formatter) -> Result<(), IntErr<Error, I>>>(
    func: F,
) -> Result<String, IntErr<Never, I>> {
    struct Fmt<I: Interrupt, F: Fn(&mut Formatter) -> Result<(), IntErr<Error, I>>> {
        format: F,
        error: Cell<Option<IntErr<Never, I>>>,
    }

    impl<F, I: Interrupt> Display for Fmt<I, F>
    where
        F: Fn(&mut Formatter) -> Result<(), IntErr<Error, I>>,
    {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
            let interrupt = match (self.format)(f) {
                Ok(()) => return Ok(()),
                Err(IntErr::Interrupt(i)) => i,
                Err(IntErr::Error(e)) => return Err(e),
            };
            self.error.set(Some(IntErr::Interrupt(interrupt)));
            Ok(())
        }
    }

    let fmt = Fmt {
        format: func,
        error: Cell::new(None),
    };
    let string = fmt.to_string();
    if let Some(e) = fmt.error.into_inner() {
        return Err(e);
    }
    Ok(string)
}

pub struct ValueTooLarge<T: Display> {
    max_allowed: T,
}

impl<T: Display> Display for ValueTooLarge<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Value must be less than or equal to {}",
            self.max_allowed
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum IntegerPowerError {
    ExponentTooLarge,
    ZeroToThePowerOfZero,
}

impl Display for IntegerPowerError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::ExponentTooLarge => write!(f, "Exponent too large"),
            Self::ZeroToThePowerOfZero => write!(f, "Zero to the power of zero is undefined"),
        }
    }
}

#[derive(Debug)]
pub struct DivideByZero {}
impl Display for DivideByZero {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Division by zero")
    }
}
