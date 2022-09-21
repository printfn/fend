#![deny(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(elided_lifetimes_in_paths)]

use std::fmt::Write;
use std::{
    io,
    process::{self, ExitCode},
};

mod args;
mod color;
mod config;
mod context;
mod exchange_rates;
mod file_paths;
mod helper;
mod interrupt;
mod terminal;

use args::Action as ArgsAction;
use context::Context;

enum EvalResult {
    Ok,
    Err,
    NoInput,
}

fn print_spans(spans: Vec<fend_core::SpanRef<'_>>, config: &config::Config) -> String {
    let mut result = String::new();
    for span in spans {
        let style = config.colors.get_color(span.kind());
        let styled_str = style.force_styling(true).apply_to(span.string());
        write!(result, "{styled_str}").unwrap();
    }
    result
}

fn eval_and_print_res(
    line: &str,
    context: &mut Context<'_>,
    print_res: bool,
    int: &impl fend_core::Interrupt,
    config: &config::Config,
) -> EvalResult {
    match context.eval(line, int) {
        Ok(res) => {
            let result: Vec<_> = res.get_main_result_spans().collect();
            if result.is_empty() || res.is_unit_type() {
                return EvalResult::NoInput;
            }
            if print_res {
                if config.enable_colors {
                    println!("{}", print_spans(result, config));
                } else {
                    println!("{}", res.get_main_result());
                }
            }
            EvalResult::Ok
        }
        Err(msg) => {
            eprintln!("Error: {msg}");
            EvalResult::Err
        }
    }
}

fn print_help(explain_quitting: bool) {
    println!("For more information on how to use fend, please take a look at the manual:");
    println!("https://printfn.github.io/fend/documentation/");
    println!();
    println!("Version: {}", fend_core::get_version());
    if let Ok(config_path) = file_paths::get_config_file_location() {
        println!("Config file: {}", config_path.to_string_lossy());
    } else {
        println!("Failed to get config file location");
    }
    if let Ok(history_path) = file_paths::get_history_file_location(file_paths::DirMode::DontCreate)
    {
        println!("History file: {}", history_path.to_string_lossy());
    } else {
        println!("Failed to get history file location");
    }
    if let Ok(cache_path) = file_paths::get_cache_dir(file_paths::DirMode::DontCreate) {
        println!("Cache directory: {}", cache_path.to_string_lossy());
    } else {
        println!("Failed to get cache directory location");
    }
    if explain_quitting {
        println!("\nTo quit, type `quit`.");
    }
}

fn repl_loop(config: &config::Config) -> ExitCode {
    let core_context = std::cell::RefCell::new(context::InnerCtx::new(config));
    let mut context = Context::new(&core_context);
    let mut prompt_state = match terminal::init_prompt(config, &context) {
        Ok(prompt_state) => prompt_state,
        Err(err) => {
            println!("Error: {err}");
            return ExitCode::FAILURE;
        }
    };
    let mut initial_run = true; // set to false after first successful command
    let mut last_command_success = true;
    let interrupt = interrupt::register_handler();
    loop {
        match prompt_state.read_line() {
            Ok(line) => match line.as_str() {
                "exit" | "exit()" | ".exit" | ":exit" | "quit" | "quit()" | ":quit" | ":q"
                | ":wq" | ":q!" | ":wq!" | ":qa" | ":wqa" | ":qa!" | ":wqa!" => break,
                "help" | "?" => {
                    print_help(true);
                }
                "!serialize" => match context.serialize() {
                    Ok(res) => println!("{res:?}"),
                    Err(e) => eprintln!("{e}"),
                },
                line => {
                    interrupt.reset();
                    match eval_and_print_res(line, &mut context, true, &interrupt, config) {
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
            Err(terminal::ReadLineError::Interrupted) => {
                match (initial_run, context.get_input_typed()) {
                    (_, true) => {
                        // input has been typed => do nothing
                    }
                    (true, false) => {
                        // initial run, no input => terminate
                        break;
                    }
                    (false, false) => {
                        // later run, no input => show message
                        println!("Use Ctrl-D (i.e. EOF) to exit");
                    }
                }
            }
            Err(terminal::ReadLineError::Eof) => break,
            Err(terminal::ReadLineError::Error(err)) => {
                println!("Error: {err}");
                break;
            }
        }
    }
    if last_command_success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn eval_exprs(exprs: &[String]) -> ExitCode {
    let config = config::read();
    let core_context = std::cell::RefCell::new(context::InnerCtx::new(&config));
    for (i, expr) in exprs.iter().enumerate() {
        let print_res = i == exprs.len() - 1;
        match eval_and_print_res(
            expr.as_str(),
            &mut Context::new(&core_context),
            print_res,
            &interrupt::Never::default(),
            &config,
        ) {
            EvalResult::Ok | EvalResult::NoInput => (),
            EvalResult::Err => return ExitCode::FAILURE,
        }
    }
    ExitCode::SUCCESS
}

fn real_main() -> ExitCode {
    // Assemble the action from all but the first argument.
    let action = ArgsAction::get();
    match action {
        ArgsAction::Help => {
            print_help(false);
        }
        ArgsAction::Version => {
            println!("{}", fend_core::get_version());
        }
        ArgsAction::DefaultConfig => {
            println!("{}", config::DEFAULT_CONFIG_FILE);
        }
        ArgsAction::Eval { exprs } => {
            return eval_exprs(&exprs);
        }
        ArgsAction::Repl => {
            if terminal::atty_stdin() {
                let config = config::read();
                return repl_loop(&config);
            }
            let mut input = String::new();
            match io::Read::read_to_string(&mut io::stdin(), &mut input) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            }
            return eval_exprs(&[input]);
        }
    }
    ExitCode::SUCCESS
}

fn main() -> process::ExitCode {
    real_main()
}
