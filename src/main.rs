use fend_core;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
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
                if let Some(ch) = line.as_str().chars().nth(0) {
                    if !ch.is_whitespace() {
                        rl.add_history_entry(line.as_str());
                    }
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
