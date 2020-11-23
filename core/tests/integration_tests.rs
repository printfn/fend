use fend_core::{evaluate, Context};

macro_rules! test_eval_simple {
    ($e:ident, $input:literal, $expected:literal) => {
        #[test]
        fn $e() {
            let mut context = Context::new();
            assert_eq!(
                evaluate($input, &mut context).unwrap().get_main_result(),
                $expected
            );
        }
    };
}

macro_rules! test_eval {
    ($e:ident, $input:literal, $expected:literal) => {
        #[test]
        fn $e() {
            let mut context = Context::new();
            assert_eq!(
                evaluate($input, &mut context).unwrap().get_main_result(),
                $expected
            );
            // try parsing the output again, and make sure it matches
            assert_eq!(
                evaluate($expected, &mut context).unwrap().get_main_result(),
                $expected
            );
        }
    };
}

macro_rules! test_same {
    ($e:ident, $a:literal, $b:literal) => {
        #[test]
        fn $e() {
            let mut context = Context::new();
            assert_eq!(
                evaluate($a, &mut context).unwrap().get_main_result(),
                evaluate($b, &mut context).unwrap().get_main_result()
            );
        }
    };
}

macro_rules! expect_error {
    ($e:ident, $input:literal) => {
        #[test]
        fn $e() {
            let mut context = Context::new();
            assert!(evaluate($input, &mut context).is_err());
        }
    };
    ($e:ident, $input:literal, $message:expr) => {
        #[test]
        fn $e() {
            let mut context = Context::new();
            assert_eq!(evaluate($input, &mut context), Err($message.to_string()));
        }
    };
}

test_eval!(two, "2", "2");
test_eval!(nine, "9", "9");
test_eval!(ten, "10", "10");
test_eval!(
    large_integer,
    "39456720983475234523452345",
    "39456720983475234523452345"
);
test_eval!(ten_whitespace_after, "10 ", "10");
test_eval!(ten_whitespace_before, " 10", "10");
test_eval!(ten_whitespace_both, " 10\n\r\n", "10");
test_eval!(blank_input, "", "");

#[test]
fn version() {
    let mut ctx = Context::new();
    let result = evaluate("version", &mut ctx).unwrap();
    for c in result.get_main_result().chars() {
        assert!(c.is_ascii_digit() || c == '.');
    }
}

test_eval!(pi, "pi", "approx. 3.1415926535");
test_eval!(pi_times_two, "pi * 2", "approx. 6.2831853071");
test_eval!(two_pi, "2 pi", "approx. 6.2831853071");

#[test]
fn pi_to_fraction() {
    let mut ctx = Context::new();
    assert!(evaluate("pi to fraction", &mut ctx)
        .unwrap()
        .get_main_result()
        .starts_with("approx."));
}

const DIVISION_BY_ZERO_ERROR: &str = "Division by zero";
expect_error!(one_over_zero, "1/0", DIVISION_BY_ZERO_ERROR);
expect_error!(zero_over_zero, "0/0", DIVISION_BY_ZERO_ERROR);
expect_error!(minus_one_over_zero, "-1/0", DIVISION_BY_ZERO_ERROR);
expect_error!(
    three_pi_over_zero_pi,
    "(3pi) / (0pi)",
    DIVISION_BY_ZERO_ERROR
);
expect_error!(
    minus_one_over_zero_indirect,
    "-1/(2-2)",
    DIVISION_BY_ZERO_ERROR
);

test_eval!(two_zeroes, "00", "0");
test_eval!(six_zeroes, "000000", "0");
test_eval!(multiple_zeroes_with_decimal_point, "000000.01", "0.01");
test_eval!(leading_zeroes_and_decimal_point, "0000001.01", "1.01");

test_eval!(binary_leading_zeroes, "0b01", "0b1");
test_eval!(hex_leading_zeroes, "0x0000_00ff", "0xff");
test_eval!(explicit_base_10_leading_zeroes, "10#04", "10#4");
test_eval!(leading_zeroes_after_decimal_point, "1.001", "1.001");
test_eval!(leading_zeroes_in_exponent, "1e01", "10");
test_eval!(leading_zeroes_in_negative_exponent, "1e-01", "0.1");

expect_error!(no_recurring_digits, "0.()");

test_eval_simple!(to_float_1, "0.(3) to float", "0.(3)");
test_eval_simple!(to_float_2, "0.(33) to float", "0.(3)");
test_eval_simple!(to_float_3, "0.(34) to float", "0.(34)");
test_eval_simple!(to_float_4, "0.(12345) to float", "0.(12345)");
test_eval_simple!(to_float_5, "0.(0) to float", "0");
test_eval_simple!(to_float_6, "0.123(00) to float", "0.123");
test_eval_simple!(to_float_7, "0.0(34) to float", "0.0(34)");
test_eval_simple!(to_float_8, "0.00(34) to float", "0.00(34)");
test_eval_simple!(to_float_9, "0.0000(34) to float", "0.0000(34)");
test_eval_simple!(to_float_10, "0.123434(34) to float", "0.12(34)");
test_eval_simple!(to_float_11, "0.123434(34)i to float", "0.12(34)i");
test_eval_simple!(
    to_float_12,
    "0.(3) + 0.123434(34)i to float",
    "0.(3) + 0.12(34)i"
);
test_eval_simple!(to_float_13, "6#0.(1) to float", "6#0.(1)");
test_eval_simple!(to_float_14, "6#0.(1) to float to base 10", "0.2");

test_eval!(two_times_two, "2*2", "4");
test_eval!(two_times_two_whitespace, "\n2\n*\n2\n", "4");
test_eval!(
    large_multiplication,
    "315427679023453451289740 * 927346502937456234523452",
    "292510755072077978255166497050046859223676982480"
);
test_eval!(pi_times_pi, "pi * pi", "approx. 9.869604401");
test_eval!(four_pi_plus_one, "4pi + 1", "approx. 13.5663706143");

test_eval!(implicit_lambda_1, "-sin (-pi/2)", "1");
test_eval!(implicit_lambda_2, "+sin (-pi/2)", "-1");
test_eval!(implicit_lambda_3, "/sin (-pi/2)", "-1");
test_eval!(implicit_lambda_4, "cos! 0", "1");
test_eval!(implicit_lambda_5, "sqrt! 16", "24");
test_eval!(implicit_lambda_6, "///sqrt! 16", "approx. 0.0416666666");
test_eval!(implicit_lambda_7, "(x: sin^2 x + cos^2 x) 1", "approx. 1");
test_eval!(implicit_lambda_8, "cos^2 pi", "1");
test_eval!(implicit_lambda_9, "sin pi/cos pi", "0");
test_eval!(implicit_lambda_10, "sin + 1) pi", "1");
test_eval!(implicit_lambda_11, "3sin pi", "0");
test_eval!(implicit_lambda_12, "(-sqrt) 4", "-2");
//test_eval!(implicit_lambda_13, "-sqrt 4", "-2");

test_eval!(inverse_sin, "sin^-1", "asin");
test_eval!(inverse_sin_point_five, "sin^-1 0.5", "approx. 0.5235987755");
test_eval!(inverse_sin_nested, "sin^-1 (sin 0.5", "approx. 0.5");
test_eval!(inverse_sin_nested_2, "(sin^-1)^-1", "sin");
test_eval!(inverse_cos, "cos^-1", "acos");
test_eval!(inverse_tan, "tan^-1", "atan");
test_eval!(inverse_asin, "asin^-1", "sin");
test_eval!(inverse_acos, "acos^-1", "cos");
test_eval!(inverse_atan, "atan^-1", "tan");
test_eval!(inverse_sinh, "sinh^-1", "asinh");
test_eval!(inverse_cosh, "cosh^-1", "acosh");
test_eval!(inverse_tanh, "tanh^-1", "atanh");
test_eval!(inverse_asinh, "asinh^-1", "sinh");
test_eval!(inverse_acosh, "acosh^-1", "cosh");
test_eval!(inverse_atanh, "atanh^-1", "tanh");

test_eval!(two_plus_two, "2+2", "4");
test_eval!(two_plus_two_whitespace, "\n2\n+\n2\n", "4");
test_eval!(plus_two, "+2", "2");
test_eval!(unary_pluses_two, "++++2", "2");
test_eval!(
    large_simple_addition,
    "315427679023453451289740 + 927346502937456234523452",
    "1242774181960909685813192"
);

test_eval!(minus_zero, "-0", "0");
test_eval!(two_minus_two, "2-2", "0");
test_eval!(three_minus_two, "3-2", "1");
test_eval!(two_minus_three, "2-3", "-1");
test_eval!(minus_two, "-2", "-2");
test_eval!(minus_minus_two, "--2", "2");
test_eval!(minus_minus_minus_two, "---2", "-2");
test_eval!(minus_minus_minus_two_parens, "-(--2)", "-2");
test_eval!(two_minus_64, "\n2\n-\n64\n", "-62");
test_eval!(
    large_simple_subtraction,
    "315427679023453451289740 - 927346502937456234523452",
    "-611918823914002783233712"
);
test_eval!(three_pi_minus_two_pi, "3pi - 2pi", "approx. 3.1415926535");
test_eval!(
    four_pi_plus_one_over_pi,
    "4pi-1)/pi",
    "approx. 3.6816901138"
);
test_eval!(
    large_simple_subtraction_2,
    "36893488123704996004 - 18446744065119617025",
    "18446744058585378979"
);

test_eval!(sqrt_half, "sqrt (1/2)", "approx. 0.7071067814");

test_eval!(sqrt_0, "sqrt 0", "0");
test_eval!(sqrt_1, "sqrt 1", "1");
test_eval!(sqrt_2, "sqrt 2", "approx. 1.4142135619");
test_eval!(sqrt_pi, "sqrt pi", "approx. 1.7724538509");
test_eval!(sqrt_4, "sqrt 4", "2");
test_eval!(sqrt_9, "sqrt 9", "3");
test_eval!(sqrt_16, "sqrt 16", "4");
test_eval!(sqrt_25, "sqrt 25", "5");
test_eval!(sqrt_36, "sqrt 36", "6");
test_eval!(sqrt_49, "sqrt 49", "7");
test_eval!(sqrt_64, "sqrt 64", "8");
test_eval!(sqrt_81, "sqrt 81", "9");
test_eval!(sqrt_100, "sqrt 100", "10");
test_eval!(sqrt_10000, "sqrt 10000", "100");
test_eval!(sqrt_1000000, "sqrt 1000000", "1000");
test_eval!(sqrt_quarter, "sqrt 0.25", "0.5");
test_eval!(sqrt_sixteenth, "sqrt 0.0625", "0.25");

test_eval!(cbrt_0, "cbrt 0", "0");
test_eval!(cbrt_1, "cbrt 1", "1");
test_eval!(cbrt_8, "cbrt 8", "2");
test_eval!(cbrt_27, "cbrt 27", "3");
test_eval!(cbrt_64, "cbrt 64", "4");
test_eval!(cbrt_eighth, "cbrt (1/8)", "0.5");
test_eval!(cbrt_1_over_125, "cbrt (125/8)", "2.5");

test_eval!(sqrt_kg_squared_1, "sqrt(kg^2)", "1 kg");
test_eval!(sqrt_kg_squared_2, "(sqrt kg)^2", "1 kg");
test_eval!(
    lightyear_to_parsecs,
    "1 lightyear to parsecs",
    "approx. 0.3066013937 parsecs"
);

test_eval!(order_of_operations_1, "2+2*3", "8");
test_eval!(order_of_operations_2, "2*2+3", "7");
test_eval!(order_of_operations_3, "2+2+3", "7");
test_eval!(order_of_operations_4, "2+2-3", "1");
test_eval!(order_of_operations_5, "2-2+3", "3");
test_eval!(order_of_operations_6, "2-2-3", "-3");
test_eval!(order_of_operations_7, "2*2*3", "12");
test_eval!(order_of_operations_8, "2*2*-3", "-12");
test_eval!(order_of_operations_9, "2*-2*3", "-12");
test_eval!(order_of_operations_10, "-2*2*3", "-12");
test_eval!(order_of_operations_11, "-2*-2*3", "12");
test_eval!(order_of_operations_12, "-2*2*-3", "12");
test_eval!(order_of_operations_13, "2*-2*-3", "12");
test_eval!(order_of_operations_14, "-2*-2*-3", "-12");
test_eval!(order_of_operations_15, "-2*-2*-3/2", "-6");
test_eval!(order_of_operations_16, "-2*-2*-3/-2", "6");
test_eval!(order_of_operations_17, "-3 -1/2", "-3.5");

test_eval!(
    yobibyte,
    "1 YiB to bytes",
    "1208925819614629174706176 bytes"
);

test_eval!(div_1_over_1, "1/1", "1");
test_eval!(div_1_over_2, "1/2", "0.5");
test_eval!(div_1_over_4, "1/4", "0.25");
test_eval!(div_1_over_8, "1/8", "0.125");
test_eval!(div_1_over_16, "1/16", "0.0625");
test_eval!(div_1_over_32, "1/32", "0.03125");
test_eval!(div_1_over_64, "1/64", "0.015625");
test_eval!(div_2_over_64, "2/64", "0.03125");
test_eval!(div_4_over_64, "4/64", "0.0625");
test_eval!(div_8_over_64, "8/64", "0.125");
test_eval!(div_16_over_64, "16/64", "0.25");
test_eval!(div_32_over_64, "32/64", "0.5");
test_eval!(div_64_over_64, "64/64", "1");
test_eval!(div_2_over_1, "2/1", "2");
test_eval!(div_27_over_3, "27/3", "9");
test_eval!(div_100_over_4, "100/4", "25");
test_eval!(div_100_over_5, "100/5", "20");
test_eval!(div_large_1, "18446744073709551616/2", "9223372036854775808");
test_eval!(
    div_large_2,
    "184467440737095516160000000000000/2",
    "92233720368547758080000000000000"
);
test_eval!(div_exact_pi, "(3pi) / (2pi)", "1.5");

test_eval!(zero_point_zero, "0.0", "0");
test_eval!(zero_point_multiple_zeroes, "0.000000", "0");
test_eval!(zero_point_zero_one, "0.01", "0.01");
test_eval!(zero_point_zero_one_zeroes, "0.01000", "0.01");
test_eval!(zero_point_two_five, "0.25", "0.25");
expect_error!(one_point, "1.");
test_eval!(point_one, ".1", "0.1");
test_eval!(point_one_e_minus_one, ".1e-1", "0.01");
test_eval!(leading_zeroes_with_dp, "001.01000", "1.01");
test_eval!(
    very_long_decimal,
    "0.251974862348971623412341534273261435",
    "0.251974862348971623412341534273261435"
);
test_eval!(
    one_point_zeroes_1_as_1_dp,
    "1.00000001 as 1 dp",
    "approx. 1"
);
test_eval!(
    one_point_zeroes_1_as_2_dp,
    "1.00000001 as 2 dp",
    "approx. 1"
);
test_eval!(
    one_point_zeroes_1_as_3_dp,
    "1.00000001 as 3 dp",
    "approx. 1"
);
test_eval!(
    one_point_zeroes_1_as_4_dp,
    "1.00000001 as 4 dp",
    "approx. 1"
);
test_eval!(
    one_point_zeroes_1_as_10_dp,
    "1.00000001 as 10 dp",
    "1.00000001"
);
test_eval!(
    one_point_zeroes_1_as_30_dp,
    "1.00000001 as 30 dp",
    "1.00000001"
);
test_eval!(
    one_point_zeroes_1_as_1000_dp,
    "1.00000001 as 1000 dp",
    "1.00000001"
);
test_eval!(
    one_point_zeroes_1_as_0_dp,
    "1.00000001 as 0 dp",
    "approx. 1"
);
test_eval!(point_1_zero_recurring, ".1(0)", "0.1");
test_eval!(recurring_product_whitespace_1, ".1( 0)", "0");
test_eval!(recurring_product_whitespace_2, ".1 ( 0)", "0");
expect_error!(point_1_zero_recurring_whitespace_error, ".1(0 )");
expect_error!(point_1_zero_recurring_letters_error, ".1(0a)");
test_eval!(recurring_product_with_e, "2.0(e)", "approx. 5.4365636569");
test_eval!(
    recurring_product_with_function,
    "2.0(ln 5)",
    "approx. 3.2188758248"
);
test_eval!(integer_product_whitespace_1, "2 (5)", "10");
test_eval!(integer_product_whitespace_2, "2( 5)", "10");
test_eval!(
    large_division,
    "60153992292001127921539815855494266880 / 9223372036854775808",
    "6521908912666391110"
);

test_eval!(parentheses_1, "(1)", "1");
test_eval!(parentheses_2, "(0.0)", "0");
test_eval!(parentheses_3, "(1+-2)", "-1");
test_eval!(parentheses_4, "1+2*3", "7");
test_eval!(parentheses_5, "(1+2)*3", "9");
test_eval!(parentheses_6, "((1+2))*3", "9");
test_eval!(parentheses_7, "((1)+2)*3", "9");
test_eval!(parentheses_8, "(1+(2))*3", "9");
test_eval!(parentheses_9, "(1+(2)*3)", "7");
test_eval!(parentheses_10, "1+(2*3)", "7");
test_eval!(parentheses_11, "1+((2 )*3)", "7");
test_eval!(parentheses_12, " 1 + ( (\r\n2 ) * 3 ) ", "7");
test_eval!(parentheses_13, "2*(1+3", "8");
test_eval!(parentheses_14, "4+5+6)*(1+2", "45");
test_eval!(parentheses_15, "4+5+6))*(1+2", "45");

test_eval!(powers_1, "1^1", "1");
test_eval!(powers_2, "1**1", "1");
test_eval!(powers_3, "1**1.0", "1");
test_eval!(powers_4, "1.0**1", "1");
test_eval!(powers_5, "2^4", "16");
test_eval!(powers_6, "4^2", "16");
test_eval!(powers_7, "4^3", "64");
test_eval!(powers_8, "4^(3^1)", "64");
test_eval!(powers_9, "4^3^1", "64");
test_eval!(powers_10, "(4^3)^1", "64");
test_eval!(powers_11, "(2^3)^4", "4096");
test_eval!(powers_12, "2^3^2", "512");
test_eval!(powers_13, "(2^3)^2", "64");
test_eval!(powers_14, "4^0.5", "2");
test_eval!(powers_15, "4^(1/2)", "2");
test_eval!(powers_16, "4^(1/4)", "approx. 1.4142135619");
test_eval!(powers_17, "(2/3)^(4/5)", "approx. 0.7229811807");
test_eval!(
    powers_18,
    "5.2*10^15*300^(3/2)",
    "approx. 27019992598076723515.9873962402"
);
test_eval!(pi_to_the_power_of_ten, "pi^10", "approx. 93648.047476083");
expect_error!(zero_to_the_power_of_zero, "0^0");
test_eval!(zero_to_the_power_of_one, "0^1", "0");
test_eval!(one_to_the_power_of_zero, "1^0", "1");
test_eval!(one_to_the_power_of_huge_exponent, "1^1e1000", "1");
expect_error!(exponent_too_large, "2^1e1000");
expect_error!(i_cubed, "i^3");
expect_error!(four_to_the_power_of_i, "4^i");
expect_error!(i_to_the_power_of_i, "i^i");
test_eval!(unit_to_approx_power, "kg^(approx. 1)", "approx. 1 kg");

test_eval!(negative_decimal, "-0.125", "-0.125");
test_eval!(two_pow_one_pow_two, "2^1^2", "2");
test_eval!(two_pow_one_parens_one_pow_two, "2^(1^2)", "2");
test_eval!(two_pow_parens_one, "2^(1)", "2");
test_eval!(negative_power_1, "2 * (-2^3)", "-16");
test_eval!(negative_power_2, "2 * -2^3", "-16");
test_eval!(negative_power_3, "2^-3 * 4", "0.5");
test_eval!(negative_power_4, "2^3 * 4", "32");
test_eval!(negative_power_5, "-2^-3", "-0.125");
test_eval!(negative_product, "2 * -3 * 4", "-24");
test_eval!(negative_power_6, "4^-1^2", "0.25");
test_same!(negative_power_7, "2^-3^4", "1 / 2^81");

test_eval!(i, "i", "i");
test_eval!(three_i, "3i", "3i");
test_eval!(three_i_plus_four, "3i+4", "4 + 3i");
test_eval!(three_i_plus_four_plus_i, "(3i+4) + i", "4 + 4i");
test_eval!(three_i_plus_four_plus_i_2, "3i+(4 + i)", "4 + 4i");
test_eval!(minus_three_i, "-3i", "-3i");
test_eval!(i_over_i, "i/i", "1");
test_eval!(i_times_i, "i*i", "-1");
test_eval!(i_times_i_times_i, "i*i*i", "-i");
test_eval!(i_times_i_times_i_times_i, "i*i*i*i", "1");
test_eval!(minus_three_plus_i, "-3+i", "-3 + i");
test_eval!(i_plus_i, "1+i", "1 + i");
test_eval!(i_minus_i, "1-i", "1 - i");
test_eval!(minus_one_plus_i, "-1 + i", "-1 + i");
test_eval!(minus_one_minus_i, "-1 - i", "-1 - i");
test_eval!(minus_one_minus_two_i, "-1 - 2i", "-1 - 2i");
test_eval!(minus_one_minus_half_i, "-1 - 0.5i", "-1 - 0.5i");
test_eval!(
    minus_one_minus_half_i_plus_half_i,
    "-1 - 0.5i + 1.5i",
    "-1 + i"
);
test_eval!(minus_i, "-i", "-i");
test_eval!(plus_i, "+i", "i");
test_eval!(two_i, "2i", "2i");
test_eval!(i_over_3, "i/3", "i/3");
test_eval!(two_i_over_three, "2i/3", "2i/3");
test_eval!(two_i_over_minus_three_minus_one, "2i/-3-1", "-1 - 2i/3");
expect_error!(i_is_not_a_binary_digit, "2#i");

test_eval!(digit_separators_1, "1_1", "11");
test_eval!(digit_separators_2, "11_1", "111");
test_eval!(digit_separators_3, "1_1_1", "111");
test_eval!(digit_separators_4, "123_456_789_123", "123456789123");
test_eval!(digit_separators_5, "1_2_3_4_5_6", "123456");
test_eval!(digit_separators_6, "1.1_1", "1.11");
test_eval!(digit_separators_7, "1_1.1_1", "11.11");
expect_error!(digit_separators_8, "_1");
expect_error!(digit_separators_9, "1_");
expect_error!(digit_separators_10, "1__1");
expect_error!(digit_separators_11, "_");
expect_error!(digit_separators_12, "1_.1");
expect_error!(digit_separators_13, "1._1");
expect_error!(digit_separators_14, "1.1_");
test_eval!(digit_separators_15, "1,1", "11");
test_eval!(digit_separators_16, "11,1", "111");
test_eval!(digit_separators_17, "1,1,1", "111");
test_eval!(digit_separators_18, "123,456,789,123", "123456789123");
test_eval!(digit_separators_19, "1,2,3,4,5,6", "123456");
test_eval!(digit_separators_20, "1.1,1", "1.11");
test_eval!(digit_separators_21, "1,1.1,1", "11.11");
expect_error!(digit_separators_22, ",1");
expect_error!(digit_separators_23, "1,");
expect_error!(digit_separators_24, "1,,1");
expect_error!(digit_separators_25, ",");
expect_error!(digit_separators_26, "1,.1");
expect_error!(digit_separators_27, "1.,1");
expect_error!(digit_separators_28, "1.1,");

test_eval!(different_base_1, "0x10", "0x10");
test_eval!(different_base_2, "0o10", "0o10");
test_eval!(different_base_3, "0b10", "0b10");
test_eval!(different_base_4, "0x10 - 1", "0xf");
test_eval!(different_base_5, "0x0 + sqrt 16", "0x4");
test_eval!(different_base_6, "16#0 + sqrt 16", "16#4");
test_eval!(different_base_7, "0 + 6#100", "36");
test_eval!(different_base_8, "0 + 36#z", "35");
test_eval!(different_base_9, "16#dead_beef", "16#deadbeef");
test_eval!(different_base_10, "16#DEAD_BEEF", "16#deadbeef");
expect_error!(different_base_11, "#");
expect_error!(different_base_12, "0#0");
expect_error!(different_base_13, "1#0");
expect_error!(different_base_14, "2_2#0");
expect_error!(different_base_15, "22 #0");
expect_error!(different_base_16, "22# 0");
test_eval!(different_base_17, "36#i i", "36#i i");
test_eval!(different_base_18, "16#1 i", "16#1 i");
test_eval!(different_base_19, "16#f i", "16#f i");
test_eval!(different_base_20, "0 + 36#ii", "666");
expect_error!(different_base_21, "18#i/i");
test_eval!(different_base_22, "19#i/i", "-19#i i");
// verified using a ruby program
test_eval!(
    different_base_23,
    "0+36#0123456789abcdefghijklmnopqrstuvwxyz",
    "86846823611197163108337531226495015298096208677436155"
);
test_eval!(
    different_base_24,
    "36#0 + 86846823611197163108337531226495015298096208677436155",
    "36#123456789abcdefghijklmnopqrstuvwxyz"
);
test_eval!(different_base_25, "18#100/65537 i", "18#100i/18#b44h");
test_eval!(different_base_26, "19#100/65537 i", "19#100 i/19#9aa6");
expect_error!(different_base_27, "5 to base 1.5");
expect_error!(different_base_28, "5 to base pi");
expect_error!(different_base_29, "5 to base (0pi)");
expect_error!(different_base_30, "5 to base 1");
expect_error!(different_base_31, "5 to base (-5)");
expect_error!(different_base_32, "5 to base 1000000000");
expect_error!(different_base_33, "5 to base 100");
expect_error!(different_base_34, "5 to base i");
expect_error!(different_base_35, "5 to base kg");
expect_error!(different_base_36, "6#3e9");
expect_error!(different_base_37, "6#3e39");
test_eval!(different_base_38, "9#5i", "9#5i");

test_eval!(
    three_electroncharge,
    "3electroncharge",
    "0.0000000000000000004806529902 C"
);
test_eval!(e_to_1, "ℯ to 1", "approx. 2.7182818284");

test_eval_simple!(base_conversion_1, "16 to base 2", "10000");
test_eval_simple!(base_conversion_2, "0x10ffff to decimal", "1114111");
test_eval_simple!(base_conversion_3, "0o400 to decimal", "256");
test_eval_simple!(base_conversion_4, "100 to base 6", "244");
test_eval_simple!(base_conversion_5, "65536 to hex", "10000");
test_eval_simple!(base_conversion_6, "65536 to octal", "200000");

test_eval!(exponents_1, "1e10", "10000000000");
test_eval!(exponents_2, "1.5e10", "15000000000");
test_eval!(exponents_3, "0b1e10", "0b100");
test_eval!(exponents_4, "0b1e+10", "0b100");
test_eval!(exponents_5, "0 + 0b1e100", "16");
test_eval!(exponents_6, "0 + 0b1e1000", "256");
test_eval!(exponents_7, "0 + 0b1e10000", "65536");
test_eval!(exponents_8, "0 + 0b1e100000", "4294967296");
test_eval!(exponents_9, "16#1e10", "16#1e10");
test_eval!(exponents_10, "0d1e10", "0d10000000000");
expect_error!(exponents_11, "11#1e10");
test_eval!(
    binary_exponent,
    "0 + 0b1e10000000",
    "340282366920938463463374607431768211456"
);
test_eval!(exponents_12, "1.5e-1", "0.15");
test_eval!(exponents_13, "1.5e0", "1.5");
test_eval!(exponents_14, "1.5e-0", "1.5");
test_eval!(exponents_15, "1.5e+0", "1.5");
test_eval!(exponents_16, "1.5e1", "15");
test_eval!(exponents_17, "1.5e+1", "15");
expect_error!(exponents_18, "1e- 1");
test_eval!(exponents_19, "0 + 0b1e-110", "0.015625");
test_eval!(exponents_20, "e", "approx. 2.7182818284");
test_eval!(exponents_21, "2 e", "approx. 5.4365636569");
test_eval!(exponents_22, "2e", "approx. 5.4365636569");
test_eval!(exponents_23, "2e/2", "approx. 2.7182818284");
test_eval!(exponents_24, "2e / 2", "approx. 2.7182818284");
expect_error!(exponents_25, "2e+");
expect_error!(exponents_26, "2e-");
expect_error!(exponents_27, "2ehello");
test_eval!(exponents_28, "e^10", "approx. 22026.4657948067");

test_eval!(one_kg, "1kg", "1 kg");
test_eval!(one_g, "1g", "1 g");
test_eval!(one_kg_plus_one_g, "1kg + 1g", "1.001 kg");
test_eval!(one_kg_plus_100_g, "1kg + 100g", "1.1 kg");
test_eval!(zero_g_plus_1_kg_plus_100_g, "0g + 1kg + 100g", "1100 g");
test_eval!(zero_g_plus_1_kg, "0g + 1kg", "1000 g");
test_eval!(one_over_half_kg, "1/0.5 kg", "2 kg");
test_eval!(one_over_one_over_half_kg, "1/(1/0.5 kg)", "0.5 kg^-1");
test_eval!(cbrt_kg, "cbrt (1kg)", "1 kg^(1/3)");

test_eval!(one_kg_plug_i_g, "1 kg + i g", "(1 + 0.001i) kg");

test_eval!(abs_2, "abs 2", "2");
test_eval!(five_meters, "5 m", "5 m");
test_eval!(parentheses_multiplication, "(4)(6)", "24");
test_eval!(parentheses_multiplication_2, "5(6)", "30");
expect_error!(multiply_number_without_parentheses, "(5)6");
expect_error!(simple_adjacent_numbers, "7165928\t761528765");

test_eval!(three_feet_six_inches, "3’6”", "3.5’");
test_eval!(five_feet_twelve_inches, "5 feet 12 inch", "6 feet");
test_eval!(three_feet_six_inches_ascii, "3'6\"", "3.5'");
test_eval!(three_meters_15_cm, "3 m 15 cm", "3.15 m");

test_eval!(five_percent, "5%", "5%");
test_eval!(five_percent_plus_point_one, "5% + 0.1", "15%");
test_eval!(five_percent_plus_one, "5% + 1", "105%");
test_eval!(point_one_plus_five_percent, "0.1 + 5%", "0.15");
test_eval!(one_plus_five_percent, "1 + 5%", "1.05");
//test_eval!(five_percent_times_five_percent, "5% * 5%", "0.25%");

test_eval!(units_1, "0m + 1kph * 1 hr", "1000 m");
test_eval!(units_2, "0GiB + 1GB", "0.931322574615478515625 GiB");
test_eval!(units_3, "0m/s + 1 km/hr", "approx. 0.2777777777 m / s");
test_eval!(units_4, "0m/s + i km/hr", "5i/18 m / s");
test_eval!(units_5, "0m/s + i kilometers per hour", "5i/18 m / s");
test_eval!(units_6, "0m/s + (1 + i) km/hr", "(5/18 + 5i/18) m / s");
test_eval!(units_9, "365.25 light days -> ly", "1 ly");
test_eval!(units_10, "365.25 light days as ly", "1 ly");
test_eval!(units_11, "1 light year", "1 light year");
expect_error!(units_12, "1 2 m");
test_eval!(units_13, "5pi", "approx. 15.7079632679");
test_eval!(units_14, "5 pi/2", "approx. 7.8539816339");
test_eval!(units_15, "5 i/2", "2.5i");
test_eval!(units_22, "1psi -> kPa -> 5dp", "approx. 6.89475 kPa");
test_eval!(units_23, "1NM to m", "1852 m");
test_eval!(units_24, "1NM + 1cm as m", "1852.01 m");
test_eval!(units_25, "1 m / (s kg cd)", "1 m s^-1 kg^-1 cd^-1");
test_eval!(units_26, "1 watt hour / lb", "1 watt hour / lb");
test_eval!(units_27, "4 watt hours / lb", "4 watt hours / lb");
test_eval!(units_28, "1 second second", "1 second second");
test_eval!(units_29, "2 second seconds", "2 second seconds");
test_eval!(units_30, "1 lb^-1", "1 lb^-1");
test_eval!(units_31, "2 lb^-1", "2 lb^-1");
test_eval!(units_32, "2 lb^-1 kg^-1", "2 lb^-1 kg^-1");
test_eval!(units_33, "1 lb^-1 kg^-1", "1 lb^-1 kg^-1");
test_eval!(units_34, "1 light year", "1 light year");
test_eval!(units_35, "1 light year / second", "1 light year / second");
test_eval!(units_36, "2 light years / second", "2 light years / second");
test_eval!(
    units_37,
    "2 light years second^-1 lb^-1",
    "2 light years second^-1 lb^-1"
);
test_eval!(units_38, "1 feet", "1 foot");
test_eval!(units_39, "5 foot", "5 feet");
test_eval!(units_40, "5 foot 2 inches", "approx. 5.1666666666 feet");
test_eval!(
    units_41,
    "5 foot 1 inch 1 inch",
    "approx. 5.1666666666 feet"
);

// this tests if "e" is parsed as the electron charge (instead of Euler's number)
// in unit definitions
test_eval!(
    electroncharge_and_bohrmagneton,
    "(bohrmagneton to C J s/kg) * 1e35",
    "approx. 927401007831.8305442879 C J s / kg"
);

expect_error!(plain_adjacent_numbers, "1 2");
expect_error!(multiple_plain_adjacent_numbers, "1 2 3 4 5");
expect_error!(implicit_sum_missing_unit, "1 inch 5");
expect_error!(implicit_sum_incompatible_unit, "1 inch 5 kg");

expect_error!(too_many_args, "abs 1 2");
test_eval!(abs_4_with_coefficient, "5 (abs 4)", "20");

test_eval_simple!(
    mixed_fraction_to_improper_fraction,
    "1 2/3 to fraction",
    "5/3"
);

test_eval!(mixed_fractions_1, "5/3", "approx. 1.6666666666");
test_eval!(mixed_fractions_2, "4 + 1 2/3", "approx. 5.6666666666");
test_eval!(mixed_fractions_3, "-8 1/2", "-8.5");
test_eval!(mixed_fractions_4, "-8 1/2'", "-8.5'");
test_eval!(mixed_fractions_5, "1.(3)i", "1 1/3 i");
test_eval!(mixed_fractions_6, "1*1 1/2", "1.5");
test_eval!(mixed_fractions_7, "2*1 1/2", "3");
test_eval!(mixed_fractions_8, "3*2*1 1/2", "9");
test_eval!(mixed_fractions_9, "3 + 2*1 1/2", "6");
test_eval!(mixed_fractions_10, "abs 2*1 1/2", "3");
expect_error!(mixed_fractions_11, "1/1 1/2");
expect_error!(mixed_fractions_12, "2/1 1/2");
test_eval!(mixed_fractions_13, "1 1/2 m/s^2", "1.5 m / s^2");
expect_error!(mixed_fractions_14, "(x:2x) 1 1/2");
expect_error!(mixed_fractions_15, "pi 1 1/2");

expect_error!(lone_conversion_arrow, "->");
expect_error!(conversion_arrow_no_rhs, "1m->");
expect_error!(conversion_arrow_with_space_in_the_middle, "1m - >");
expect_error!(conversion_arrow_no_lhs, "->1ft");
expect_error!(meter_to_feet, "1m -> 45ft");
expect_error!(meter_to_kg_ft, "1m -> 45 kg ft");
test_eval!(one_foot_to_inches, "1' -> inches", "12 inches");

test_eval!(abs_1, "abs 1", "1");
test_eval!(abs_i, "abs i", "1");
test_eval!(abs_minus_1, "abs (-1)", "1");
test_eval!(abs_minus_i, "abs (-i)", "1");
test_eval!(abs_2_i, "abs (2i)", "2");
test_eval!(abs_1_plus_i, "abs (1 + i)", "approx. 1.4142135619");

test_eval!(two_kg_squared, "2 kg^2", "2 kg^2");
test_eval!(quarter_kg_pow_minus_two, "((1/4) kg)^-2", "16 kg^-2");
test_eval!(newton_subtraction, "1 N - 1 kg m s^-2", "0 N");
test_eval!(
    joule_subtraction,
    "1 J - 1 kg m^2 s^-2 + 1 kg / (m^-2 s^2)",
    "1 J"
);
test_eval!(two_to_the_power_of_abs_one, "2^abs 1", "2");
expect_error!(adjacent_numbers_rhs_cubed, "2 4^3");
expect_error!(negative_adjacent_numbers_rhs_cubed, "-2 4^3");
test_eval!(product_with_unary_minus_1, "3*-2", "-6");
test_eval!(product_with_unary_minus_2, "-3*-2", "6");
test_eval!(product_with_unary_minus_3, "-3*2", "-6");
expect_error!(illegal_mixed_fraction_with_pow_1, "1 2/3^2");
expect_error!(illegal_mixed_fraction_with_pow_2, "1 2^2/3");
expect_error!(illegal_mixed_fraction_with_pow_3, "1^2 2/3");
expect_error!(illegal_mixed_fraction_with_pow_4, "1 2/-3");
test_eval!(positive_mixed_fraction_sum, "1 2/3 + 4 5/6", "6.5");
test_eval!(
    negative_mixed_fraction_sum,
    "1 2/3 + -4 5/6",
    "approx. -3.1666666666"
);
test_eval!(
    positive_mixed_fraction_subtraction,
    "1 2/3 - 4 5/6",
    "approx. -3.1666666666"
);
test_eval!(
    negative_mixed_fraction_subtraction,
    "1 2/3 - 4 + 5/6",
    "-1.5"
);
test_eval!(
    barn_to_meters_squared,
    "1 barn -> m^2",
    "0.0000000000000000000000000001 m^2"
);
test_eval!(liter_to_cubic_meters, "1L -> m^3", "0.001 m^3");
test_eval!(five_feet_to_meters, "5 ft to m", "1.524 m");
test_eval!(log10_4, "log10 4", "approx. 0.6020599913");

test_eval!(factorial_of_0, "0!", "1");
test_eval!(factorial_of_1, "1!", "1");
test_eval!(factorial_of_2, "2!", "2");
test_eval!(factorial_of_3, "3!", "6");
test_eval!(factorial_of_4, "4!", "24");
test_eval!(factorial_of_5, "5!", "120");
test_eval!(factorial_of_6, "6!", "720");
test_eval!(factorial_of_7, "7!", "5040");
test_eval!(factorial_of_8, "8!", "40320");
expect_error!(factorial_of_half, "0.5!");
expect_error!(factorial_of_minus_two, "(-2)!");
expect_error!(factorial_of_three_i, "3i!");
expect_error!(factorial_of_three_kg, "(3 kg)!");

test_eval_simple!(recurring_digits_1, "9/11 -> float", "0.(81)");
test_eval_simple!(recurring_digits_2, "6#1 / 11 -> float", "6#0.(0313452421)");
test_eval_simple!(recurring_digits_3, "6#0 + 6#1 / 7 -> float", "6#0.(05)");
test_eval_simple!(recurring_digits_4, "0.25 -> fraction", "1/4");
test_eval_simple!(recurring_digits_5, "0.21 -> 1 dp", "approx. 0.2");
test_eval_simple!(recurring_digits_6, "0.21 -> 1 dp -> auto", "0.21");
test_eval_simple!(recurring_digits_7, "502938/700 -> float", "718.48(285714)");

test_eval!(builtin_function_name_abs, "abs", "abs");
test_eval!(builtin_function_name_sin, "sin", "sin");
test_eval!(builtin_function_name_cos, "cos", "cos");
test_eval!(builtin_function_name_tan, "tan", "tan");
test_eval!(builtin_function_name_asin, "asin", "asin");
test_eval!(builtin_function_name_acos, "acos", "acos");
test_eval!(builtin_function_name_atan, "atan", "atan");
test_eval!(builtin_function_name_sinh, "sinh", "sinh");
test_eval!(builtin_function_name_cosh, "cosh", "cosh");
test_eval!(builtin_function_name_tanh, "tanh", "tanh");
test_eval!(builtin_function_name_asinh, "asinh", "asinh");
test_eval!(builtin_function_name_acosh, "acosh", "acosh");
test_eval!(builtin_function_name_atanh, "atanh", "atanh");
test_eval!(builtin_function_name_ln, "ln", "ln");
test_eval!(builtin_function_name_log2, "log2", "log2");
test_eval!(builtin_function_name_log10, "log10", "log10");
test_eval!(builtin_function_name_base, "base", "base");

// values from https://en.wikipedia.org/wiki/Trigonometric_constants_expressed_in_real_radicals#Table_of_some_common_angles
test_eval!(sin_0, "sin 0", "0");
test_eval!(sin_1, "sin 1", "approx. 0.8414709848");
test_eval!(sin_1m, "sin (1m)", "approx. 0.8414709848 m");
test_eval!(sin_pi, "sin pi", "0");
test_eval!(sin_2_pi, "sin (2pi)", "0");
test_eval!(sin_minus_pi, "sin (-pi)", "0");
test_eval!(sin_minus_1000_pi, "sin (-1000pi)", "0");
test_eval!(sin_pi_over_2, "sin (pi/2)", "1");
test_eval!(sin_3_pi_over_2, "sin (3pi/2)", "-1");
test_eval!(sin_5_pi_over_2, "sin (5pi/2)", "1");
test_eval!(sin_7_pi_over_2, "sin (7pi/2)", "-1");
test_eval!(sin_minus_pi_over_2, "sin (-pi/2)", "-1");
test_eval!(sin_minus_3_pi_over_2, "sin (-3pi/2)", "1");
test_eval!(sin_minus_5_pi_over_2, "sin (-5pi/2)", "-1");
test_eval!(sin_minus_7_pi_over_2, "sin (-7pi/2)", "1");
test_eval!(sin_minus_1023_pi_over_2, "sin (-1023pi/2)", "1");
test_eval!(sin_pi_over_6, "sin (pi/6)", "0.5");
test_eval!(sin_5_pi_over_6, "sin (5pi/6)", "0.5");
test_eval!(sin_7_pi_over_6, "sin (7pi/6)", "-0.5");
test_eval!(sin_11_pi_over_6, "sin (11pi/6)", "-0.5");
test_eval!(sin_minus_pi_over_6, "sin (-pi/6)", "-0.5");
test_eval!(sin_minus_5_pi_over_6, "sin (-5pi/6)", "-0.5");
test_eval!(sin_minus_7_pi_over_6, "sin (-7pi/6)", "0.5");
test_eval!(sin_minus_11_pi_over_6, "sin (-11pi/6)", "0.5");
test_eval!(sin_180_degrees, "sin (180°)", "0");
test_eval!(sin_30_degrees, "sin (30°)", "0.5");
test_eval!(sin_one_degree, "sin (1°)", "approx. 0.0174524064");

test_eval!(cos_0, "cos 0", "1");
test_eval!(cos_1, "cos 1", "approx. 0.5403023058");
test_eval!(cos_pi, "cos pi", "-1");
test_eval!(cos_2_pi, "cos (2pi)", "1");
test_eval!(cos_minus_pi, "cos (-pi)", "-1");
test_eval!(cos_minus_1000_pi, "cos (-1000pi)", "1");
test_eval!(cos_pi_over_2, "cos (pi/2)", "0");
test_eval!(cos_3_pi_over_2, "cos (3pi/2)", "0");
test_eval!(cos_5_pi_over_2, "cos (5pi/2)", "0");
test_eval!(cos_7_pi_over_2, "cos (7pi/2)", "0");
test_eval!(cos_minus_pi_over_2, "cos (-pi/2)", "0");
test_eval!(cos_minus_3_pi_over_2, "cos (-3pi/2)", "0");
test_eval!(cos_minus_5_pi_over_2, "cos (-5pi/2)", "0");
test_eval!(cos_minus_7_pi_over_2, "cos (-7pi/2)", "0");
test_eval!(cos_minus_1023_pi_over_2, "cos (-1023pi/2)", "0");
test_eval!(cos_pi_over_3, "cos (pi/3)", "0.5");
test_eval!(cos_2_pi_over_3, "cos (2pi/3)", "-0.5");
test_eval!(cos_4_pi_over_3, "cos (4pi/3)", "-0.5");
test_eval!(cos_5_pi_over_3, "cos (5pi/3)", "0.5");
test_eval!(cos_minus_pi_over_3, "cos (-pi/3)", "0.5");
test_eval!(cos_minus_2_pi_over_3, "cos (-2pi/3)", "-0.5");
test_eval!(cos_minus_4_pi_over_3, "cos (-4pi/3)", "-0.5");
test_eval!(cos_minus_5_pi_over_3, "cos (-5pi/3)", "0.5");

test_eval!(tau, "tau", "approx. 6.2831853071");
test_eval!(sin_tau_over_two, "sin (tau / 2)", "0");
test_eval!(greek_pi_symbol, "π", "approx. 3.1415926535");
test_eval!(greek_tau_symbol, "τ", "approx. 6.2831853071");

test_eval!(tan_0, "tan 0", "0");
test_eval!(tan_1m, "tan (1meter)", "approx. 1.5574077246 meters");
test_eval!(tan_pi, "tan pi", "0");
test_eval!(tan_2pi, "tan (2pi)", "0");
test_eval!(asin_1, "asin 1", "approx. 1.5707963267");
expect_error!(asin_3, "asin 3");
expect_error!(asin_minus_3, "asin (-3)");
expect_error!(asin_one_point_zero_one, "asin 1.01");
expect_error!(asin_minus_one_point_zero_one, "asin (-1.01)");

test_eval!(acos_0, "acos 0", "approx. 1.5707963267");
expect_error!(acos_3, "acos 3");
expect_error!(acos_minus_3, "acos (-3)");
expect_error!(acos_one_point_zero_one, "acos 1.01");
expect_error!(acos_minus_one_point_zero_one, "acos (-1.01)");
test_eval!(atan_1, "atan 1", "approx. 0.7853981633");
test_eval!(sinh_0, "sinh 0", "approx. 0");
test_eval!(cosh_0, "cosh 0", "approx. 1");
test_eval!(tanh_0, "tanh 0", "approx. 0");
test_eval!(asinh_0, "asinh 0", "approx. 0");
expect_error!(acosh_0, "acosh 0");
test_eval!(acosh_2, "acosh 2", "approx. 1.3169578969");
test_eval!(atanh_0, "atanh 0", "approx. 0");
expect_error!(atanh_3, "atanh 3");
expect_error!(atanh_minus_3, "atanh (-3)");
expect_error!(atanh_one_point_zero_one, "atanh 1.01");
expect_error!(atanh_minus_one_point_zero_one, "atanh (-1.01)");
expect_error!(atanh_1, "atanh 1");
expect_error!(atanh_minus_1, "atanh (-1)");
test_eval!(ln_2, "ln 2", "approx. 0.6931471805");
expect_error!(ln_0, "ln 0");
test_eval!(exp_2, "exp 2", "approx. 7.3890560989");
test_eval!(log10_100, "log10 100", "approx. 2");
test_eval!(log10_1000, "log10 1000", "approx. 3");
test_eval!(log10_10000, "log10 10000", "approx. 4");
test_eval!(log10_100000, "log10 100000", "approx. 5");
test_eval!(log2_65536, "log2 65536", "approx. 16");
expect_error!(log10_minus_1, "log10 (-1)");
expect_error!(log2_minus_1, "log2 (-1)");
expect_error!(sqrt_minus_two, "sqrt (-2)");
test_eval!(minus_two_cubed, "(-2)^3", "-8");
test_eval!(minus_two_pow_five, "(-2)^5", "-32");
test_eval!(two_pow_minus_two, "2^-2", "0.25");
test_eval!(minus_two_to_the_power_of_minus_two, "(-2)^-2", "0.25");
test_eval!(minus_two_to_the_power_of_minus_three, "(-2)^-3", "-0.125");
test_eval!(minus_two_to_the_power_of_minus_four, "(-2)^-4", "0.0625");
expect_error!(invalid_function_call, "oishfod 3");
test_eval!(ln, "ln", "ln");
test_eval!(dp, "dp", "dp");
test_eval!(ten_dp, "10 dp", "10 dp");
test_eval!(float, "float", "float");
test_eval!(fraction, "fraction", "fraction");
test_eval!(auto, "auto", "auto");
expect_error!(sqrt_i, "sqrt i");
expect_error!(sqrt_minus_two_i, "sqrt (-2i)");
expect_error!(cbrt_i, "cbrt i");
expect_error!(cbrt_minus_two_i, "cbrt (-2i)");
expect_error!(sin_i, "sin i");
expect_error!(dp_1, "dp 1");

test_eval!(unary_div_seconds, "/s", "1 s^-1");
test_eval!(per_second, "per second", "1 second^-1");
test_eval!(hertz_plus_unary_div_seconds, "1 Hz + /s", "2 Hz");

test_eval!(lambda_1, "(x: x) 1", "1");
test_eval!(lambda_2, "(x: y: x) 1 2", "1");
test_eval!(
    lambda_3,
    "(cis: (cis (pi/3))) (x: cos x + i * (sin x))",
    "approx. 0.5 + 0.8660254037i"
);
test_eval!(lambda_4, "(x: iuwhe)", "\\x.iuwhe");
test_eval!(lambda_5, "(b: 5 + b) 1", "6");
test_eval!(lambda_6, "(addFive: 4)(b: 5 + b)", "4");
test_eval!(lambda_7, "(addFive: addFive 4)(b: 5 + b)", "9");
test_eval!(lambda_8, "(x: y: z: x) 1 2 3", "1");
test_eval!(lambda_9, "(x: y: z: y) 1 2 3", "2");
test_eval!(lambda_10, "(x: y: z: z) 1 2 3", "3");
test_eval!(lambda_11, "(one: one + 4) 1", "5");
test_eval!(lambda_12, "(one: one + one) 1", "2");
test_eval!(lambda_13, "(x: x to kg) (5 g)", "0.005 kg");
test_eval!(lambda_14, "(p: q: p p q) (x: y: y) (x: y: y) 1 0", "0");
test_eval!(lambda_15, "(p: q: p p q) (x: y: y) (x: y: x) 1 0", "1");
test_eval!(lambda_16, "(p: q: p p q) (x: y: x) (x: y: y) 1 0", "1");
test_eval!(lambda_17, "(p: q: p p q) (x: y: x) (x: y: x) 1 0", "1");
test_eval!(lambda_18, "(x => x) 1", "1");
test_eval!(lambda_19, "(x: y => x) 1 2", "1");
test_eval!(lambda_20, "(\\x. y => x) 1 2", "1");
test_eval!(lambda_21, "(\\x.\\y.x)1 2", "1");
test_eval!(lambda_22, "a. => 0", "a.:0");

test_eval!(unit_to_the_power_of_pi, "kg^pi", "1 kg^π");
test_eval!(
    more_complex_unit_power_of_pi,
    "kg^(2pi) / kg^(2pi) to 1",
    "1"
);

test_eval!(cis_0, "cis 0", "1");
test_eval!(cis_pi, "cis pi", "-1");
test_eval!(cis_half_pi, "cis (pi/2)", "i");
test_eval!(cis_three_pi_over_two, "cis (3pi/2)", "-i");
test_eval!(cis_two_pi, "cis (2pi)", "1");
test_eval!(cis_minus_two_pi, "cis -(2pi)", "1");
test_eval!(cis_pi_over_six, "cis (pi/6)", "approx. 0.8660254037 + 0.5i");

test_eval!(name_one, "one", "1");
test_eval!(name_two, "two", "2");
test_eval!(name_pair, "pair", "2");
test_eval!(name_three, "three", "3");
test_eval!(name_four, "four", "4");
test_eval!(name_five, "five", "5");
test_eval!(name_six, "six", "6");
test_eval!(name_seven, "seven", "7");
test_eval!(name_eight, "eight", "8");
test_eval!(name_nine, "nine", "9");
test_eval!(name_ten, "ten", "10");
test_eval!(name_eleven, "eleven", "11");
test_eval!(name_twelve, "twelve", "12");
test_eval!(name_thirteen, "thirteen", "13");
test_eval!(name_fourteen, "fourteen", "14");
test_eval!(name_fifteen, "fifteen", "15");
test_eval!(name_sixteen, "sixteen", "16");
test_eval!(name_seventeen, "seventeen", "17");
test_eval!(name_eighteen, "eighteen", "18");
test_eval!(name_nineteen, "nineteen", "19");
test_eval!(name_twenty, "twenty", "20");
test_eval!(name_thirty, "thirty", "30");
test_eval!(name_forty, "forty", "40");
test_eval!(name_fifty, "fifty", "50");
test_eval!(name_sixty, "sixty", "60");
test_eval!(name_seventy, "seventy", "70");
test_eval!(name_eighty, "eighty", "80");
test_eval!(name_ninety, "ninety", "90");
test_eval!(name_hundred, "hundred", "100");
test_eval!(name_thousand, "thousand", "1000");
test_eval!(name_million, "million", "1000000");

test_eval!(name_dozen, "dozen", "12");
test_eval!(name_one_dozen, "one dozen", "12");
test_eval!(name_two_dozen, "two dozen", "24");
test_eval!(name_three_dozen, "three dozen", "36");
test_eval!(name_four_dozen, "four dozen", "48");
test_eval!(name_five_dozen, "five dozen", "60");
test_eval!(name_six_dozen, "six dozen", "72");
test_eval!(name_seven_dozen, "seven dozen", "84");
test_eval!(name_eight_dozen, "eight dozen", "96");
test_eval!(name_nine_dozen, "nine dozen", "108");
test_eval!(name_ten_dozen, "ten dozen", "120");
test_eval!(name_eleven_dozen, "eleven dozen", "132");
test_eval!(name_twelve_dozen, "twelve dozen", "144");
test_eval!(name_gross, "gross", "144");
test_eval!(name_thirteen_dozen, "thirteen dozen", "156");
test_eval!(name_fourteen_dozen, "fourteen dozen", "168");
test_eval!(name_fifteen_dozen, "fifteen dozen", "180");
test_eval!(name_sixteen_dozen, "sixteen dozen", "192");
test_eval!(name_seventeen_dozen, "seventeen dozen", "204");
test_eval!(name_eighteen_dozen, "eighteen dozen", "216");
test_eval!(name_nineteen_dozen, "nineteen dozen", "228");
test_eval!(name_twenty_dozen, "twenty dozen", "240");
test_eval!(name_thirty_dozen, "thirty dozen", "360");
test_eval!(name_forty_dozen, "forty dozen", "480");
test_eval!(name_fifty_dozen, "fifty dozen", "600");
test_eval!(name_sixty_dozen, "sixty dozen", "720");
test_eval!(name_seventy_dozen, "seventy dozen", "840");
test_eval!(name_eighty_dozen, "eighty dozen", "960");
test_eval!(name_ninety_dozen, "ninety dozen", "1080");
test_eval!(name_hundred_dozen, "hundred dozen", "1200");
test_eval!(name_thousand_dozen, "thousand dozen", "12000");
test_eval!(name_million_dozen, "million dozen", "12000000");

test_eval!(lone_prefix_yotta, "yotta", "1000000000000000000000000");
test_eval!(lone_prefix_zetta, "zetta", "1000000000000000000000");
test_eval!(lone_prefix_exa, "exa", "1000000000000000000");
test_eval!(lone_prefix_peta, "peta", "1000000000000000");
test_eval!(lone_prefix_tera, "tera", "1000000000000");
test_eval!(lone_prefix_giga, "giga", "1000000000");
test_eval!(lone_prefix_mega, "mega", "1000000");
test_eval!(lone_prefix_myria, "myria", "10000");
test_eval!(lone_prefix_kilo, "kilo", "1000");
test_eval!(lone_prefix_hecto, "hecto", "100");
test_eval!(lone_prefix_deca, "deca", "10");
test_eval!(lone_prefix_deka, "deka", "10");
test_eval!(lone_prefix_deci, "deci", "0.1");
test_eval!(lone_prefix_centi, "centi", "0.01");
test_eval!(lone_prefix_milli, "milli", "0.001");
test_eval!(lone_prefix_micro, "micro", "0.000001");
test_eval!(lone_prefix_nano, "nano", "0.000000001");
test_eval!(lone_prefix_pico, "pico", "0.000000000001");
test_eval!(lone_prefix_femto, "femto", "0.000000000000001");
test_eval!(lone_prefix_atto, "atto", "0.000000000000000001");
test_eval!(lone_prefix_zepto, "zepto", "0.000000000000000000001");
test_eval!(lone_prefix_yocto, "yocto", "0.000000000000000000000001");

test_eval!(billion, "billion", "1000000000");
test_eval!(trillion, "trillion", "1000000000000");
test_eval!(quadrillion, "quadrillion", "1000000000000000");
test_eval!(quintillion, "quintillion", "1000000000000000000");
test_eval!(sextillion, "sextillion", "1000000000000000000000");
test_eval!(septillion, "septillion", "1000000000000000000000000");
test_eval!(octillion, "octillion", "1000000000000000000000000000");
test_eval!(nonillion, "nonillion", "1000000000000000000000000000000");
test_eval!(
    noventillion,
    "noventillion",
    "1000000000000000000000000000000"
);
test_eval!(decillion, "decillion", "1000000000000000000000000000000000");
test_eval!(
    undecillion,
    "undecillion",
    "1000000000000000000000000000000000000"
);
test_eval!(
    duodecillion,
    "duodecillion",
    "1000000000000000000000000000000000000000"
);
test_eval!(
    tredecillion,
    "tredecillion",
    "1000000000000000000000000000000000000000000"
);
test_eval!(
    quattuordecillion,
    "quattuordecillion",
    "1000000000000000000000000000000000000000000000"
);
test_eval!(
    quindecillion,
    "quindecillion",
    "1000000000000000000000000000000000000000000000000"
);
test_eval!(
    sexdecillion,
    "sexdecillion",
    "1000000000000000000000000000000000000000000000000000"
);
test_eval!(
    septendecillion,
    "septendecillion",
    "1000000000000000000000000000000000000000000000000000000"
);
test_eval!(
    octodecillion,
    "octodecillion",
    "1000000000000000000000000000000000000000000000000000000000"
);
test_eval!(
    novemdecillion,
    "novemdecillion",
    "1000000000000000000000000000000000000000000000000000000000000"
);
test_eval!(
    vigintillion,
    "vigintillion",
    "1000000000000000000000000000000000000000000000000000000000000000"
);

test_eval!(one_cent, "cent", "1 cent");
test_eval!(two_cent, "2 cent", "2 cents");
expect_error!(to_dp, "1 to dp");
expect_error!(to_sf, "1 to sf");

test_eval!(sf, "sf", "sf");
test_eval!(one_sf, "1 sf", "1 sf");
test_eval!(ten_sf, "10 sf", "10 sf");

test_eval_simple!(one_over_sin, "1/sin", "\\x.(1/(sin x))");

expect_error!(zero_sf, "0 sf");
test_eval!(sf_1, "1234567.55645 to 1 sf", "approx. 1000000");
test_eval!(sf_2, "1234567.55645 to 2 sf", "approx. 1200000");
test_eval!(sf_3, "1234567.55645 to 3 sf", "approx. 1230000");
test_eval!(sf_4, "1234567.55645 to 4 sf", "approx. 1234000");
test_eval!(sf_5, "1234567.55645 to 5 sf", "approx. 1234500");
test_eval!(sf_6, "1234567.55645 to 6 sf", "approx. 1234560");
test_eval!(sf_7, "1234567.55645 to 7 sf", "approx. 1234567");
test_eval!(sf_8, "1234567.55645 to 8 sf", "approx. 1234567.5");
test_eval!(sf_9, "1234567.55645 to 9 sf", "approx. 1234567.55");
test_eval!(sf_10, "1234567.55645 to 10 sf", "approx. 1234567.556");
test_eval!(sf_11, "1234567.55645 to 11 sf", "approx. 1234567.5564");
test_eval!(sf_12, "1234567.55645 to 12 sf", "1234567.55645");
test_eval!(sf_13, "1234567.55645 to 13 sf", "1234567.55645");
test_eval_simple!(sf_small_1, "pi / 1000000 to 1 sf", "approx. 0.000003");
test_eval_simple!(sf_small_2, "pi / 1000000 to 2 sf", "approx. 0.0000031");
test_eval_simple!(sf_small_3, "pi / 1000000 to 3 sf", "approx. 0.00000314");
test_eval_simple!(sf_small_4, "pi / 1000000 to 4 sf", "approx. 0.000003141");
test_eval_simple!(sf_small_5, "pi / 1000000 to 5 sf", "approx. 0.0000031415");
test_eval_simple!(sf_small_6, "pi / 1000000 to 6 sf", "approx. 0.00000314159");
test_eval_simple!(sf_small_7, "pi / 1000000 to 7 sf", "approx. 0.000003141592");
test_eval_simple!(
    sf_small_8,
    "pi / 1000000 to 8 sf",
    "approx. 0.0000031415926"
);
test_eval_simple!(
    sf_small_9,
    "pi / 1000000 to 9 sf",
    "approx. 0.00000314159265"
);
test_eval_simple!(
    sf_small_10,
    "pi / 1000000 to 10 sf",
    "approx. 0.000003141592653"
);
test_eval_simple!(
    sf_small_11,
    "pi / 1000000 to 11 sf",
    "approx. 0.0000031415926535"
);

expect_error!(no_prefixes_for_speed_of_light, "mc");

test_eval!(quarter, "quarter", "0.25");

test_eval_simple!(million_pi_1_sf, "1e6 pi to 1 sf", "approx. 3000000");
test_eval_simple!(million_pi_2_sf, "1e6 pi to 2 sf", "approx. 3100000");
test_eval_simple!(million_pi_3_sf, "1e6 pi to 3 sf", "approx. 3140000");
test_eval_simple!(million_pi_4_sf, "1e6 pi to 4 sf", "approx. 3141000");
test_eval_simple!(million_pi_5_sf, "1e6 pi to 5 sf", "approx. 3141500");
test_eval_simple!(million_pi_6_sf, "1e6 pi to 6 sf", "approx. 3141590");
test_eval_simple!(million_pi_7_sf, "1e6 pi to 7 sf", "approx. 3141592");
test_eval_simple!(million_pi_8_sf, "1e6 pi to 8 sf", "approx. 3141592.6");
test_eval_simple!(million_pi_9_sf, "1e6 pi to 9 sf", "approx. 3141592.65");
test_eval_simple!(million_pi_10_sf, "1e6 pi to 10 sf", "approx. 3141592.653");

test_eval_simple!(large_integer_to_1_sf, "1234567 to 1 sf", "approx. 1000000");
test_eval_simple!(large_integer_to_2_sf, "1234567 to 2 sf", "approx. 1200000");
test_eval_simple!(large_integer_to_3_sf, "1234567 to 3 sf", "approx. 1230000");
test_eval_simple!(large_integer_to_4_sf, "1234567 to 4 sf", "approx. 1234000");
test_eval_simple!(large_integer_to_5_sf, "1234567 to 5 sf", "approx. 1234500");
test_eval_simple!(large_integer_to_6_sf, "1234567 to 6 sf", "approx. 1234560");
test_eval_simple!(large_integer_to_7_sf, "1234567 to 7 sf", "1234567");
test_eval_simple!(large_integer_to_8_sf, "1234567 to 8 sf", "1234567");
test_eval_simple!(large_integer_to_9_sf, "1234567 to 9 sf", "1234567");
test_eval_simple!(large_integer_to_10_sf, "1234567 to 10 sf", "1234567");

test_eval_simple!(trailing_zeroes_sf_1, "1234560 to 5sf", "approx. 1234500");
test_eval_simple!(trailing_zeroes_sf_2, "1234560 to 6sf", "1234560");
test_eval_simple!(trailing_zeroes_sf_3, "1234560 to 7sf", "1234560");
test_eval_simple!(trailing_zeroes_sf_4, "1234560.1 to 6sf", "approx. 1234560");
test_eval_simple!(trailing_zeroes_sf_5, "12345601 to 6sf", "approx. 12345600");
test_eval_simple!(trailing_zeroes_sf_6, "12345601 to 7sf", "approx. 12345600");
test_eval_simple!(trailing_zeroes_sf_7, "12345601 to 8sf", "12345601");

test_eval!(
    kwh_conversion,
    "100 kWh/yr to watt",
    "approx. 11.4079552707 watts"
);
