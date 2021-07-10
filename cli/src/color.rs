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

#[derive(Default, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Color {
    foreground: Option<Base>,
    underline: bool,
    bold: bool,
}

impl Color {
    fn new(foreground: Base) -> Self {
        Self {
            foreground: Some(foreground),
            underline: false,
            bold: false,
        }
    }

    fn bold(foreground: Base) -> Self {
        Self {
            bold: true,
            ..Self::new(foreground)
        }
    }

    fn to_ansi(&self) -> ansi_term::Style {
        let mut style = ansi_term::Style::default();
        if let Some(foreground) = self.foreground {
            style = style.fg(foreground.as_ansi());
        }
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
#[serde(deny_unknown_fields, default, rename_all = "kebab-case")]
pub struct OutputColours {
    number: Color,
    string: Color,
    identifier: Color,
    keyword: Color,
    built_in_function: Color,
    date: Color,
    other: Color,
}

impl Default for OutputColours {
    fn default() -> Self {
        Self {
            number: Color::default(),
            string: Color::bold(Base::Yellow),
            identifier: Color::new(Base::White),
            keyword: Color::bold(Base::Blue),
            built_in_function: Color::bold(Base::Blue),
            date: Color::default(),
            other: Color::default(),
        }
    }
}

impl OutputColours {
    pub fn get_color(&self, kind: fend_core::SpanKind) -> ansi_term::Style {
        use fend_core::SpanKind;

        match kind {
            SpanKind::Number => self.number.to_ansi(),
            SpanKind::String => self.string.to_ansi(),
            SpanKind::Ident => self.identifier.to_ansi(),
            SpanKind::Keyword => self.keyword.to_ansi(),
            SpanKind::BuiltInFunction => self.built_in_function.to_ansi(),
            SpanKind::Date => self.date.to_ansi(),
            _ => self.other.to_ansi(),
        }
    }
}
