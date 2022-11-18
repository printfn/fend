use std::sync::Arc;

use crate::{
    ast,
    error::{FendError, Interrupt},
    lexer, parser,
    scope::Scope,
    value::Value,
    Attrs, Span,
};

pub(crate) fn evaluate_to_value<'a, I: Interrupt>(
    input: &'a str,
    scope: Option<Arc<Scope>>,
    attrs: Attrs,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    let lex = lexer::lex(input, int);
    let mut tokens = vec![];
    let mut missing_open_parens: i32 = 0;
    for token in lex {
        let token = token?;
        if let lexer::Token::Symbol(lexer::Symbol::CloseParens) = token {
            missing_open_parens += 1;
        }
        tokens.push(token);
    }
    for _ in 0..missing_open_parens {
        tokens.insert(0, lexer::Token::Symbol(lexer::Symbol::OpenParens));
    }
    let parsed = parser::parse_tokens(&tokens)?;
    let result = ast::evaluate(parsed, scope, attrs, context, int)?;
    Ok(result)
}

/// This also saves the calculation result in a variable `_` and `ans`
pub(crate) fn evaluate_to_spans<'a, I: Interrupt>(
    mut input: &'a str,
    scope: Option<Arc<Scope>>,
    context: &mut crate::Context,
    int: &I,
) -> Result<(Vec<Span>, bool), FendError> {
    let debug = input.strip_prefix("!debug ").map_or(false, |remaining| {
        input = remaining;
        true
    });
    let show_approx = input.strip_prefix("!noapprox ").map_or(true, |remaining| {
        input = remaining;
        false
    });
    let attrs = Attrs { show_approx };
    let value = evaluate_to_value(input, scope, attrs, context, int)?;
    context.variables.insert("_".to_string(), value.clone());
    context.variables.insert("ans".to_string(), value.clone());
    Ok((
        if debug {
            vec![Span::from_string(format!("{:?}", value))]
        } else {
            let mut spans = vec![];
            value.format(0, &mut spans, attrs, context, int)?;
            spans
        },
        value.is_unit(),
    ))
}
