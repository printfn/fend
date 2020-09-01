#![forbid(unsafe_code)]
#![forbid(clippy::all)]
#![deny(clippy::pedantic)]
#![doc(html_root_url = "https://docs.rs/fend-core/0.1.1")]

mod ast;
mod interrupt;
mod lexer;
mod num;
mod parser;
mod value;

use interrupt::Interrupt;
use std::collections::HashMap;
use value::Value;

/// This contains the result of a computation.
#[derive(PartialEq, Eq, Debug)]
pub struct FendResult {
    main_result: String,
    other_info: Vec<String>,
}

impl FendResult {
    /// This retrieves the main result of the computation.
    #[must_use]
    pub fn get_main_result(&self) -> &str {
        self.main_result.as_str()
    }

    /// This retrieves a list of other results of the computation. It is less
    /// stable than the main result, and should only be shown for when used
    /// interactively.
    pub fn get_other_info(&self) -> impl Iterator<Item = &str> {
        self.other_info.iter().map(std::string::String::as_str)
    }
}

fn evaluate_to_value(
    input: &str,
    scope: &HashMap<String, Value>,
    int: &impl Interrupt,
) -> Result<Value, String> {
    let parsed = parser::parse_string(input, int)?;
    let result = ast::evaluate(parsed, scope, int)?;
    Ok(result)
}

/// This struct contains context used for `fend`. It should only be created once
/// at startup.
#[derive(Clone)]
pub struct Context {
    scope: HashMap<String, Value>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new context instance. This can be fairly slow, and should
    /// only be done once if possible.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scope: crate::num::Number::create_initial_units(&crate::interrupt::Never::default())
                .unwrap(),
        }
    }
}

/// This function evaluates a string using the given context.
///
/// For example, passing in the string `"1 + 1"` will return a result of `"2"`.
///
/// # Errors
/// It returns an error if the given string is invalid.
/// This may be due to parser or runtime errors.
pub fn evaluate(input: &str, context: &mut Context) -> Result<FendResult, String> {
    if input.is_empty() {
        // no or blank input: return no output
        return Ok(FendResult {
            main_result: "".to_string(),
            other_info: vec![],
        });
    }
    let result = evaluate_to_value(input, &context.scope, &interrupt::Never::default())?;
    Ok(FendResult {
        main_result: format!("{}", result),
        other_info: vec![],
    })
}

/// Returns the current version of `fend-core`.
#[must_use]
pub fn get_version() -> String {
    "0.1.1".to_string()
}
