use std::{fmt, io};

use crate::{
    error::FendError,
    serialize::{deserialize_u8, deserialize_usize, serialize_u8, serialize_usize},
};

#[derive(PartialEq, Eq, Clone, Copy, Default)]
#[must_use]
pub(crate) enum FormattingStyle {
    /// Print value as an improper fraction
    ImproperFraction,
    /// Print as a mixed fraction, e.g. 1 1/2
    MixedFraction,
    /// Print as a float, possibly indicating recurring digits
    /// with parentheses, e.g. 7/9 => 0.(81)
    ExactFloat,
    /// Print with the given number of decimal places
    DecimalPlaces(usize),
    /// Print with the given number of significant figures (not including any leading zeroes)
    SignificantFigures(usize),
    /// If exact and no recurring digits: ExactFloat, if complex/imag: MixedFraction,
    /// otherwise: DecimalPlaces(10)
    #[default]
    Auto,
    /// If not exact: DecimalPlaces(10). If no recurring digits: ExactFloat.
    /// Other numbers: MixedFraction, albeit possibly including fractions of pi
    Exact,
}

impl fmt::Display for FormattingStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::ImproperFraction => write!(f, "fraction"),
            Self::MixedFraction => write!(f, "mixed_fraction"),
            Self::ExactFloat => write!(f, "float"),
            Self::Exact => write!(f, "exact"),
            Self::DecimalPlaces(d) => write!(f, "{d} dp"),
            Self::SignificantFigures(s) => write!(f, "{s} sf"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl fmt::Debug for FormattingStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::ImproperFraction => write!(f, "improper fraction"),
            Self::MixedFraction => write!(f, "mixed fraction"),
            Self::ExactFloat => write!(f, "exact float"),
            Self::Exact => write!(f, "exact"),
            Self::DecimalPlaces(d) => write!(f, "{d} dp"),
            Self::SignificantFigures(s) => write!(f, "{s} sf"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl FormattingStyle {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        match self {
            Self::ImproperFraction => serialize_u8(1, write)?,
            Self::MixedFraction => serialize_u8(2, write)?,
            Self::ExactFloat => serialize_u8(3, write)?,
            Self::Exact => serialize_u8(4, write)?,
            Self::DecimalPlaces(d) => {
                serialize_u8(5, write)?;
                serialize_usize(*d, write)?;
            }
            Self::SignificantFigures(s) => {
                serialize_u8(6, write)?;
                serialize_usize(*s, write)?;
            }
            Self::Auto => serialize_u8(7, write)?,
        }
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(match deserialize_u8(read)? {
            1 => Self::ImproperFraction,
            2 => Self::MixedFraction,
            3 => Self::ExactFloat,
            4 => Self::Exact,
            5 => Self::DecimalPlaces(deserialize_usize(read)?),
            6 => Self::SignificantFigures(deserialize_usize(read)?),
            7 => Self::Auto,
            _ => return Err(FendError::DeserializationError),
        })
    }
}
