use std::env::var_os;
use std::fs;
use std::path::PathBuf;

fn get_config_dir() -> Option<PathBuf> {
    // first try $FEND_CONFIG_DIR
    if let Some(config_dir) = var_os("FEND_CONFIG_DIR") {
        return Some(PathBuf::from(config_dir));
    }
    // Next, look for XDG_CONFIG_HOME
    if let Some(config_dir) = var_os("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(config_dir);
        path.push("fend");
        return Some(path);
    }
    // Otherwise use $HOME/.config/fend
    if let Some(home_dir) = var_os("HOME") {
        let mut path = PathBuf::from(home_dir);
        path.push(".config");
        path.push("fend");
        return Some(path);
    }
    // Otherwise return None
    None
}

pub fn get_history_file_path() -> Option<PathBuf> {
    let mut config_dir = get_config_dir()?;
    match fs::create_dir_all(config_dir.as_path()) {
        Ok(_) => (),
        Err(_) => return None,
    }
    config_dir.push(".history");
    Some(config_dir)
}
