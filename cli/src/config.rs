use crate::color;
use std::{env, fmt, fs, io, path};

#[derive(Debug, Eq, PartialEq)]
pub struct Config {
    pub prompt: String,
    pub enable_colors: bool,
    pub coulomb_and_farad: bool,
    pub colors: color::OutputColors,
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
                        unknown_key => {
                            return Err(serde::de::Error::unknown_field(unknown_key, FIELDS));
                        }
                    }
                }
                if !seen_prompt {
                    return Err(serde::de::Error::missing_field("prompt"));
                }
                if !seen_enable_colors {
                    return Err(serde::de::Error::missing_field("enable-colors"));
                }
                // other fields are optional
                Ok(result)
            }
        }

        const FIELDS: &[&str] = &["prompt", "enable-colors", "coulomb-and-farad", "colors"];
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
        }
    }
}

static DEFAULT_CONFIG_FILE: &str = include_str!("default_config.toml");

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
    match toml::de::from_slice(bytes.as_slice()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: invalid config file in {path:?}:\n{e}");
            eprintln!("Using default config file:\n{DEFAULT_CONFIG_FILE}");
            Config::default()
        }
    }
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
