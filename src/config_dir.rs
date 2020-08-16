use std::env::var_os;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

fn get_config_dir() -> Option<PathBuf> {
    // first try $FEND_CONFIG_DIR
    if let Some(config_dir) = var_os("FEND_CONFIG_DIR") {
        Some(PathBuf::from(config_dir))
    } else if let Some(proj_dirs) = ProjectDirs::from("", "",  "fend") {
        // Linux: $XDG_CONFIG_HOME/fend or $HOME/.config/fend
        // macOS: $HOME/Library/Application Support/fend
        // Windows: {FOLDERID_RoamingAppData}\fend\config
        Some(PathBuf::from(proj_dirs.config_dir()))
    } else {
        None
    }
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
