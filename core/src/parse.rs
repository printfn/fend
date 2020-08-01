use crate::num::bigrat::BigRat;
use crate::ast::Expr;
use std::convert::TryInto;

type ParseResult<'a, T> = Result<(T, &'a str), String>;

fn parse_char(input: &str) -> ParseResult<char> {
    let mut char_indices = input.char_indices();
    if let Some((_, ch)) = char_indices.next() {
        if let Some((idx, _)) = char_indices.next() {
            let (_a, b) = input.split_at(idx);
            Ok((ch, b))
        } else {
            let (empty, _b) = input.split_at(0);
            Ok((ch, empty))
        }
    } else {
        Err("Expected a character".to_string())
    }
}

pub fn skip_whitespace(mut input: &str) -> ParseResult<()> {
    loop {
        match parse_char(input) {
            Ok((ch, remaining)) => {
                if ch.is_whitespace() {
                    input = remaining;
                } else {
                    return Ok(((), input));
                }
            }
            Err(_) => return Ok(((), input)),
        }
    }
}

fn parse_ascii_digit(input: &str) -> ParseResult<i32> {
    let (ch, input) = parse_char(input)?;
    if let Some(digit) = ch.to_digit(10) {
        Ok((digit.try_into().unwrap(), input))
    } else {
        Err(format!("Expected a digit, found '{}'", ch))
    }
}

// parse an integer consisting of only digits in base 10
fn parse_number(input: &str) -> ParseResult<Expr> {
    let (_, mut input) = skip_whitespace(input)?;
    let negative = if let Ok((_, remaining)) = parse_fixed_char(input, '-') {
        let (_, remaining) = skip_whitespace(remaining)?;
        input = remaining;
        true
    } else {
        false
    };
    let (digit, mut input) = parse_ascii_digit(input)?;
    let mut res = BigRat::from(digit);
    loop {
        match parse_ascii_digit(input) {
            Err(_) => {
                let (_, input) = skip_whitespace(input)?;
                if negative {
                    res = BigRat::from(0) - res;
                }
                return Ok((Expr::Num(res), input));
            }
            Ok((digit, next_input)) => {
                res = res * 10.into();
                res = res + digit.into();
                input = next_input;
            }
        }
    }
}

fn parse_fixed_char(input: &str, ch: char) -> ParseResult<()> {
    let (parsed_ch, input) = parse_char(input)?;
    if parsed_ch == ch {
        Ok(((), input))
    } else {
        Err(format!("Expected '{}', found '{}'", parsed_ch, ch))
    }
}

fn parse_multiplication_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '*')?;
    let (b, input) = parse_number(input)?;
    Ok((b, input))
}

fn parse_division_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '/')?;
    let (b, input) = parse_number(input)?;
    Ok((b, input))
}

fn parse_multiplicative(input: &str) -> ParseResult<Expr> {
    let (mut res, mut input) = parse_number(input)?;
    loop {
        if let Ok((term, remaining)) = parse_multiplication_cont(input) {
            res = Expr::Mul(Box::new(res), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_division_cont(input) {
            res = Expr::Div(Box::new(res), Box::new(term));
            input = remaining;
        } else {
            break;
        }
    }
    Ok((res, input))
}

fn parse_addition_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '+')?;
    let (b, input) = parse_multiplicative(input)?;
    Ok((b, input))
}

fn parse_subtraction_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '-')?;
    let (b, input) = parse_multiplicative(input)?;
    Ok((b, input))
}

fn parse_additive(input: &str) -> ParseResult<Expr> {
    let (mut res, mut input) = parse_multiplicative(input)?;
    loop {
        if let Ok((term, remaining)) = parse_addition_cont(input) {
            res = Expr::Add(Box::new(res), Box::new(term));
            input = remaining;
        } else if let Ok((term, remaining)) = parse_subtraction_cont(input) {
            res = Expr::Sub(Box::new(res), Box::new(term));
            input = remaining;
        } else {
            break;
        }
    }
    Ok((res, input))
}

pub fn parse_expression(input: &str) -> ParseResult<Expr> {
    parse_additive(input)
}
