use crate::color;
use std::{env, fmt, fs, io, path};

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
                            result.enable_colors = map.next_value()?;
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
            enable_colors: false,
            coulomb_and_farad: false,
            colors: color::OutputColors::default(),
            max_history_size: 1000,
            unknown_settings: UnknownSettings::Warn,
            unknown_keys: vec![],
        }
    }
}

pub static DEFAULT_CONFIG_FILE: &str = include_str!("default_config.toml");

fn get_config_dir() -> Option<path::PathBuf> {
    // first try $FEND_CONFIG_DIR
    env::var_os("FEND_CONFIG_DIR").map_or_else(
        || {
            // if not, then use these paths:
            // Linux: $XDG_CONFIG_HOME/fend or $HOME/.config/fend
            // macOS: $HOME/Library/Application Support/fend
            // Windows: {FOLDERID_RoamingAppData}\fend\config
            directories::ProjectDirs::from("", "", "fend")
                .map(|proj_dirs| path::PathBuf::from(proj_dirs.config_dir()))
        },
        |config_dir| Some(path::PathBuf::from(config_dir)),
    )
}

pub fn get_config_file_dir() -> Option<path::PathBuf> {
    get_config_dir().map(|mut dir| {
        dir.push("config.toml");
        dir
    })
}

fn read_config_file() -> Config {
    let path = match get_config_file_dir() {
        Some(path) => path,
        None => return Config::default(),
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
            eprintln!("Error: invalid config file in {path:?}:\n{e}");
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

pub fn read(interactive: bool) -> Config {
    let mut config = read_config_file();
    if !interactive {
        config.enable_colors = false;
    }
    if env::var_os("NO_COLOR").is_some() {
        config.enable_colors = false;
    }
    config
}

pub fn get_history_file_path() -> Option<path::PathBuf> {
    let mut config_dir = get_config_dir()?;
    match fs::create_dir_all(config_dir.as_path()) {
        Ok(_) => (),
        Err(_) => return None,
    }
    config_dir.push(".history");
    Some(config_dir)
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
