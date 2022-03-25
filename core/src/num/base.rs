use std::{fmt, io};

use crate::{
    error::FendError,
    serialize::{deserialize_u8, serialize_u8},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Base(BaseEnum);

#[derive(Copy, Clone, PartialEq, Eq)]
enum BaseEnum {
    /// Binary with 0b prefix
    Binary,
    /// Octal with 0o prefix
    Octal,
    /// Hex with 0x prefix
    Hex,
    /// Custom base between 2 and 36 (inclusive), written as base#number
    Custom(u8),
    /// Plain (no prefix)
    Plain(u8),
}

impl Base {
    pub(crate) const HEX: Self = Self(BaseEnum::Hex);

    pub(crate) const fn base_as_u8(self) -> u8 {
        match self.0 {
            BaseEnum::Binary => 2,
            BaseEnum::Octal => 8,
            BaseEnum::Hex => 16,
            BaseEnum::Custom(b) | BaseEnum::Plain(b) => b,
        }
    }

    pub(crate) const fn from_zero_based_prefix_char(ch: char) -> Result<Self, FendError> {
        Ok(match ch {
            'x' => Self(BaseEnum::Hex),
            'o' => Self(BaseEnum::Octal),
            'b' => Self(BaseEnum::Binary),
            _ => return Err(FendError::InvalidBasePrefix),
        })
    }

    pub(crate) const fn from_plain_base(base: u8) -> Result<Self, FendError> {
        if base < 2 {
            return Err(FendError::BaseTooSmall);
        } else if base > 36 {
            return Err(FendError::BaseTooLarge);
        }
        Ok(Self(BaseEnum::Plain(base)))
    }

    pub(crate) const fn from_custom_base(base: u8) -> Result<Self, FendError> {
        if base < 2 {
            return Err(FendError::BaseTooSmall);
        } else if base > 36 {
            return Err(FendError::BaseTooLarge);
        }
        Ok(Self(BaseEnum::Custom(base)))
    }

    pub(crate) fn write_prefix(self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.0 {
            BaseEnum::Binary => write!(f, "0b")?,
            BaseEnum::Octal => write!(f, "0o")?,
            BaseEnum::Hex => write!(f, "0x")?,
            BaseEnum::Custom(b) => write!(f, "{}#", b)?,
            BaseEnum::Plain(_) => (),
        }
        Ok(())
    }

    pub(crate) const fn has_prefix(self) -> bool {
        !matches!(self.0, BaseEnum::Plain(_))
    }

    pub(crate) const fn digit_as_char(digit: u64) -> Option<char> {
        Some(match digit {
            0 => '0',
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            10 => 'a',
            11 => 'b',
            12 => 'c',
            13 => 'd',
            14 => 'e',
            15 => 'f',
            16 => 'g',
            17 => 'h',
            18 => 'i',
            19 => 'j',
            20 => 'k',
            21 => 'l',
            22 => 'm',
            23 => 'n',
            24 => 'o',
            25 => 'p',
            26 => 'q',
            27 => 'r',
            28 => 's',
            29 => 't',
            30 => 'u',
            31 => 'v',
            32 => 'w',
            33 => 'x',
            34 => 'y',
            35 => 'z',
            _ => return None,
        })
    }

    pub(crate) fn serialize(self, write: &mut impl io::Write) -> Result<(), FendError> {
        match self.0 {
            BaseEnum::Binary => serialize_u8(1, write)?,
            BaseEnum::Octal => serialize_u8(2, write)?,
            BaseEnum::Hex => serialize_u8(3, write)?,
            BaseEnum::Custom(b) => {
                serialize_u8(4, write)?;
                serialize_u8(b, write)?;
            }
            BaseEnum::Plain(b) => {
                serialize_u8(5, write)?;
                serialize_u8(b, write)?;
            }
        }
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(Self(match deserialize_u8(read)? {
            1 => BaseEnum::Binary,
            2 => BaseEnum::Octal,
            3 => BaseEnum::Hex,
            4 => BaseEnum::Custom(deserialize_u8(read)?),
            5 => BaseEnum::Plain(deserialize_u8(read)?),
            _ => return Err(FendError::DeserializationError),
        }))
    }
}

impl Default for Base {
    fn default() -> Self {
        Self(BaseEnum::Plain(10))
    }
}

impl fmt::Debug for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            BaseEnum::Binary => write!(f, "binary"),
            BaseEnum::Octal => write!(f, "octal"),
            BaseEnum::Hex => write!(f, "hex"),
            BaseEnum::Custom(b) => write!(f, "base {} (with prefix)", b),
            BaseEnum::Plain(b) => write!(f, "base {}", b),
        }
    }
}
