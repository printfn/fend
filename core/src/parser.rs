use crate::ast::Expr;
use crate::lexer::{Symbol, Token};
use std::fmt;

pub(crate) enum ParseError {
    ExpectedAToken,
    ExpectedToken(Symbol, Symbol),
    FoundInvalidTokenWhileExpecting(Symbol),
    ExpectedANumber,
    ExpectedIdentifier,
    ExpectedNumIdentOrParen,
    // TODO remove this
    InvalidApplyOperands,
    UnexpectedInput,
    ExpectedIdentifierAsArgument,
    ExpectedDotInLambda(Box<ParseError>),
    InvalidMixedFraction,
    UnexpectedWhitespace,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::ExpectedAToken => write!(f, "Expected a token"),
            Self::ExpectedToken(fnd, ex) => write!(f, "Found '{}' while expecting '{}'", fnd, ex),
            Self::FoundInvalidTokenWhileExpecting(sym) => {
                write!(f, "Found an invalid token while expecting '{}'", sym)
            }
            Self::ExpectedANumber => write!(f, "Expected a number"),
            Self::ExpectedIdentifier | Self::ExpectedIdentifierAsArgument => {
                write!(f, "Expected an identifier")
            }
            Self::ExpectedNumIdentOrParen => {
                write!(f, "Expected a number, an identifier or an open parenthesis")
            }
            // TODO improve this message or remove this error type
            Self::InvalidApplyOperands => write!(f, "Error"),
            Self::UnexpectedInput => write!(f, "Unexpected input found"),
            Self::ExpectedDotInLambda(_) => {
                write!(f, "Missing '.' in lambda (expected e.g. \\x.x)")
            }
            Self::InvalidMixedFraction => write!(f, "Invalid mixed fraction"),
            Self::UnexpectedWhitespace => write!(f, "Unexpected whitespace"),
        }
    }
}
impl crate::err::Error for ParseError {}

type ParseResult<'a, 'b, T = Expr<'a>> = Result<(T, &'b [Token<'a>]), ParseError>;

fn parse_token<'a, 'b>(
    input: &'b [Token<'a>],
    skip_whitespace: bool,
) -> ParseResult<'a, 'b, Token<'a>> {
    if input.is_empty() {
        Err(ParseError::ExpectedAToken)
    } else if let Token::Whitespace = input[0] {
        if skip_whitespace {
            parse_token(&input[1..], skip_whitespace)
        } else {
            Ok((input[0].clone(), &input[1..]))
        }
    } else {
        Ok((input[0].clone(), &input[1..]))
    }
}

fn parse_fixed_symbol<'a, 'b>(input: &'b [Token<'a>], symbol: Symbol) -> ParseResult<'a, 'b, ()> {
    let (token, remaining) = parse_token(input, true)?;
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

fn parse_number<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    match parse_token(input, true)? {
        (Token::Num(num), remaining) => Ok((Expr::Num(num), remaining)),
        _ => Err(ParseError::ExpectedANumber),
    }
}

fn parse_ident<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    match parse_token(input, true)? {
        (Token::Ident(ident), remaining) => {
            if let Ok(((), remaining2)) = parse_fixed_symbol(remaining, Symbol::Of) {
                let inner = parse_ident(remaining2)?;
                match inner {
                    (Expr::Of(mut other), remaining3) => {
                        other.push(ident);
                        Ok((Expr::Of(other), remaining3))
                    }
                    (Expr::Ident(ident2), remaining3) => {
                        Ok((Expr::Of(vec![ident2, ident]), remaining3))
                    }
                    _ => Err(ParseError::ExpectedIdentifier),
                }
            } else {
                Ok((Expr::Ident(ident), remaining))
            }
        }
        _ => Err(ParseError::ExpectedIdentifier),
    }
}

fn parse_parens<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (_, input) = parse_fixed_symbol(input, Symbol::OpenParens)?;
    let (inner, mut input) = parse_expression(input)?;
    // allow omitting closing parentheses at end of input
    if !input.is_empty() {
        let (_, remaining) = parse_fixed_symbol(input, Symbol::CloseParens)?;
        input = remaining;
    }
    Ok((Expr::Parens(Box::new(inner)), input))
}

fn parse_backslash_lambda<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Backslash)?;
    let (ident, input) = if let (Expr::Ident(ident), input) = parse_ident(input)? {
        (ident, input)
    } else {
        return Err(ParseError::ExpectedIdentifier);
    };
    let (_, input) = parse_fixed_symbol(input, Symbol::Dot)
        .map_err(|e| ParseError::ExpectedDotInLambda(Box::new(e)))?;
    let (rhs, input) = parse_function(input)?;
    Ok((Expr::Fn(ident, Box::new(rhs)), input))
}

fn parse_parens_or_literal<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (token, _) = parse_token(input, true)?;

    match token {
        Token::Num(_) => parse_number(input),
        Token::Ident(_) => parse_ident(input),
        Token::Symbol(Symbol::OpenParens) => parse_parens(input),
        Token::Symbol(Symbol::Backslash) => parse_backslash_lambda(input),
        Token::Symbol(..) => Err(ParseError::ExpectedNumIdentOrParen),
        Token::Whitespace => Err(ParseError::UnexpectedWhitespace),
    }
}

fn parse_factorial<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (mut res, mut input) = parse_parens_or_literal(input)?;
    while let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Factorial) {
        res = Expr::Factorial(Box::new(res));
        input = remaining;
    }
    Ok((res, input))
}

fn parse_power<'a, 'b>(input: &'b [Token<'a>], allow_unary: bool) -> ParseResult<'a, 'b> {
    if allow_unary {
        if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Sub) {
            let (result, remaining) = parse_power(remaining, true)?;
            return Ok((Expr::UnaryMinus(Box::new(result)), remaining));
        }
        if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Add) {
            let (result, remaining) = parse_power(remaining, true)?;
            return Ok((Expr::UnaryPlus(Box::new(result)), remaining));
        }
        // The precedence of unary division relative to exponentiation
        // is not important because /a^b -> (1/a)^b == 1/(a^b)
        if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Div) {
            let (result, remaining) = parse_power(remaining, true)?;
            return Ok((Expr::UnaryDiv(Box::new(result)), remaining));
        }
    }
    let (mut result, mut input) = parse_factorial(input)?;
    if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Pow) {
        let (rhs, remaining) = parse_power(remaining, true)?;
        result = Expr::Pow(Box::new(result), Box::new(rhs));
        input = remaining;
    }
    Ok((result, input))
}

fn parse_apply_cont<'a, 'b>(input: &'b [Token<'a>], lhs: &Expr<'a>) -> ParseResult<'a, 'b> {
    let (rhs, input) = parse_power(input, false)?;
    Ok((
        match (lhs, &rhs) {
            (Expr::Num(_), Expr::Num(_))
            | (Expr::UnaryMinus(_), Expr::Num(_))
            | (Expr::ApplyMul(_, _), Expr::Num(_)) => {
                // this may later be parsed as a compound fraction, e.g. 1 2/3
                // or as an addition, e.g. 6 feet 1 inch
                return Err(ParseError::InvalidApplyOperands);
            }
            (Expr::Num(_), Expr::Pow(a, _))
            | (Expr::UnaryMinus(_), Expr::Pow(a, _))
            | (Expr::ApplyMul(_, _), Expr::Pow(a, _)) => {
                if let Expr::Num(_) = **a {
                    return Err(ParseError::InvalidApplyOperands);
                }
                Expr::Apply(Box::new(lhs.clone()), Box::new(rhs))
            }
            (_, Expr::Num(_)) => Expr::ApplyFunctionCall(Box::new(lhs.clone()), Box::new(rhs)),
            (Expr::Num(_), _) | (Expr::ApplyMul(_, _), _) => {
                Expr::ApplyMul(Box::new(lhs.clone()), Box::new(rhs))
            }
            _ => Expr::Apply(Box::new(lhs.clone()), Box::new(rhs)),
        },
        input,
    ))
}

#[allow(clippy::option_if_let_else)]
fn parse_mixed_fraction<'a, 'b>(input: &'b [Token<'a>], lhs: &Expr<'a>) -> ParseResult<'a, 'b> {
    let (positive, lhs, other_factor) = match lhs {
        Expr::Num(_) => (true, lhs, None),
        Expr::UnaryMinus(x) => {
            if let Expr::Num(_) = &**x {
                (false, lhs, None)
            } else {
                return Err(ParseError::InvalidMixedFraction);
            }
        }
        Expr::Mul(a, b) => match &**b {
            Expr::Num(_) => (true, &**b, Some(&**a)),
            Expr::UnaryMinus(x) => {
                if let Expr::Num(_) = &**x {
                    (false, &**b, Some(&**a))
                } else {
                    return Err(ParseError::InvalidMixedFraction);
                }
            }
            _ => return Err(ParseError::InvalidMixedFraction),
        },
        _ => return Err(ParseError::InvalidMixedFraction),
    };
    let (rhs_top, input) = parse_power(input, false)?;
    if let Expr::Num(_) = rhs_top {
    } else {
        return Err(ParseError::InvalidMixedFraction);
    }
    let (_, input) = parse_fixed_symbol(input, Symbol::Div)?;
    let (rhs_bottom, input) = parse_power(input, false)?;
    if let Expr::Num(_) = rhs_bottom {
    } else {
        return Err(ParseError::InvalidMixedFraction);
    }
    let rhs = Box::new(Expr::Div(Box::new(rhs_top), Box::new(rhs_bottom)));
    let mixed_fraction = if positive {
        Expr::Add(Box::new(lhs.clone()), rhs)
    } else {
        Expr::Sub(Box::new(lhs.clone()), rhs)
    };
    let mixed_fraction = if let Some(other_factor) = other_factor {
        Expr::Mul(Box::new(other_factor.clone()), Box::new(mixed_fraction))
    } else {
        mixed_fraction
    };
    Ok((mixed_fraction, input))
}

fn parse_multiplication_cont<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Mul)?;
    let (b, input) = parse_power(input, true)?;
    Ok((b, input))
}

fn parse_division_cont<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Div)?;
    let (b, input) = parse_power(input, true)?;
    Ok((b, input))
}

fn parse_multiplicative<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (mut res, mut input) = parse_power(input, true)?;
    loop {
        if let Ok((term, remaining)) = parse_multiplication_cont(input) {
            res = Expr::Mul(Box::new(res.clone()), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_division_cont(input) {
            res = Expr::Div(Box::new(res.clone()), Box::new(term));
            input = remaining;
        } else if let Ok((new_res, remaining)) = parse_mixed_fraction(input, &res) {
            res = new_res;
            input = remaining;
        } else if let Ok((new_res, remaining)) = parse_apply_cont(input, &res) {
            res = new_res;
            input = remaining;
        } else {
            break;
        }
    }
    Ok((res, input))
}

fn parse_implicit_addition<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (res, input) = parse_multiplicative(input)?;
    if let Ok((rhs, remaining)) = parse_implicit_addition(input) {
        match (&res, &rhs) {
            // n i n i, n i i n i i, etc. (n: number literal, i: identifier)
            (Expr::ApplyMul(_, _), Expr::ApplyMul(_, _))
            | (Expr::ApplyMul(_, _), Expr::ImplicitAdd(_, _)) => {
                return Ok((Expr::ImplicitAdd(Box::new(res), Box::new(rhs)), remaining))
            }
            _ => (),
        };
    }
    Ok((res, input))
}

fn parse_addition_cont<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Add)?;
    let (b, input) = parse_implicit_addition(input)?;
    Ok((b, input))
}

fn parse_subtraction_cont<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Sub)?;
    let (b, input) = parse_implicit_addition(input)?;
    Ok((b, input))
}

fn parse_to_cont<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (_, input) = parse_fixed_symbol(input, Symbol::ArrowConversion)?;
    let (b, input) = parse_implicit_addition(input)?;
    Ok((b, input))
}

fn parse_additive<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (mut res, mut input) = parse_implicit_addition(input)?;
    loop {
        if let Ok((term, remaining)) = parse_addition_cont(input) {
            res = Expr::Add(Box::new(res), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_subtraction_cont(input) {
            res = Expr::Sub(Box::new(res), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_to_cont(input) {
            res = Expr::As(Box::new(res), Box::new(term));
            input = remaining;
        } else {
            break;
        }
    }
    Ok((res, input))
}

fn parse_function<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    let (lhs, input) = parse_additive(input)?;
    if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Fn) {
        if let Expr::Ident(s) = lhs {
            let (rhs, remaining) = parse_function(remaining)?;
            return Ok((Expr::Fn(s, Box::new(rhs)), remaining));
        } else {
            return Err(ParseError::ExpectedIdentifierAsArgument);
        }
    }
    Ok((lhs, input))
}

pub(crate) fn parse_expression<'a, 'b>(input: &'b [Token<'a>]) -> ParseResult<'a, 'b> {
    parse_function(input)
}

pub(crate) fn parse_tokens<'a, 'b>(input: &'b [Token<'a>]) -> Result<Expr<'a>, ParseError> {
    let (res, remaining) = parse_expression(input)?;
    if !remaining.is_empty() {
        return Err(ParseError::UnexpectedInput);
    }
    Ok(res)
}
