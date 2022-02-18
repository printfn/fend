use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Base {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
}

struct BaseVisitor;

impl<'de> serde::de::Visitor<'de> for BaseVisitor {
    type Value = Base;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .write_str("`black`, `red`, `green`, `yellow`, `blue`, `purple`, `cyan` or `white`")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(match v {
            "black" => Base::Black,
            "red" => Base::Red,
            "green" => Base::Green,
            "yellow" => Base::Yellow,
            "blue" => Base::Blue,
            "purple" => Base::Purple,
            "cyan" => Base::Cyan,
            "white" => Base::White,
            _ => {
                return Err(E::unknown_variant(
                    v,
                    &[
                        "black", "red", "green", "yellow", "blue", "purple", "cyan", "white",
                    ],
                ))
            }
        })
    }
}

impl<'de> serde::Deserialize<'de> for Base {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(BaseVisitor)
    }
}

impl Base {
    pub fn as_ansi(self) -> ansi_term::Color {
        match self {
            Self::Black => ansi_term::Color::Black,
            Self::Red => ansi_term::Color::Red,
            Self::Green => ansi_term::Color::Green,
            Self::Yellow => ansi_term::Color::Yellow,
            Self::Blue => ansi_term::Color::Blue,
            Self::Purple => ansi_term::Color::Purple,
            Self::Cyan => ansi_term::Color::Cyan,
            Self::White => ansi_term::Color::White,
        }
    }
}
