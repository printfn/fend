use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Base {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Color256(u8),
    Unknown(String),
}

struct BaseVisitor;

impl<'de> serde::de::Visitor<'de> for BaseVisitor {
    type Value = Base;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "`black`, `red`, `green`, `yellow`, `blue`, `magenta`, \
                `cyan`, `white` or `256:n` (e.g. `256:42`)",
        )
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        if let Some(color_n) = v.strip_prefix("256:") {
            if let Ok(n) = color_n.parse::<u8>() {
                return Ok(Base::Color256(n));
            }
        }
        Ok(match v {
            "black" => Base::Black,
            "red" => Base::Red,
            "green" => Base::Green,
            "yellow" => Base::Yellow,
            "blue" => Base::Blue,
            "magenta" | "purple" => Base::Magenta,
            "cyan" => Base::Cyan,
            "white" => Base::White,
            unknown_color_name => Base::Unknown(unknown_color_name.to_string()),
        })
    }
}

impl<'de> serde::Deserialize<'de> for Base {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(BaseVisitor)
    }
}

impl Base {
    pub fn as_ansi(&self) -> console::Color {
        match self {
            Self::Black => console::Color::Black,
            Self::Red => console::Color::Red,
            Self::Green => console::Color::Green,
            Self::Yellow => console::Color::Yellow,
            Self::Blue => console::Color::Blue,
            Self::Magenta => console::Color::Magenta,
            Self::Cyan => console::Color::Cyan,
            Self::White | Self::Unknown(_) => console::Color::White,
            Self::Color256(n) => console::Color::Color256(*n),
        }
    }

    pub fn warn_about_unknown_colors(&self) {
        if let Self::Unknown(name) = self {
            eprintln!("Warning: ignoring unknown color `{name}`");
        }
    }
}
