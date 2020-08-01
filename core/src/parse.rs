use crate::ast::Expr;
use crate::num::bigrat::BigRat;
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

fn parse_fixed_char(input: &str, ch: char) -> ParseResult<()> {
    let (parsed_ch, input) = parse_char(input)?;
    if parsed_ch == ch {
        Ok(((), input))
    } else {
        Err(format!("Expected '{}', found '{}'", parsed_ch, ch))
    }
}

// parse an integer consisting of only digits in base 10
fn parse_number(input: &str) -> ParseResult<Expr> {
    let (_, input) = skip_whitespace(input)?;
    let (digit, mut input) = parse_ascii_digit(input)?;
    let mut res = BigRat::from(digit);
    loop {
        match parse_ascii_digit(input) {
            Err(_) => {
                // parse decimal point and at least one digit
                if let Ok((_, remaining)) = parse_fixed_char(input, '.') {
                    let (digit, remaining) = parse_ascii_digit(remaining)?;
                    input = remaining;
                    res.add_decimal_digit(digit);
                    loop {
                        match parse_ascii_digit(input) {
                            Err(_) => break,
                            Ok((digit, next_input)) => {
                                res.add_decimal_digit(digit);
                                input = next_input;
                            }
                        }
                    }
                }
                let (_, input) = skip_whitespace(input)?;
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

fn parse_ident(input: &str) -> ParseResult<Expr> {
    let (_, input) = skip_whitespace(input)?;
    let (ch, mut input) = parse_char(input)?;
    if !ch.is_alphabetic() {
        return Err(format!("Found invalid character in identifier: '{}'", ch))
    }
    let mut ident = ch.to_string();
    loop {
        if let Ok((ch, remaining)) = parse_char(input) {
            if ch.is_alphanumeric() {
                ident.push(ch);
                input = remaining;
                continue;
            }
        }
        break;
    }
    Ok((Expr::Ident(ident), input))
}

fn parse_parens(input: &str) -> ParseResult<Expr> {
    let (_, input) = skip_whitespace(input)?;
    let (_, input) = parse_fixed_char(input, '(')?;
    let (inner, input) = parse_expression(input)?;
    let (_, input) = parse_fixed_char(input, ')')?;
    let (_, input) = skip_whitespace(input)?;
    Ok((Expr::Parens(Box::new(inner)), input))
}

fn parse_parens_or_literal(input: &str) -> ParseResult<Expr> {
    if let Ok((res, remaining)) = parse_number(input) {
        Ok((res, remaining))
    } else if let Ok((res, remaining)) = parse_ident(input) {
        Ok((res, remaining))
    } else if let Ok((res, remaining)) = parse_parens(input) {
        Ok((res, remaining))
    } else {
        Err("Expected literal or parentheses".to_string())
    }
}

fn parse_apply(input: &str) -> ParseResult<Expr> {
    let (a, input) = parse_parens_or_literal(input)?;
    Ok(if let Ok((b, input)) = parse_apply(input) {
        (Expr::Apply(Box::new(a), Box::new(b)), input)
    } else {
        (a, input)
    })
}

fn parse_power_cont(mut input: &str) -> ParseResult<Expr> {
    if let Ok((_, remaining)) = parse_fixed_char(input, '^') {
        input = remaining;
    } else if let Ok((_, remaining)) = parse_fixed_char(input, '*') {
        let (_, remaining) = parse_fixed_char(remaining, '*')?;
        input = remaining;
    } else {
        return Err("Expected ^ or **".to_string());
    }
    let (b, input) = parse_power(input)?;
    Ok((b, input))
}

fn parse_power(input: &str) -> ParseResult<Expr> {
    let (_, input) = skip_whitespace(input)?;
    if let Ok((_, remaining)) = parse_fixed_char(input, '-') {
        let (res, remaining) = parse_power(remaining)?;
        return Ok((Expr::Negate(Box::new(res)), remaining));
    }
    let (mut res, mut input) = parse_apply(input)?;
    if let Ok((term, remaining)) = parse_power_cont(input) {
        res = Expr::Pow(Box::new(res), Box::new(term));
        input = remaining;
    }
    Ok((res, input))
}

fn parse_multiplication_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '*')?;
    let (b, input) = parse_power(input)?;
    Ok((b, input))
}

fn parse_division_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '/')?;
    let (b, input) = parse_power(input)?;
    Ok((b, input))
}

fn parse_multiplicative(input: &str) -> ParseResult<Expr> {
    let (mut res, mut input) = parse_power(input)?;
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
