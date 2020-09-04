use crate::err::IntErr;
use crate::interrupt::{test_int, Interrupt};
use crate::num::{Base, Number};
use std::{
    convert::TryInto,
    fmt::{Display, Error, Formatter},
};

#[derive(Clone)]
pub enum Token {
    Num(Number),
    Ident(String),
    Symbol(Symbol),
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Symbol {
    OpenParens,
    CloseParens,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    ArrowConversion,
    Factorial,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::OpenParens => "(",
            Self::CloseParens => ")",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Pow => "^",
            Self::ArrowConversion => "->",
            Self::Factorial => "!",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

fn parse_char(input: &str) -> Result<(char, &str), IntErr<String>> {
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
        Err("Expected a character".to_string())?
    }
}

fn consume_char(input: &mut &str) -> Result<char, IntErr<String>> {
    match parse_char(input) {
        Ok((ch, remaining_input)) => {
            *input = remaining_input;
            Ok(ch)
        }
        Err(_) => Err("Expected a character".to_string())?,
    }
}

fn parse_ascii_digit(input: &str, base: Base) -> Result<(u8, &str), IntErr<String>> {
    let (ch, input) = parse_char(input)?;
    let possible_digit = ch.to_digit(base.base_as_u8().into());
    if let Some(digit) = possible_digit.and_then(|d| <u32 as TryInto<u8>>::try_into(d).ok()) {
        Ok((digit, input))
    } else {
        Err(format!("Expected a digit, found '{}'", ch))?
    }
}

fn parse_fixed_char(input: &str, ch: char) -> Result<((), &str), IntErr<String>> {
    let (parsed_ch, input) = parse_char(input)?;
    if parsed_ch == ch {
        Ok(((), input))
    } else {
        Err(format!("Expected '{}', found '{}'", parsed_ch, ch))?
    }
}

// Parses a plain integer with no whitespace and no base prefix.
// Leading minus sign is not allowed.
fn parse_integer<'a>(
    input: &'a str,
    allow_digit_separator: bool,
    allow_leading_zeroes: bool,
    base: Base,
    process_digit: &mut impl FnMut(u8) -> Result<(), IntErr<String>>,
) -> Result<((), &'a str), IntErr<String>> {
    let (digit, mut input) = parse_ascii_digit(input, base)?;
    process_digit(digit)?;
    let leading_zero = digit == 0;
    let mut parsed_digit_separator;
    loop {
        if let Ok((_, remaining)) = parse_fixed_char(input, '_') {
            input = remaining;
            parsed_digit_separator = true;
            if !allow_digit_separator {
                return Err("Digit separators are not allowed".to_string())?;
            }
        } else {
            parsed_digit_separator = false;
        }
        match parse_ascii_digit(input, base) {
            Err(_) => {
                if parsed_digit_separator {
                    return Err("Digit separators can only occur between digits".to_string())?;
                }
                break;
            }
            Ok((digit, next_input)) => {
                if leading_zero && !allow_leading_zeroes {
                    return Err("Integer literals cannot have leading zeroes".to_string())?;
                }
                process_digit(digit)?;
                input = next_input;
            }
        }
    }
    Ok(((), input))
}

fn parse_base_prefix(input: &str) -> Result<(Base, &str), IntErr<String>> {
    // 0x -> 16
    // 0o -> 8
    // 0b -> 2
    // base# -> base (where 2 <= base <= 36)
    // case-sensitive, no whitespace allowed
    if let Ok((_, input)) = parse_fixed_char(input, '0') {
        let (ch, input) = parse_char(input)?;
        Ok((
            match ch {
                'x' => Base::Hex,
                'o' => Base::Octal,
                'b' => Base::Binary,
                _ => {
                    return Err(
                        "Unable to parse a valid base prefix, expected 0x, 0o or 0b".to_string()
                    )?
                }
            },
            input,
        ))
    } else {
        let mut custom_base: u8 = 0;
        let (_, input) = parse_integer(input, false, false, Base::Decimal, &mut |digit| {
            if custom_base > 3 {
                return Err("Base cannot be larger than 36".to_string())?;
            }
            custom_base = 10 * custom_base + digit;
            if custom_base > 36 {
                return Err("Base cannot be larger than 36".to_string())?;
            }
            Ok(())
        })?;
        if custom_base < 2 {
            return Err("Base must be at least 2".to_string())?;
        }
        let (_, input) = parse_fixed_char(input, '#')?;
        Ok((Base::Custom(custom_base), input))
    }
}

fn parse_basic_number<'a>(
    input: &'a str,
    base: Base,
    allow_zero: bool,
    int: &impl Interrupt,
) -> Result<(Number, &'a str), IntErr<String>> {
    // parse integer component
    let mut res = Number::zero_with_base(base);
    let (_, mut input) = parse_integer(
        input,
        true,
        base.allow_leading_zeroes(),
        base,
        &mut |digit| {
            let base_as_u64: u64 = base.base_as_u8().into();
            res = res
                .clone()
                .mul(base_as_u64.into(), int)?
                .add(u64::from(digit).into(), int)?;
            Ok(())
        },
    )?;

    // parse decimal point and at least one digit
    if let Ok((_, remaining)) = parse_fixed_char(input, '.') {
        let (_, remaining) = parse_integer(remaining, true, true, base, &mut |digit| {
            res.add_digit_in_base(digit.into(), base, int)?;
            Ok(())
        })?;
        input = remaining;
    }

    if !allow_zero && res.is_zero() {
        return Err("Invalid number: 0".to_string())?;
    }

    // parse optional exponent, but only for base 10 and below
    if base.base_as_u8() <= 10 {
        if let Ok((_, remaining)) = parse_fixed_char(input, 'e') {
            input = remaining;
            let mut negative_exponent = false;
            if let Ok((_, remaining)) = parse_fixed_char(input, '-') {
                negative_exponent = true;
                input = remaining;
            }
            let mut exp = Number::zero_with_base(Base::Decimal);
            let (_, remaining) = parse_integer(input, true, true, Base::Decimal, &mut |digit| {
                exp = (exp.clone().mul(10.into(), int)?).add(u64::from(digit).into(), int)?;
                Ok(())
            })?;
            if negative_exponent {
                exp = -exp;
            }
            let base_as_u64: u64 = base.base_as_u8().into();
            let base_as_number: Number = base_as_u64.into();
            res = res.mul(base_as_number.pow(exp, int)?, int)?;
            input = remaining;
        }
    }

    Ok((res, input))
}

fn parse_number_internal<'a>(
    mut input: &'a str,
    int: &impl Interrupt,
) -> Result<(Number, &'a str), IntErr<String>> {
    let base = if let Ok((base, remaining)) = parse_base_prefix(input) {
        input = remaining;
        base
    } else {
        Base::Decimal
    };

    let (res, input) = parse_basic_number(input, base, true, int)?;

    Ok((res, input))
}

fn parse_number(input: &mut &str, int: &impl Interrupt) -> Result<Token, IntErr<String>> {
    let (num, remaining_input) = parse_number_internal(input, int)?;
    *input = remaining_input;
    Ok(Token::Num(num))
}

// checks if the char is valid only by itself
fn is_valid_in_ident_char(ch: char) -> bool {
    // percent, per mille, double quote, single quote, unicode single and double quotes
    // %‰\"'’”
    let allowed_symbols = "%\u{2030}\"'\u{2019}\u{201d}";
    allowed_symbols.contains(ch)
}

// normal rules for identifiers
fn is_valid_in_ident(ch: char, first: bool) -> bool {
    ch.is_alphabetic() || (!first && ".0123456789".contains(ch))
}

fn parse_ident(input: &mut &str) -> Result<Token, IntErr<String>> {
    let first_char = consume_char(input)?;
    if !is_valid_in_ident(first_char, true) {
        if is_valid_in_ident_char(first_char) {
            return Ok(Token::Ident(first_char.to_string()));
        }
        return Err(format!(
            "Character '{}' is not valid at the beginning of an identifier",
            first_char
        ))?;
    }
    let mut ident = first_char.to_string();
    while let Some(next_char) = input.chars().next() {
        if !is_valid_in_ident(next_char, false) {
            break;
        }
        consume_char(input)?;
        ident.push(next_char);
    }
    Ok(match ident.as_str() {
        "to" => Token::Symbol(Symbol::ArrowConversion),
        _ => Token::Ident(ident),
    })
}

pub fn lex(mut input: &str, int: &impl Interrupt) -> Result<Vec<Token>, IntErr<String>> {
    let mut res = vec![];
    loop {
        test_int(int)?;
        match input.chars().next() {
            Some(ch) => {
                if ch.is_whitespace() {
                    consume_char(&mut input)?;
                } else if ch.is_ascii_digit() {
                    res.push(parse_number(&mut input, int)?);
                } else if is_valid_in_ident(ch, true) || is_valid_in_ident_char(ch) {
                    res.push(parse_ident(&mut input)?);
                } else {
                    match consume_char(&mut input)? {
                        '(' => res.push(Token::Symbol(Symbol::OpenParens)),
                        ')' => res.push(Token::Symbol(Symbol::CloseParens)),
                        '+' => res.push(Token::Symbol(Symbol::Add)),
                        '!' => res.push(Token::Symbol(Symbol::Factorial)),
                        '-' => {
                            if input.starts_with('>') {
                                consume_char(&mut input)?;
                                res.push(Token::Symbol(Symbol::ArrowConversion))
                            } else {
                                res.push(Token::Symbol(Symbol::Sub))
                            }
                        }
                        '*' => {
                            if input.starts_with('*') {
                                consume_char(&mut input)?;
                                res.push(Token::Symbol(Symbol::Pow))
                            } else {
                                res.push(Token::Symbol(Symbol::Mul))
                            }
                        }
                        '/' => res.push(Token::Symbol(Symbol::Div)),
                        '^' => res.push(Token::Symbol(Symbol::Pow)),
                        _ => return Err(format!("Unexpected character '{}'", ch))?,
                    }
                }
            }
            None => return Ok(res),
        }
    }
}
