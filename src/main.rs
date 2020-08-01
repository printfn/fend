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
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                if line.as_str().is_empty() {
                    continue;
                }
                match fend_core::evaluate(line.as_str()) {
                    Ok(res) => println!("{}", res),
                    Err(msg) => println!("Error: {}", msg),
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
