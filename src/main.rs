#![forbid(unsafe_code)]

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn repl_loop() -> i32 {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::with_config(
        rustyline::config::Builder::new()
            .history_ignore_space(true)
            .auto_add_history(true)
            .build(),
    );
    if rl.load_history("history.txt").is_err() {
        // No previous history
    }
    let mut initial_run = true; // set to false after first successful command
    let mut last_command_success = true;
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => match line.as_str() {
                "exit" | "quit" | ":q" => break,
                line => match fend_core::evaluate(line) {
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
                    },
                },
            },
            Err(ReadlineError::Interrupted) => {
                if initial_run {
                    break;
                } else {
                    println!("Use Ctrl-D (i.e. EOF) to exit");
                }
            },
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
    if last_command_success { 0 } else { 1 }
}

fn main() {
    let exit_code = repl_loop();
    std::process::exit(exit_code);
}
