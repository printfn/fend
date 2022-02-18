use super::{base::Base, style::Color};
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub struct OutputColors {
    number: Color,
    string: Color,
    identifier: Color,
    keyword: Color,
    built_in_function: Color,
    date: Color,
    other: Color,
}

impl<'de> serde::Deserialize<'de> for OutputColors {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OutputColorsVisitor;

        impl<'de> serde::de::Visitor<'de> for OutputColorsVisitor {
            type Value = OutputColors;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a list of output colors")
            }

            fn visit_map<V: serde::de::MapAccess<'de>>(
                self,
                mut map: V,
            ) -> Result<OutputColors, V::Error> {
                let mut result = OutputColors::default();
                while let Some(key) = map.next_key()? {
                    match key {
                        "number" => result.number = map.next_value()?,
                        "string" => result.string = map.next_value()?,
                        "identifier" => result.identifier = map.next_value()?,
                        "keyword" => result.keyword = map.next_value()?,
                        "built-in-function" => result.built_in_function = map.next_value()?,
                        "date" => result.date = map.next_value()?,
                        "other" => result.other = map.next_value()?,
                        unknown_key => {
                            return Err(serde::de::Error::unknown_field(unknown_key, FIELDS));
                        }
                    }
                }
                Ok(result)
            }
        }

        const FIELDS: &[&str] = &[
            "number",
            "string",
            "identifier",
            "keyword",
            "built-in-function",
            "date",
            "other",
        ];
        deserializer.deserialize_struct("OutputColors", FIELDS, OutputColorsVisitor)
    }
}

impl Default for OutputColors {
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

impl OutputColors {
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
