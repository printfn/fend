use fend_core::DecimalSeparatorStyle;

use crate::{color, custom_units::CustomUnitDefinition};
use std::{env, fmt, fs, io};

#[derive(Debug, Eq, PartialEq)]
pub struct Config {
	pub prompt: String,
	pub enable_colors: bool,
	pub coulomb_and_farad: bool,
	pub colors: color::OutputColors,
	pub max_history_size: usize,
	pub enable_internet_access: bool,
	pub exchange_rate_source: ExchangeRateSource,
	pub exchange_rate_max_age: u64,
	pub custom_units: Vec<CustomUnitDefinition>,
	pub decimal_separator: DecimalSeparatorStyle,
	unknown_settings: UnknownSettings,
	unknown_keys: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ExchangeRateSource {
	Disabled,
	EuropeanUnion,
	UnitedNations,
}

struct ExchangeRateSourceVisitor;

impl serde::de::Visitor<'_> for ExchangeRateSourceVisitor {
	type Value = ExchangeRateSource;

	fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str("`EU`, `UN`, or `disabled`")
	}

	fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
		Ok(match v {
			"EU" => ExchangeRateSource::EuropeanUnion,
			"UN" => ExchangeRateSource::UnitedNations,
			"disabled" => ExchangeRateSource::Disabled,
			_ => {
				return Err(serde::de::Error::unknown_variant(
					v,
					&["EU", "UN", "disabled"],
				));
			}
		})
	}
}

impl<'de> serde::Deserialize<'de> for ExchangeRateSource {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_str(ExchangeRateSourceVisitor)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnknownSettings {
	Ignore,
	Warn,
}

struct ConfigVisitor;

impl<'de> serde::de::Visitor<'de> for ConfigVisitor {
	type Value = Config;

	fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str("a fend configuration struct")
	}

	#[allow(clippy::too_many_lines)]
	fn visit_map<V: serde::de::MapAccess<'de>>(self, mut map: V) -> Result<Config, V::Error> {
		let mut result = Config::default();
		let mut seen_prompt = false;
		let mut seen_enable_colors = false;
		let mut seen_coulomb_farad = false;
		let mut seen_colors = false;
		let mut seen_max_hist_size = false;
		let mut seen_enable_internet_access = false;
		let mut seen_exchange_rate_source = false;
		let mut seen_exchange_rate_max_age = false;
		let mut seen_custom_units = false;
		let mut seen_decimal_separator_style = false;
		while let Some(key) = map.next_key::<String>()? {
			match key.as_str() {
				"prompt" => {
					if seen_prompt {
						return Err(serde::de::Error::duplicate_field("prompt"));
					}
					result.prompt = map.next_value::<String>()?;
					seen_prompt = true;
				}
				"enable-colors" | "color" => {
					if seen_enable_colors {
						return Err(serde::de::Error::duplicate_field("enable-colors"));
					}
					let enable_colors: toml::Value = map.next_value()?;
					if enable_colors == false.into() || enable_colors == "never".into() {
						result.enable_colors = false;
					} else if enable_colors == true.into() || enable_colors == "auto".into() {
						result.enable_colors = use_colors_if_auto();
					} else if enable_colors == "always".into() {
						result.enable_colors = true;
					} else {
						eprintln!(
							"Error: unknown config setting for `{key}`, expected one of `'never'`, `'auto'` or `'always'`"
						);
					}
					seen_enable_colors = true;
				}
				"coulomb-and-farad" => {
					if seen_coulomb_farad {
						return Err(serde::de::Error::duplicate_field("coulomb-and-farad"));
					}
					result.coulomb_and_farad = map.next_value()?;
					seen_coulomb_farad = true;
				}
				"exchange-rate-source" => {
					if seen_exchange_rate_source {
						return Err(serde::de::Error::duplicate_field("exchange-rate-source"));
					}
					result.exchange_rate_source = map.next_value()?;
					seen_exchange_rate_source = true;
				}
				"exchange-rate-max-age" => {
					if seen_exchange_rate_max_age {
						return Err(serde::de::Error::duplicate_field("exchange-rate-max-age"));
					}
					result.exchange_rate_max_age = map.next_value()?;
					seen_exchange_rate_max_age = true;
				}
				"colors" => {
					if seen_colors {
						return Err(serde::de::Error::duplicate_field("colors"));
					}
					result.colors = map.next_value()?;
					seen_colors = true;
				}
				"max-history-size" => {
					if seen_max_hist_size {
						return Err(serde::de::Error::duplicate_field("max-history-size"));
					}
					result.max_history_size = map.next_value()?;
					seen_max_hist_size = true;
				}
				"enable-internet-access" => {
					if seen_enable_internet_access {
						return Err(serde::de::Error::duplicate_field("enable-internet-access"));
					}
					result.enable_internet_access = map.next_value()?;
					seen_enable_internet_access = true;
				}
				"unknown-settings" => {
					let unknown_settings: String = map.next_value()?;
					result.unknown_settings = match unknown_settings.as_str() {
						"ignore" => UnknownSettings::Ignore,
						"warn" => UnknownSettings::Warn,
						v => {
							return Err(serde::de::Error::invalid_value(
								serde::de::Unexpected::Str(v),
								&"`ignore` or `warn`",
							));
						}
					};
				}
				"custom-units" => {
					if seen_custom_units {
						return Err(serde::de::Error::duplicate_field("custom-units"));
					}
					result.custom_units = map.next_value()?;
					seen_custom_units = true;
				}
				"decimal-separator-style" => {
					if seen_decimal_separator_style {
						return Err(serde::de::Error::duplicate_field("decimal-separator-style"));
					}
					let style: String = map.next_value()?;
					result.decimal_separator = match style.as_str() {
						"dot" | "default" => DecimalSeparatorStyle::Dot,
						"comma" => DecimalSeparatorStyle::Comma,
						v => {
							return Err(serde::de::Error::invalid_value(
								serde::de::Unexpected::Str(v),
								&"`default`, `dot` or `comma`",
							));
						}
					};
					seen_decimal_separator_style = true;
				}
				unknown_key => {
					// this may occur if the user has multiple fend versions installed
					map.next_value::<toml::Value>()?;
					result.unknown_keys.push(unknown_key.to_string());
				}
			}
		}
		Ok(result)
	}
}

impl<'de> serde::Deserialize<'de> for Config {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		const FIELDS: &[&str] = &[
			"prompt",
			"enable-colors",
			"coulomb-and-farad",
			"colors",
			"max-history-size",
			"unknown-settings",
			"enable-internet-access",
			"custom-units",
			"decimal-separator-style",
			"exchange-rate-source",
			"exchange-rate-max-age",
		];
		deserializer.deserialize_struct("Config", FIELDS, ConfigVisitor)
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			prompt: "> ".to_string(),
			enable_colors: use_colors_if_auto(),
			coulomb_and_farad: false,
			colors: color::OutputColors::default(),
			max_history_size: 1000,
			enable_internet_access: true,
			unknown_settings: UnknownSettings::Warn,
			exchange_rate_source: ExchangeRateSource::EuropeanUnion,
			exchange_rate_max_age: 60 * 60 * 24 * 3,
			custom_units: vec![],
			decimal_separator: DecimalSeparatorStyle::Dot,
			unknown_keys: vec![],
		}
	}
}

pub static DEFAULT_CONFIG_FILE: &str = include_str!("default_config.toml");

fn read_config_file() -> Config {
	let Ok(path) = crate::file_paths::get_config_file_location() else {
		return Config::default();
	};
	let Ok(mut file) = fs::File::open(&path) else {
		return Config::default();
	};
	let mut bytes = vec![];
	let Ok(_) = <fs::File as io::Read>::read_to_end(&mut file, &mut bytes) else {
		return Config::default();
	};
	let Ok(config_string) = String::from_utf8(bytes) else {
		eprintln!("Error: config file is not UTF-8 encoded");
		return Config::default();
	};
	let config = match toml::from_str(&config_string) {
		Ok(config) => config,
		Err(e) => {
			eprintln!("Error: invalid config file in {}:\n{e}", path.display());
			eprint!("Using the default config file instead, you can view it ");
			eprintln!("by running `fend --default-config`");
			Config::default()
		}
	};
	print_warnings_about_unknown_keys(&config);
	config
}

fn print_warnings_about_unknown_keys(config: &Config) {
	match config.unknown_settings {
		UnknownSettings::Ignore => return,
		UnknownSettings::Warn => (),
	}
	for key in &config.unknown_keys {
		eprintln!("Warning: ignoring unknown configuration setting `{key}`");
	}
	config.colors.print_warnings_about_unknown_keys();
}

// if the enable-colors setting is set to 'auto', should we use colors?
fn use_colors_if_auto() -> bool {
	if cfg!(test) {
		return false;
	}
	if env::consts::OS == "windows" {
		// Colors are broken in the Windows command prompt, but do work
		// in the Windows Terminal. Disable them by default until
		// there's a better way to detect what kind of terminal we're
		// running in.
		return false;
	}
	if env::var_os("NO_COLOR").is_some() {
		return false;
	}
	if env::var_os("CLICOLOR_FORCE").unwrap_or_else(|| "0".into()) != "0" {
		return true;
	}
	if env::var_os("CLICOLOR").unwrap_or_else(|| "1".into()) != "0"
		&& crate::terminal::is_terminal_stdout()
	{
		return true;
	}
	false
}

pub fn read() -> Config {
	read_config_file()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_config_file() {
		let deserialized: Config = toml::from_str(DEFAULT_CONFIG_FILE).unwrap();
		assert_eq!(deserialized, Config::default());
	}
}
