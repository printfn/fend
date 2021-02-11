use directories::ProjectDirs;
use std::env::var_os;
use std::fs;
use std::path::PathBuf;

fn get_config_dir() -> Option<PathBuf> {
    // first try $FEND_CONFIG_DIR
    var_os("FEND_CONFIG_DIR").map_or_else(
        || {
            // if not, then use these paths:
            // Linux: $XDG_CONFIG_HOME/fend or $HOME/.config/fend
            // macOS: $HOME/Library/Application Support/fend
            // Windows: {FOLDERID_RoamingAppData}\fend\config
            ProjectDirs::from("", "", "fend").map(|proj_dirs| PathBuf::from(proj_dirs.config_dir()))
        },
        |config_dir| Some(PathBuf::from(config_dir)),
    )
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
