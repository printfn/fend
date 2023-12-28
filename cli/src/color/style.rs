use super::base::Base;
use std::fmt::{self, Write};

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Color {
	foreground: Option<Base>,
	underline: bool,
	bold: bool,
	unknown_keys: Vec<String>,
}

impl<'de> serde::Deserialize<'de> for Color {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		struct ColorVisitor;

		impl<'de> serde::de::Visitor<'de> for ColorVisitor {
			type Value = Color;

			fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
				formatter.write_str("a color, with properties `foreground`, `underline` and `bold`")
			}

			fn visit_map<V: serde::de::MapAccess<'de>>(
				self,
				mut map: V,
			) -> Result<Color, V::Error> {
				let mut result = Color::default();
				let mut seen_foreground = false;
				let mut seen_underline = false;
				let mut seen_bold = false;
				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"foreground" => {
							if seen_foreground {
								return Err(serde::de::Error::duplicate_field("foreground"));
							}
							result.foreground = Some(map.next_value()?);
							seen_foreground = true;
						}
						"underline" => {
							if seen_underline {
								return Err(serde::de::Error::duplicate_field("underline"));
							}
							result.underline = map.next_value()?;
							seen_underline = true;
						}
						"bold" => {
							if seen_bold {
								return Err(serde::de::Error::duplicate_field("bold"));
							}
							result.bold = map.next_value()?;
							seen_bold = true;
						}
						unknown_key => {
							map.next_value::<toml::Value>()?;
							result.unknown_keys.push(unknown_key.to_string());
						}
					}
				}
				Ok(result)
			}
		}

		const FIELDS: &[&str] = &["foreground", "underline", "bold"];
		deserializer.deserialize_struct("Color", FIELDS, ColorVisitor)
	}
}

impl Color {
	pub fn new(foreground: Base) -> Self {
		Self {
			foreground: Some(foreground),
			underline: false,
			bold: false,
			unknown_keys: vec![],
		}
	}

	pub fn bold(foreground: Base) -> Self {
		Self {
			bold: true,
			..Self::new(foreground)
		}
	}

	pub fn to_ansi(&self) -> String {
		let mut result = "\x1b[".to_string();
		if self.underline {
			result.push_str("4;");
		}
		if self.bold {
			result.push_str("1;");
		}
		if let Some(foreground) = &self.foreground {
			match foreground {
				Base::Black => result.push_str("30"),
				Base::Red => result.push_str("31"),
				Base::Green => result.push_str("32"),
				Base::Yellow => result.push_str("33"),
				Base::Blue => result.push_str("34"),
				Base::Magenta => result.push_str("35"),
				Base::Cyan => result.push_str("36"),
				Base::White => result.push_str("37"),
				Base::Color256(n) => {
					result.push_str("38;5;");
					write!(result, "{n}").unwrap();
					result.push_str(&n.to_string());
				}
				Base::Unknown(_) => result.push_str("39"),
			}
		} else {
			result.push_str("39");
		}
		result.push('m');
		result
	}

	pub fn print_warnings_about_unknown_keys(&self, style_assignment: &str) {
		for key in &self.unknown_keys {
			eprintln!(
				"Warning: ignoring unknown configuration setting `colors.{style_assignment}.{key}`"
			);
		}
		if let Some(base) = &self.foreground {
			base.warn_about_unknown_colors();
		}
	}
}
