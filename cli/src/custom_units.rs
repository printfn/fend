use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug, Hash, Default)]
pub enum CustomUnitAttribute {
	#[default]
	None,
	AllowLongPrefix,
	AllowShortPrefix,
	IsLongPrefix,
	Alias,
}

struct CustomUnitAttributeVisitor;

impl serde::de::Visitor<'_> for CustomUnitAttributeVisitor {
	type Value = CustomUnitAttribute;

	fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str(
			"`none`, `allow-long-prefix`, `allow-short-prefix`, `is-long-prefix` or `alias`",
		)
	}

	fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
		Ok(match v {
			"none" => CustomUnitAttribute::None,
			"allow-long-prefix" => CustomUnitAttribute::AllowLongPrefix,
			"allow-short-prefix" => CustomUnitAttribute::AllowShortPrefix,
			"is-long-prefix" => CustomUnitAttribute::IsLongPrefix,
			"alias" => CustomUnitAttribute::Alias,
			unknown => {
				return Err(serde::de::Error::unknown_variant(
					unknown,
					&[
						"none",
						"allow-long-prefix",
						"allow-short-prefix",
						"is-long-prefix",
						"alias",
					],
				))
			}
		})
	}
}

impl<'de> serde::Deserialize<'de> for CustomUnitAttribute {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_str(CustomUnitAttributeVisitor)
	}
}

impl CustomUnitAttribute {
	pub fn to_fend_core(&self) -> fend_core::CustomUnitAttribute {
		match self {
			CustomUnitAttribute::None => fend_core::CustomUnitAttribute::None,
			CustomUnitAttribute::AllowLongPrefix => fend_core::CustomUnitAttribute::AllowLongPrefix,
			CustomUnitAttribute::AllowShortPrefix => {
				fend_core::CustomUnitAttribute::AllowShortPrefix
			}
			CustomUnitAttribute::IsLongPrefix => fend_core::CustomUnitAttribute::IsLongPrefix,
			CustomUnitAttribute::Alias => fend_core::CustomUnitAttribute::Alias,
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct CustomUnitDefinition {
	pub singular: String,
	pub plural: String,
	pub definition: String,
	pub attribute: CustomUnitAttribute,
}

impl<'de> serde::Deserialize<'de> for CustomUnitDefinition {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		const FIELDS: &[&str] = &["singular", "plural", "definition", "attribute"];

		struct CustomUnitDefinitionVisitor;

		impl<'de> serde::de::Visitor<'de> for CustomUnitDefinitionVisitor {
			type Value = CustomUnitDefinition;

			fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
				formatter.write_str("a custom unit definition, with properties `singular`, `plural`, `definition` and `attribute`")
			}

			fn visit_map<V: serde::de::MapAccess<'de>>(
				self,
				mut map: V,
			) -> Result<CustomUnitDefinition, V::Error> {
				let mut result = CustomUnitDefinition {
					attribute: CustomUnitAttribute::None,
					definition: String::new(),
					singular: String::new(),
					plural: String::new(),
				};
				let mut seen_singular = false;
				let mut seen_plural = false;
				let mut seen_definition = false;
				let mut seen_attribute = false;
				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"singular" => {
							if seen_singular {
								return Err(serde::de::Error::duplicate_field("singular"));
							}
							result.singular = map.next_value()?;
							seen_singular = true;
						}
						"plural" => {
							if seen_plural {
								return Err(serde::de::Error::duplicate_field("plural"));
							}
							result.plural = map.next_value()?;
							if result.plural.is_empty() {
								return Err(serde::de::Error::invalid_value(
									serde::de::Unexpected::Str(&result.plural),
									&"a non-empty string describing the plural form of this unit",
								));
							}
							seen_plural = true;
						}
						"definition" => {
							if seen_definition {
								return Err(serde::de::Error::duplicate_field("definition"));
							}
							result.definition = map.next_value()?;
							if result.definition.is_empty() {
								return Err(serde::de::Error::invalid_value(
									serde::de::Unexpected::Str(&result.definition),
									&"a non-empty string that contains the definition of this unit",
								));
							}
							seen_definition = true;
						}
						"attribute" => {
							if seen_attribute {
								return Err(serde::de::Error::duplicate_field("attribute"));
							}
							result.attribute = map.next_value()?;
							seen_attribute = true;
						}
						unknown_key => {
							map.next_value::<toml::Value>()?;
							return Err(serde::de::Error::unknown_field(unknown_key, FIELDS));
						}
					}
				}
				if result.singular.is_empty() {
					return Err(serde::de::Error::missing_field("singular"));
				}
				if result.definition.is_empty() {
					return Err(serde::de::Error::missing_field("definition"));
				}
				Ok(result)
			}
		}

		deserializer.deserialize_struct("CustomUnitDefinition", FIELDS, CustomUnitDefinitionVisitor)
	}
}
