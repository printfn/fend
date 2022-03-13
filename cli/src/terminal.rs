use crate::{config, context, file_paths, helper};
use std::{error, path};

// contains wrapper code for terminal handling, using third-party
// libraries where necessary

pub fn atty_stdout() -> bool {
    // check if stdout is a tty (which affects whether to show colors)
    atty::is(atty::Stream::Stdout)
}

pub fn atty_stdin() -> bool {
    // check if stdin is a tty (used for whether to show an
    // interactive prompt)
    atty::is(atty::Stream::Stdin)
}

pub struct PromptState<'a> {
    rl: rustyline::Editor<helper::Helper<'a>>,
    config: &'a config::Config,
    history_path: Option<path::PathBuf>,
}

pub fn init_prompt<'a>(
    config: &'a config::Config,
    context: &context::Context<'a>,
) -> PromptState<'a> {
    let mut rl = rustyline::Editor::<helper::Helper<'_>>::with_config(
        rustyline::config::Builder::new()
            .history_ignore_space(true)
            .auto_add_history(true)
            .max_history_size(config.max_history_size)
            .build(),
    );
    rl.set_helper(Some(helper::Helper::new(context.clone(), config)));
    let history_path = file_paths::get_history_file_location();
    if let Some(history_path) = &history_path {
        if rl.load_history(history_path.as_path()).is_err() {
            // No previous history
        }
    }
    PromptState {
        rl,
        config,
        history_path,
    }
}

pub enum ReadLineError {
    Interrupted, // e.g. Ctrl-C
    Eof,
    Error(Box<dyn error::Error>),
}

fn save_history(rl: &mut rustyline::Editor<helper::Helper<'_>>, path: &Option<path::PathBuf>) {
    if let Some(history_path) = path {
        if rl.save_history(history_path.as_path()).is_err() {
            // Error trying to save history
        }
    }
}

impl PromptState<'_> {
    pub fn read_line(&mut self) -> Result<String, ReadLineError> {
        let res = self.rl.readline(self.config.prompt.as_str());
        save_history(&mut self.rl, &self.history_path);
        match res {
            Ok(line) => Ok(line),
            Err(rustyline::error::ReadlineError::Interrupted) => Err(ReadLineError::Interrupted),
            Err(rustyline::error::ReadlineError::Eof) => Err(ReadLineError::Eof),
            Err(err) => Err(ReadLineError::Error(err.into())),
        }
    }
}
