#![forbid(unsafe_code)]
#![forbid(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use rustyline::error::ReadlineError;
use rustyline::Editor;

use fend_core::Context;
use std::path::PathBuf;

mod changelog;
mod config;
mod helper;
mod interrupt;

enum EvalResult {
    Ok,
    Err,
    NoInput,
}
fn eval_and_print_res(
    line: &str,
    context: &mut Context,
    int: &impl fend_core::Interrupt,
    show_other_info: bool,
) -> EvalResult {
    match fend_core::evaluate_with_interrupt(line, context, int) {
        Ok(res) => {
            let main_result = res.get_main_result();
            if main_result.is_empty() {
                return EvalResult::NoInput;
            }
            println!("{}", main_result);
            if show_other_info {
                let extra_info = res.get_other_info();
                for info in extra_info {
                    println!("-> {}", info);
                }
            }
            EvalResult::Ok
        }
        Err(msg) => {
            eprintln!("Error: {}", msg);
            EvalResult::Err
        }
    }
}

fn print_help(explain_quitting: bool) {
    println!(concat!(
        "For more information on how to use fend, ",
        "please take a look at the manual:\n",
        "https://github.com/printfn/fend-rs/wiki\n\n",
        "Version:"
    ));
    print_version();
    if explain_quitting {
        println!("\nTo quit, type `quit`.")
    }
}

fn print_changelog() {
    println!("{}", changelog::get_changelog());
}

fn print_version() {
    println!("fend v0.1.6 (2020-10-05)");
    println!("fend-core v{}", fend_core::get_extended_version());
}

fn save_history(rl: &mut Editor<helper::FendHelper>, path: &Option<PathBuf>) {
    if let Some(history_path) = path {
        if rl.save_history(history_path.as_path()).is_err() {
            // Error trying to save history
        }
    }
}

fn repl_loop() -> i32 {
    // `()` can be used when no completer is required
    let mut rl = Editor::<helper::FendHelper>::with_config(
        rustyline::config::Builder::new()
            .history_ignore_space(true)
            .auto_add_history(true)
            .max_history_size(10000)
            .build(),
    );
    let mut context = Context::new();
    rl.set_helper(Some(helper::FendHelper::new(context.clone())));
    let history_path = config::get_history_file_path();
    if let Some(history_path) = history_path.clone() {
        if rl.load_history(history_path.as_path()).is_err() {
            // No previous history
        }
    }
    let mut initial_run = true; // set to false after first successful command
    let mut last_command_success = true;
    let interrupt = interrupt::register_handler();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => match line.as_str() {
                "exit" | "quit" | ":q" => break,
                "help" | "?" => {
                    print_help(true);
                }
                "changelog" => print_changelog(),
                "version" => print_version(),
                line => {
                    interrupt.reset();
                    match eval_and_print_res(line, &mut context, &interrupt, true) {
                        EvalResult::Ok => {
                            last_command_success = true;
                            initial_run = false;
                        }
                        EvalResult::NoInput => {
                            last_command_success = true;
                        }
                        EvalResult::Err => {
                            last_command_success = false;
                        }
                    }
                }
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
        save_history(&mut rl, &history_path);
    }
    save_history(&mut rl, &history_path);
    if last_command_success {
        0
    } else {
        1
    }
}

fn main() {
    let mut args = std::env::args();
    if args.len() >= 3 {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }
    let _ = args.next();
    if let Some(expr) = args.next() {
        if expr == "help" || expr == "--help" || expr == "-h" {
            print_help(false);
            return;
        }
        if expr == "changelog" || expr == "--changelog" {
            print_changelog();
            return;
        }
        if expr == "version" || expr == "--version" || expr == "-v" || expr == "-V" {
            print_version();
            return;
        }
        std::process::exit(
            match eval_and_print_res(
                expr.as_str(),
                &mut Context::new(),
                &interrupt::NeverInterrupt::default(),
                false,
            ) {
                EvalResult::Ok | EvalResult::NoInput => 0,
                EvalResult::Err => 1,
            },
        )
    } else {
        let exit_code = repl_loop();
        std::process::exit(exit_code);
    }
}
