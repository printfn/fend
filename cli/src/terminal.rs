use crate::{config, context, file_paths, helper};
use std::{error, io, mem, path};

// contains wrapper code for terminal handling, using third-party
// libraries where necessary

pub fn is_terminal_stdout() -> bool {
	// check if stdout is a tty (which affects whether to show colors)
	std::io::IsTerminal::is_terminal(&std::io::stdout())
}

pub fn is_terminal_stdin() -> bool {
	// check if stdin is a tty (used for whether to show an
	// interactive prompt)
	std::io::IsTerminal::is_terminal(&std::io::stdin())
}

pub struct PromptState<'a> {
	rl: rustyline::Editor<helper::Helper<'a>, rustyline::history::FileHistory>,
	config: &'a config::Config,
	history_path: Option<path::PathBuf>,
}

pub fn init_prompt<'a>(
	config: &'a config::Config,
	context: &context::Context<'a>,
) -> Result<PromptState<'a>, Box<dyn error::Error>> {
	let mut rl =
		rustyline::Editor::<helper::Helper<'_>, rustyline::history::FileHistory>::with_config(
			rustyline::config::Builder::new()
				.history_ignore_space(true)
				.auto_add_history(true)
				.max_history_size(config.max_history_size)?
				.build(),
		)?;
	rl.set_helper(Some(helper::Helper::new(context.clone(), config)));
	let history_path = match file_paths::get_history_file_location(file_paths::DirMode::DontCreate)
	{
		Ok(history_path) => {
			// ignore error if e.g. no history file exists
			mem::drop(rl.load_history(history_path.as_path()));
			Some(history_path)
		}
		Err(_) => None,
	};
	Ok(PromptState {
		rl,
		config,
		history_path,
	})
}

pub enum ReadLineError {
	Interrupted, // e.g. Ctrl-C
	Eof,
	Error(Box<dyn error::Error>),
}

impl From<rustyline::error::ReadlineError> for ReadLineError {
	fn from(err: rustyline::error::ReadlineError) -> Self {
		match err {
			rustyline::error::ReadlineError::Interrupted => ReadLineError::Interrupted,
			rustyline::error::ReadlineError::Eof => ReadLineError::Eof,
			err => ReadLineError::Error(err.into()),
		}
	}
}

fn save_history(
	rl: &mut rustyline::Editor<helper::Helper<'_>, rustyline::history::FileHistory>,
	path: &Option<path::PathBuf>,
) -> io::Result<()> {
	if let Some(history_path) = path {
		file_paths::get_state_dir(file_paths::DirMode::Create)?;
		if rl.save_history(history_path.as_path()).is_err() {
			// Error trying to save history
		}
	}
	Ok(())
}

impl PromptState<'_> {
	pub fn read_line(&mut self) -> Result<String, ReadLineError> {
		let res = self.rl.readline(self.config.prompt.as_str());
		// ignore errors when saving history
		mem::drop(save_history(&mut self.rl, &self.history_path));
		Ok(res?)
	}
}
