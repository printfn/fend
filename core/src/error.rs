use std::{error, fmt, io};

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
	FactorialComplex,
	DeserializationError,
	Wrap(Box<dyn error::Error + Send + Sync + 'static>),
	NoExchangeRatesAvailable,
	OutOfRange {
		value: Box<dyn crate::format::DisplayDebug>,
		range: Range<Box<dyn crate::format::DisplayDebug>>,
	},
	NegativeNumbersNotAllowed,
	ProbabilityDistributionsNotAllowed,
	FractionToInteger,
	ModuloByZero,
	RandomNumbersNotAvailable,
	MustBeAnInteger(Box<dyn crate::format::DisplayDebug>),
	ExpectedARationalNumber,
	CannotConvertToInteger,
	ComplexToInteger,
	NumberWithUnitToInt,
	InexactNumberToInt,
	ExpectedANumber,
	ExpectedABool(&'static str),
	InvalidDiceSyntax,
	SpecifyNumDp,
	SpecifyNumSf,
	UnableToInvertFunction(&'static str),
	InvalidOperandsForSubtraction,
	InversesOfLambdasUnsupported,
	CouldNotFindKeyInObject,
	CouldNotFindKey(String),
	CannotFormatWithZeroSf,
	IsNotAFunction(String),
	IsNotAFunctionOrNumber(String),
	IdentifierNotFound(crate::ident::Ident),
	ExpectedACharacter,
	StringCannotBeLonger,
	StringCannotBeEmpty,
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
	ExpectedAUnitlessNumber,
	ExpectedAnObject,
	InvalidUnicodeEscapeSequence,
	FormattingError(fmt::Error),
	IoError(io::Error),
	ParseDateError(String),
	ParseError(crate::parser::ParseError),
	ExpectedAString,
	ExpComplex,
	ExpectedARealNumber,
	ConversionRhsNumerical,
	FactorialUnitless,
	RootsComplex,
	ModuloForPositiveInts,
	ExpUnitless,
	IncompatibleConversion {
		from: String,
		to: String,
		from_base: String,
		to_base: String,
	},
	ModuloUnitless,
	RootsOfNegativeNumbers,
	NonIntegerNegRoots,
	CannotConvertValueTo(&'static str),
	ExpectedADateLiteral,
}

impl fmt::Display for FendError {
	#[allow(clippy::too_many_lines)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Interrupted => write!(f, "interrupted"),
			Self::ParseError(e) => write!(f, "{e}"),
			Self::FactorialUnitless => {
				write!(f, "factorial is only supported for unitless numbers")
			}
			Self::DeserializationError => write!(f, "failed to deserialize object"),
			Self::ModuloUnitless => write!(f, "modulo is only supported for unitless numbers"),
			Self::FactorialComplex => write!(f, "factorial is not supported for complex numbers"),
			Self::RootsComplex => write!(f, "roots are currently unsupported for complex numbers"),
			Self::ExpComplex => write!(f, "exponentiation is not supported for complex numbers"),
			Self::ExpUnitless => write!(f, "exponentiation is only supported for unitless numbers"),
			Self::IoError(_) => write!(f, "I/O error"),
			Self::InvalidBasePrefix => write!(
				f,
				"unable to parse a valid base prefix, expected 0b, 0o, or 0x"
			),
			Self::NoExchangeRatesAvailable => write!(f, "exchange rates are not available"),
			Self::IncompatibleConversion {
				from,
				to,
				from_base,
				to_base,
			} => {
				write!(
                    f,
                    "cannot convert from {from} to {to}: units '{from_base}' and '{to_base}' are incompatible"
                )
			}
			Self::NonIntegerNegRoots => write!(f, "cannot compute non-integer or negative roots"),
			Self::RootsOfNegativeNumbers => {
				write!(f, "roots of negative numbers are not supported")
			}
			Self::ModuloForPositiveInts => {
				write!(f, "modulo is only supported for positive integers")
			}
			Self::CannotConvertValueTo(ty) => write!(f, "cannot convert value to {ty}"),
			Self::BaseTooSmall => write!(f, "base must be at least 2"),
			Self::ConversionRhsNumerical => write!(
				f,
				"right-hand side of unit conversion has a numerical value"
			),
			Self::BaseTooLarge => write!(f, "base cannot be larger than 36"),
			Self::UnableToConvertToBase => write!(f, "unable to convert number to a valid base"),
			Self::DivideByZero => write!(f, "division by zero"),
			Self::ExponentTooLarge => write!(f, "exponent too large"),
			Self::ZeroToThePowerOfZero => write!(f, "zero to the power of zero is undefined"),
			Self::OutOfRange { range, value } => {
				write!(f, "{value} must lie in the interval {range}")
			}
			Self::ModuloByZero => write!(f, "modulo by zero"),
			Self::SpecifyNumDp => write!(
				f,
				"you need to specify what number of decimal places to use, e.g. '10 dp'"
			),
			Self::SpecifyNumSf => write!(
				f,
				"you need to specify what number of significant figures to use, e.g. '10 sf'"
			),
			Self::ExpectedAUnitlessNumber => write!(f, "expected a unitless number"),
			Self::ExpectedARealNumber => write!(f, "expected a real number"),
			Self::StringCannotBeLonger => write!(f, "string cannot be longer than one codepoint"),
			Self::StringCannotBeEmpty => write!(f, "string cannot be empty"),
			Self::NegativeNumbersNotAllowed => write!(f, "negative numbers are not allowed"),
			Self::ProbabilityDistributionsNotAllowed => {
				write!(
					f,
					"probability distributions are not allowed (consider using `sample`)"
				)
			}
			Self::ParseDateError(s) => write!(f, "failed to convert '{s}' to a date"),
			Self::ExpectedAString => write!(f, "expected a string"),
			Self::UnableToInvertFunction(name) => write!(f, "unable to invert function {name}"),
			Self::FractionToInteger => write!(f, "cannot convert fraction to integer"),
			Self::RandomNumbersNotAvailable => write!(f, "random numbers are not available"),
			Self::MustBeAnInteger(x) => write!(f, "{x} is not an integer"),
			Self::ExpectedABool(t) => write!(f, "expected a bool (found {t})"),
			Self::CouldNotFindKeyInObject => write!(f, "could not find key in object"),
			Self::CouldNotFindKey(k) => write!(f, "could not find key {k}"),
			Self::InversesOfLambdasUnsupported => write!(
				f,
				"inverses of lambda functions are not currently supported"
			),
			Self::ExpectedARationalNumber => write!(f, "expected a rational number"),
			Self::CannotConvertToInteger => write!(f, "number cannot be converted to an integer"),
			Self::ComplexToInteger => write!(f, "cannot convert complex number to integer"),
			Self::NumberWithUnitToInt => write!(f, "cannot convert number with unit to integer"),
			Self::InexactNumberToInt => write!(f, "cannot convert inexact number to integer"),
			Self::ExpectedANumber => write!(f, "expected a number"),
			Self::InvalidDiceSyntax => write!(f, "invalid dice syntax, try e.g. `4d6`"),
			Self::InvalidOperandsForSubtraction => write!(f, "invalid operands for subtraction"),
			Self::CannotFormatWithZeroSf => {
				write!(f, "cannot format a number with zero significant figures")
			}
			Self::IsNotAFunction(s) => write!(f, "'{s}' is not a function"),
			Self::IsNotAFunctionOrNumber(s) => write!(f, "'{s}' is not a function or number"),
			Self::IdentifierNotFound(s) => write!(f, "unknown identifier '{s}'"),
			Self::ExpectedACharacter => write!(f, "expected a character"),
			Self::ExpectedADigit(ch) => write!(f, "expected a digit, found '{ch}'"),
			Self::ExpectedChar(ex, fnd) => write!(f, "expected '{ex}', found '{fnd}'"),
			Self::ExpectedDigitSeparator(ch) => {
				write!(f, "expected a digit separator, found {ch}")
			}
			Self::DigitSeparatorsNotAllowed => write!(f, "digit separators are not allowed"),
			Self::DigitSeparatorsOnlyBetweenDigits => {
				write!(f, "digit separators can only occur between digits")
			}
			Self::InvalidCharAtBeginningOfIdent(ch) => {
				write!(f, "'{ch}' is not valid at the beginning of an identifier")
			}
			Self::UnexpectedChar(ch) => write!(f, "unexpected character '{ch}'"),
			Self::UnterminatedStringLiteral => write!(f, "unterminated string literal"),
			Self::UnknownBackslashEscapeSequence(ch) => {
				write!(f, "unknown escape sequence: \\{ch}")
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
			Self::ExpectedAnObject => write!(f, "expected an object"),
			Self::InvalidUnicodeEscapeSequence => {
				write!(
					f,
					"invalid Unicode escape sequence, expected e.g. \\u{{7e}}"
				)
			}
			Self::FormattingError(_) => write!(f, "error during formatting"),
			Self::Wrap(e) => write!(f, "{e}"),
			Self::ExpectedADateLiteral => write!(f, "Expected a date literal, e.g. @1970-01-01"),
		}
	}
}

impl error::Error for FendError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		match self {
			Self::FormattingError(e) => Some(e),
			Self::IoError(e) => Some(e),
			Self::Wrap(e) => Some(e.as_ref()),
			_ => None,
		}
	}
}

impl From<fmt::Error> for FendError {
	fn from(e: fmt::Error) -> Self {
		Self::FormattingError(e)
	}
}

impl From<io::Error> for FendError {
	fn from(e: io::Error) -> Self {
		Self::IoError(e)
	}
}

impl From<Box<dyn error::Error + Send + Sync + 'static>> for FendError {
	fn from(e: Box<dyn error::Error + Send + Sync + 'static>) -> Self {
		Self::Wrap(e)
	}
}

pub(crate) use crate::interrupt::Interrupt;
