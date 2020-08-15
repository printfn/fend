mod bigrat;
mod biguint;
mod complex;
mod exact_base;
mod unit;

pub type Number = exact_base::ExactBase;

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
        return Ok(match self {
            Base::Binary => write!(f, "0b")?,
            Base::Octal => write!(f, "0o")?,
            Base::Decimal => (),
            Base::Hex => write!(f, "0x")?,
            Base::Custom(b) => write!(f, "{}#", b)?,
        });
    }
}
