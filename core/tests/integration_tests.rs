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
fn test_same(a: &str, b: &str) {
    let mut context = Context::new();
    assert_eq!(
        evaluate(a, &mut context).unwrap().get_main_result(),
        evaluate(b, &mut context).unwrap().get_main_result()
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
fn expect_error(input: &str) {
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
    let msg = "Division by zero";
    assert_err_msg("1/0", msg);
    assert_err_msg("0/0", msg);
    assert_err_msg("-1/0", msg);
    assert_err_msg("(3pi) / (0pi)", msg);
    assert_err_msg("-1/(2-2)", msg);
}

#[test]
fn test_leading_zeroes() {
    let msg = "Integer literals cannot have leading zeroes";
    assert_err_msg("00", msg);
    assert_err_msg("000000", msg);
    assert_err_msg("000000.01", msg);
    assert_err_msg("0000001.01", msg);
    test_evaluation("0b01", "0b1");
    test_evaluation("0x0000_00ff", "0xff");
    test_evaluation("10#04", "10#4");
    test_evaluation("1.001", "1.001");
    test_evaluation("1e01", "10");
    test_evaluation("1e-01", "0.1");
}

#[test]
fn test_parsing_recurring_digits() {
    expect_error("0.()");
    test_eval_simple("0.(3) to float", "0.(3)");
    test_eval_simple("0.(33) to float", "0.(3)");
    test_eval_simple("0.(34) to float", "0.(34)");
    test_eval_simple("0.(12345) to float", "0.(12345)");
    test_eval_simple("0.(0) to float", "0");
    test_eval_simple("0.123(00) to float", "0.123");
    test_eval_simple("0.0(34) to float", "0.0(34)");
    test_eval_simple("0.00(34) to float", "0.00(34)");
    test_eval_simple("0.0000(34) to float", "0.0000(34)");
    test_eval_simple("0.123434(34) to float", "0.12(34)");
    test_eval_simple("0.123434(34)i to float", "0.12(34)i");
    test_eval_simple("0.(3) + 0.123434(34)i to float", "0.(3) + 0.12(34)i");
    test_eval_simple("6#0.(1) to float", "6#0.(1)");
    test_eval_simple("6#0.(1) to float to base 10", "0.2");
}

#[test]
fn test_multiplication() {
    test_evaluation("2*2", "4");
    test_evaluation("\n2\n*\n2\n", "4");
    test_evaluation(
        "315427679023453451289740 * 927346502937456234523452",
        "292510755072077978255166497050046859223676982480",
    );
    test_evaluation("pi * pi", "approx. 9.869604401");
    test_evaluation("4pi + 1", "approx. 13.5663706143");
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
    test_evaluation("3pi - 2pi", "approx. 3.1415926535");
    test_evaluation("4pi-1)/pi", "approx. 3.6816901138");
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
    test_evaluation("sqrt (1/2)", "approx. 0.7071067814");
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

    test_evaluation("sqrt(kg^2)", "1 kg");
    test_evaluation("(sqrt kg)^2", "1 kg");
}

#[test]
fn test_approx_roots() {
    test_evaluation("sqrt 2", "approx. 1.4142135619");
    test_evaluation("sqrt pi", "approx. 1.7724538509");
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
    test_evaluation("(3pi) / (2pi)", "approx. 1.5");
}

#[test]
fn test_decimal_point() {
    test_evaluation("0.0", "0");
    test_evaluation("0.000000", "0");
    test_evaluation("0.01", "0.01");
    test_evaluation("0.01000", "0.01");
    test_evaluation("0.25", "0.25");
    expect_error("1.");
    test_evaluation(".1", "0.1");
    test_evaluation(".1e-1", "0.01");
    expect_error("001.01000");
    test_evaluation(
        "0.251974862348971623412341534273261435",
        "0.251974862348971623412341534273261435",
    );
    test_eval_simple("1.00000001 as 1 dp", "approx. 1");
    test_eval_simple("1.00000001 as 2 dp", "approx. 1");
    test_eval_simple("1.00000001 as 3 dp", "approx. 1");
    test_eval_simple("1.00000001 as 4 dp", "approx. 1");
    test_evaluation("1.00000001 as 10 dp", "1.00000001");
    test_evaluation("1.00000001 as 30 dp", "1.00000001");
    test_evaluation("1.00000001 as 1000 dp", "1.00000001");
    test_evaluation("1.00000001 as 0 dp", "approx. 1");
    test_evaluation(".1(0)", "0.1");
    test_evaluation(".1( 0)", "0");
    test_evaluation(".1 ( 0)", "0");
    expect_error(".1(0 )");
    expect_error(".1(0a)");
    test_evaluation("2.0(e)", "approx. 5.4365636569");
    test_evaluation("2.0(ln 5)", "approx. 3.2188758248");
    test_evaluation("2 (5)", "10");
    test_evaluation("2( 5)", "10");
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
    test_evaluation("2*(1+3", "8");
    test_evaluation("4+5+6)*(1+2", "45");
    test_evaluation("4+5+6))*(1+2", "45");
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
    );
    test_evaluation("pi^10", "approx. 93648.047476083");
    expect_error("0^0");
    test_evaluation("0^1", "0");
    test_evaluation("1^0", "1");
    // this exponent is currently too large
    expect_error("1^1e1000");
    expect_error("i^3");
    expect_error("4^i");
    expect_error("i^i");
    test_evaluation("kg^(approx. 1)", "approx. 1 kg")
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
    test_same("2^-3^4", "1 / 2^81");
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
    expect_error("2#i");
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
    expect_error("_1");
    expect_error("1_");
    expect_error("1__1");
    expect_error("_");
    expect_error("1_.1");
    expect_error("1._1");
    expect_error("1.1_");
    test_evaluation("1,1", "11");
    test_evaluation("11,1", "111");
    test_evaluation("1,1,1", "111");
    test_evaluation("123,456,789,123", "123456789123");
    test_evaluation("1,2,3,4,5,6", "123456");
    test_evaluation("1.1,1", "1.11");
    test_evaluation("1,1.1,1", "11.11");
    expect_error(",1");
    expect_error("1,");
    expect_error("1,,1");
    expect_error(",");
    expect_error("1,.1");
    expect_error("1.,1");
    expect_error("1.1,");
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
    expect_error("#");
    expect_error("0#0");
    expect_error("1#0");
    expect_error("2_2#0");
    expect_error("22 #0");
    expect_error("22# 0");
    test_evaluation("36#i i", "36#i i");
    test_evaluation("16#1i", "16#1i");
    test_evaluation("16#fi", "16#fi");
    test_evaluation("0 + 36#ii", "666");
    expect_error("18#i/i");
    test_evaluation("19#i/i", "-19#i i");
    // verified using a ruby program
    test_evaluation(
        "0+36#0123456789abcdefghijklmnopqrstuvwxyz",
        "86846823611197163108337531226495015298096208677436155",
    );
    test_evaluation(
        "36#0 + 86846823611197163108337531226495015298096208677436155",
        "36#123456789abcdefghijklmnopqrstuvwxyz",
    );
    test_evaluation("18#100/65537 i", "18#100i/18#b44h");
    test_evaluation("19#100/65537 i", "19#100 i/19#9aa6");
    test_eval_simple("16 to base 2", "10000");
    test_eval_simple("0x10ffff to decimal", "1114111");
    test_eval_simple("0o400 to decimal", "256");
    test_eval_simple("100 to base 6", "244");
    test_eval_simple("65536 to hex", "10000");
    test_eval_simple("65536 to octal", "200000");
    expect_error("5 to base 1.5");
    expect_error("5 to base pi");
    expect_error("5 to base (0pi)");
    expect_error("5 to base 1");
    expect_error("5 to base (-5)");
    expect_error("5 to base 1000000000");
    expect_error("5 to base 100");
    expect_error("5 to base i");
    expect_error("5 to base kg");
    expect_error("6#3e9");
    expect_error("6#3e39");
    test_evaluation("3electroncharge", "3 electroncharge");
    test_evaluation("ℯ to 1", "approx. 2.7182818284");
}

#[test]
fn test_exponents() {
    test_evaluation("1e10", "10000000000");
    test_evaluation("1.5e10", "15000000000");
    test_evaluation("0b1e10", "0b100");
    test_evaluation("0b1e+10", "0b100");
    test_evaluation("0 + 0b1e100", "16");
    test_evaluation("0 + 0b1e1000", "256");
    test_evaluation("0 + 0b1e10000", "65536");
    test_evaluation("0 + 0b1e100000", "4294967296");
    test_evaluation("16#1e10", "16#1e10");
    test_evaluation("0d1e10", "0d10000000000");
    expect_error("11#1e10");
    test_evaluation(
        "0 + 0b1e10000000",
        "340282366920938463463374607431768211456",
    );
    test_evaluation("1.5e-1", "0.15");
    test_evaluation("1.5e0", "1.5");
    test_evaluation("1.5e-0", "1.5");
    test_evaluation("1.5e+0", "1.5");
    test_evaluation("1.5e1", "15");
    test_evaluation("1.5e+1", "15");
    expect_error("1e- 1");
    test_evaluation("0 + 0b1e-110", "0.015625");
    test_evaluation("e", "approx. 2.7182818284");
    test_evaluation("2 e", "approx. 5.4365636569");
    test_evaluation("2e", "approx. 5.4365636569");
    test_evaluation("2e/2", "approx. 2.7182818284");
    test_evaluation("2e / 2", "approx. 2.7182818284");
    expect_error("2e+");
    expect_error("2e-");
    expect_error("2ehello");
    test_evaluation("e^10", "approx. 22026.4657948067");
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
    test_evaluation("cbrt (1kg)", "1 kg^(1/3)");
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
    test_evaluation("0m/s + i kilometers per hour", "5i/18 m / s");
    test_evaluation("0m/s + (1 + i) km/hr", "(5/18 + 5i/18) m / s");
    expect_error("7165928\t761528765");
    test_eval_simple("1 2/3 to fraction", "5/3");
    test_evaluation("abs 2", "2");
    test_evaluation("5 m", "5 m");
    test_evaluation("(4)(6)", "24");
    test_evaluation("5(6)", "30");
    expect_error("(5)6");
    test_evaluation("3’6”", "3.5’");
    test_evaluation("365.25 light days -> ly", "1 ly");
    test_evaluation("365.25 light days as ly", "1 ly");
    test_evaluation("1 light year", "1 light year");
    expect_error("1 2 m");
    test_evaluation("5pi", "approx. 15.7079632679");
    test_evaluation("5 pi/2", "approx. 7.8539816339");
    test_evaluation("5 i/2", "2.5i");
    test_evaluation("3 m 15 cm", "3.15 m");
    test_evaluation("5%", "5%");
    test_evaluation("5% + 0.1", "15%");
    test_evaluation("5% + 1", "105%");
    test_evaluation("0.1 + 5%", "0.15");
    test_evaluation("1 + 5%", "1.05");
    // should be approx.
    test_evaluation("1psi -> kPa -> 5dp", "approx. 6.89475 kPa");
    //test_evaluation("5% * 5%", "0.25%");
    test_evaluation("1NM to m", "1852 m");
    test_evaluation("1NM + 1cm as m", "1852.01 m");
    test_evaluation("1 m / (s kg cd)", "1 m s^-1 kg^-1 cd^-1");
    test_evaluation("1 watt hour / lb", "1 watt hour / lb");
    test_evaluation("4 watt hours / lb", "4 watt hours / lb");
    test_evaluation("1 second second", "1 second second");
    test_evaluation("2 second seconds", "2 second seconds");
    test_evaluation("1 lb^-1", "1 lb^-1");
    test_evaluation("2 lb^-1", "2 lb^-1");
    test_evaluation("2 lb^-1 kg^-1", "2 lb^-1 kg^-1");
    test_evaluation("1 lb^-1 kg^-1", "1 lb^-1 kg^-1");
    test_evaluation("1 light year", "1 light year");
    test_evaluation("1 light year / second", "1 light year / second");
    test_evaluation("2 light years / second", "2 light years / second");
    test_evaluation(
        "2 light years second^-1 lb^-1",
        "2 light years second^-1 lb^-1",
    );
    test_evaluation("1 feet", "1 foot");
    test_evaluation("5 foot", "5 feet");
    test_eval_simple("5 foot 2 inches", "5 1/6 feet");
    test_eval_simple("5 foot 1 inch 1 inch", "5 1/6 feet");
    // this tests if "e" is parsed as the electron charge (instead of Euler's number)
    // in unit definitions
    test_eval_simple(
        "bohrmagneton to C J s/kg to 35 dp",
        "approx. 0.00000000000000000000000927401007831 C J s / kg",
    )
}

#[test]
fn test_no_adjacent_numbers() {
    expect_error("1 2");
    expect_error("1 2 3 4 5");
    expect_error("1 inch 5");
    expect_error("abs 1 2");
    expect_error("1 inch 5 kg");
    test_evaluation("5 (abs 4)", "20");
}

#[test]
fn test_mixed_fractions() {
    test_evaluation("5/3", "1 2/3");
    test_evaluation("4 + 1 2/3", "5 2/3");
    test_evaluation("-8 1/2", "-8.5");
    test_evaluation("-8 1/2'", "-8.5'");
    test_evaluation("1.(3)i", "1 1/3 i");
    test_evaluation("1*1 1/2", "1.5");
    test_evaluation("2*1 1/2", "3");
    test_evaluation("3*2*1 1/2", "9");
    test_evaluation("3 + 2*1 1/2", "6");
    test_evaluation("abs 2*1 1/2", "3");
    expect_error("1/1 1/2");
    expect_error("2/1 1/2");
    test_evaluation("1 1/2 m/s^2", "1.5 m / s^2");
    expect_error("(x:2x) 1 1/2");
    expect_error("pi 1 1/2");
}

#[test]
fn test_unit_sums() {
    test_evaluation("5 feet 12 inch", "6 feet");
    test_evaluation("3'6\"", "3.5'");
    test_evaluation("3’6”", "3.5’");
}

#[test]
fn test_unit_conversions() {
    expect_error("->");
    expect_error("1m->");
    expect_error("1m - >");
    expect_error("->1ft");
    expect_error("1m -> 45ft");
    expect_error("1m -> 45 kg ft");
    test_evaluation("1' -> inches", "12 inches");
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
    expect_error("2^abs 1");
    expect_error("2 4^3");
    expect_error("-2 4^3");
    test_evaluation("3*-2", "-6");
    test_evaluation("-3*-2", "6");
    test_evaluation("-3*2", "-6");
    expect_error("1 2/3^2");
    expect_error("1 2^2/3");
    expect_error("1^2 2/3");
    expect_error("1 2/-3");
    test_evaluation("1 2/3 + 4 5/6", "6.5");
    test_evaluation("1 2/3 + -4 5/6", "-3 1/6");
    test_evaluation("1 2/3 - 4 5/6", "-3 1/6");
    test_evaluation("1 2/3 - 4 + 5/6", "-1.5");
    test_evaluation("1 barn -> m^2", "0.0000000000000000000000000001 m^2");
    test_evaluation("1L -> m^3", "0.001 m^3");
    test_evaluation("5 ft to m", "1.524 m");
    test_evaluation("log10 4", "approx. 0.6020599913");
    test_evaluation("0!", "1");
    test_evaluation("1!", "1");
    test_evaluation("2!", "2");
    test_evaluation("3!", "6");
    test_evaluation("4!", "24");
    test_evaluation("5!", "120");
    test_evaluation("6!", "720");
    test_evaluation("7!", "5040");
    test_evaluation("8!", "40320");
    expect_error("0.5!");
    expect_error("(-2)!");
    expect_error("3i!");
    expect_error("(3 kg)!");
}

#[test]
fn test_recurring_digits() {
    test_eval_simple("9/11 -> float", "0.(81)");
    test_eval_simple("6#1 / 11 -> float", "6#0.(0313452421)");
    test_eval_simple("6#0 + 6#1 / 7 -> float", "6#0.(05)");
    test_eval_simple("0.25 -> fraction", "1/4");
    test_eval_simple("0.21 -> 1 dp", "approx. 0.2");
    test_eval_simple("0.21 -> 1 dp -> auto", "0.21");
    test_eval_simple("502938/700 -> float", "718.48(285714)");
}

#[test]
fn test_builtin_function_names() {
    test_evaluation("abs", "abs");
    test_evaluation("sin", "sin");
    test_evaluation("cos", "cos");
    test_evaluation("tan", "tan");
    test_evaluation("asin", "asin");
    test_evaluation("acos", "acos");
    test_evaluation("atan", "atan");
    test_evaluation("sinh", "sinh");
    test_evaluation("cosh", "cosh");
    test_evaluation("tanh", "tanh");
    test_evaluation("asinh", "asinh");
    test_evaluation("acosh", "acosh");
    test_evaluation("atanh", "atanh");
    test_evaluation("ln", "ln");
    test_evaluation("log2", "log2");
    test_evaluation("log10", "log10");
}

#[test]
fn test_exact_sin() {
    // values from https://en.wikipedia.org/wiki/Trigonometric_constants_expressed_in_real_radicals#Table_of_some_common_angles
    test_evaluation("sin 0", "0");
    test_evaluation("sin pi", "0");
    test_evaluation("sin (2pi)", "0");
    test_evaluation("sin (-pi)", "0");
    test_evaluation("sin (-1000pi)", "0");
    test_evaluation("sin (pi/2)", "1");
    test_evaluation("sin (3pi/2)", "-1");
    test_evaluation("sin (5pi/2)", "1");
    test_evaluation("sin (7pi/2)", "-1");
    test_evaluation("sin (-pi/2)", "-1");
    test_evaluation("sin (-3pi/2)", "1");
    test_evaluation("sin (-5pi/2)", "-1");
    test_evaluation("sin (-7pi/2)", "1");
    test_evaluation("sin (-1023pi/2)", "1");
    test_evaluation("sin (pi/6)", "0.5");
    test_evaluation("sin (5pi/6)", "0.5");
    test_evaluation("sin (7pi/6)", "-0.5");
    test_evaluation("sin (11pi/6)", "-0.5");
    test_evaluation("sin (-pi/6)", "-0.5");
    test_evaluation("sin (-5pi/6)", "-0.5");
    test_evaluation("sin (-7pi/6)", "0.5");
    test_evaluation("sin (-11pi/6)", "0.5");
}

#[test]
fn test_exact_cos() {
    test_evaluation("cos 0", "1");
    test_evaluation("cos pi", "-1");
    test_evaluation("cos (2pi)", "1");
    test_evaluation("cos (-pi)", "-1");
    test_evaluation("cos (-1000pi)", "1");
    test_evaluation("cos (pi/2)", "0");
    test_evaluation("cos (3pi/2)", "0");
    test_evaluation("cos (5pi/2)", "0");
    test_evaluation("cos (7pi/2)", "0");
    test_evaluation("cos (-pi/2)", "0");
    test_evaluation("cos (-3pi/2)", "0");
    test_evaluation("cos (-5pi/2)", "0");
    test_evaluation("cos (-7pi/2)", "0");
    test_evaluation("cos (-1023pi/2)", "0");
    test_evaluation("cos (pi/3)", "0.5");
    test_evaluation("cos (2pi/3)", "-0.5");
    test_evaluation("cos (4pi/3)", "-0.5");
    test_evaluation("cos (5pi/3)", "0.5");
    test_evaluation("cos (-pi/3)", "0.5");
    test_evaluation("cos (-2pi/3)", "-0.5");
    test_evaluation("cos (-4pi/3)", "-0.5");
    test_evaluation("cos (-5pi/3)", "0.5");
}

#[test]
fn test_various_functions() {
    test_evaluation("sin (1m)", "approx. 0.8414709848 m");
    test_evaluation("sin (1°)", "approx. 0.0174524064");
    test_evaluation("tan 0", "0");
    test_evaluation("tan (1meter)", "approx. 1.5574077246 meters");
    test_same("cos 0", "cos (2pi)");
    test_evaluation("cos 1", "approx. 0.5403023058");
    test_same("cos 0", "sin (pi/2)");
    test_same("tan (2pi)", "tan pi");
    test_evaluation("asin 1", "approx. 1.5707963267");
    expect_error("asin 3");
    expect_error("asin (-3)");
    expect_error("asin 1.01");
    expect_error("asin (-1.01)");
    test_evaluation("acos 0", "approx. 1.5707963267");
    expect_error("acos 3");
    expect_error("acos (-3)");
    expect_error("acos 1.01");
    expect_error("acos (-1.01)");
    test_evaluation("atan 1", "approx. 0.7853981633");

    test_same("sinh 0", "approx. 0");
    test_same("cosh 0", "approx. 1");
    test_same("tanh 0", "approx. 0");

    test_same("asinh 0", "asin 0");
    expect_error("acosh 0");
    test_evaluation("acosh 2", "approx. 1.3169578969");
    test_same("atanh 0", "atan 0");
    expect_error("atanh 3");
    expect_error("atanh (-3)");
    expect_error("atanh 1.01");
    expect_error("atanh (-1.01)");
    expect_error("atanh 1");
    expect_error("atanh (-1)");
    test_evaluation("ln 2", "approx. 0.6931471805");
    expect_error("ln 0");
    test_evaluation("exp 2", "approx. 7.3890560989");
    test_evaluation("log10 100", "approx. 2");
    test_evaluation("log10 1000", "approx. 3");
    test_evaluation("log10 10000", "approx. 4");
    test_evaluation("log10 100000", "approx. 5");
    test_evaluation("log2 65536", "approx. 16");
    expect_error("log10 (-1)");
    expect_error("log2 (-1)");
    expect_error("sqrt (-2)");
    test_evaluation("(-2)^3", "-8");
    test_evaluation("(-2)^5", "-32");
    test_evaluation("2^-2", "0.25");
    test_evaluation("(-2)^-2", "0.25");
    test_evaluation("(-2)^-3", "-0.125");
    test_evaluation("(-2)^-4", "0.0625");
    expect_error("oishfod 3");
    test_evaluation("ln", "ln");
    //test_evaluation("sqrt", "x:x^(1/2)");
    test_evaluation("dp", "dp");
    test_evaluation("10 dp", "10 dp");
    test_evaluation("float", "float");
    test_evaluation("fraction", "fraction");
    test_evaluation("auto", "auto");
    expect_error("sqrt i");
    expect_error("sqrt (-2i)");
    expect_error("cbrt i");
    expect_error("cbrt (-2i)");
    expect_error("sin i");
}

#[test]
fn test_unary_div() {
    test_evaluation("/s", "1 s^-1");
    test_evaluation("per second", "1 second^-1");
    test_evaluation("1 Hz + /s", "2 Hz");
}

#[test]
fn test_lambdas() {
    let mut ctx = Context::new();
    test_evaluation("(x: x) 1", "1");
    test_evaluation("(x: y: x) 1 2", "1");
    test_evaluation(
        "(cis: (cis (pi/3))) (x: cos x + i * (sin x))",
        "approx. 0.5 + 0.8660254037i",
    );
    assert!(evaluate("(x: iuwhe)", &mut ctx).is_ok());
    test_evaluation("(b: 5 + b) 1", "6");
    test_evaluation("(addFive: 4)(b: 5 + b)", "4");
    test_evaluation("(addFive: addFive 4)(b: 5 + b)", "9");
    test_evaluation("(x: y: z: x) 1 2 3", "1");
    test_evaluation("(x: y: z: y) 1 2 3", "2");
    test_evaluation("(x: y: z: z) 1 2 3", "3");
    test_evaluation("(one: one + 4) 1", "5");
    test_evaluation("(one: one + one) 1", "2");
    test_evaluation("(x: x to kg) (5 g)", "0.005 kg");
    test_evaluation("(p: q: p p q) (x: y: y) (x: y: y) 1 0", "0");
    test_evaluation("(p: q: p p q) (x: y: y) (x: y: x) 1 0", "1");
    test_evaluation("(p: q: p p q) (x: y: x) (x: y: y) 1 0", "1");
    test_evaluation("(p: q: p p q) (x: y: x) (x: y: x) 1 0", "1");
    test_evaluation("(x => x) 1", "1");
    test_evaluation("(x: y => x) 1 2", "1");
    test_evaluation("(\\x. y => x) 1 2", "1");
    test_evaluation("(\\x.\\y.x)1 2", "1");
    test_evaluation("a. => 0", "a.:0");
}
