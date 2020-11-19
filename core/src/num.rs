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

#[allow(clippy::pub_enum_variant_names)]
pub enum ValueOutOfRange<T: fmt::Display> {
    MustBeLessThanOrEqualTo(T),
    MustBeBetween(T, T),
    MustNotBeLessThan(T),
    MustBeGreaterThan(T),
}

impl<T: fmt::Display> fmt::Display for ValueOutOfRange<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::MustBeLessThanOrEqualTo(x) => {
                write!(f, "Value must be less than or equal to {}", x)
            }
            Self::MustBeBetween(a, b) => {
                write!(f, "Value must be between {} and {}", a, b)
            }
            Self::MustNotBeLessThan(x) => {
                write!(f, "Value must not be less than {}", x)
            }
            Self::MustBeGreaterThan(x) => {
                write!(f, "Value must be greater than {}", x)
            }
        }
    }
}

impl<T: fmt::Display> crate::err::Error for ValueOutOfRange<T> {}

pub enum ConvertToUsizeError {
    OutOfRange(ValueOutOfRange<usize>),
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
            Self::OutOfRange(value_out_of_range_error) => write!(f, "{}", value_out_of_range_error),
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
