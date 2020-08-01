#![forbid(unsafe_code)]

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
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
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                match fend_core::evaluate(line.as_str()) {
                    Ok(res) => {
                        if res.is_empty() {
                            continue;
                        }
                        println!("{}", res);
                    }
                    Err(msg) => eprintln!("Error: {}", msg),
                }
            }
            Err(ReadlineError::Interrupted) => println!("Use Ctrl-D (i.e. EOF) to exit"),
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
