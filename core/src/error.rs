use std::{error, fmt};

use crate::num::Range;

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum FendError {
    Interrupted,
    InvalidBasePrefix,
    BaseTooSmall,
    BaseTooLarge,
    UnableToConvertToBase,
    DivideByZero,
    ExponentTooLarge,
    ZeroToThePowerOfZero,
    OutOfRange {
        value: Box<dyn crate::format::DisplayDebug>,
        range: Range<Box<dyn crate::format::DisplayDebug>>,
    },
    NegativeNumbersNotAllowed,
    ProbabilityDistributionsNotAllowed,
    FractionToInteger,
    RandomNumbersNotAvailable,
    MustBeAnInteger(Box<dyn crate::format::DisplayDebug>),
    ExpectedARationalNumber,
    CannotConvertToInteger,
    ComplexToInteger,
    NumberWithUnitToInt,
    InexactNumberToInt,
    ExpectedANumber,
    InvalidType,
    CannotFormatWithZeroSf,
    IsNotAFunction(String),
    IsNotAFunctionOrNumber(String),
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
            Self::UnableToConvertToBase => write!(f, "unable to convert number to a valid base"),
            Self::DivideByZero => write!(f, "division by zero"),
            Self::ExponentTooLarge => write!(f, "exponent too large"),
            Self::ZeroToThePowerOfZero => write!(f, "zero to the power of zero is undefined"),
            Self::OutOfRange { range, value } => {
                write!(f, "{} must lie in the interval {}", value, range)
            }
            Self::NegativeNumbersNotAllowed => write!(f, "negative numbers are not allowed"),
            Self::ProbabilityDistributionsNotAllowed => {
                write!(f, "probability distributions are not allowed")
            }
            Self::FractionToInteger => write!(f, "cannot convert fraction to integer"),
            Self::RandomNumbersNotAvailable => write!(f, "random numbers are not available"),
            Self::MustBeAnInteger(x) => write!(f, "{} is not an integer", x),
            Self::ExpectedARationalNumber => write!(f, "expected a rational number"),
            Self::CannotConvertToInteger => write!(f, "number cannot be converted to an integer"),
            Self::ComplexToInteger => write!(f, "cannot convert complex number to integer"),
            Self::NumberWithUnitToInt => write!(f, "cannot convert number with unit to integer"),
            Self::InexactNumberToInt => write!(f, "cannot convert inexact number to integer"),
            Self::ExpectedANumber => write!(f, "expected a number"),
            Self::InvalidType => write!(f, "invalid type"),
            Self::CannotFormatWithZeroSf => {
                write!(f, "cannot format a number with zero significant figures")
            }
            Self::IsNotAFunction(s) => write!(f, "'{}' is not a function", s),
            Self::IsNotAFunctionOrNumber(s) => write!(f, "'{}' is not a function or number", s),
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

// todo remove this impl
impl From<String> for FendError {
    fn from(e: String) -> Self {
        Self::String(e)
    }
}

pub(crate) use crate::interrupt::Interrupt;
