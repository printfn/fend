use crate::err::{IntErr, Interrupt, NeverInterrupt};
use crate::interrupt::test_int;
use crate::num::{Base, BaseOutOfRangeError, InvalidBasePrefixError, Number};
use std::{
    convert::TryInto,
    fmt::{Display, Error, Formatter},
};

#[derive(Clone)]
pub enum Token<'a> {
    Num(Number),
    Ident(&'a str),
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
    // In GNU mode only: | is division with higher precedence. Otherwise
    // it is an error
    InnerDiv,
    Pow,
    ArrowConversion,
    Factorial,
}

enum LexerError {
    ExpectedACharacter,
    ExpectedADigit(char),
    ExpectedChar(char, char),
    ExpectedDigitSeparator(char),
    DigitSeparatorsNotAllowed,
    DigitSeparatorsOnlyBetweenDigits,
    NoLeadingZeroes,
    BaseOutOfRange(BaseOutOfRangeError),
    InvalidBasePrefix(InvalidBasePrefixError),
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            LexerError::ExpectedACharacter => write!(f, "Expected a character"),
            LexerError::ExpectedADigit(ch) => write!(f, "Expected a digit, found '{}'", ch),
            LexerError::ExpectedChar(ex, fnd) => write!(f, "Expected '{}', found '{}'", ex, fnd),
            LexerError::ExpectedDigitSeparator(ch) => {
                write!(f, "Expected a digit separator, found {}", ch)
            }
            LexerError::DigitSeparatorsNotAllowed => write!(f, "Digit separators are not allowed"),
            LexerError::DigitSeparatorsOnlyBetweenDigits => {
                write!(f, "Digit separators can only occur between digits")
            }
            LexerError::NoLeadingZeroes => write!(f, "Integer literals cannot have leading zeroes"),
            LexerError::BaseOutOfRange(e) => write!(f, "{}", e),
            LexerError::InvalidBasePrefix(e) => write!(f, "{}", e),
        }
    }
}
impl crate::err::Error for LexerError {}

impl<I: Interrupt> From<BaseOutOfRangeError> for IntErr<LexerError, I> {
    fn from(e: BaseOutOfRangeError) -> Self {
        LexerError::BaseOutOfRange(e).into()
    }
}

impl<I: Interrupt> From<InvalidBasePrefixError> for IntErr<LexerError, I> {
    fn from(e: InvalidBasePrefixError) -> Self {
        LexerError::InvalidBasePrefix(e).into()
    }
}

impl<I: Interrupt> From<LexerError> for IntErr<String, I> {
    fn from(e: LexerError) -> Self {
        e.to_string().into()
    }
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
            Self::InnerDiv => "|",
            Self::Pow => "^",
            Self::ArrowConversion => "->",
            Self::Factorial => "!",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

fn parse_char(input: &str) -> Result<(char, &str), LexerError> {
    if let Some(ch) = input.chars().next() {
        let (_, b) = input.split_at(ch.len_utf8());
        Ok((ch, b))
    } else {
        Err(LexerError::ExpectedACharacter)
    }
}

fn parse_ascii_digit(input: &str, base: Base) -> Result<(u8, &str), LexerError> {
    let (ch, input) = parse_char(input)?;
    let possible_digit = ch.to_digit(base.base_as_u8().into());
    if let Some(digit) = possible_digit.and_then(|d| <u32 as TryInto<u8>>::try_into(d).ok()) {
        Ok((digit, input))
    } else {
        Err(LexerError::ExpectedADigit(ch))
    }
}

fn parse_fixed_char(input: &str, ch: char) -> Result<((), &str), LexerError> {
    let (parsed_ch, input) = parse_char(input)?;
    if parsed_ch == ch {
        Ok(((), input))
    } else {
        Err(LexerError::ExpectedChar(parsed_ch, ch))
    }
}

fn parse_digit_separator(input: &str) -> Result<((), &str), LexerError> {
    let (parsed_ch, input) = parse_char(input)?;
    if parsed_ch == '_' || parsed_ch == ',' {
        Ok(((), input))
    } else {
        Err(LexerError::ExpectedDigitSeparator(parsed_ch))
    }
}

// Parses a plain integer with no whitespace and no base prefix.
// Leading minus sign is not allowed.
fn parse_integer<'a, E, I: Interrupt>(
    input: &'a str,
    allow_digit_separator: bool,
    allow_leading_zeroes: bool,
    base: Base,
    process_digit: &mut impl FnMut(u8) -> Result<(), IntErr<E, I>>,
) -> Result<((), &'a str), IntErr<E, I>>
where
    IntErr<E, I>: From<LexerError>,
{
    let (digit, mut input) = parse_ascii_digit(input, base)?;
    process_digit(digit)?;
    let leading_zero = digit == 0;
    let mut parsed_digit_separator;
    loop {
        if let Ok((_, remaining)) = parse_digit_separator(input) {
            input = remaining;
            parsed_digit_separator = true;
            if !allow_digit_separator {
                return Err(LexerError::DigitSeparatorsNotAllowed)?;
            }
        } else {
            parsed_digit_separator = false;
        }
        match parse_ascii_digit(input, base) {
            Err(_) => {
                if parsed_digit_separator {
                    return Err(LexerError::DigitSeparatorsOnlyBetweenDigits)?;
                }
                break;
            }
            Ok((digit, next_input)) => {
                if leading_zero && !allow_leading_zeroes {
                    return Err(LexerError::NoLeadingZeroes)?;
                }
                process_digit(digit)?;
                input = next_input;
            }
        }
    }
    Ok(((), input))
}

fn parse_base_prefix(input: &str) -> Result<(Base, &str), IntErr<LexerError, NeverInterrupt>> {
    // 0x -> 16
    // 0d -> 10
    // 0o -> 8
    // 0b -> 2
    // base# -> base (where 2 <= base <= 36)
    // case-sensitive, no whitespace allowed
    if let Ok((_, input)) = parse_fixed_char(input, '0') {
        let (ch, input) = parse_char(input)?;
        Ok((Base::from_zero_based_prefix_char(ch)?, input))
    } else {
        let mut custom_base: u8 = 0;
        let (_, input) = parse_integer(
            input,
            false,
            false,
            Base::default(),
            &mut |digit| -> Result<(), IntErr<LexerError, NeverInterrupt>> {
                if custom_base > 3 {
                    return Err(BaseOutOfRangeError::BaseTooLarge)?;
                }
                custom_base = 10 * custom_base + digit;
                if custom_base > 36 {
                    return Err(BaseOutOfRangeError::BaseTooLarge)?;
                }
                Ok(())
            },
        )?;
        if custom_base < 2 {
            return Err(BaseOutOfRangeError::BaseTooSmall)?;
        }
        let (_, input) = parse_fixed_char(input, '#')?;
        Ok((Base::from_custom_base(custom_base)?, input))
    }
}

fn parse_basic_number<'a, I: Interrupt>(
    input: &'a str,
    base: Base,
    allow_zero: bool,
    int: &I,
) -> Result<(Number, &'a str), IntErr<String, I>> {
    // parse integer component
    let mut res = Number::zero_with_base(base);
    let (_, mut input) = parse_integer(
        input,
        true,
        base.allow_leading_zeroes(),
        base,
        &mut |digit| -> Result<(), IntErr<String, I>> {
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
        let (_, remaining) = parse_integer(remaining, true, true, base, &mut |digit| -> Result<
            (),
            IntErr<String, I>,
        > {
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
            // peek ahead to the next char to determine if we should continue parsing an exponent
            let abort = if let Ok((ch, _)) = parse_char(remaining) {
                // abort if there is a non-alphanumeric non-plus or minus char after 'e',
                // such as '(' or '/'
                !(ch.is_alphanumeric() || ch == '+' || ch == '-')
            } else {
                // if there is no more input after the 'e', abort
                true
            };
            if !abort {
                input = remaining;
                let mut negative_exponent = false;
                if let Ok((_, remaining)) = parse_fixed_char(input, '-') {
                    negative_exponent = true;
                    input = remaining;
                } else if let Ok((_, remaining)) = parse_fixed_char(input, '+') {
                    input = remaining;
                }
                let mut exp = Number::zero_with_base(base);
                let base_num = Number::from(u64::from(base.base_as_u8()));
                let (_, remaining) =
                    parse_integer(input, true, true, base, &mut |digit| -> Result<
                        (),
                        IntErr<String, I>,
                    > {
                        exp = (exp.clone().mul(base_num.clone(), int)?)
                            .add(u64::from(digit).into(), int)?;
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
    }

    Ok((res, input))
}

fn parse_number<'a, I: Interrupt>(
    input: &'a str,
    int: &I,
) -> Result<(Number, &'a str), IntErr<String, I>> {
    let (base, input) = parse_base_prefix(input).unwrap_or((Base::default(), input));
    let (res, input) = parse_basic_number(input, base, true, int)?;
    Ok((res, input))
}

pub fn is_valid_in_ident(ch: char, prev: Option<char>) -> bool {
    let allowed_chars = ",&_⅛¼⅜½⅝¾⅞⅙⅓⅔⅚⅕⅖⅗⅘°$℃℉℧℈℥℔¢£¥€₩₪₤₨฿₡₣₦₧₫₭₮₯₱﷼﹩￠￡￥￦㍱㍲㍳㍴㍶㎀㎁㎂㎃㎄㎅㎆㎇㎈㎉㎊㎋㎌㎍㎎㎏㎐㎑㎒㎓㎔㎕㎖㎗㎘㎙㎚㎛㎜㎝㎞㎟㎠㎡㎢㎣㎤㎥㎦㎧㎨㎩㎪㎫㎬㎭㎮㎯㎰㎱㎲㎳㎴㎵㎶㎷㎸㎹㎺㎻㎼㎽㎾㎿㏀㏁㏃㏄㏅㏆㏈㏉㏊㏌㏏㏐㏓㏔㏕㏖㏗㏙㏛㏜㏝";
    // these chars are only valid by themselves
    let only_valid_by_themselves = "%‰‱′″\"'’”";
    if only_valid_by_themselves.contains(ch)
        || only_valid_by_themselves.contains(prev.unwrap_or('a'))
    {
        prev.is_none()
    } else if ch.is_alphabetic() || allowed_chars.contains(ch) {
        true
    } else {
        prev.is_some() && ".0123456789".contains(ch)
    }
}

fn parse_ident(input: &str) -> Result<(Token, &str), String> {
    let (first_char, _) = parse_char(input).map_err(|e| e.to_string())?;
    if !is_valid_in_ident(first_char, None) {
        return Err(format!(
            "Character '{}' is not valid at the beginning of an identifier",
            first_char
        ))?;
    }
    let mut byte_idx = first_char.len_utf8();
    let (_, mut remaining) = input.split_at(byte_idx);
    let mut prev_char = first_char;
    while let Ok((next_char, remaining_input)) = parse_char(remaining) {
        if !is_valid_in_ident(next_char, Some(prev_char)) {
            break;
        }
        remaining = remaining_input;
        byte_idx += next_char.len_utf8();
        prev_char = next_char;
    }
    let (ident, input) = input.split_at(byte_idx);
    Ok((
        match ident {
            "to" | "as" => Token::Symbol(Symbol::ArrowConversion),
            "per" => Token::Symbol(Symbol::Div),
            _ => Token::Ident(ident),
        },
        input,
    ))
}

pub fn lex<'a, I: Interrupt>(
    mut input: &'a str,
    int: &I,
) -> Result<Vec<Token<'a>>, IntErr<String, I>> {
    let mut res = vec![];
    loop {
        test_int(int)?;
        match input.chars().next() {
            Some(ch) => {
                if ch.is_whitespace() {
                    let (_, remaining) = input.split_at(ch.len_utf8());
                    input = remaining;
                } else if ch.is_ascii_digit() {
                    let (num, remaining) = parse_number(input, int)?;
                    input = remaining;
                    res.push(Token::Num(num));
                } else if is_valid_in_ident(ch, None) {
                    let (ident, remaining) = parse_ident(input)?;
                    input = remaining;
                    res.push(ident);
                } else {
                    {
                        let (_, remaining) = input.split_at(ch.len_utf8());
                        input = remaining;
                    }
                    match ch {
                        '(' => res.push(Token::Symbol(Symbol::OpenParens)),
                        ')' => res.push(Token::Symbol(Symbol::CloseParens)),
                        '+' => res.push(Token::Symbol(Symbol::Add)),
                        '!' => res.push(Token::Symbol(Symbol::Factorial)),
                        '-' => {
                            if input.starts_with('>') {
                                let (_, remaining) = input.split_at('>'.len_utf8());
                                input = remaining;
                                res.push(Token::Symbol(Symbol::ArrowConversion))
                            } else {
                                res.push(Token::Symbol(Symbol::Sub))
                            }
                        }
                        '*' => {
                            if input.starts_with('*') {
                                let (_, remaining) = input.split_at('*'.len_utf8());
                                input = remaining;
                                res.push(Token::Symbol(Symbol::Pow))
                            } else {
                                res.push(Token::Symbol(Symbol::Mul))
                            }
                        }
                        '/' => res.push(Token::Symbol(Symbol::Div)),
                        '|' => res.push(Token::Symbol(Symbol::InnerDiv)),
                        '^' => res.push(Token::Symbol(Symbol::Pow)),
                        _ => return Err(format!("Unexpected character '{}'", ch))?,
                    }
                }
            }
            None => return Ok(res),
        }
    }
}
