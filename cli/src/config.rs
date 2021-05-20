use crate::color;
use std::{env, fs, io, path};

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    pub prompt: String,
    #[serde(alias = "color")]
    pub enable_colors: bool,
    #[serde(default)]
    pub colors: color::OutputColours,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompt: "> ".to_string(),
            enable_colors: false,
            colors: color::OutputColours::default(),
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
    if let Ok(config) = toml::de::from_slice(bytes.as_slice()) {
        config
    } else {
        eprintln!("Invalid config file in {:?}", &path);
        eprintln!("Using default config file:\n{}", DEFAULT_CONFIG_FILE);
        Config::default()
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
        assert_eq!(deserialized, Default::default());
    }
}
