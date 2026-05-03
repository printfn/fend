use crate::ImplicitMultiplicationPrecedence;
use crate::ast::{Bop, Expr};
use crate::lexer::{Symbol, Token};
use crate::value::Value;
use std::fmt;

#[derive(Debug)]
pub(crate) enum ParseError {
	ExpectedAToken,
	ExpectedToken(Symbol, Symbol),
	FoundInvalidTokenWhileExpecting(Symbol),
	ExpectedANumber,
	ExpectedIdentifier,
	UnexpectedSymbol(Symbol),
	// TODO remove this
	InvalidApplyOperands,
	UnexpectedInput,
	ExpectedIdentifierAsArgument,
	ExpectedIdentifierInAssignment,
	ExpectedDotInLambda,
	InvalidMixedFraction,
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match self {
			Self::ExpectedAToken => write!(f, "expected a token"),
			Self::ExpectedToken(fnd, ex) => write!(f, "found '{fnd}' while expecting '{ex}'"),
			Self::FoundInvalidTokenWhileExpecting(sym) => {
				write!(f, "found an invalid token while expecting '{sym}'")
			}
			Self::ExpectedANumber => write!(f, "expected a number"),
			Self::ExpectedIdentifier
			| Self::ExpectedIdentifierAsArgument
			| Self::ExpectedIdentifierInAssignment => {
				write!(f, "expected an identifier")
			}
			Self::UnexpectedSymbol(s) => {
				write!(f, "expected a value, instead found '{s}'")
			}
			// TODO improve this message or remove this error type
			Self::InvalidApplyOperands => write!(f, "error"),
			Self::UnexpectedInput => write!(f, "unexpected input found"),
			Self::ExpectedDotInLambda => {
				write!(f, "missing '.' in lambda (expected e.g. \\x.x)")
			}
			Self::InvalidMixedFraction => write!(f, "invalid mixed fraction"),
		}
	}
}

type ParseResult<'a, T = Expr> = Result<(T, &'a [Token]), ParseError>;

impl From<ParseError> for crate::error::FendError {
	fn from(e: ParseError) -> Self {
		Self::ParseError(e)
	}
}

fn parse_token(input: &[Token]) -> ParseResult<'_, Token> {
	if input.is_empty() {
		return Err(ParseError::ExpectedAToken);
	}
	Ok((input[0].clone(), &input[1..]))
}

fn parse_fixed_symbol(input: &[Token], symbol: Symbol) -> ParseResult<'_, ()> {
	let (token, remaining) = parse_token(input)?;
	if let Token::Symbol(sym) = token {
		if sym == symbol {
			Ok(((), remaining))
		} else {
			Err(ParseError::ExpectedToken(sym, symbol))
		}
	} else {
		Err(ParseError::FoundInvalidTokenWhileExpecting(symbol))
	}
}

fn parse_number(input: &[Token]) -> ParseResult<'_> {
	match parse_token(input)? {
		(Token::Num(num), remaining) => Ok((Expr::Literal(Value::Num(Box::new(num))), remaining)),
		_ => Err(ParseError::ExpectedANumber),
	}
}

fn parse_ident(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	match parse_token(input)? {
		(Token::Ident(ident), remaining) => {
			if ident.as_str() == "light"
				&& let Ok((ident2, remaining2)) =
					parse_ident(remaining, implicit_multiplication_precedence)
			{
				return Ok((
					Expr::Apply(Box::new(Expr::Ident(ident)), Box::new(ident2)),
					remaining2,
				));
			}
			if let Ok(((), remaining2)) = parse_fixed_symbol(remaining, Symbol::Of) {
				let (inner, remaining3) =
					parse_parens_or_literal(remaining2, implicit_multiplication_precedence)?;
				Ok((Expr::Of(ident, Box::new(inner)), remaining3))
			} else {
				Ok((Expr::Ident(ident), remaining))
			}
		}
		_ => Err(ParseError::ExpectedIdentifier),
	}
}

fn parse_parens(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::OpenParens)?;
	if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::CloseParens) {
		return Ok((Expr::Literal(Value::Unit), remaining));
	}
	let (inner, mut input) = parse_expression(input, implicit_multiplication_precedence)?;
	// allow omitting closing parentheses at end of input
	if !input.is_empty() {
		let ((), remaining) = parse_fixed_symbol(input, Symbol::CloseParens)?;
		input = remaining;
	}
	Ok((Expr::Parens(Box::new(inner)), input))
}

fn parse_backslash_lambda(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::Backslash)?;
	let (Expr::Ident(ident), input) = parse_ident(input, implicit_multiplication_precedence)?
	else {
		return Err(ParseError::ExpectedIdentifier);
	};
	let ((), input) =
		parse_fixed_symbol(input, Symbol::Dot).map_err(|_| ParseError::ExpectedDotInLambda)?;
	let (rhs, input) = parse_function(input, implicit_multiplication_precedence)?;
	Ok((Expr::Fn(ident, Box::new(rhs)), input))
}

fn parse_parens_or_literal(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (token, remaining) = parse_token(input)?;

	match token {
		Token::Num(_) => parse_number(input),
		Token::Ident(_) => parse_ident(input, implicit_multiplication_precedence),
		Token::StringLiteral(s) => Ok((Expr::Literal(Value::String(s)), remaining)),
		Token::Symbol(Symbol::OpenParens) => {
			parse_parens(input, implicit_multiplication_precedence)
		}
		Token::Symbol(Symbol::Backslash) => {
			parse_backslash_lambda(input, implicit_multiplication_precedence)
		}
		Token::Symbol(s) => Err(ParseError::UnexpectedSymbol(s)),
		Token::Date(d) => Ok((Expr::Literal(Value::Date(d)), remaining)),
	}
}

fn parse_factorial(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut res, mut input) = parse_parens_or_literal(input, implicit_multiplication_precedence)?;
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Factorial) {
		res = Expr::Factorial(Box::new(res));
		input = remaining;
	}
	Ok((res, input))
}

fn parse_power(
	input: &[Token],
	allow_unary: bool,
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	if allow_unary {
		if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Sub) {
			let (result, remaining) =
				parse_power(remaining, true, implicit_multiplication_precedence)?;
			return Ok((Expr::UnaryMinus(Box::new(result)), remaining));
		}
		if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Add) {
			let (result, remaining) =
				parse_power(remaining, true, implicit_multiplication_precedence)?;
			return Ok((Expr::UnaryPlus(Box::new(result)), remaining));
		}
		// The precedence of unary division relative to exponentiation
		// is not important because /a^b -> (1/a)^b == 1/(a^b)
		if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Div) {
			let (result, remaining) =
				parse_power(remaining, true, implicit_multiplication_precedence)?;
			return Ok((Expr::UnaryDiv(Box::new(result)), remaining));
		}
	}
	let (mut result, mut input) = parse_factorial(input, implicit_multiplication_precedence)?;
	if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Pow) {
		let (rhs, remaining) = parse_power(remaining, true, implicit_multiplication_precedence)?;
		result = Expr::Bop(Bop::Pow, Box::new(result), Box::new(rhs));
		input = remaining;
	}
	Ok((result, input))
}

fn parse_apply_cont<'a>(
	input: &'a [Token],
	lhs: &Expr,
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'a> {
	let (rhs, input) = parse_power(input, false, implicit_multiplication_precedence)?;
	Ok((
		match (lhs, &rhs) {
			(
				Expr::Literal(Value::Num(_)) | Expr::UnaryMinus(_) | Expr::ApplyMul(_, _),
				Expr::Literal(Value::Num(_)),
			) => {
				if let Expr::UnaryMinus(b) = &lhs
					&& !matches!(b.as_ref(), Expr::Literal(Value::Num(_)))
				{
					return Ok((Expr::Apply(Box::new(lhs.clone()), Box::new(rhs)), input));
				}
				// this may later be parsed as a compound fraction, e.g. 1 2/3
				// or as an addition, e.g. 6 feet 1 inch
				return Err(ParseError::InvalidApplyOperands);
			}
			(
				Expr::Literal(Value::Num(_)) | Expr::UnaryMinus(_) | Expr::ApplyMul(_, _),
				Expr::Bop(Bop::Pow, a, _),
			) => {
				if let Expr::Literal(Value::Num(_)) = **a {
					return Err(ParseError::InvalidApplyOperands);
				}
				Expr::Apply(Box::new(lhs.clone()), Box::new(rhs))
			}
			// support e.g. '$5', '£3' or '¥10'
			(Expr::Ident(i), Expr::Literal(Value::Num(_))) if i.is_prefix_unit() => {
				Expr::Apply(Box::new(lhs.clone()), Box::new(rhs))
			}
			(_, Expr::Literal(Value::Num(_))) => {
				Expr::ApplyFunctionCall(Box::new(lhs.clone()), Box::new(rhs))
			}
			(Expr::Literal(Value::Num(_)) | Expr::ApplyMul(_, _), _) => {
				Expr::ApplyMul(Box::new(lhs.clone()), Box::new(rhs))
			}
			_ => Expr::Apply(Box::new(lhs.clone()), Box::new(rhs)),
		},
		input,
	))
}

fn parse_mixed_fraction<'a>(
	input: &'a [Token],
	lhs: &Expr,
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'a> {
	let (positive, lhs, other_factor) = match lhs {
		Expr::Literal(Value::Num(_)) => (true, lhs, None),
		Expr::UnaryMinus(x) => {
			if let Expr::Literal(Value::Num(_)) = &**x {
				(false, lhs, None)
			} else {
				return Err(ParseError::InvalidMixedFraction);
			}
		}
		Expr::Bop(Bop::Mul, a, b) => match &**b {
			Expr::Literal(Value::Num(_)) => (true, &**b, Some(&**a)),
			Expr::UnaryMinus(x) => {
				if let Expr::Literal(Value::Num(_)) = &**x {
					(false, &**b, Some(&**a))
				} else {
					return Err(ParseError::InvalidMixedFraction);
				}
			}
			_ => return Err(ParseError::InvalidMixedFraction),
		},
		_ => return Err(ParseError::InvalidMixedFraction),
	};
	let (rhs_top, input) = parse_power(input, false, implicit_multiplication_precedence)?;
	if let Expr::Literal(Value::Num(_)) = rhs_top {
	} else {
		return Err(ParseError::InvalidMixedFraction);
	}
	let ((), input) = parse_fixed_symbol(input, Symbol::Div)?;
	let (rhs_bottom, input) = parse_power(input, false, implicit_multiplication_precedence)?;
	if let Expr::Literal(Value::Num(_)) = rhs_bottom {
	} else {
		return Err(ParseError::InvalidMixedFraction);
	}
	let rhs = Box::new(Expr::Bop(Bop::Div, Box::new(rhs_top), Box::new(rhs_bottom)));
	let mixed_fraction = if positive {
		Expr::Bop(Bop::Plus, Box::new(lhs.clone()), rhs)
	} else {
		Expr::Bop(Bop::Minus, Box::new(lhs.clone()), rhs)
	};
	let mixed_fraction = other_factor.map_or(mixed_fraction.clone(), |other_factor| {
		Expr::Bop(
			Bop::Mul,
			Box::new(other_factor.clone()),
			Box::new(mixed_fraction),
		)
	});
	Ok((mixed_fraction, input))
}

fn parse_multiplication_cont(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::Mul)?;
	let (b, input) = parse_multiplicative_operand(input, implicit_multiplication_precedence)?;
	Ok((b, input))
}

fn parse_division_cont(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::Div)?;
	let (b, input) = parse_multiplicative_operand(input, implicit_multiplication_precedence)?;
	Ok((b, input))
}

fn parse_modulo_cont(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::Mod)?;
	let (b, input) = parse_multiplicative_operand(input, implicit_multiplication_precedence)?;
	Ok((b, input))
}

// try parsing `%` as modulo
fn parse_modulo2_cont(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (token, input) = parse_token(input)?;
	if let Token::Ident(ident) = token {
		if ident.as_str() != "%" {
			return Err(ParseError::UnexpectedInput);
		}
	} else {
		return Err(ParseError::UnexpectedInput);
	}
	// extra restriction: `%` can't be directly followed by an operator, since we
	// assume that e.g. `1 % + ...` should be a percentage
	if input.first().is_some_and(|t| {
		matches!(t, Token::Symbol(_)) && !matches!(t, Token::Symbol(Symbol::OpenParens))
	}) {
		return Err(ParseError::UnexpectedInput);
	}
	let (b, input) = parse_multiplicative_operand(input, implicit_multiplication_precedence)?;
	Ok((b, input))
}

fn parse_apply_chain(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut res, mut input) = parse_power(input, true, implicit_multiplication_precedence)?;
	while let Ok((new_res, remaining)) =
		parse_apply_cont(input, &res, implicit_multiplication_precedence)
	{
		res = new_res;
		input = remaining;
	}
	Ok((res, input))
}

fn parse_multiplicative_operand(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	match implicit_multiplication_precedence {
		ImplicitMultiplicationPrecedence::SameAsDivision => {
			parse_power(input, true, implicit_multiplication_precedence)
		}
		ImplicitMultiplicationPrecedence::HigherThanDivision => {
			parse_apply_chain(input, implicit_multiplication_precedence)
		}
	}
}

fn parse_multiplicative(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut res, mut input) =
		parse_multiplicative_operand(input, implicit_multiplication_precedence)?;
	loop {
		if let Ok((term, remaining)) =
			parse_multiplication_cont(input, implicit_multiplication_precedence)
		{
			res = Expr::Bop(Bop::Mul, Box::new(res.clone()), Box::new(term));
			input = remaining;
		} else if let Ok((term, remaining)) =
			parse_division_cont(input, implicit_multiplication_precedence)
		{
			res = Expr::Bop(Bop::Div, Box::new(res.clone()), Box::new(term));
			input = remaining;
		} else if let Ok((term, remaining)) =
			parse_modulo_cont(input, implicit_multiplication_precedence)
		{
			res = Expr::Bop(Bop::Mod, Box::new(res.clone()), Box::new(term));
			input = remaining;
		} else if let Ok((term, remaining)) =
			parse_modulo2_cont(input, implicit_multiplication_precedence)
		{
			res = Expr::Bop(Bop::Mod, Box::new(res.clone()), Box::new(term));
			input = remaining;
		} else if let Ok((new_res, remaining)) =
			parse_mixed_fraction(input, &res, implicit_multiplication_precedence)
		{
			res = new_res;
			input = remaining;
		} else if implicit_multiplication_precedence
			== ImplicitMultiplicationPrecedence::SameAsDivision
			&& let Ok((new_res, remaining)) =
				parse_apply_cont(input, &res, implicit_multiplication_precedence)
		{
			res = new_res;
			input = remaining;
		} else {
			break;
		}
	}
	Ok((res, input))
}

fn parse_implicit_addition(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (res, input) = parse_multiplicative(input, implicit_multiplication_precedence)?;
	if let Ok((rhs, remaining)) = parse_implicit_addition(input, implicit_multiplication_precedence)
	{
		// n i n i, n i i n i i, etc. (n: number literal, i: identifier)
		if let (
			Expr::ApplyMul(_, _),
			Expr::ApplyMul(_, _) | Expr::Bop(Bop::ImplicitPlus, _, _) | Expr::Literal(_),
		) = (&res, &rhs)
		{
			return Ok((
				Expr::Bop(Bop::ImplicitPlus, Box::new(res), Box::new(rhs)),
				remaining,
			));
		}
	}
	Ok((res, input))
}

fn parse_addition_cont(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::Add)?;
	let (b, input) = parse_implicit_addition(input, implicit_multiplication_precedence)?;
	Ok((b, input))
}

fn parse_subtraction_cont(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::Sub)?;
	let (b, input) = parse_implicit_addition(input, implicit_multiplication_precedence)?;
	Ok((b, input))
}

fn parse_to_cont(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let ((), input) = parse_fixed_symbol(input, Symbol::UnitConversion)?;
	let (b, input) = parse_implicit_addition(input, implicit_multiplication_precedence)?;
	Ok((b, input))
}

fn parse_additive(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut res, mut input) = parse_implicit_addition(input, implicit_multiplication_precedence)?;
	loop {
		if let Ok((term, remaining)) =
			parse_addition_cont(input, implicit_multiplication_precedence)
		{
			res = Expr::Bop(Bop::Plus, Box::new(res), Box::new(term));
			input = remaining;
		} else if let Ok((term, remaining)) =
			parse_subtraction_cont(input, implicit_multiplication_precedence)
		{
			res = Expr::Bop(Bop::Minus, Box::new(res), Box::new(term));
			input = remaining;
		} else if let Ok((term, remaining)) =
			parse_to_cont(input, implicit_multiplication_precedence)
		{
			res = Expr::As(Box::new(res), Box::new(term));
			input = remaining;
		} else {
			break;
		}
	}
	Ok((res, input))
}

fn parse_bitshifts(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut result, mut input) = parse_additive(input, implicit_multiplication_precedence)?;
	loop {
		if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::ShiftLeft) {
			let (rhs, remaining) = parse_additive(remaining, implicit_multiplication_precedence)?;
			result = Expr::Bop(
				Bop::Bitwise(crate::ast::BitwiseBop::LeftShift),
				Box::new(result),
				Box::new(rhs),
			);
			input = remaining;
		} else if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::ShiftRight) {
			let (rhs, remaining) = parse_additive(remaining, implicit_multiplication_precedence)?;
			result = Expr::Bop(
				Bop::Bitwise(crate::ast::BitwiseBop::RightShift),
				Box::new(result),
				Box::new(rhs),
			);
			input = remaining;
		} else {
			break;
		}
	}
	Ok((result, input))
}

fn parse_bitwise_and(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut result, mut input) = parse_bitshifts(input, implicit_multiplication_precedence)?;
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::BitwiseAnd) {
		let (rhs, remaining) = parse_bitshifts(remaining, implicit_multiplication_precedence)?;
		result = Expr::Bop(
			Bop::Bitwise(crate::ast::BitwiseBop::And),
			Box::new(result),
			Box::new(rhs),
		);
		input = remaining;
	}
	Ok((result, input))
}

fn parse_bitwise_xor(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut result, mut input) = parse_bitwise_and(input, implicit_multiplication_precedence)?;
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::BitwiseXor) {
		let (rhs, remaining) = parse_bitwise_and(remaining, implicit_multiplication_precedence)?;
		result = Expr::Bop(
			Bop::Bitwise(crate::ast::BitwiseBop::Xor),
			Box::new(result),
			Box::new(rhs),
		);
		input = remaining;
	}
	Ok((result, input))
}

fn parse_bitwise_or(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut result, mut input) = parse_bitwise_xor(input, implicit_multiplication_precedence)?;
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::BitwiseOr) {
		let (rhs, remaining) = parse_bitwise_xor(remaining, implicit_multiplication_precedence)?;
		result = Expr::Bop(
			Bop::Bitwise(crate::ast::BitwiseBop::Or),
			Box::new(result),
			Box::new(rhs),
		);
		input = remaining;
	}
	Ok((result, input))
}

fn parse_combination(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut result, mut input) = parse_bitwise_or(input, implicit_multiplication_precedence)?;
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Combination) {
		let (rhs, remaining) = parse_bitwise_or(remaining, implicit_multiplication_precedence)?;
		result = Expr::Bop(Bop::Combination, Box::new(result), Box::new(rhs));
		input = remaining;
	}
	Ok((result, input))
}

fn parse_permutation(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (mut result, mut input) = parse_combination(input, implicit_multiplication_precedence)?;
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Permutation) {
		let (rhs, remaining) = parse_combination(remaining, implicit_multiplication_precedence)?;
		result = Expr::Bop(Bop::Permutation, Box::new(result), Box::new(rhs));
		input = remaining;
	}
	Ok((result, input))
}

fn parse_function(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (lhs, input) = parse_permutation(input, implicit_multiplication_precedence)?;
	if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Fn) {
		if let Expr::Ident(s) = lhs {
			let (rhs, remaining) = parse_function(remaining, implicit_multiplication_precedence)?;
			return Ok((Expr::Fn(s, Box::new(rhs)), remaining));
		}
		return Err(ParseError::ExpectedIdentifierAsArgument);
	}
	Ok((lhs, input))
}

fn parse_equality(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (lhs, input) = parse_function(input, implicit_multiplication_precedence)?;
	if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::DoubleEquals) {
		let (rhs, remaining) = parse_function(remaining, implicit_multiplication_precedence)?;
		Ok((
			Expr::Equality(true, Box::new(lhs), Box::new(rhs)),
			remaining,
		))
	} else if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::NotEquals) {
		let (rhs, remaining) = parse_function(remaining, implicit_multiplication_precedence)?;
		Ok((
			Expr::Equality(false, Box::new(lhs), Box::new(rhs)),
			remaining,
		))
	} else {
		Ok((lhs, input))
	}
}

fn parse_assignment(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	let (lhs, input) = parse_equality(input, implicit_multiplication_precedence)?;
	if let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Equals) {
		if let Expr::Ident(s) = lhs {
			let (rhs, remaining) = parse_assignment(remaining, implicit_multiplication_precedence)?;
			return Ok((Expr::Assign(s, Box::new(rhs)), remaining));
		}
		return Err(ParseError::ExpectedIdentifierInAssignment);
	}
	Ok((lhs, input))
}

fn parse_statements(
	mut input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Semicolon) {
		input = remaining;
	}
	if input.is_empty() {
		return Ok((Expr::Literal(Value::Unit), &[]));
	}
	let (mut result, mut input) = parse_assignment(input, implicit_multiplication_precedence)?;
	while let Ok(((), remaining)) = parse_fixed_symbol(input, Symbol::Semicolon) {
		if remaining.is_empty() || matches!(remaining[0], Token::Symbol(Symbol::Semicolon)) {
			input = remaining;
			continue;
		}
		let (rhs, remaining) = parse_assignment(remaining, implicit_multiplication_precedence)?;
		result = Expr::Statements(Box::new(result), Box::new(rhs));
		input = remaining;
	}
	Ok((result, input))
}

pub(crate) fn parse_expression(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> ParseResult<'_> {
	parse_statements(input, implicit_multiplication_precedence)
}

pub(crate) fn parse_tokens(
	input: &[Token],
	implicit_multiplication_precedence: ImplicitMultiplicationPrecedence,
) -> Result<Expr, ParseError> {
	let (res, remaining) = parse_expression(input, implicit_multiplication_precedence)?;
	if !remaining.is_empty() {
		return Err(ParseError::UnexpectedInput);
	}
	Ok(res)
}
