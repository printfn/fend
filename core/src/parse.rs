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

// parse an integer consisting of only digits in base 10
fn parse_number(input: &str) -> ParseResult<BigRat> {
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
                return Ok((res, input));
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

fn parse_multiplicative(input: &str) -> ParseResult<BigRat> {
    let mut factors = vec![];
    let (a, mut input) = parse_number(input)?;
    factors.push(a);
    loop {
        match parse_fixed_char(input, '*') {
            Err(_) => break,
            Ok((_, remaining)) => input = remaining,
        }
        match parse_number(input) {
            Err(_) => break,
            Ok((next_factor, remaining)) => {
                factors.push(next_factor);
                input = remaining;
            }
        }
    }
    let mut product = 1.into();
    for factor in factors {
        product = product * factor;
    }
    Ok((product, input))
}

fn parse_addition_cont(input: &str) -> ParseResult<BigRat> {
    let (_, input) = parse_fixed_char(input, '+')?;
    let (b, input) = parse_multiplicative(input)?;
    Ok((b, input))
}

fn parse_subtraction_cont(input: &str) -> ParseResult<BigRat> {
    let (_, input) = parse_fixed_char(input, '-')?;
    let (b, input) = parse_multiplicative(input)?;
    Ok((BigRat::from(0) - b, input))
}

fn parse_additive(input: &str) -> ParseResult<BigRat> {
    let mut terms = vec![];
    let (a, mut input) = parse_multiplicative(input)?;
    terms.push(a);
    loop {
        if let Ok((term, remaining)) = parse_addition_cont(input) {
            terms.push(term);
            input = remaining;
        } else if let Ok((term, remaining)) = parse_subtraction_cont(input) {
            terms.push(term);
            input = remaining;
        } else {
            break;
        }
    }
    let mut sum = 0.into();
    for term in terms {
        sum = sum + term;
    }
    Ok((sum, input))
}

pub fn parse_expression(input: &str) -> ParseResult<BigRat> {
    parse_additive(input)
}
