#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(elided_lifetimes_in_paths)]

use std::{env, path, process};

mod color;
mod config;
mod context;
mod file_paths;
mod helper;
mod interrupt;

use context::Context;

enum EvalResult {
    Ok,
    Err,
    NoInput,
}

/// Which action should be executed?
///
/// This implements [`FromIterator`] and can be `collect`ed from
/// the [`env::args()`]`.skip(1)` iterator.
#[derive(Debug, PartialEq, Eq, Clone)]
enum ArgsAction {
    /// Print the help message (without quitting explaination).
    Help,
    /// Print the current version.
    Version,
    /// Enter the REPL.
    Repl,
    /// Evaluate the arguments.
    Eval(String),
    /// Show the default config file
    DefaultConfig,
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
    match context.eval(line, true, int) {
        Ok(res) => {
            let result: Vec<_> = res.get_main_result_spans().collect();
            if result.is_empty() || res.is_unit_type() {
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
            eprintln!("Error: {msg}");
            EvalResult::Err
        }
    }
}

fn print_help(explain_quitting: bool) {
    println!("For more information on how to use fend, please take a look at the manual:");
    println!("https://github.com/printfn/fend/wiki");
    println!();
    println!("Version: {}", fend_core::get_version());
    if let Some(config_path) = file_paths::get_config_file_location() {
        println!("Config file: {}", config_path.to_string_lossy());
    } else {
        println!("Failed to get config file location");
    }
    if let Some(history_path) = file_paths::get_history_file_location() {
        println!("History file: {}", history_path.to_string_lossy());
    } else {
        println!("Failed to get history file location");
    }
    if explain_quitting {
        println!("\nTo quit, type `quit`.");
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
            .max_history_size(config.max_history_size)
            .build(),
    );
    let core_context = std::cell::RefCell::new(fend_core::Context::new());
    if config.coulomb_and_farad {
        core_context.borrow_mut().use_coulomb_and_farad();
    }
    let mut context = Context::new(&core_context);
    rl.set_helper(Some(helper::Helper::new(context.clone(), config)));
    let history_path = file_paths::get_history_file_location();
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
                println!("Error: {err}");
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
    process::exit(real_main())
}

fn real_main() -> i32 {
    // Assemble the action from all but the first argument.
    let action: ArgsAction = env::args().skip(1).collect();
    match action {
        ArgsAction::Help => {
            print_help(false);
            0
        }
        ArgsAction::Version => {
            println!("{}", fend_core::get_version());
            0
        }
        ArgsAction::DefaultConfig => {
            println!("{}", config::DEFAULT_CONFIG_FILE);
            0
        }
        ArgsAction::Eval(expr) => {
            let config = config::read(false);
            let core_context = std::cell::RefCell::new(fend_core::Context::new());
            if config.coulomb_and_farad {
                core_context.borrow_mut().use_coulomb_and_farad();
            }
            match eval_and_print_res(
                expr.as_str(),
                &mut Context::new(&core_context),
                &interrupt::Never::default(),
                &config,
            ) {
                EvalResult::Ok | EvalResult::NoInput => 0,
                EvalResult::Err => 1,
            }
        }
        ArgsAction::Repl => {
            let config = config::read(true);
            repl_loop(&config)
        }
    }
}

impl FromIterator<String> for ArgsAction {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        iter.into_iter().fold(ArgsAction::Repl, |action, arg| {
            use ArgsAction::{DefaultConfig, Eval, Help, Repl, Version};
            match (action, arg.as_str()) {
                // If any argument is shouting for help, print help!
                (_, "help" | "--help" | "-h") | (Help, _) => Help,
                // If no help is requested, but the version, print the version
                // Once we're set on printing the version, only a request for help
                // can overwrite that
                // NOTE: 'version' is already handled by fend itself
                (Repl | Eval(_) | DefaultConfig, "--version" | "-v" | "-V") | (Version, _) => {
                    Version
                }

                (Repl | Eval(_), "--default-config") | (DefaultConfig, _) => DefaultConfig,
                // If neither help nor version is requested, evaluate the arguments
                // Ignore empty arguments, so that `$ fend "" ""` will enter the repl.
                (Repl, arg) if !arg.trim().is_empty() => Eval(String::from(arg)),
                (Repl, _) => Repl,
                (Eval(eval), arg) => Eval(eval + " " + arg),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ArgsAction;

    macro_rules! action {
        ($( $arg:literal ),*) => {
            vec![ $( $arg.to_string() ),* ]
                .into_iter()
                .collect::<ArgsAction>()
        }
    }

    #[test]
    fn help_argument_works() {
        // The --help argument wins!
        assert_eq!(ArgsAction::Help, action!["-h"]);
        assert_eq!(ArgsAction::Help, action!["--help"]);
        assert_eq!(ArgsAction::Help, action!["help"]);
        assert_eq!(ArgsAction::Help, action!["1", "+ 1", "help"]);
        assert_eq!(ArgsAction::Help, action!["--version", "1!", "--help"]);
        assert_eq!(ArgsAction::Help, action!["-h", "some", "arguments"]);
    }

    #[test]
    fn version_argument_works() {
        // --version wins over normal arguments
        assert_eq!(ArgsAction::Version, action!["-v"]);
        assert_eq!(ArgsAction::Version, action!["-V"]);
        assert_eq!(ArgsAction::Version, action!["--version"]);
        // `version` is handled by the eval
        assert_eq!(
            ArgsAction::Eval(String::from("version")),
            action!["version"]
        );
        assert_eq!(ArgsAction::Version, action!["before", "-v", "and", "after"]);
        assert_eq!(ArgsAction::Version, action!["-V", "here"]);
        assert_eq!(
            ArgsAction::Version,
            action!["--version", "-v", "+1", "version"]
        );
    }

    #[test]
    fn normal_arguments_are_collected_correctly() {
        use ArgsAction::Eval;
        assert_eq!(Eval(String::from("1 + 1")), action!["1", "+", "1"]);
        assert_eq!(Eval(String::from("1 + 1")), action!["1 + 1"]);
        assert_eq!(Eval(String::from("1 '+' 1 ")), action!["1 '+' 1 "]);
    }

    #[test]
    fn empty_arguments() {
        assert_eq!(ArgsAction::Repl, action![]);
        assert_eq!(ArgsAction::Repl, action![""]);
        assert_eq!(ArgsAction::Repl, action!["", ""]);
        assert_eq!(ArgsAction::Repl, action!["\t", " "]);
        assert_eq!(ArgsAction::Eval(String::from("1")), action!["\t", " ", "1"]);
    }
}
