#![forbid(unsafe_code)]

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
    pub fn get_main_result(&self) -> &str {
        self.main_result.as_str()
    }

    pub fn get_other_info(&self) -> impl Iterator<Item = &str> {
        self.other_info.iter().map(|string| string.as_str())
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

impl Context {
    pub fn new() -> Self {
        Self {
            scope: crate::num::Number::create_initial_units(),
        }
    }
}

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
        main_result: format!("{:?}", result),
        other_info: vec![],
    })
}
