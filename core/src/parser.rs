use crate::ast::Expr;
use crate::num::{Base, Number};

/*

Grammar:

expression = additive
additive = compound_fraction [addition_cont subtraction_cont]*
addition_cont = '+' compound_fraction
subtraction_cont = '-' compound_fraction
compound_fraction = A:multiplicative B:multiplicative?
  // if there are two terms, they are matched as follows:
  // <Num>, <Num> / <Num> => A + B # e.g. 8 1/2
  // -<Num>, <Num / <Num> => A - B # e.g. -8 1/2
  // <ApplyMul>, <ApplyMul> => A + B # e.g. 1 foot 3 inches
multiplicative = power [multiplication_cont division_cont]*
multiplication_cont = '*' power
division_cont = '/' power
power = whitespace? [('-' power) ('+' power) (apply power_cont)]
power_cont = ['^' '**'] power
apply = parens_or_literal [parens_or_literal]*
  // every pair of terms is matched as follows
  // <Num>, <Num> => error
  // <ApplyMul>, <Num> => error
  // _, <Num> => ApplyFunctionCall
  // <Num>, _ => ApplyMul
  // <ApplyMul>, _ => ApplyMul
  // _ => Apply
parens_or_literal = [number parens ident]
parens = whitespace? '(' expression ')' whitespace?
ident = whitespace? alphabetic [alphabetic '.']*
number =
    whitespace?
    base_prefix?
    basic_number
basic_number(base) = A:integer
    ('.' B:integer)?
    ('e' '-'? C:integer)?

  // A can have digit separators but no leading zero
  // B can have digit separators and leading zeroes
  // C can have digit separators but no leading zero,
  //   and is only considered if the base <= 10
  // A and B are in the parsed base, or 10 if unspecified.
  // C is always in base 10
  // If C is present, the number is multiplied by base^C

base_prefix = ['0x' '0o' '0b' (A:integer '#')]
  // A is decimal, and may not have leading zeroes or digit separators,
  // and must be between 2 and 36 inclusive.

*/

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

fn parse_ascii_digit(input: &str, base: Base) -> ParseResult<u64> {
    let (ch, input) = parse_char(input)?;
    if let Some(digit) = ch.to_digit(base.base_as_u8().into()) {
        Ok((digit.into(), input))
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

// Parses a plain integer with no whitespace and no base prefix.
// Leading minus sign is not allowed.
fn parse_integer<'a>(
    input: &'a str,
    allow_digit_separator: bool,
    allow_leading_zeroes: bool,
    base: Base,
    process_digit: &mut impl FnMut(u64) -> Result<(), String>,
) -> ParseResult<'a, ()> {
    let (digit, mut input) = parse_ascii_digit(input, base)?;
    process_digit(digit)?;
    let leading_zero = digit == 0;
    let mut parsed_digit_separator;
    loop {
        if let Ok((_, remaining)) = parse_fixed_char(input, '_') {
            input = remaining;
            parsed_digit_separator = true;
            if !allow_digit_separator {
                return Err("Digit separators are not allowed".to_string());
            }
        } else {
            parsed_digit_separator = false;
        }
        match parse_ascii_digit(input, base) {
            Err(_) => {
                if parsed_digit_separator {
                    return Err("Digit separators can only occur between digits".to_string());
                }
                break;
            }
            Ok((digit, next_input)) => {
                if leading_zero && !allow_leading_zeroes {
                    return Err("Integer literals cannot have leading zeroes".to_string());
                }
                process_digit(digit)?;
                input = next_input;
            }
        }
    }
    Ok(((), input))
}

fn parse_base_prefix(input: &str) -> ParseResult<Base> {
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
                    )
                }
            },
            input,
        ))
    } else {
        let mut custom_base: u8 = 0;
        let (_, input) = parse_integer(input, false, false, Base::Decimal, &mut |digit| {
            if custom_base > 3 {
                return Err("Base cannot be larger than 36".to_string());
            }
            custom_base = 10 * custom_base + digit as u8;
            if custom_base > 36 {
                return Err("Base cannot be larger than 36".to_string());
            }
            Ok(())
        })?;
        if custom_base < 2 {
            return Err("Base must be at least 2".to_string());
        }
        let (_, input) = parse_fixed_char(input, '#')?;
        Ok((Base::Custom(custom_base), input))
    }
}

fn parse_basic_number(input: &str, base: Base, allow_zero: bool) -> ParseResult<Number> {
    // parse integer component
    let mut res = Number::zero_with_base(base);
    let (_, mut input) = parse_integer(input, true, false, base, &mut |digit| {
        let base_as_u64: u64 = base.base_as_u8().into();
        res = (res.clone() * base_as_u64.into()).add(digit.into())?;
        Ok(())
    })?;

    // parse decimal point and at least one digit
    if let Ok((_, remaining)) = parse_fixed_char(input, '.') {
        let (_, remaining) = parse_integer(remaining, true, true, base, &mut |digit| {
            res.add_digit_in_base(digit, base)?;
            Ok(())
        })?;
        input = remaining;
    }

    if !allow_zero && res.is_zero() {
        return Err("Invalid number: 0".to_string());
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
            let (_, remaining) = parse_integer(input, true, false, Base::Decimal, &mut |digit| {
                exp = (exp.clone() * 10.into()).add(digit.into())?;
                Ok(())
            })?;
            if negative_exponent {
                exp = -exp;
            }
            let base_as_u64: u64 = base.base_as_u8().into();
            let base_as_number: Number = base_as_u64.into();
            res = res * base_as_number.pow(exp)?;
            input = remaining;
        }
    }

    Ok((res, input))
}

fn parse_number(input: &str) -> ParseResult<Expr> {
    let (_, mut input) = skip_whitespace(input)?;

    let base = if let Ok((base, remaining)) = parse_base_prefix(input) {
        input = remaining;
        base
    } else {
        Base::Decimal
    };

    let (res, input) = parse_basic_number(input, base, true)?;

    let (_, input) = skip_whitespace(input)?;
    return Ok((Expr::Num(res), input));
}

fn parse_ident(input: &str) -> ParseResult<Expr> {
    let (_, input) = skip_whitespace(input)?;
    let (ch, mut input) = parse_char(input)?;
    if !ch.is_alphabetic() {
        return Err(format!("Found invalid character in identifier: '{}'", ch));
    }
    let mut ident = ch.to_string();
    loop {
        if let Ok((ch, remaining)) = parse_char(input) {
            if ch.is_alphanumeric() || ch == '.' {
                ident.push(ch);
                input = remaining;
                continue;
            }
        }
        break;
    }
    let (_, input) = skip_whitespace(input)?;
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
    let (ch, _) = parse_char(input)?;

    if ch.is_ascii_digit() {
        return parse_number(input);
    } else if ch == '(' {
        return parse_parens(input);
    } else {
        return parse_ident(input);
    }
}

fn parse_apply(input: &str) -> ParseResult<Expr> {
    let (mut res, mut input) = parse_parens_or_literal(input)?;
    loop {
        if let Ok((term, remaining)) = parse_parens_or_literal(input) {
            res = match (&res, &term) {
                (Expr::Num(_), Expr::Num(_)) => {
                    // this may later be parsed as a compound fraction
                    return Ok((res, input));
                }
                (Expr::ApplyMul(_, _), Expr::Num(_)) => {
                    // this may later become an addition, e.g. 6 feet 1 inch
                    return Ok((res, input));
                }
                (_, Expr::Num(_)) => Expr::ApplyFunctionCall(Box::new(res), Box::new(term)),
                (Expr::Num(_), _) | (Expr::ApplyMul(_, _), _) => {
                    Expr::ApplyMul(Box::new(res), Box::new(term))
                }
                _ => Expr::Apply(Box::new(res), Box::new(term)),
            };
            input = remaining;
        } else {
            break;
        }
    }
    Ok((res, input))
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
        return Ok((Expr::UnaryMinus(Box::new(res)), remaining));
    }
    if let Ok((_, remaining)) = parse_fixed_char(input, '+') {
        let (res, remaining) = parse_power(remaining)?;
        return Ok((Expr::UnaryPlus(Box::new(res)), remaining));
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

fn parse_compound_fraction(input: &str) -> ParseResult<Expr> {
    let (res, input) = parse_multiplicative(input)?;
    if let Ok((rhs, remaining)) = parse_multiplicative(input) {
        match (&res, &rhs) {
            (Expr::Num(_), Expr::Div(b, c)) => match (&**b, &**c) {
                (Expr::Num(_), Expr::Num(_)) => {
                    return Ok((Expr::Add(Box::new(res), Box::new(rhs)), remaining))
                }
                _ => (),
            },
            (Expr::UnaryMinus(a), Expr::Div(b, c)) => match (&**a, &**b, &**c) {
                (Expr::Num(_), Expr::Num(_), Expr::Num(_)) => {
                    return Ok((
                        // note that res = '-<num>' here because unary minus has a higher precedence
                        Expr::Sub(Box::new(res), Box::new(rhs)),
                        remaining,
                    ));
                }
                _ => (),
            },
            (Expr::ApplyMul(_, _), Expr::ApplyMul(_, _)) => {
                return Ok((Expr::Add(Box::new(res), Box::new(rhs)), remaining))
            }
            _ => (),
        };
    }
    Ok((res, input))
}

fn parse_addition_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '+')?;
    let (b, input) = parse_compound_fraction(input)?;
    Ok((b, input))
}

fn parse_subtraction_cont(input: &str) -> ParseResult<Expr> {
    let (_, input) = parse_fixed_char(input, '-')?;
    let (b, input) = parse_compound_fraction(input)?;
    Ok((b, input))
}

fn parse_additive(input: &str) -> ParseResult<Expr> {
    let (mut res, mut input) = parse_compound_fraction(input)?;
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
