#![forbid(unsafe_code)]
// enable all clippy warnings
#![forbid(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::restriction)]
// selectively disable these warnings, all of them are under clippy::restriction
#![allow(clippy::implicit_return)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::unreachable)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

mod ast;
mod lexer;
mod num;
mod parser;
mod value;

use std::collections::HashMap;
use value::Value;

#[derive(PartialEq, Eq, Debug)]
pub struct FendResult {
    main_result: String,
    other_info: Vec<String>,
}

impl FendResult {
    #[must_use]
    pub fn get_main_result(&self) -> &str {
        self.main_result.as_str()
    }

    pub fn get_other_info(&self) -> impl Iterator<Item = &str> {
        self.other_info.iter().map(std::string::String::as_str)
    }
}

fn evaluate_to_value(input: &str, scope: &HashMap<String, Value>) -> Result<Value, String> {
    let parsed = parser::parse_string(input)?;
    let result = ast::evaluate(parsed, scope)?;
    Ok(result)
}

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
    #[must_use]
    pub fn new() -> Self {
        Self {
            scope: crate::num::Number::create_initial_units(),
        }
    }
}

/// This function evaluates a string using the given context. For example,
/// passing in the string "1 + 1" will return a result of "2".
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
    let result = evaluate_to_value(input, &context.scope)?;
    Ok(FendResult {
        main_result: format!("{}", result),
        other_info: vec![],
    })
}

pub fn get_version() -> String {
    "0.1.0".to_string()
}
