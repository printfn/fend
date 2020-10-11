use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Base(BaseEnum);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum BaseEnum {
    /// Binary with 0b prefix
    Binary,
    /// Octal with 0o prefix
    Octal,
    /// Decimal with 0d prefix
    Decimal,
    /// Hex with 0x prefix
    Hex,
    /// Custom base between 2 and 36 (inclusive), written as base#number
    Custom(u8),
    /// Plain (no prefix)
    Plain(u8),
}

pub struct InvalidBasePrefixError {}

impl fmt::Display for InvalidBasePrefixError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Unable to parse a valid base prefix, expected 0b, 0o, 0d or 0x"
        )
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum BaseOutOfRangeError {
    BaseTooSmall,
    BaseTooLarge,
}

impl fmt::Display for BaseOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::BaseTooSmall => write!(f, "Base must be at least 2"),
            Self::BaseTooLarge => write!(f, "Base cannot be larger than 36"),
        }
    }
}

impl Base {
    pub const fn base_as_u8(self) -> u8 {
        match self.0 {
            BaseEnum::Binary => 2,
            BaseEnum::Octal => 8,
            BaseEnum::Decimal => 10,
            BaseEnum::Hex => 16,
            BaseEnum::Custom(b) | BaseEnum::Plain(b) => b,
        }
    }

    pub const fn from_zero_based_prefix_char(ch: char) -> Result<Self, InvalidBasePrefixError> {
        Ok(match ch {
            'x' => Self(BaseEnum::Hex),
            'd' => Self(BaseEnum::Decimal),
            'o' => Self(BaseEnum::Octal),
            'b' => Self(BaseEnum::Binary),
            _ => return Err(InvalidBasePrefixError {}),
        })
    }

    pub const fn from_plain_base(base: u8) -> Result<Self, BaseOutOfRangeError> {
        if base < 2 {
            return Err(BaseOutOfRangeError::BaseTooSmall);
        } else if base > 36 {
            return Err(BaseOutOfRangeError::BaseTooLarge);
        }
        Ok(Self(BaseEnum::Plain(base)))
    }

    pub const fn from_custom_base(base: u8) -> Result<Self, BaseOutOfRangeError> {
        if base < 2 {
            return Err(BaseOutOfRangeError::BaseTooSmall);
        } else if base > 36 {
            return Err(BaseOutOfRangeError::BaseTooLarge);
        }
        Ok(Self(BaseEnum::Custom(base)))
    }

    pub fn write_prefix(self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self.0 {
            BaseEnum::Binary => write!(f, "0b")?,
            BaseEnum::Octal => write!(f, "0o")?,
            BaseEnum::Decimal => write!(f, "0d")?,
            BaseEnum::Hex => write!(f, "0x")?,
            BaseEnum::Custom(b) => write!(f, "{}#", b)?,
            BaseEnum::Plain(_) => (),
        }
        Ok(())
    }

    pub const fn has_prefix(self) -> bool {
        !matches!(self.0, BaseEnum::Plain(_))
    }

    pub const fn digit_as_char(digit: u64) -> Option<char> {
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

    pub const fn allow_leading_zeroes(self) -> bool {
        !matches!(self.0, BaseEnum::Plain(10))
    }
}

impl Default for Base {
    fn default() -> Self {
        Self(BaseEnum::Plain(10))
    }
}
