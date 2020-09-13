use crate::{
    ast,
    err::{IntErr, Interrupt},
    parser,
    scope::Scope,
    value::Value,
};

pub fn evaluate_to_value<I: Interrupt>(
    input: &str,
    scope: &mut Scope,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    let parsed = parser::parse_string(input, int)?;
    let result = ast::evaluate(parsed, scope, int)?;
    Ok(result)
}

pub fn evaluate_to_string<I: Interrupt>(
    input: &str,
    scope: &mut Scope,
    int: &I,
) -> Result<String, IntErr<String, I>> {
    let value = evaluate_to_value(input, scope, int)?;
    let s = crate::num::to_string(|f| value.format(f, int))?;
    Ok(s)
}
