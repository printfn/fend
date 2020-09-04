use crate::{ast, err::IntErr, interrupt::Interrupt, parser, value::Value};
use std::collections::HashMap;

pub fn evaluate_to_value(
    input: &str,
    scope: &HashMap<String, Value>,
    int: &impl Interrupt,
) -> Result<Value, IntErr<String>> {
    let parsed = parser::parse_string(input, int)?;
    let result = ast::evaluate(parsed, scope, int)?;
    Ok(result)
}
