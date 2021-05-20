#[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
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

impl Base {
    fn as_ansi(self) -> ansi_term::Color {
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

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Color {
    foreground: Base,
    #[serde(default)]
    underline: bool,
    #[serde(default)]
    bold: bool,
}

impl Color {
    fn new(foreground: Base) -> Self {
        Self {
            foreground,
            underline: false,
            bold: false,
        }
    }

    fn to_ansi(&self) -> ansi_term::Style {
        let mut style = ansi_term::Style::new();
        style = style.fg(self.foreground.as_ansi());
        if self.underline {
            style = style.underline();
        }
        if self.bold {
            style = style.bold();
        }
        style
    }
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct OutputColours {
    string: Color,
}

impl Default for OutputColours {
    fn default() -> Self {
        Self {
            string: Color {
                bold: true,
                ..Color::new(Base::Yellow)
            },
        }
    }
}

impl OutputColours {
    pub fn get_color(&self, kind: fend_core::SpanKind) -> ansi_term::Style {
        use fend_core::SpanKind;

        match kind {
            SpanKind::String => self.string.to_ansi(),
            SpanKind::Ident => ansi_term::Colour::White.normal(),
            SpanKind::Keyword | SpanKind::BuiltInFunction => ansi_term::Colour::Red.bold(),
            _ => ansi_term::Style::default(),
        }
    }
}
