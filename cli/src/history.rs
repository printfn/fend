use std::{env, fs, path};

fn get_history_dir() -> Option<path::PathBuf> {
    // first try $FEND_STATE_DIR
    if let Some(env_var_history_dir) = env::var_os("FEND_STATE_DIR") {
        return Some(path::PathBuf::from(env_var_history_dir));
    }

    // otherwise try $XDG_STATE_HOME/fend/
    if let Some(env_var_xdg_state_dir) = env::var_os("XDG_STATE_HOME") {
        let mut res = path::PathBuf::from(env_var_xdg_state_dir);
        res.push("fend");
        return Some(res);
    }

    // otherwise use $HOME/.local/state/fend/
    let userdirs = directories::UserDirs::new()?;
    let home_dir = userdirs.home_dir();
    let mut res = path::PathBuf::from(home_dir);
    res.push(".local");
    res.push("state");
    res.push("fend");
    Some(res)
}

pub fn get_history_file_path() -> Option<path::PathBuf> {
    let mut history_path = get_history_dir()?;
    match fs::create_dir_all(history_path.as_path()) {
        Ok(_) => (),
        Err(_) => return None,
    }
    history_path.push("history");
    Some(history_path)
}
