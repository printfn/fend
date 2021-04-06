use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::env::var_os;
use std::{fs, io, path};

#[derive(Deserialize, Serialize)]
pub struct ConfigOptions {
    pub prompt: String,
    pub color: bool,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            prompt: "> ".to_string(),
            color: false,
        }
    }
}

fn get_config_dir() -> Option<path::PathBuf> {
    // first try $FEND_CONFIG_DIR
    var_os("FEND_CONFIG_DIR").map_or_else(
        || {
            // if not, then use these paths:
            // Linux: $XDG_CONFIG_HOME/fend or $HOME/.config/fend
            // macOS: $HOME/Library/Application Support/fend
            // Windows: {FOLDERID_RoamingAppData}\fend\config
            ProjectDirs::from("", "", "fend")
                .map(|proj_dirs| path::PathBuf::from(proj_dirs.config_dir()))
        },
        |config_dir| Some(path::PathBuf::from(config_dir)),
    )
}

enum ReadConfigErr {
    FileReadingError(io::Error),
    DeserializationError(toml::de::Error),
}

impl From<io::Error> for ReadConfigErr {
    fn from(err: io::Error) -> Self {
        Self::FileReadingError(err)
    }
}

impl From<toml::de::Error> for ReadConfigErr {
    fn from(err: toml::de::Error) -> Self {
        Self::DeserializationError(err)
    }
}

fn read_config_file() -> Result<ConfigOptions, ReadConfigErr> {
    let mut path = match get_config_dir() {
        Some(path) => path,
        None => return Ok(ConfigOptions::default()),
    };
    path.push("config.toml");
    let mut file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(_) => return Ok(ConfigOptions::default()),
    };
    let mut bytes = vec![];
    match <fs::File as io::Read>::read_to_end(&mut file, &mut bytes) {
        Ok(_) => (),
        Err(_) => return Ok(ConfigOptions::default()),
    }
    Ok(match toml::de::from_slice(bytes.as_slice()) {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Invalid config file in {:?}", &path);
            let default_config = ConfigOptions::default();
            match toml::ser::to_string_pretty(&default_config) {
                Ok(s) => eprintln!("Using default config file:\n{}", s),
                Err(_) => (),
            }
            default_config
        }
    })
}

pub fn read_config(interactive: bool) -> ConfigOptions {
    let mut config = match read_config_file() {
        Ok(config) => config,
        Err(_) => ConfigOptions::default(),
    };
    if !interactive {
        config.color = false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        config.color = false;
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
