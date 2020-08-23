mod bigrat;
mod biguint;
mod complex;
mod exact_base;
mod unit;

pub type Number = unit::UnitValue;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Base {
    /// Binary with 0b prefix
    Binary,
    /// Octal with 0o prefix
    Octal,
    /// Decimal with no prefix
    Decimal,
    /// Hex with 0x prefix
    Hex,
    /// Custom base between 2 and 36 (inclusive), written as base#number
    Custom(u8),
}

impl Base {
    pub fn base_as_u8(self) -> u8 {
        match self {
            Base::Binary => 2,
            Base::Octal => 8,
            Base::Decimal => 10,
            Base::Hex => 16,
            Base::Custom(b) => b,
        }
    }

    pub fn write_prefix(self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Ok(match self {
            Base::Binary => write!(f, "0b")?,
            Base::Octal => write!(f, "0o")?,
            Base::Decimal => (),
            Base::Hex => write!(f, "0x")?,
            Base::Custom(b) => write!(f, "{}#", b)?,
        })
    }

    pub fn digit_as_char(digit: u64) -> Option<char> {
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
            _ => return None
        })
    }
}
