use std::{convert, error, fmt};

use crate::num::Range;

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum FendError {
    Interrupted,
    InvalidBasePrefix,
    BaseTooSmall,
    BaseTooLarge,
    DivideByZero,
    ExponentTooLarge,
    ZeroToThePowerOfZero,
    OutOfRange {
        value: Box<dyn crate::format::DisplayDebug>,
        range: Range<Box<dyn crate::format::DisplayDebug>>,
    },
    NegativeNumbersNotAllowed,
    FractionToInteger,
    MustBeAnInteger(Box<dyn crate::format::DisplayDebug>),
    ExpectedARationalNumber,
    CannotConvertToInteger,
    ComplexToInteger,
    NumberWithUnitToInt,
    InexactNumberToInt,
    IdentifierNotFound(crate::ident::Ident),
    ExpectedACharacter,
    ExpectedADigit(char),
    ExpectedChar(char, char),
    ExpectedDigitSeparator(char),
    DigitSeparatorsNotAllowed,
    DigitSeparatorsOnlyBetweenDigits,
    InvalidCharAtBeginningOfIdent(char),
    UnexpectedChar(char),
    UnterminatedStringLiteral,
    UnknownBackslashEscapeSequence(char),
    BackslashXOutOfRange,
    ExpectedALetterOrCode,
    InvalidUnicodeEscapeSequence,
    // todo remove this
    String(String),
}

impl fmt::Display for FendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Interrupted => write!(f, "interrupted"),
            Self::InvalidBasePrefix => write!(
                f,
                "unable to parse a valid base prefix, expected 0b, 0o, or 0x"
            ),
            Self::BaseTooSmall => write!(f, "base must be at least 2"),
            Self::BaseTooLarge => write!(f, "base cannot be larger than 36"),
            Self::DivideByZero => write!(f, "division by zero"),
            Self::ExponentTooLarge => write!(f, "exponent too large"),
            Self::ZeroToThePowerOfZero => write!(f, "zero to the power of zero is undefined"),
            Self::OutOfRange { range, value } => {
                write!(f, "{} must lie in the interval {}", value, range)
            }
            Self::NegativeNumbersNotAllowed => write!(f, "negative numbers are not allowed"),
            Self::FractionToInteger => write!(f, "cannot convert fraction to integer"),
            Self::MustBeAnInteger(x) => write!(f, "{} is not an integer", x),
            Self::ExpectedARationalNumber => write!(f, "expected a rational number"),
            Self::CannotConvertToInteger => write!(f, "number cannot be converted to an integer"),
            Self::ComplexToInteger => write!(f, "cannot convert complex number to integer"),
            Self::NumberWithUnitToInt => write!(f, "cannot convert number with unit to integer"),
            Self::InexactNumberToInt => write!(f, "cannot convert inexact number to integer"),
            Self::IdentifierNotFound(s) => write!(f, "unknown identifier '{}'", s),
            Self::ExpectedACharacter => write!(f, "expected a character"),
            Self::ExpectedADigit(ch) => write!(f, "expected a digit, found '{}'", ch),
            Self::ExpectedChar(ex, fnd) => write!(f, "expected '{}', found '{}'", ex, fnd),
            Self::ExpectedDigitSeparator(ch) => {
                write!(f, "expected a digit separator, found {}", ch)
            }
            Self::DigitSeparatorsNotAllowed => write!(f, "digit separators are not allowed"),
            Self::DigitSeparatorsOnlyBetweenDigits => {
                write!(f, "digit separators can only occur between digits")
            }
            Self::InvalidCharAtBeginningOfIdent(ch) => {
                write!(f, "'{}' is not valid at the beginning of an identifier", ch)
            }
            Self::UnexpectedChar(ch) => write!(f, "unexpected character '{}'", ch),
            Self::UnterminatedStringLiteral => write!(f, "unterminated string literal"),
            Self::UnknownBackslashEscapeSequence(ch) => {
                write!(f, "unknown escape sequence: \\{}", ch)
            }
            Self::BackslashXOutOfRange => {
                write!(f, "expected an escape sequence between \\x00 and \\x7f")
            }
            Self::ExpectedALetterOrCode => {
                write!(
                    f,
                    "expected an uppercase letter, or one of @[\\]^_? (e.g. \\^H or \\^@)"
                )
            }
            Self::InvalidUnicodeEscapeSequence => {
                write!(
                    f,
                    "invalid Unicode escape sequence, expected e.g. \\u{{7e}}"
                )
            }
            Self::String(s) => write!(f, "{}", s),
        }
    }
}

impl error::Error for FendError {}

impl Error for FendError {}

// todo remove these impls
impl<I: Interrupt> From<FendError> for IntErr<String, I> {
    fn from(e: FendError) -> Self {
        e.to_string().into()
    }
}

impl<I: Interrupt> From<String> for IntErr<FendError, I> {
    fn from(e: String) -> Self {
        Self::Error(FendError::String(e))
    }
}

impl<I: Interrupt> From<IntErr<String, I>> for IntErr<FendError, I> {
    fn from(e: IntErr<String, I>) -> Self {
        match e {
            IntErr::Interrupt(i) => Self::Interrupt(i),
            IntErr::Error(e) => e.into(),
        }
    }
}

impl<I: Interrupt> From<IntErr<FendError, I>> for IntErr<String, I> {
    fn from(e: IntErr<FendError, I>) -> Self {
        match e {
            IntErr::Interrupt(i) => Self::Interrupt(i),
            IntErr::Error(e) => Self::Error(e.to_string()),
        }
    }
}

pub(crate) trait Error: fmt::Display {}

pub(crate) type Never = convert::Infallible;

pub(crate) enum IntErr<E, I: Interrupt> {
    Interrupt(I),
    Error(E),
}

#[allow(clippy::use_self)]
impl<E, I: Interrupt> IntErr<E, I> {
    pub(crate) fn unwrap(self) -> IntErr<Never, I> {
        match self {
            Self::Interrupt(i) => IntErr::<Never, I>::Interrupt(i),
            Self::Error(_) => panic!("unwrap"),
        }
    }
}

impl<E, I: Interrupt> From<E> for IntErr<E, I> {
    fn from(e: E) -> Self {
        Self::Error(e)
    }
}

#[allow(clippy::use_self)]
impl<E: Error, I: Interrupt> From<IntErr<Never, I>> for IntErr<E, I> {
    fn from(e: IntErr<Never, I>) -> Self {
        match e {
            IntErr::Error(never) => match never {},
            IntErr::Interrupt(i) => Self::Interrupt(i),
        }
    }
}

impl<E: fmt::Debug, I: Interrupt> fmt::Debug for IntErr<E, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Interrupt(_) => write!(f, "interrupt")?,
            Self::Error(e) => write!(f, "{:?}", e)?,
        }
        Ok(())
    }
}

impl Error for String {}

pub(crate) trait Interrupt {
    fn test(&self) -> Result<(), ()>;
}

#[derive(Default)]
pub(crate) struct NeverInterrupt {}
impl Interrupt for NeverInterrupt {
    fn test(&self) -> Result<(), ()> {
        Ok(())
    }
}
