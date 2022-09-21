use std::{env, fs};

/// Which action should be executed?
///
/// This implements [`FromIterator`] and can be `collect`ed from
/// the [`env::args()`]`.skip(1)` iterator.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    /// Print the help message (without quitting explaination).
    Help,
    /// Print the current version.
    Version,
    /// Enter the REPL.
    Repl,
    /// Evaluate the arguments.
    Eval { exprs: Vec<String> },
    /// Show the default config file
    DefaultConfig,
}

impl Action {
    pub fn from_args<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let mut print_help = false;
        let mut print_version = false;
        let mut print_default_config = false;
        let mut before_double_dash = true;
        let mut exprs = vec![];
        let mut expr = String::new();

        for arg in iter {
            match (before_double_dash, arg.as_str()) {
                (true, "help" | "--help" | "-h") => print_help = true,
                // NOTE: 'version' is already handled by fend itself
                (true, "--version" | "-v" | "-V") => print_version = true,
                (true, "--default-config") => print_default_config = true,
                (true, "--") => before_double_dash = false,
                (_, arg) => {
                    if before_double_dash {
                        if let Ok(contents) = fs::read_to_string(arg) {
                            if !expr.is_empty() {
                                exprs.push(expr);
                                expr = String::new();
                            }
                            exprs.push(contents);
                            continue;
                        }
                    }
                    if arg.trim().is_empty() {
                        continue;
                    }
                    if !expr.is_empty() {
                        expr.push(' ');
                    }
                    expr.push_str(arg);
                }
            }
        }

        if print_help {
            // If any argument is shouting for help, print help!
            Self::Help
        } else if print_version {
            // If no help is requested, but the version, print the version
            Self::Version
        } else if print_default_config {
            Self::DefaultConfig
        } else if exprs.is_empty() && expr.is_empty() {
            Self::Repl
        } else {
            // If neither help nor version is requested, evaluate the arguments
            if !expr.is_empty() {
                exprs.push(expr);
            }
            Self::Eval { exprs }
        }
    }

    pub fn get() -> Self {
        Self::from_args(env::args().skip(1))
    }
}

#[cfg(test)]
mod tests {
    use super::Action;

    macro_rules! action {
        ($( $arg:literal ),*) => {
            Action::from_args(vec![ $( $arg.to_string() ),* ])
        }
    }

    fn eval(expr: &str) -> Action {
        Action::Eval {
            exprs: vec![expr.to_string()],
        }
    }

    #[test]
    fn help_argument_works() {
        // The --help argument wins!
        assert_eq!(Action::Help, action!["-h"]);
        assert_eq!(Action::Help, action!["--help"]);
        assert_eq!(Action::Help, action!["help"]);
        assert_eq!(Action::Help, action!["1", "+ 1", "help"]);
        assert_eq!(Action::Help, action!["--version", "1!", "--help"]);
        assert_eq!(Action::Help, action!["-h", "some", "arguments"]);
    }

    #[test]
    fn version_argument_works() {
        // --version wins over normal arguments
        assert_eq!(Action::Version, action!["-v"]);
        assert_eq!(Action::Version, action!["-V"]);
        assert_eq!(Action::Version, action!["--version"]);
        // `version` is handled by the eval
        assert_eq!(eval("version"), action!["version"]);
        assert_eq!(Action::Version, action!["before", "-v", "and", "after"]);
        assert_eq!(Action::Version, action!["-V", "here"]);
        assert_eq!(Action::Version, action!["--version", "-v", "+1", "version"]);
    }

    #[test]
    fn normal_arguments_are_collected_correctly() {
        assert_eq!(eval("1 + 1"), action!["1", "+", "1"]);
        assert_eq!(eval("1 + 1"), action!["1 + 1"]);
        assert_eq!(eval("1 '+' 1 "), action!["1 '+' 1 "]);
    }

    #[test]
    fn empty_arguments() {
        assert_eq!(Action::Repl, action![]);
        assert_eq!(Action::Repl, action![""]);
        assert_eq!(Action::Repl, action!["", ""]);
        assert_eq!(Action::Repl, action!["\t", " "]);
        assert_eq!(eval("1"), action!["\t", " ", "1"]);
    }
}
