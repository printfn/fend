use super::{base::Base, style::Color};
use std::collections;

#[derive(Debug, Default)]
pub struct OutputColors {
    styles: collections::HashMap<String, Color>,
}

impl<'de> serde::Deserialize<'de> for OutputColors {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(OutputColors {
            styles: collections::HashMap::deserialize(deserializer)?,
        })
    }
}

impl PartialEq for OutputColors {
    fn eq(&self, other: &Self) -> bool {
        self.get_style("number") == other.get_style("number")
            && self.get_style("string") == other.get_style("string")
            && self.get_style("identifier") == other.get_style("identifier")
            && self.get_style("keyword") == other.get_style("keyword")
            && self.get_style("built-in-function") == other.get_style("built-in-function")
            && self.get_style("date") == other.get_style("date")
            && self.get_style("other") == other.get_style("other")
    }
}

impl Eq for OutputColors {}

impl OutputColors {
    fn get_style(&self, name: &str) -> Color {
        self.styles.get(name).cloned().unwrap_or_else(|| {
            match name {
                "number" | "date" | "string" | "other" => Color::default(),
                "identifier" => Color::new(Base::White),
                "keyword" | "built-in-function" => Color::bold(Base::Blue),
                _ => {
                    // this should never happen
                    Color::default()
                }
            }
        })
    }

    pub fn print_warnings_about_unknown_keys(&self) {
        for (key, style) in &self.styles {
            if !matches!(
                key.as_str(),
                "number"
                    | "string"
                    | "identifier"
                    | "keyword"
                    | "built-in-function"
                    | "date"
                    | "other"
            ) {
                eprintln!(
                    "Warning: ignoring unknown configuration setting `colors.{}`",
                    key
                );
            }
            style.print_warnings_about_unknown_keys(key);
        }
    }

    pub fn get_color(&self, kind: fend_core::SpanKind) -> ansi_term::Style {
        use fend_core::SpanKind;

        match kind {
            SpanKind::Number => self.get_style("number").to_ansi(),
            SpanKind::String => self.get_style("string").to_ansi(),
            SpanKind::Ident => self.get_style("identifier").to_ansi(),
            SpanKind::Keyword => self.get_style("keyword").to_ansi(),
            SpanKind::BuiltInFunction => self.get_style("built_in_function").to_ansi(),
            SpanKind::Date => self.get_style("date").to_ansi(),
            _ => self.get_style("other").to_ansi(),
        }
    }
}
