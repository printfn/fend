use std::fmt;

mod base;
mod bigrat;
mod biguint;
mod complex;
mod exact;
mod formatting_style;
mod real;
mod unit;

pub use formatting_style::FormattingStyle;

pub type Number<'a> = unit::UnitValue<'a>;
pub type FormattedNumber = unit::FormattedUnitValue;
pub type Base = base::Base;
type Exact<T> = exact::Exact<T>;
pub type BaseOutOfRangeError = base::BaseOutOfRangeError;
pub type InvalidBasePrefixError = base::InvalidBasePrefixError;

pub struct ValueTooLarge<T: fmt::Display> {
    max_allowed: T,
}

impl<T: fmt::Display> fmt::Display for ValueTooLarge<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Value must be less than or equal to {}",
            self.max_allowed
        )?;
        Ok(())
    }
}

impl<T: fmt::Display> crate::err::Error for ValueTooLarge<T> {}

pub enum ConvertToUsizeError {
    TooLarge(ValueTooLarge<usize>),
    NegativeNumber,
    Fraction,
    InvalidRealNumber,
    ComplexNumber,
    NumberWithUnit,
    InexactNumber,
}

impl fmt::Display for ConvertToUsizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::TooLarge(value_too_large_error) => write!(f, "{}", value_too_large_error),
            Self::NegativeNumber => write!(f, "Negative numbers are not allowed"),
            Self::Fraction => write!(f, "Cannot convert fraction to integer"),
            Self::InvalidRealNumber => write!(f, "Number cannot be converted to an integer"),
            Self::ComplexNumber => write!(f, "Cannot convert complex number to integer"),
            Self::NumberWithUnit => write!(f, "Cannot convert number with unit to integer"),
            Self::InexactNumber => write!(f, "Cannot convert inexact number to integer"),
        }
    }
}

impl crate::err::Error for ConvertToUsizeError {}

#[derive(Debug)]
pub enum IntegerPowerError {
    ExponentTooLarge,
    ZeroToThePowerOfZero,
}

impl fmt::Display for IntegerPowerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::ExponentTooLarge => write!(f, "Exponent too large"),
            Self::ZeroToThePowerOfZero => write!(f, "Zero to the power of zero is undefined"),
        }
    }
}
impl crate::err::Error for IntegerPowerError {}

#[derive(Debug)]
pub struct DivideByZero {}
impl fmt::Display for DivideByZero {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Division by zero")
    }
}
impl crate::err::Error for DivideByZero {}
