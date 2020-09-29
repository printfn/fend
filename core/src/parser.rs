use crate::ast::Expr;
use crate::lexer::{Symbol, Token};
use std::fmt::{Display, Error, Formatter};

pub enum ParseError {
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
    ExpectedDotInLambda,
}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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
            Self::ExpectedDotInLambda => write!(f, "Missing '.' in lambda (expected e.g. \\x.x)"),
        }
    }
}
impl crate::err::Error for ParseError {}

type ParseResult<'a, T> = Result<(T, &'a [Token<'a>]), ParseError>;

fn parse_token<'a>(input: &'a [Token<'a>]) -> ParseResult<Token<'a>> {
    if input.is_empty() {
        Err(ParseError::ExpectedAToken)
    } else {
        Ok((input[0].clone(), &input[1..]))
    }
}

fn parse_fixed_symbol<'a>(input: &'a [Token], symbol: Symbol) -> ParseResult<'a, ()> {
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

fn parse_number<'a>(input: &'a [Token]) -> ParseResult<'a, Expr> {
    match parse_token(input)? {
        (Token::Num(num), remaining) => Ok((Expr::Num(num), remaining)),
        _ => Err(ParseError::ExpectedANumber),
    }
}

fn parse_ident<'a>(input: &'a [Token]) -> ParseResult<'a, Expr> {
    match parse_token(input)? {
        (Token::Ident(ident), remaining) => Ok((Expr::Ident(ident.to_string()), remaining)),
        _ => Err(ParseError::ExpectedIdentifier),
    }
}

fn parse_parens<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (_, input) = parse_fixed_symbol(input, Symbol::OpenParens)?;
    let (inner, input) = parse_expression(input, options)?;
    let (_, input) = parse_fixed_symbol(input, Symbol::CloseParens)?;
    Ok((Expr::Parens(Box::new(inner)), input))
}

fn parse_backslash_lambda<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Backslash)?;
    let (ident, input) = if let (Expr::Ident(ident), input) = parse_ident(input)? {
        (ident, input)
    } else {
        return Err(ParseError::ExpectedIdentifier);
    };
    let (_, input) =
        parse_fixed_symbol(input, Symbol::Dot).map_err(|_| ParseError::ExpectedDotInLambda)?;
    let (rhs, input) = parse_function(input, options)?;
    Ok((Expr::Fn(ident, Box::new(rhs)), input))
}

fn parse_parens_or_literal<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (token, _) = parse_token(input)?;

    match token {
        Token::Num(_) => parse_number(input),
        Token::Ident(_) => parse_ident(input),
        Token::Symbol(Symbol::OpenParens) => parse_parens(input, options),
        Token::Symbol(Symbol::Backslash) => parse_backslash_lambda(input, options),
        Token::Symbol(..) => Err(ParseError::ExpectedNumIdentOrParen),
    }
}

// parse inner division with the '|' operator.
fn parse_inner_div<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (mut result, mut input) = parse_parens_or_literal(input, options)?;
    if options.gnu_compatible {
        while let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::InnerDiv) {
            let (rhs, remaining) = parse_parens_or_literal(remaining, options)?;
            result = Expr::Div(Box::new(result), Box::new(rhs));
            input = remaining;
        }
    }
    Ok((result, input))
}

fn parse_factorial<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (mut res, mut input) = parse_inner_div(input, options)?;
    while let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Factorial) {
        res = Expr::Factorial(Box::new(res));
        input = remaining;
    }
    Ok((res, input))
}

fn parse_power<'a>(
    input: &'a [Token],
    allow_unary: bool,
    options: ParseOptions,
) -> ParseResult<'a, Expr> {
    if allow_unary {
        if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Sub) {
            let (result, remaining) = parse_power(remaining, true, options)?;
            return Ok((Expr::UnaryMinus(Box::new(result)), remaining));
        }
        if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Add) {
            let (result, remaining) = parse_power(remaining, true, options)?;
            return Ok((Expr::UnaryPlus(Box::new(result)), remaining));
        }
        if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Div) {
            let (result, remaining) = parse_power(remaining, true, options)?;
            return Ok((Expr::UnaryDiv(Box::new(result)), remaining));
        }
    }
    let (mut result, mut input) = parse_factorial(input, options)?;
    if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Pow) {
        let (rhs, remaining) = parse_power(remaining, true, options)?;
        result = Expr::Pow(Box::new(result), Box::new(rhs));
        input = remaining;
    }
    Ok((result, input))
}

fn parse_apply_cont<'a>(
    input: &'a [Token],
    lhs: &Expr,
    options: ParseOptions,
) -> ParseResult<'a, Expr> {
    let (rhs, input) = parse_power(input, false, options)?;
    // This should never be called in GNU-compatible mode
    assert!(!options.gnu_compatible);
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

// parse apply/multiply with higher precedence (left-associative)
fn parse_inner_apply<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (mut res, mut input) = parse_power(input, true, options)?;
    if options.gnu_compatible {
        while let Ok((rhs, remaining)) = parse_power(input, false, options) {
            res = Expr::Apply(Box::new(res), Box::new(rhs));
            input = remaining;
        }
    }
    Ok((res, input))
}

fn parse_multiplication_cont<'a>(
    input: &'a [Token],
    options: ParseOptions,
) -> ParseResult<'a, Expr> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Mul)?;
    let (b, input) = parse_inner_apply(input, options)?;
    Ok((b, input))
}

fn parse_division_cont<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Div)?;
    let (b, input) = parse_inner_apply(input, options)?;
    Ok((b, input))
}

fn parse_multiplicative<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (mut res, mut input) = parse_inner_apply(input, options)?;
    loop {
        if let Ok((term, remaining)) = parse_multiplication_cont(input, options) {
            res = Expr::Mul(Box::new(res), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_division_cont(input, options) {
            res = Expr::Div(Box::new(res), Box::new(term));
            input = remaining;
        } else if options.gnu_compatible {
            // apply can't be parsed here if we're GNU compatible
            break;
        } else if let Ok((new_res, remaining)) = parse_apply_cont(input, &res, options) {
            res = new_res;
            input = remaining;
        } else {
            break;
        }
    }
    Ok((res, input))
}

fn parse_compound_fraction<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (res, input) = parse_multiplicative(input, options)?;
    if options.gnu_compatible {
        // don't parse mixed fractions or implicit addition (e.g. 3'6") when gnu_compatible is set
        return Ok((res, input));
    }
    if let Ok((rhs, remaining)) = parse_multiplicative(input, options) {
        match (&res, &rhs) {
            // n n/n (n: number literal)
            (Expr::Num(_), Expr::Div(b, c)) => {
                if let (Expr::Num(_), Expr::Num(_)) = (&**b, &**c) {
                    return Ok((Expr::Add(Box::new(res), Box::new(rhs)), remaining));
                }
            }
            // -n n/n
            (Expr::UnaryMinus(a), Expr::Div(b, c)) => {
                if let (Expr::Num(_), Expr::Num(_), Expr::Num(_)) = (&**a, &**b, &**c) {
                    return Ok((
                        // note that res = '-<num>' here because unary minus has a higher precedence
                        Expr::Sub(Box::new(res), Box::new(rhs)),
                        remaining,
                    ));
                }
            }
            // n i n i, n i i n i i, etc. (n: number literal, i: identifier)
            (Expr::ApplyMul(_, _), Expr::ApplyMul(_, _)) => {
                return Ok((Expr::Add(Box::new(res), Box::new(rhs)), remaining))
            }
            _ => (),
        };
    }
    Ok((res, input))
}

fn parse_addition_cont<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Add)?;
    let (b, input) = parse_compound_fraction(input, options)?;
    Ok((b, input))
}

fn parse_subtraction_cont<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (_, input) = parse_fixed_symbol(input, Symbol::Sub)?;
    let (b, input) = parse_compound_fraction(input, options)?;
    Ok((b, input))
}

fn parse_to_cont<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (_, input) = parse_fixed_symbol(input, Symbol::ArrowConversion)?;
    let (b, input) = parse_compound_fraction(input, options)?;
    Ok((b, input))
}

fn parse_additive<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (mut res, mut input) = parse_compound_fraction(input, options)?;
    loop {
        if let Ok((term, remaining)) = parse_addition_cont(input, options) {
            res = Expr::Add(Box::new(res), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_subtraction_cont(input, options) {
            res = Expr::Sub(Box::new(res), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_to_cont(input, options) {
            res = Expr::As(Box::new(res), Box::new(term));
            input = remaining;
        } else {
            break;
        }
    }
    Ok((res, input))
}

fn parse_function<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    let (lhs, input) = parse_additive(input, options)?;
    if let Ok((_, remaining)) = parse_fixed_symbol(input, Symbol::Fn) {
        if let Expr::Ident(s) = lhs {
            let (rhs, remaining) = parse_function(remaining, options)?;
            return Ok((Expr::Fn(s, Box::new(rhs)), remaining));
        } else {
            return Err(ParseError::ExpectedIdentifierAsArgument);
        }
    }
    Ok((lhs, input))
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ParseOptions {
    pub gnu_compatible: bool,
}

impl ParseOptions {
    pub const fn new_for_gnu_units() -> Self {
        Self {
            gnu_compatible: true,
        }
    }
}

pub fn parse_expression<'a>(input: &'a [Token], options: ParseOptions) -> ParseResult<'a, Expr> {
    parse_function(input, options)
}

pub fn parse_tokens(input: &[Token], options: ParseOptions) -> Result<Expr, ParseError> {
    let (res, remaining) = parse_expression(input, options)?;
    if !remaining.is_empty() {
        return Err(ParseError::UnexpectedInput);
    }
    Ok(res)
}
