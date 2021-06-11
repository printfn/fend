#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(elided_lifetimes_in_paths)]

use std::{env, mem, path, process};

mod color;
mod config;
mod context;
mod helper;
mod interrupt;

use context::Context;

enum EvalResult {
    Ok,
    Err,
    NoInput,
}

fn print_spans(spans: Vec<fend_core::SpanRef<'_>>, config: &config::Config) -> String {
    let mut strings = vec![];
    for span in spans {
        let style = config.colors.get_color(span.kind());
        strings.push(style.paint(span.string()));
    }
    ansi_term::ANSIStrings(strings.as_slice()).to_string()
}

fn eval_and_print_res(
    line: &str,
    context: &mut Context<'_>,
    int: &impl fend_core::Interrupt,
    config: &config::Config,
) -> EvalResult {
    /*
    let ms_since_1970 = chrono::Utc::now().timestamp_millis();
    let tz_offset_secs = chrono::Local::now().offset().local_minus_utc();
    context.set_current_time_v1(convert::TryInto::try_into(ms_since_1970).unwrap(), tz_offset_secs.into());
    */
    match context.eval(line, true, int) {
        Ok(res) => {
            let result: Vec<_> = res.get_main_result_spans().collect();
            if result.is_empty() {
                return EvalResult::NoInput;
            }
            if config.enable_colors {
                println!("{}", print_spans(result, config));
            } else {
                println!("{}", res.get_main_result());
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
    println!(
        concat!(
            "For more information on how to use fend, ",
            "please take a look at the manual:\n",
            "https://github.com/printfn/fend/wiki\n\n",
            "Version: {}{}",
        ),
        fend_core::get_version(),
        config::get_config_file_dir().map_or_else(
            || "Failed to get config file location".to_string(),
            |d| format!("\nConfig file: {}", d.to_string_lossy())
        )
    );
    if explain_quitting {
        println!("\nTo quit, type `quit`.")
    }
}

fn save_history(rl: &mut rustyline::Editor<helper::Helper<'_>>, path: &Option<path::PathBuf>) {
    if let Some(history_path) = path {
        if rl.save_history(history_path.as_path()).is_err() {
            // Error trying to save history
        }
    }
}

fn repl_loop(config: &config::Config) -> i32 {
    // `()` can be used when no completer is required
    let mut rl = rustyline::Editor::<helper::Helper<'_>>::with_config(
        rustyline::config::Builder::new()
            .history_ignore_space(true)
            .auto_add_history(true)
            .max_history_size(10000)
            .build(),
    );
    let core_context = std::cell::RefCell::new(fend_core::Context::new());
    let mut context = Context::new(&core_context);
    rl.set_helper(Some(helper::Helper::new(context.clone(), &config)));
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
        let readline = rl.readline(&config.prompt);
        match readline {
            Ok(line) => match line.as_str() {
                "exit" | "exit()" | ".exit" | ":exit" | "quit" | "quit()" | ":quit" | ":q"
                | ":wq" | ":q!" | ":wq!" | ":qa" | ":wqa" | ":qa!" | ":wqa!" => break,
                "help" | "?" => {
                    print_help(true);
                }
                line => {
                    interrupt.reset();
                    match eval_and_print_res(line, &mut context, &interrupt, config) {
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
            Err(rustyline::error::ReadlineError::Interrupted) => {
                if initial_run {
                    break;
                }
                println!("Use Ctrl-D (i.e. EOF) to exit");
            }
            Err(rustyline::error::ReadlineError::Eof) => break,
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
    let mut args = env::args();
    if args.len() >= 3 {
        eprintln!("Too many arguments");
        process::exit(1);
    }
    mem::drop(args.next());
    if let Some(expr) = args.next() {
        if expr == "help" || expr == "--help" || expr == "-h" {
            print_help(false);
            return;
        }
        // 'version' is already handled by fend itself
        if expr == "--version" || expr == "-v" || expr == "-V" {
            println!("{}", fend_core::get_version());
            return;
        }
        let config = config::read(false);
        let core_context = std::cell::RefCell::new(fend_core::Context::new());
        process::exit(
            match eval_and_print_res(
                expr.as_str(),
                &mut Context::new(&core_context),
                &interrupt::Never::default(),
                &config,
            ) {
                EvalResult::Ok | EvalResult::NoInput => 0,
                EvalResult::Err => 1,
            },
        )
    } else {
        let config = config::read(true);
        let exit_code = repl_loop(&config);
        process::exit(exit_code);
    }
}
