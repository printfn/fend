#![forbid(unsafe_code)]
// enable almost all clippy warnings
#![forbid(clippy::all)]
#![forbid(clippy::pedantic)]
#![forbid(clippy::nursery)]
#![deny(clippy::restriction)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::implicit_return)]
#![allow(clippy::print_stdout)]
#![allow(clippy::exit)]

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod config_dir;

fn repl_loop() -> i32 {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::with_config(
        rustyline::config::Builder::new()
            .history_ignore_space(true)
            .auto_add_history(true)
            .max_history_size(10000)
            .build(),
    );
    let history_path = config_dir::get_history_file_path();
    if let Some(history_path) = history_path.clone() {
        if rl.load_history(history_path.as_path()).is_err() {
            // No previous history
        }
    }
    let mut context = fend_core::Context::new();
    let mut initial_run = true; // set to false after first successful command
    let mut last_command_success = true;
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => match line.as_str() {
                "exit" | "quit" | ":q" => break,
                line => match fend_core::evaluate(line, &mut context) {
                    Ok(res) => {
                        last_command_success = true;
                        let main_result = res.get_main_result();
                        if main_result.is_empty() {
                            continue;
                        }
                        println!("{}", main_result);
                        let extra_info = res.get_other_info();
                        for info in extra_info {
                            println!("-> {}", info);
                        }
                        initial_run = false;
                    }
                    Err(msg) => {
                        last_command_success = false;
                        eprintln!("Error: {}", msg);
                    }
                },
            },
            Err(ReadlineError::Interrupted) => {
                if initial_run {
                    break;
                } else {
                    println!("Use Ctrl-D (i.e. EOF) to exit");
                }
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
    if let Some(history_path) = history_path {
        if rl.save_history(history_path.as_path()).is_err() {
            // Error trying to save history
        }
    }
    if last_command_success {
        0
    } else {
        1
    }
}

fn main() {
    let exit_code = repl_loop();
    std::process::exit(exit_code);
}
