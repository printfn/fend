use std::{env, error, fmt, fs, io, path};

#[derive(Debug)]
pub struct HomeDirError;

impl fmt::Display for HomeDirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to find home directory")
    }
}

impl error::Error for HomeDirError {}

impl From<HomeDirError> for io::Error {
    fn from(e: HomeDirError) -> Self {
        Self::new(io::ErrorKind::Other, e)
    }
}

fn get_home_dir() -> Result<path::PathBuf, HomeDirError> {
    let home_dir = match home::home_dir() {
        Some(home_dir) => home_dir,
        None => return Err(HomeDirError),
    };
    Ok(home_dir)
}

fn get_config_dir() -> Result<path::PathBuf, HomeDirError> {
    // first try $FEND_CONFIG_DIR
    if let Some(env_var_config_dir) = env::var_os("FEND_CONFIG_DIR") {
        return Ok(path::PathBuf::from(env_var_config_dir));
    }

    // otherwise try $XDG_CONFIG_HOME/fend/
    if let Some(env_var_xdg_config_dir) = env::var_os("XDG_CONFIG_HOME") {
        let mut res = path::PathBuf::from(env_var_xdg_config_dir);
        res.push("fend");
        return Ok(res);
    }

    // otherwise use $HOME/.config/fend/
    let mut res = get_home_dir()?;
    res.push(".config");
    res.push("fend");
    Ok(res)
}

pub fn get_config_file_location() -> Result<path::PathBuf, HomeDirError> {
    let mut config_path = get_config_dir()?;
    config_path.push("config.toml");
    Ok(config_path)
}

fn get_state_dir() -> Result<path::PathBuf, HomeDirError> {
    // first try $FEND_STATE_DIR
    if let Some(env_var_history_dir) = env::var_os("FEND_STATE_DIR") {
        return Ok(path::PathBuf::from(env_var_history_dir));
    }

    // otherwise try $XDG_STATE_HOME/fend/
    if let Some(env_var_xdg_state_dir) = env::var_os("XDG_STATE_HOME") {
        let mut res = path::PathBuf::from(env_var_xdg_state_dir);
        res.push("fend");
        return Ok(res);
    }

    // otherwise use $HOME/.local/state/fend/
    let mut res = get_home_dir()?;
    res.push(".local");
    res.push("state");
    res.push("fend");
    Ok(res)
}

pub fn create_state_dir() -> io::Result<()> {
    let state_dir = get_state_dir()?;
    fs::create_dir_all(state_dir)?;
    Ok(())
}

pub fn get_history_file_location() -> Result<path::PathBuf, HomeDirError> {
    let mut history_path = get_state_dir()?;
    history_path.push("history");
    Ok(history_path)
}
