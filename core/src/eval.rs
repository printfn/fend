use crate::{
    ast,
    err::{IntErr, Interrupt},
    lexer, parser,
    scope::Scope,
    value::Value,
};

pub fn evaluate_to_value<I: Interrupt>(
    input: &str,
    options: parser::ParseOptions,
    scope: &mut Scope,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    let tokens = lexer::lex(input, int)?;
    let parsed = parser::parse_tokens(tokens.as_slice(), options).map_err(|e| e.to_string())?;
    let result = ast::evaluate(parsed, scope, options, int)?;
    Ok(result)
}

pub fn evaluate_to_string<I: Interrupt>(
    input: &str,
    scope: &mut Scope,
    int: &I,
) -> Result<String, IntErr<String, I>> {
    let value = evaluate_to_value(input, parser::ParseOptions::default(), scope, int)?;
    let s = crate::num::to_string(|f| value.format(f, int))?;
    Ok(s)
}
