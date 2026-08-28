use std::borrow::Cow;
use std::sync::Arc;

use crate::{
	Span, ast, error::Interrupt, lexer, parser, result::FResult, scope::Scope, value::Value,
};

pub(crate) fn evaluate_to_value<I: Interrupt>(
	input: &str,
	scope: Option<Arc<Scope>>,
	attrs: Attrs,
	spans: &mut Vec<Span>,
	context: &mut crate::Context,
	int: &I,
) -> FResult<Value> {
	let lex = lexer::lex(input, context, int);
	let mut tokens = vec![];
	let mut missing_open_parens: i32 = 0;
	for token in lex {
		let token = token?;
		if matches!(token, lexer::Token::Symbol(lexer::Symbol::CloseParens)) {
			missing_open_parens += 1;
		}
		tokens.push(token);
	}
	for _ in 0..missing_open_parens {
		tokens.insert(0, lexer::Token::Symbol(lexer::Symbol::OpenParens));
	}
	let parsed = parser::parse_tokens(&tokens)?;
	let result = ast::evaluate(parsed, scope, attrs, spans, context, int)?;
	Ok(result)
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Attrs {
	pub(crate) debug: bool,
	pub(crate) show_approx: bool,
	pub(crate) plain_number: bool,
	pub(crate) trailing_newline: bool,
}

impl Default for Attrs {
	fn default() -> Self {
		Self {
			debug: false,
			show_approx: true,
			plain_number: false,
			trailing_newline: true,
		}
	}
}

fn parse_attrs(mut input: &str) -> (Attrs, &str) {
	let mut attrs = Attrs::default();
	while input.starts_with('@') {
		if let Some(remaining) = input.strip_prefix("@debug ") {
			attrs.debug = true;
			input = remaining;
		} else if let Some(remaining) = input.strip_prefix("@noapprox ") {
			attrs.show_approx = false;
			input = remaining;
		} else if let Some(remaining) = input.strip_prefix("@plain_number ") {
			attrs.plain_number = true;
			input = remaining;
		} else if let Some(remaining) = input.strip_prefix("@no_trailing_newline ") {
			attrs.trailing_newline = false;
			input = remaining;
		} else {
			break;
		}
	}
	(attrs, input)
}

/// If a previous result is available (stored in `_`) and the input begins with
/// a binary operator that cannot otherwise start an expression (such as `* 2`
/// or `to miles`), rewrite it to continue from that result, e.g. `_ * 2` or
/// `_ to miles`. This mirrors how pocket and on-screen calculators let an
/// operator key off the previously-displayed value.
fn continue_from_previous_result<'a, I: Interrupt>(
	input: &'a str,
	context: &crate::Context,
	int: &I,
) -> Cow<'a, str> {
	if !context.variables.contains_key("_") {
		return Cow::Borrowed(input);
	}
	let mut tokens = lexer::lex(input, context, int);
	if let Some(Ok(lexer::Token::Symbol(symbol))) = tokens.next()
		&& symbol.expects_preceding_operand()
	{
		return Cow::Owned(format!("_ {input}"));
	}
	Cow::Borrowed(input)
}

/// This also saves the calculation result in a variable `_` and `ans`
pub(crate) fn evaluate_to_spans<I: Interrupt>(
	input: &str,
	scope: Option<Arc<Scope>>,
	context: &mut crate::Context,
	int: &I,
) -> FResult<(Vec<Span>, Attrs)> {
	let (attrs, input) = parse_attrs(input);
	let input = continue_from_previous_result(input, context, int);
	let mut spans = vec![];
	let value = evaluate_to_value(input.as_ref(), scope, attrs, &mut spans, context, int)?;
	context.variables.insert("_".to_string(), value.clone());
	context.variables.insert("ans".to_string(), value.clone());
	Ok((
		if attrs.debug {
			vec![Span::from_string(format!("{value:?}"))]
		} else {
			if context.echo_result {
				value.format(0, &mut spans, attrs, false, context, int)?;
			}
			spans
		},
		attrs,
	))
}
