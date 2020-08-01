#![forbid(unsafe_code)]

mod ast;
mod num;
mod parse;

pub fn evaluate(input: &str) -> Result<String, String> {
    let (_, input) = parse::skip_whitespace(input)?;
    if input.is_empty() {
        // no or blank input: return no output
        return Ok("".to_string());
    }
    let (parsed, input) = parse::parse_expression(input)?;
    if !input.is_empty() {
        return Err(format!("Unexpected input found: '{}'", input));
    }
    let result = ast::evaluate(parsed)?;
    Ok(format!("{}", result))
}

#[cfg(test)]
mod tests {
    use crate::evaluate;

    fn test_evaluation(input: &str, expected: &str) {
        assert_eq!(evaluate(input), Ok(expected.to_string()));
    }

    fn expect_parse_error(input: &str) {
        assert!(evaluate(input).is_err());
    }

    #[test]
    fn test_blank_input() {
        test_evaluation("", "");
    }

    #[test]
    fn test_div_by_zero() {
        assert_eq!(
            evaluate("1/0"),
            Err("Attempt to divide by zero".to_string())
        );
        assert_eq!(
            evaluate("0/0"),
            Err("Attempt to divide by zero".to_string())
        );
        assert_eq!(
            evaluate("-1/0"),
            Err("Attempt to divide by zero".to_string())
        );
    }

    #[test]
    fn test_basic_integers() {
        test_evaluation("2", "2");
        test_evaluation("9", "9");
        test_evaluation("10", "10");
        test_evaluation("39456720983475234523452345", "39456720983475234523452345");
        test_evaluation("10 ", "10");
        test_evaluation(" 10", "10");
        test_evaluation(" 10\n\r\n", "10");
        expect_parse_error("10a");
    }

    #[test]
    fn test_multiplication() {
        test_evaluation("2*2", "4");
        test_evaluation("\n2\n*\n2\n", "4");
        test_evaluation(
            "315427679023453451289740 * 927346502937456234523452",
            "292510755072077978255166497050046859223676982480",
        );
    }

    #[test]
    fn test_addition() {
        test_evaluation("2+2", "4");
        test_evaluation("\n2\n+\n2\n", "4");
        test_evaluation(
            "315427679023453451289740 + 927346502937456234523452",
            "1242774181960909685813192",
        );
    }

    #[test]
    fn test_subtraction() {
        test_evaluation("2-2", "0");
        test_evaluation("3-2", "1");
        test_evaluation("2-3", "-1");
        test_evaluation("\n2\n-\n64\n", "-62");
        test_evaluation(
            "315427679023453451289740 - 927346502937456234523452",
            "-611918823914002783233712",
        );
    }

    #[test]
    fn test_basic_order_of_operations() {
        test_evaluation("2+2*3", "8");
        test_evaluation("2*2+3", "7");
        test_evaluation("2+2+3", "7");
        test_evaluation("2+2-3", "1");
        test_evaluation("2-2+3", "3");
        test_evaluation("2-2-3", "-3");
        test_evaluation("2*2*3", "12");
        test_evaluation("2*2*-3", "-12");
        test_evaluation("2*-2*3", "-12");
        test_evaluation("-2*2*3", "-12");
        test_evaluation("-2*-2*3", "12");
        test_evaluation("-2*2*-3", "12");
        test_evaluation("2*-2*-3", "12");
        test_evaluation("-2*-2*-3", "-12");
        test_evaluation("-2*-2*-3/2", "-6");
        test_evaluation("-2*-2*-3/-2", "6");
    }

    #[test]
    fn test_exact_division() {
        test_evaluation("1/1", "1");
        test_evaluation("1/2", "0.5");
        test_evaluation("1/4", "0.25");
        test_evaluation("1/8", "0.125");
        test_evaluation("1/16", "0.0625");
        test_evaluation("1/32", "0.03125");
        test_evaluation("1/64", "0.015625");
        test_evaluation("2/64", "0.03125");
        test_evaluation("4/64", "0.0625");
        test_evaluation("8/64", "0.125");
        test_evaluation("16/64", "0.25");
        test_evaluation("32/64", "0.5");
        test_evaluation("64/64", "1");
        test_evaluation("2/1", "2");
        test_evaluation("27/3", "9");
        test_evaluation("100/4", "25");
        test_evaluation("100/5", "20");
        test_evaluation("18446744073709551616/2", "9223372036854775808");
        test_evaluation(
            "184467440737095516160000000000000/2",
            "92233720368547758080000000000000",
        );
    }

    #[test]
    fn test_decimal_point() {
        test_evaluation("0.0", "0");
        test_evaluation("0.000000", "0");
        test_evaluation("0.01", "0.01");
        test_evaluation("0.01000", "0.01");
        test_evaluation("001.01000", "1.01");
        test_evaluation("0.25", "0.25");
        expect_parse_error("1.");
        expect_parse_error(".1");
        test_evaluation(
            "0.251974862348971623412341534273261435",
            "0.251974862348971623412341534273261435",
        );
    }

    #[test]
    fn test_parens() {
        test_evaluation("(1)", "1");
        test_evaluation("(0.0)", "0");
        test_evaluation("(1+-2)", "-1");
        test_evaluation("1+2*3", "7");
        test_evaluation("(1+2)*3", "9");
        test_evaluation("((1+2))*3", "9");
        test_evaluation("((1)+2)*3", "9");
        test_evaluation("(1+(2))*3", "9");
        test_evaluation("(1+(2)*3)", "7");
        test_evaluation("1+(2*3)", "7");
        test_evaluation("1+((2 )*3)", "7");
        test_evaluation(" 1 + ( (\r\n2 ) * 3 ) ", "7");
    }
}
