use crate::color;
use std::{env, fmt, fs, io};

#[derive(Debug, Eq, PartialEq)]
pub struct Config {
    pub prompt: String,
    pub enable_colors: bool,
    pub coulomb_and_farad: bool,
    pub colors: color::OutputColors,
    pub max_history_size: usize,
    unknown_settings: UnknownSettings,
    unknown_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnknownSettings {
    Ignore,
    Warn,
}

impl<'de> serde::Deserialize<'de> for Config {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ConfigVisitor;

        impl<'de> serde::de::Visitor<'de> for ConfigVisitor {
            type Value = Config;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a fend configuration struct")
            }

            fn visit_map<V: serde::de::MapAccess<'de>>(
                self,
                mut map: V,
            ) -> Result<Config, V::Error> {
                let mut result = Config::default();
                let mut seen_prompt = false;
                let mut seen_enable_colors = false;
                let mut seen_coulomb_farad = false;
                let mut seen_colors = false;
                let mut seen_max_hist_size = false;
                while let Some(key) = map.next_key()? {
                    match key {
                        "prompt" => {
                            if seen_prompt {
                                return Err(serde::de::Error::duplicate_field("prompt"));
                            }
                            result.prompt = map.next_value()?;
                            seen_prompt = true;
                        }
                        "enable-colors" | "color" => {
                            if seen_enable_colors {
                                return Err(serde::de::Error::duplicate_field("enable-colors"));
                            }
                            let enable_colors: toml::Value = map.next_value()?;
                            if enable_colors == false.into() || enable_colors == "never".into() {
                                result.enable_colors = false;
                            } else if enable_colors == true.into() || enable_colors == "auto".into()
                            {
                                result.enable_colors = use_colors_if_auto();
                            } else if enable_colors == "always".into() {
                                result.enable_colors = true;
                            } else {
                                eprintln!("Error: unknown config setting for `{}`, expected one of `'never'`, `'auto'` or `'always'`", key);
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
                        "unknown-settings" => {
                            let unknown_settings: &str = map.next_value()?;
                            result.unknown_settings = match unknown_settings {
                                "ignore" => UnknownSettings::Ignore,
                                "warn" => UnknownSettings::Warn,
                                v => {
                                    return Err(serde::de::Error::invalid_value(
                                        serde::de::Unexpected::Str(v),
                                        &"`ignore` or `warn`",
                                    ))
                                }
                            };
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

        const FIELDS: &[&str] = &[
            "prompt",
            "enable-colors",
            "coulomb-and-farad",
            "colors",
            "max-history-size",
            "unknown-settings",
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
            unknown_settings: UnknownSettings::Warn,
            unknown_keys: vec![],
        }
    }
}

pub static DEFAULT_CONFIG_FILE: &str = include_str!("default_config.toml");

fn read_config_file() -> Config {
    let path = match crate::file_paths::get_config_file_location() {
        Ok(path) => path,
        Err(_) => return Config::default(),
    };
    let mut file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(_) => return Config::default(),
    };
    let mut bytes = vec![];
    match <fs::File as io::Read>::read_to_end(&mut file, &mut bytes) {
        Ok(_) => (),
        Err(_) => return Config::default(),
    }
    let config = match toml::de::from_slice(bytes.as_slice()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: invalid config file in {:?}:\n{}", path, e);
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
        eprintln!("Warning: ignoring unknown configuration setting `{}`", key);
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
        && crate::terminal::atty_stdout()
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
        let deserialized: Config = toml::de::from_str(DEFAULT_CONFIG_FILE).unwrap();
        assert_eq!(deserialized, Config::default());
    }
}
