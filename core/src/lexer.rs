use crate::error::{IntErr, Interrupt};
use crate::num::{Base, BaseOutOfRangeError, InvalidBasePrefixError, Number};
use std::{convert::TryInto, fmt};

#[derive(Clone, Debug)]
pub(crate) enum Token<'a> {
    Num(Number<'a>),
    Ident(&'a str),
    Symbol(Symbol),
    Whitespace,
    StringLiteral(std::borrow::Cow<'a, str>),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub(crate) enum Symbol {
    OpenParens,
    CloseParens,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    ArrowConversion,
    Factorial,
    Fn,
    Backslash,
    Dot,
    Of,
}

pub(crate) enum Error {
    ExpectedACharacter,
    ExpectedADigit(char),
    ExpectedChar(char, char),
    ExpectedDigitSeparator(char),
    DigitSeparatorsNotAllowed,
    DigitSeparatorsOnlyBetweenDigits,
    BaseOutOfRange(BaseOutOfRangeError),
    InvalidBasePrefix(InvalidBasePrefixError),
    InvalidCharAtBeginningOfIdent(char),
    UnexpectedChar(char),
    BackslashInStringLiteral,
    UnterminatedStringLiteral,
    UnknownBackslashEscapeSequence(char),
    BackslashXOutOfRange,
    // todo remove this
    NumberParse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::ExpectedACharacter => write!(f, "Expected a character"),
            Self::ExpectedADigit(ch) => write!(f, "Expected a digit, found '{}'", ch),
            Self::ExpectedChar(ex, fnd) => write!(f, "Expected '{}', found '{}'", ex, fnd),
            Self::ExpectedDigitSeparator(ch) => {
                write!(f, "Expected a digit separator, found {}", ch)
            }
            Self::DigitSeparatorsNotAllowed => write!(f, "Digit separators are not allowed"),
            Self::DigitSeparatorsOnlyBetweenDigits => {
                write!(f, "Digit separators can only occur between digits")
            }
            Self::BaseOutOfRange(e) => write!(f, "{}", e),
            Self::InvalidBasePrefix(e) => write!(f, "{}", e),
            Self::InvalidCharAtBeginningOfIdent(ch) => {
                write!(f, "'{}' is not valid at the beginning of an identifier", ch)
            }
            Self::UnexpectedChar(ch) => write!(f, "Unexpected character '{}'", ch),
            Self::NumberParse(s) => write!(f, "{}", s),
            Self::BackslashInStringLiteral => {
                write!(f, "Backslash not currently allowed in string literal")
            }
            Self::UnterminatedStringLiteral => write!(f, "Unterminated string literal"),
            Self::UnknownBackslashEscapeSequence(ch) => {
                write!(f, "Unknown escape sequence: \\{}", ch)
            }
            Self::BackslashXOutOfRange => {
                write!(f, "Expected an escape sequence between \\x00 and \\x7f")
            }
        }
    }
}
impl crate::error::Error for Error {}

impl From<BaseOutOfRangeError> for Error {
    fn from(e: BaseOutOfRangeError) -> Self {
        Self::BaseOutOfRange(e)
    }
}

impl From<InvalidBasePrefixError> for Error {
    fn from(e: InvalidBasePrefixError) -> Self {
        Self::InvalidBasePrefix(e)
    }
}

impl<I: Interrupt> From<Error> for IntErr<String, I> {
    fn from(e: Error) -> Self {
        e.to_string().into()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
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
            Self::Fn => ":",
            Self::Backslash => "\"",
            Self::Dot => ".",
            Self::Of => "of",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

fn parse_char(input: &str) -> Result<(char, &str), Error> {
    input
        .chars()
        .next()
        .map_or(Err(Error::ExpectedACharacter), |ch| {
            let (_, b) = input.split_at(ch.len_utf8());
            Ok((ch, b))
        })
}

fn parse_ascii_digit(input: &str, base: Base) -> Result<(u8, &str), Error> {
    let (ch, input) = parse_char(input)?;
    let possible_digit = ch.to_digit(base.base_as_u8().into());
    possible_digit
        .and_then(|d| <u32 as TryInto<u8>>::try_into(d).ok())
        .map_or(Err(Error::ExpectedADigit(ch)), |digit| Ok((digit, input)))
}

fn parse_fixed_char(input: &str, ch: char) -> Result<((), &str), Error> {
    let (parsed_ch, input) = parse_char(input)?;
    if parsed_ch == ch {
        Ok(((), input))
    } else {
        Err(Error::ExpectedChar(ch, parsed_ch))
    }
}

fn parse_digit_separator(input: &str) -> Result<((), &str), Error> {
    let (parsed_ch, input) = parse_char(input)?;
    if parsed_ch == '_' || parsed_ch == ',' {
        Ok(((), input))
    } else {
        Err(Error::ExpectedDigitSeparator(parsed_ch))
    }
}

// Parses a plain integer with no whitespace and no base prefix.
// Leading minus sign is not allowed.
fn parse_integer<'a, E: From<Error>>(
    input: &'a str,
    allow_digit_separator: bool,
    base: Base,
    process_digit: &mut impl FnMut(u8) -> Result<(), E>,
) -> Result<((), &'a str), E> {
    let (digit, mut input) = parse_ascii_digit(input, base)?;
    process_digit(digit)?;
    let mut parsed_digit_separator;
    loop {
        if let Ok((_, remaining)) = parse_digit_separator(input) {
            input = remaining;
            parsed_digit_separator = true;
            if !allow_digit_separator {
                return Err(Error::DigitSeparatorsNotAllowed.into());
            }
        } else {
            parsed_digit_separator = false;
        }
        match parse_ascii_digit(input, base) {
            Err(_) => {
                if parsed_digit_separator {
                    return Err(Error::DigitSeparatorsOnlyBetweenDigits.into());
                }
                break;
            }
            Ok((digit, next_input)) => {
                process_digit(digit)?;
                input = next_input;
            }
        }
    }
    Ok(((), input))
}

fn parse_base_prefix(input: &str) -> Result<(Base, &str), Error> {
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
        let (_, input) = parse_integer(input, false, Base::default(), &mut |digit| -> Result<
            (),
            Error,
        > {
            let base_too_large = BaseOutOfRangeError::BaseTooLarge;
            let error = Error::BaseOutOfRange(base_too_large);
            if custom_base > 3 {
                return Err(error);
            }
            custom_base = 10 * custom_base + digit;
            if custom_base > 36 {
                return Err(error);
            }
            Ok(())
        })?;
        if custom_base < 2 {
            let base_too_small = BaseOutOfRangeError::BaseTooSmall;
            return Err(Error::BaseOutOfRange(base_too_small));
        }
        let (_, input) = parse_fixed_char(input, '#')?;
        Ok((Base::from_custom_base(custom_base)?, input))
    }
}

// Try and parse recurring digits in parentheses.
// '1.0(0)' -> success
// '1.0(a)', '1.0( 0)' -> Ok, but not parsed
// '1.0(3a)' -> Error

fn parse_recurring_digits<'a, I: Interrupt>(
    input: &'a str,
    number: &mut Number,
    num_nonrec_digits: usize,
    base: Base,
    int: &I,
) -> Result<((), &'a str), IntErr<String, I>> {
    let original_input = input;
    // If there's no '(': return Ok but don't parse anything
    if parse_fixed_char(input, '(').is_err() {
        return Ok(((), original_input));
    }
    let (_, input) = parse_fixed_char(input, '(')?;
    if parse_ascii_digit(input, base).is_err() {
        // return Ok if there were no digits
        return Ok(((), original_input));
    }
    let mut recurring_number_num = Number::from(0);
    let mut recurring_number_den = Number::from(1);
    let base_as_u64 = u64::from(base.base_as_u8());
    let (_, input) = parse_integer(input, true, base, &mut |digit| -> Result<
        (),
        IntErr<String, I>,
    > {
        let digit_as_u64 = u64::from(digit);
        recurring_number_num = recurring_number_num
            .clone()
            .mul(base_as_u64.into(), int)?
            .add(digit_as_u64.into(), int)?;
        recurring_number_den = recurring_number_den.clone().mul(base_as_u64.into(), int)?;
        Ok(())
    })?;
    recurring_number_den = recurring_number_den.clone().sub(1.into(), int)?;
    for _ in 0..num_nonrec_digits {
        recurring_number_den = recurring_number_den.clone().mul(base_as_u64.into(), int)?;
    }
    *number = number.clone().add(
        recurring_number_num
            .div(recurring_number_den, int)
            .map_err(IntErr::into_string)?,
        int,
    )?;
    // return an error if there are any other characters before the closing parentheses
    let (_, input) = parse_fixed_char(input, ')')?;
    Ok(((), input))
}

fn parse_basic_number<'a, I: Interrupt>(
    mut input: &'a str,
    base: Base,
    allow_zero: bool,
    int: &I,
) -> Result<(Number<'a>, &'a str), IntErr<String, I>> {
    // parse integer component
    let mut res = Number::zero_with_base(base);
    let base_as_u64 = u64::from(base.base_as_u8());

    if parse_fixed_char(input, '.').is_err() {
        let (_, remaining) = parse_integer(input, true, base, &mut |digit| -> Result<
            (),
            IntErr<String, I>,
        > {
            res = res
                .clone()
                .mul(base_as_u64.into(), int)?
                .add(u64::from(digit).into(), int)?;
            Ok(())
        })?;
        input = remaining;
    }

    // parse decimal point and at least one digit
    if let Ok((_, remaining)) = parse_fixed_char(input, '.') {
        let mut num_nonrec_digits = 0;
        let mut numerator = Number::zero_with_base(base);
        let mut denominator = Number::zero_with_base(base).add(1.into(), int)?;
        if parse_fixed_char(remaining, '(').is_err() {
            let (_, remaining) = parse_integer(remaining, true, base, &mut |digit| -> Result<
                (),
                IntErr<String, I>,
            > {
                numerator = numerator
                    .clone()
                    .mul(base_as_u64.into(), int)?
                    .add(u64::from(digit).into(), int)?;
                denominator = denominator.clone().mul(base_as_u64.into(), int)?;
                num_nonrec_digits += 1;
                Ok(())
            })?;
            input = remaining;
        } else {
            input = remaining;
        }
        res = res.add(
            numerator
                .div(denominator, int)
                .map_err(IntErr::into_string)?,
            int,
        )?;

        // try parsing recurring decimals
        let (_, remaining) = parse_recurring_digits(input, &mut res, num_nonrec_digits, base, int)?;
        input = remaining;
    }

    if !allow_zero && res.is_zero() {
        return Err("Invalid number: 0".to_string().into());
    }

    // parse optional exponent, but only for base 10 and below
    if base.base_as_u8() <= 10 {
        let (parsed_exponent, remaining) = if let Ok((_, remaining)) = parse_fixed_char(input, 'e')
        {
            (true, remaining)
        } else if let Ok((_, remaining)) = parse_fixed_char(input, 'E') {
            (true, remaining)
        } else {
            (false, "")
        };

        if parsed_exponent {
            // peek ahead to the next char to determine if we should continue parsing an exponent
            let abort = if let Ok((ch, _)) = parse_char(remaining) {
                // abort if there is a non-digit non-plus or minus char after 'e',
                // such as '(', '/' or 'a'. Note that this is only parsed in base <= 10,
                // so letters can never be digits. We do want to include all digits even for
                // base < 10 though to avoid 6#3e9 from being valid.
                !(ch.is_ascii_digit() || ch == '+' || ch == '-')
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
                let (_, remaining2) = parse_integer(input, true, base, &mut |digit| -> Result<
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
                let base_as_number: Number = base_as_u64.into();
                res = res.mul(base_as_number.pow(exp, int)?, int)?;
                input = remaining2;
            }
        }
    }

    Ok((res, input))
}

fn parse_number<'a, I: Interrupt>(
    input: &'a str,
    int: &I,
) -> Result<(Number<'a>, &'a str), IntErr<String, I>> {
    let (base, input) = parse_base_prefix(input).unwrap_or((Base::default(), input));
    let (res, input) = parse_basic_number(input, base, true, int)?;
    Ok((res, input))
}

pub(crate) fn is_valid_in_ident(ch: char, prev: Option<char>) -> bool {
    let allowed_chars = [
        ',', '&', '_', '⅛', '¼', '⅜', '½', '⅝', '¾', '⅞', '⅙', '⅓', '⅔', '⅚', '⅕', '⅖', '⅗', '⅘',
        '°', '$', '℃', '℉', '℧', '℈', '℥', '℔', '¢', '£', '¥', '€', '₩', '₪', '₤', '₨', '฿', '₡',
        '₣', '₦', '₧', '₫', '₭', '₮', '₯', '₱', '﷼', '﹩', '￠', '￡', '￥', '￦', '㍱', '㍲',
        '㍳', '㍴', '㍶', '㎀', '㎁', '㎂', '㎃', '㎄', '㎅', '㎆', '㎇', '㎈', '㎉', '㎊', '㎋',
        '㎌', '㎍', '㎎', '㎏', '㎐', '㎑', '㎒', '㎓', '㎔', '㎕', '㎖', '㎗', '㎘', '㎙', '㎚',
        '㎛', '㎜', '㎝', '㎞', '㎟', '㎠', '㎡', '㎢', '㎣', '㎤', '㎥', '㎦', '㎧', '㎨', '㎩',
        '㎪', '㎫', '㎬', '㎭', '㎮', '㎯', '㎰', '㎱', '㎲', '㎳', '㎴', '㎵', '㎶', '㎷', '㎸',
        '㎹', '㎺', '㎻', '㎼', '㎽', '㎾', '㎿', '㏀', '㏁', '㏃', '㏄', '㏅', '㏆', '㏈', '㏉',
        '㏊', '㏌', '㏏', '㏐', '㏓', '㏔', '㏕', '㏖', '㏗', '㏙', '㏛', '㏜', '㏝',
    ];
    // these chars are only valid by themselves
    let only_valid_by_themselves = ['%', '‰', '‱', '′', '″', '’', '”', 'π'];
    if only_valid_by_themselves.contains(&ch)
        || only_valid_by_themselves.contains(&prev.unwrap_or('a'))
    {
        prev.is_none()
    } else if ch.is_alphabetic() || allowed_chars.contains(&ch) {
        true
    } else {
        prev.is_some() && ".0123456789'\"".contains(ch)
    }
}

fn parse_ident(input: &str, allow_dots: bool) -> Result<(Token, &str), Error> {
    let (first_char, _) = parse_char(input)?;
    if !is_valid_in_ident(first_char, None) || first_char == '.' && !allow_dots {
        return Err(Error::InvalidCharAtBeginningOfIdent(first_char));
    }
    let mut byte_idx = first_char.len_utf8();
    let (_, mut remaining) = input.split_at(byte_idx);
    let mut prev_char = first_char;
    while let Ok((next_char, remaining_input)) = parse_char(remaining) {
        if !is_valid_in_ident(next_char, Some(prev_char)) || next_char == '.' && !allow_dots {
            break;
        }
        remaining = remaining_input;
        byte_idx += next_char.len_utf8();
        prev_char = next_char;
    }
    let (ident, input) = input.split_at(byte_idx);
    Ok((
        match ident {
            "to" | "as" | "in" => Token::Symbol(Symbol::ArrowConversion),
            "per" => Token::Symbol(Symbol::Div),
            "of" => Token::Symbol(Symbol::Of),
            _ => Token::Ident(ident),
        },
        input,
    ))
}

fn parse_symbol<'a>(ch: char, input: &mut &'a str) -> Result<Token<'a>, Error> {
    Ok(Token::Symbol(match ch {
        '(' => Symbol::OpenParens,
        ')' => Symbol::CloseParens,
        '+' => Symbol::Add,
        '!' => Symbol::Factorial,
        '-' => {
            if input.starts_with('>') {
                let (_, remaining) = input.split_at('>'.len_utf8());
                *input = remaining;
                Symbol::ArrowConversion
            } else {
                Symbol::Sub
            }
        }
        '*' => {
            if input.starts_with('*') {
                let (_, remaining) = input.split_at('*'.len_utf8());
                *input = remaining;
                Symbol::Pow
            } else {
                Symbol::Mul
            }
        }
        '/' => Symbol::Div,
        '^' => Symbol::Pow,
        ':' => Symbol::Fn,
        '=' => {
            if input.starts_with('>') {
                let (_, remaining) = input.split_at('>'.len_utf8());
                *input = remaining;
                Symbol::Fn
            } else {
                return Err(Error::UnexpectedChar(ch));
            }
        }
        '\\' => Symbol::Backslash,
        '.' => Symbol::Dot,
        _ => return Err(Error::UnexpectedChar(ch)),
    }))
}

fn parse_string_literal(input: &str) -> Result<(Token, &str), Error> {
    let (_, input) = input.split_at(1);
    let mut chars_iter = input.char_indices();
    let mut literal_length = None;
    let mut literal_string = String::new();
    while let Some((idx, ch)) = chars_iter.next() {
        if ch == '"' {
            literal_length = Some(idx);
            break;
        }
        if ch == '\\' {
            let (_, next) = chars_iter.next().ok_or(Error::UnterminatedStringLiteral)?;
            let escaped_char = match next {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'e' => '\u{1b}',
                'v' => '\u{0b}',
                'b' => '\u{8}',
                'x' => {
                    let (_, hex1) = chars_iter.next().ok_or(Error::UnterminatedStringLiteral)?;
                    let (_, hex2) = chars_iter.next().ok_or(Error::UnterminatedStringLiteral)?;
                    let hex1: u8 = hex1
                        .to_digit(8)
                        .ok_or(Error::BackslashXOutOfRange)?
                        .try_into()
                        .unwrap();
                    let hex2: u8 = hex2
                        .to_digit(16)
                        .ok_or(Error::BackslashXOutOfRange)?
                        .try_into()
                        .unwrap();
                    (hex1 * 16 + hex2) as u8 as char
                }
                _ => return Err(Error::UnknownBackslashEscapeSequence(next)),
            };
            literal_string.push(escaped_char);
        } else {
            literal_string.push(ch);
        }
    }
    let literal_length = literal_length.ok_or(Error::UnterminatedStringLiteral)?;
    let (_, remaining) = input.split_at(literal_length + 1);
    Ok((Token::StringLiteral(literal_string.into()), remaining))
}

pub(crate) struct Lexer<'a, 'b, I: Interrupt> {
    input: &'a str,
    // normally 0; 1 after backslash; 2 after ident after backslash
    after_backslash_state: u8,
    after_number: bool,
    int: &'b I,
}

impl<'a, 'b, I: Interrupt> Lexer<'a, 'b, I> {
    fn next_token(&mut self) -> Result<Option<Token<'a>>, IntErr<Error, I>> {
        while let Some(ch) = self.input.chars().next() {
            if !ch.is_whitespace() {
                break;
            }
            let (_, remaining) = self.input.split_at(ch.len_utf8());
            self.input = remaining;
        }
        Ok(Some(match self.input.chars().next() {
            Some(ch) => {
                if ch.is_whitespace() {
                    Token::Whitespace
                } else if ch.is_ascii_digit() || (ch == '.' && self.after_backslash_state == 0) {
                    let (num, remaining) = parse_number(self.input, self.int)
                        .map_err(|e| e.map(Error::NumberParse))?;
                    self.input = remaining;
                    Token::Num(num)
                } else if self.after_number && ['\'', '"'].contains(&ch) {
                    let (_, remaining) = self.input.split_at(1);
                    self.input = remaining;
                    if ch == '\'' {
                        Token::Ident("'")
                    } else {
                        Token::Ident("\"")
                    }
                } else if ch == '"' {
                    // normal string literal, with possible escape sequences
                    let (token, remaining) = parse_string_literal(self.input)?;
                    self.input = remaining;
                    token
                } else if self.input.starts_with("#\"") {
                    // raw string literal
                    let (_, remaining) = self.input.split_at(2);
                    let literal_length = remaining
                        .match_indices("\"#")
                        .next()
                        .ok_or(Error::UnterminatedStringLiteral)?
                        .0;
                    let (literal, remaining) = remaining.split_at(literal_length);
                    let (_terminator, remaining) = remaining.split_at(2);
                    self.input = remaining;
                    Token::StringLiteral(literal.into())
                } else if is_valid_in_ident(ch, None) {
                    // dots aren't allowed in idents after a backslash
                    let (ident, remaining) =
                        parse_ident(self.input, self.after_backslash_state != 1)?;
                    self.input = remaining;
                    ident
                } else {
                    let (_, remaining) = self.input.split_at(ch.len_utf8());
                    self.input = remaining;
                    parse_symbol(ch, &mut self.input)?
                }
            }
            None => return Ok(None),
        }))
    }
}

impl<'a, I: Interrupt> Iterator for Lexer<'a, '_, I> {
    type Item = Result<Token<'a>, IntErr<Error, I>>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.next_token() {
            Err(e) => Some(Err(e)),
            Ok(None) => None,
            Ok(Some(t)) => Some(Ok(t)),
        };
        if let Some(Ok(Token::Num(_))) = res {
            self.after_number = true;
        } else {
            self.after_number = false;
        }
        if let Some(Ok(Token::Symbol(Symbol::Backslash))) = res {
            self.after_backslash_state = 1;
        } else if self.after_backslash_state == 1 {
            if let Some(Ok(Token::Ident(_))) = res {
                self.after_backslash_state = 2;
            } else {
                self.after_backslash_state = 0;
            }
        } else {
            self.after_backslash_state = 0;
        }
        res
    }
}

pub(crate) fn lex<'a, 'b, I: Interrupt>(input: &'a str, int: &'b I) -> Lexer<'a, 'b, I> {
    Lexer {
        input,
        after_backslash_state: 0,
        after_number: false,
        int,
    }
}
