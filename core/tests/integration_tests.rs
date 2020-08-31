use fend_core::{evaluate, Context};

#[track_caller]
pub fn test_evaluation(input: &str, expected: &str) {
    let mut context = Context::new();
    assert_eq!(
        evaluate(input, &mut context).unwrap().get_main_result(),
        expected.to_string()
    );
    // try parsing the output again, and make sure it matches
    assert_eq!(
        evaluate(expected, &mut context).unwrap().get_main_result(),
        expected.to_string()
    );
}

#[track_caller]
fn test_eval_simple(input: &str, expected: &str) {
    let mut context = Context::new();
    assert_eq!(
        evaluate(input, &mut context).unwrap().get_main_result(),
        expected.to_string()
    );
}

#[track_caller]
fn expect_parse_error(input: &str) {
    let mut context = Context::new();
    assert!(evaluate(input, &mut context).is_err());
}

#[track_caller]
fn assert_err_msg(input: &str, error: &str) {
    let mut context = Context::new();
    assert_eq!(evaluate(input, &mut context), Err(error.to_string()));
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
}

#[test]
fn test_blank_input() {
    test_evaluation("", "");
}

#[test]
fn test_pi() {
    test_evaluation("pi", "approx. 3.1415926535");
    test_evaluation("pi * 2", "approx. 6.2831853071");
    test_evaluation("2 pi", "approx. 6.2831853071");
}

#[test]
fn test_div_by_zero() {
    let msg = "Attempt to divide by zero";
    assert_err_msg("1/0", msg);
    assert_err_msg("0/0", msg);
    assert_err_msg("-1/0", msg);
    assert_err_msg("-1/(2-2)", msg);
}

#[test]
fn test_leading_zeroes() {
    let msg = "Integer literals cannot have leading zeroes";
    assert_err_msg("00", msg);
    assert_err_msg("000000", msg);
    assert_err_msg("000000.01", msg);
    assert_err_msg("0000001.01", msg);
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
    test_evaluation("+2", "2");
    test_evaluation("++++2", "2");
    test_evaluation(
        "315427679023453451289740 + 927346502937456234523452",
        "1242774181960909685813192",
    );
}

#[test]
fn test_subtraction() {
    test_evaluation("-0", "0");
    test_evaluation("2-2", "0");
    test_evaluation("3-2", "1");
    test_evaluation("2-3", "-1");
    test_evaluation("-2", "-2");
    test_evaluation("--2", "2");
    test_evaluation("---2", "-2");
    test_evaluation("-(--2)", "-2");
    test_evaluation("\n2\n-\n64\n", "-62");
    test_evaluation(
        "315427679023453451289740 - 927346502937456234523452",
        "-611918823914002783233712",
    );
}

#[test]
fn test_subtraction_2() {
    test_evaluation(
        "36893488123704996004 - 18446744065119617025",
        "18446744058585378979",
    );
}

#[test]
fn test_sqrt_half() {
    test_evaluation("sqrt (1/2)", "approx. 0.7071067809");
}

#[test]
fn test_exact_roots() {
    test_evaluation("sqrt 0", "0");
    test_evaluation("sqrt 1", "1");
    test_evaluation("sqrt 4", "2");
    test_evaluation("sqrt 9", "3");
    test_evaluation("sqrt 16", "4");
    test_evaluation("sqrt 25", "5");
    test_evaluation("sqrt 36", "6");
    test_evaluation("sqrt 49", "7");
    test_evaluation("sqrt 64", "8");
    test_evaluation("sqrt 81", "9");
    test_evaluation("sqrt 100", "10");
    test_evaluation("sqrt 10000", "100");
    test_evaluation("sqrt 1000000", "1000");
    test_evaluation("sqrt 0.25", "0.5");
    test_evaluation("sqrt 0.0625", "0.25");

    test_evaluation("cbrt 0", "0");
    test_evaluation("cbrt 1", "1");
    test_evaluation("cbrt 8", "2");
    test_evaluation("cbrt 27", "3");
    test_evaluation("cbrt 64", "4");
    test_evaluation("cbrt (1/8)", "0.5");
    test_evaluation("cbrt (125/8)", "2.5");
}

#[test]
fn test_approx_roots() {
    test_evaluation("sqrt 2", "approx. 1.4142135619");
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
    test_evaluation("-3 -1/2", "-3.5");
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
    test_evaluation("0.25", "0.25");
    expect_parse_error("1.");
    expect_parse_error(".1");
    expect_parse_error("001.01000");
    test_evaluation(
        "0.251974862348971623412341534273261435",
        "0.251974862348971623412341534273261435",
    );
}

#[test]
fn test_slow_division() {
    test_evaluation(
        "60153992292001127921539815855494266880 / 9223372036854775808",
        "6521908912666391110",
    )
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

#[test]
fn test_powers() {
    test_evaluation("1^1", "1");
    test_evaluation("1**1", "1");
    test_evaluation("1**1.0", "1");
    test_evaluation("1.0**1", "1");
    test_evaluation("2^4", "16");
    test_evaluation("4^2", "16");
    test_evaluation("4^3", "64");
    test_evaluation("4^(3^1)", "64");
    test_evaluation("4^3^1", "64");
    test_evaluation("(4^3)^1", "64");
    test_evaluation("(2^3)^4", "4096");
    test_evaluation("2^3^2", "512");
    test_evaluation("(2^3)^2", "64");
    // non-integer powers
    test_evaluation("4^0.5", "2");
    test_evaluation("4^(1/2)", "2");
    test_evaluation("4^(1/4)", "approx. 1.4142135619");
    test_evaluation("(2/3)^(4/5)", "approx. 0.7229811807");
    test_evaluation(
        "5.2*10^15*300^(3/2)",
        "approx. 27019992598076723515.9873962402",
    )
}

#[test]
fn test_tricky_power_negation() {
    test_evaluation("-0.125", "-0.125");
    test_evaluation("2^1^2", "2");
    test_evaluation("2^(1^2)", "2");
    test_evaluation("2^(1)", "2");
    test_evaluation("2 * (-2^3)", "-16");
    test_evaluation("2 * -2^3", "-16");
    test_evaluation("2^-3 * 4", "0.5");
    test_evaluation("2^3 * 4", "32");
    test_evaluation("2 * -3 * 4", "-24");
    test_evaluation("-2^-3", "-0.125");
    let mut context = Context::new();
    assert_eq!(
        evaluate("2^-3^4", &mut context),
        evaluate("1 / 2^81", &mut context)
    );
    test_evaluation("4^-1^2", "0.25");
}

#[test]
fn test_basic_complex_numbers() {
    test_evaluation("i", "i");
    test_evaluation("3i", "3i");
    test_evaluation("3i+4", "4 + 3i");
    test_evaluation("(3i+4) + i", "4 + 4i");
    test_evaluation("3i+(4 + i)", "4 + 4i");
    test_evaluation("-3i", "-3i");
    test_evaluation("i/i", "1");
    test_evaluation("i*i", "-1");
    test_evaluation("i*i*i", "-i");
    test_evaluation("i*i*i*i", "1");
    test_evaluation("-3+i", "-3 + i");
    test_evaluation("1+i", "1 + i");
    test_evaluation("1-i", "1 - i");
    test_evaluation("-1 + i", "-1 + i");
    test_evaluation("-1 - i", "-1 - i");
    test_evaluation("-1 - 2i", "-1 - 2i");
    test_evaluation("-1 - 0.5i", "-1 - 0.5i");
    test_evaluation("-1 - 0.5i + 1.5i", "-1 + i");
    test_evaluation("-3i", "-3i");
    test_evaluation("-i", "-i");
    test_evaluation("+i", "i");
    test_evaluation("2i", "2i");
    test_evaluation("i/3", "i/3");
    test_evaluation("2i/3", "2i/3");
    test_evaluation("2i/-3-1", "-1 - 2i/3");
    // i is an identifier, not a number; cf. 0bi
    expect_parse_error("2#i");
}

#[test]
fn test_digit_separators() {
    test_evaluation("1_1", "11");
    test_evaluation("11_1", "111");
    test_evaluation("1_1_1", "111");
    test_evaluation("123_456_789_123", "123456789123");
    test_evaluation("1_2_3_4_5_6", "123456");
    test_evaluation("1.1_1", "1.11");
    test_evaluation("1_1.1_1", "11.11");
    expect_parse_error("_1");
    expect_parse_error("1_");
    expect_parse_error("1__1");
    expect_parse_error("_");
    expect_parse_error("1_.1");
    expect_parse_error("1._1");
    expect_parse_error("1.1_");
}

#[test]
fn test_different_bases() {
    test_evaluation("0x10", "0x10");
    test_evaluation("0o10", "0o10");
    test_evaluation("0b10", "0b10");
    test_evaluation("0x10 - 1", "0xf");
    test_evaluation("0x0 + sqrt 16", "0x4");
    test_evaluation("16#0 + sqrt 16", "16#4");
    test_evaluation("0 + 6#100", "36");
    test_evaluation("0 + 36#z", "35");
    test_evaluation("16#dead_beef", "16#deadbeef");
    test_evaluation("16#DEAD_BEEF", "16#deadbeef");
    expect_parse_error("#");
    expect_parse_error("0#0");
    expect_parse_error("1#0");
    expect_parse_error("2_2#0");
    expect_parse_error("22 #0");
    expect_parse_error("22# 0");
    test_evaluation("36#i i", "36#i i");
    test_evaluation("16#1i", "16#1i");
    test_evaluation("16#fi", "16#fi");
    test_evaluation("0 + 36#ii", "666");
    expect_parse_error("18#i/i");
    test_evaluation("19#i/i", "-19#i i");
}

#[test]
fn test_exponents() {
    test_evaluation("1e10", "10000000000");
    test_evaluation("1.5e10", "15000000000");
    test_evaluation("0b1e10", "0b10000000000");
    test_evaluation("0 + 0b1e100", "1267650600228229401496703205376");
    test_evaluation("0 + 0b1e32", "4294967296");
    test_evaluation("0 + 0b1e16", "65536");
    test_evaluation("16#1e10", "16#1e10");
    expect_parse_error("11#1e10");
    test_evaluation("0 + 0b1e1_00", "1267650600228229401496703205376");
    test_evaluation("1.5e-1", "0.15");
    expect_parse_error("1e -1");
    expect_parse_error("1e- 1");
    test_evaluation("0 + 0b1e-6", "0.015625");
}

#[test]
fn test_basic_units() {
    test_evaluation("1kg", "1 kg");
    test_evaluation("1g", "1 g");
    test_evaluation("1kg + 1g", "1.001 kg");
    test_evaluation("1kg + 100g", "1.1 kg");
    test_evaluation("0g + 1kg + 100g", "1100 g");
    test_evaluation("0g + 1kg", "1000 g");
    test_evaluation("1/0.5 kg", "2 kg");
    test_evaluation("1/(1/0.5 kg)", "0.5 kg^-1");
}

#[test]
fn test_complex_number_with_unit() {
    test_evaluation("1 kg + i g", "(1 + 0.001i) kg");
}

#[test]
fn test_more_units() {
    test_evaluation("0m + 1kph * 1 hr", "1000 m");
    test_evaluation("0GiB + 1GB", "0.931322574615478515625 GiB");
    test_evaluation("0m/s + 1 km/hr", "5/18 m / s");
    test_evaluation("0m/s + i km/hr", "5i/18 m / s");
    test_evaluation("0m/s + (1 + i) km/hr", "(5/18 + 5i/18) m / s");
    expect_parse_error("7165928\t761528765");
    test_evaluation("1 2/3", "5/3");
    test_evaluation("abs 2", "2");
    test_evaluation("5 m", "5 m");
    test_evaluation("(4)(6)", "24");
    test_evaluation("5(6)", "30");
    expect_parse_error("(5)6");
    test_evaluation("3’6”", "3.5’");
    test_evaluation("365.25 light days -> ly", "1 ly");
    test_evaluation("1 light year", "1 light year");
    expect_parse_error("1 2 m");
    test_evaluation("5pi", "approx. 15.7079632679");
    test_evaluation("5 pi/2", "approx. 7.8539816339");
    test_evaluation("5 i/2", "2.5i");
    test_evaluation("3 m 15 cm", "3.15 m");
    test_evaluation("5%", "5%");
    test_evaluation("5% + 0.1", "15%");
    test_evaluation("5% + 1", "105%");
    test_evaluation("0.1 + 5%", "0.15");
    test_evaluation("1 + 5%", "1.05");
    test_evaluation("1psi -> kPa -> 5dp", "6.89475 kPa"); // should be approx.
                                                          //test_evaluation("5% * 5%", "0.25%");
}

#[test]
fn test_no_adjacent_numbers() {
    expect_parse_error("1 2");
    expect_parse_error("1 2 3 4 5");
    expect_parse_error("1 inch 5");
    expect_parse_error("abs 1 2");
    expect_parse_error("1 inch 5 kg");
    test_evaluation("5 (abs 4)", "20");
}

#[test]
fn test_compound_fraction() {
    test_evaluation("1 2/3", "5/3");
    test_evaluation("4 + 1 2/3", "17/3");
    test_evaluation("-8 1/2", "-8.5");
}

#[test]
fn test_unit_sums() {
    test_evaluation("5 feet 12 inch", "6 foot");
    test_evaluation("3'6\"", "3.5'");
    test_evaluation("3’6”", "3.5’");
}

#[test]
fn test_unit_conversions() {
    expect_parse_error("->");
    expect_parse_error("1m->");
    expect_parse_error("1m - >");
    expect_parse_error("->1ft");
    expect_parse_error("1m -> 45ft");
    expect_parse_error("1m -> 45 kg ft");
    test_evaluation("1' -> inches", "12 inch");
}

#[test]
fn test_abs() {
    test_evaluation("abs 1", "1");
    test_evaluation("abs i", "1");
    test_evaluation("abs (-1)", "1");
    test_evaluation("abs (-i)", "1");
    test_evaluation("abs (2i)", "2");
    test_evaluation("abs (1 + i)", "approx. 1.4142135619");
}

#[test]
fn test_advanced_op_precedence() {
    test_evaluation("2 kg^2", "2 kg^2");
    test_evaluation("((1/4) kg)^-2", "16 kg^-2");
    test_evaluation("1 N - 1 kg m s^-2", "0 N");
    test_evaluation("1 J - 1 kg m^2 s^-2 + 1 kg / (m^-2 s^2)", "1 J");
    expect_parse_error("2^abs 1");
    expect_parse_error("2 4^3");
    expect_parse_error("-2 4^3");
    test_evaluation("3*-2", "-6");
    test_evaluation("-3*-2", "6");
    test_evaluation("-3*2", "-6");
    expect_parse_error("1 2/3^2");
    expect_parse_error("1 2^2/3");
    expect_parse_error("1^2 2/3");
    expect_parse_error("1 2/-3");
    test_evaluation("1 2/3 + 4 5/6", "6.5");
    test_evaluation("1 2/3 + -4 5/6", "-19/6");
    test_evaluation("1 2/3 - 4 5/6", "-19/6");
    test_evaluation("1 2/3 - 4 + 5/6", "-1.5");
    test_evaluation("1 barn -> m^2", "0.0000000000000000000000000001 m^2");
    test_evaluation("1L -> m^3", "0.001 m^3");
}

#[test]
fn test_recurring_digits() {
    test_eval_simple("9/11 -> float", "0.(81)");
    test_eval_simple("6#1 / 11 -> float", "6#0.(0313452421)");
    test_eval_simple("6#0 + 6#1 / 7 -> float", "6#0.(05)")
}
