use std::sync::Arc;

use crate::{
    ast,
    error::{IntErr, Interrupt},
    lexer, parser,
    scope::Scope,
    value::Value,
    Span,
};

pub(crate) fn evaluate_to_value<'a, I: Interrupt>(
    input: &'a str,
    scope: Option<Arc<Scope<'a>>>,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value<'a>, IntErr<String, I>> {
    let lex = lexer::lex(input, int);
    let mut tokens = vec![];
    let mut missing_open_parens: i32 = 0;
    for token in lex {
        let token = token.map_err(IntErr::into_string)?;
        if let lexer::Token::Symbol(lexer::Symbol::CloseParens) = token {
            missing_open_parens += 1
        }
        tokens.push(token);
    }
    for _ in 0..missing_open_parens {
        tokens.insert(0, lexer::Token::Symbol(lexer::Symbol::OpenParens));
    }
    let parsed = parser::parse_tokens(&tokens).map_err(|e| e.to_string())?;
    let result = ast::evaluate(parsed, scope, context, int)?;
    Ok(result)
}

pub(crate) fn evaluate_to_string<'a, I: Interrupt>(
    mut input: &'a str,
    scope: Option<Arc<Scope<'a>>>,
    context: &mut crate::Context,
    int: &I,
) -> Result<Vec<Span>, IntErr<String, I>> {
    let debug = input.strip_prefix("!debug ").map_or(false, |remaining| {
        input = remaining;
        true
    });
    let value = evaluate_to_value(input, scope, context, int)?;
    Ok(if debug {
        vec![Span::from_string(format!("{:?}", value))]
    } else {
        vec![Span::from_string(value.format(0, int)?.to_string())]
    })
}
