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
    Eval(String),
    /// Show the default config file
    DefaultConfig,
}

impl FromIterator<String> for Action {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        iter.into_iter().fold(Action::Repl, |action, arg| {
            use Action::{DefaultConfig, Eval, Help, Repl, Version};
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
    use super::Action;

    macro_rules! action {
        ($( $arg:literal ),*) => {
            vec![ $( $arg.to_string() ),* ]
                .into_iter()
                .collect::<Action>()
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
        assert_eq!(Action::Eval(String::from("version")), action!["version"]);
        assert_eq!(Action::Version, action!["before", "-v", "and", "after"]);
        assert_eq!(Action::Version, action!["-V", "here"]);
        assert_eq!(Action::Version, action!["--version", "-v", "+1", "version"]);
    }

    #[test]
    fn normal_arguments_are_collected_correctly() {
        use Action::Eval;
        assert_eq!(Eval(String::from("1 + 1")), action!["1", "+", "1"]);
        assert_eq!(Eval(String::from("1 + 1")), action!["1 + 1"]);
        assert_eq!(Eval(String::from("1 '+' 1 ")), action!["1 '+' 1 "]);
    }

    #[test]
    fn empty_arguments() {
        assert_eq!(Action::Repl, action![]);
        assert_eq!(Action::Repl, action![""]);
        assert_eq!(Action::Repl, action!["", ""]);
        assert_eq!(Action::Repl, action!["\t", " "]);
        assert_eq!(Action::Eval(String::from("1")), action!["\t", " ", "1"]);
    }
}
