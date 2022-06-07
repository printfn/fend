use crate::error::{FendError, Interrupt};
use crate::format::Format;
use crate::interrupt::test_int;
use crate::num::biguint::BigUint;
use crate::num::{Base, Exact, FormattingStyle, Range, RangeBound};
use std::{cmp, fmt, hash, io, ops};

mod sign {
    use crate::{
        error::FendError,
        serialize::{deserialize_u8, serialize_u8},
    };
    use std::io;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) enum Sign {
        Positive,
        Negative,
    }

    impl Sign {
        pub(crate) const fn flip(self) -> Self {
            match self {
                Self::Positive => Self::Negative,
                Self::Negative => Self::Positive,
            }
        }

        pub(crate) const fn sign_of_product(a: Self, b: Self) -> Self {
            match (a, b) {
                (Self::Positive, Self::Positive) | (Self::Negative, Self::Negative) => {
                    Self::Positive
                }
                (Self::Positive, Self::Negative) | (Self::Negative, Self::Positive) => {
                    Self::Negative
                }
            }
        }

        pub(crate) fn serialize(self, write: &mut impl io::Write) -> Result<(), FendError> {
            Ok(serialize_u8(
                match self {
                    Self::Positive => 1,
                    Self::Negative => 2,
                },
                write,
            )?)
        }

        pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
            Ok(match deserialize_u8(read)? {
                1 => Self::Positive,
                2 => Self::Negative,
                _ => return Err(FendError::DeserializationError),
            })
        }
    }
}

use super::biguint::{self, FormattedBigUint};
use super::out_of_range;
use sign::Sign;

#[derive(Clone)]
pub(crate) struct BigRat {
    sign: Sign,
    num: BigUint,
    den: BigUint,
}

impl fmt::Debug for BigRat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sign == Sign::Negative {
            write!(f, "-")?;
        }
        write!(f, "{:?}", self.num)?;
        if !self.den.is_definitely_one() {
            write!(f, "/{:?}", self.den)?;
        }
        Ok(())
    }
}

impl Ord for BigRat {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let int = &crate::interrupt::Never::default();
        let diff = self.clone().add(-other.clone(), int).unwrap();
        if diff.num == 0.into() {
            cmp::Ordering::Equal
        } else if diff.sign == Sign::Positive {
            cmp::Ordering::Greater
        } else {
            cmp::Ordering::Less
        }
    }
}

impl PartialOrd for BigRat {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BigRat {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl Eq for BigRat {}

impl hash::Hash for BigRat {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let int = &crate::interrupt::Never::default();
        if let Ok(res) = self.clone().simplify(int) {
            // don't hash the sign
            res.num.hash(state);
            res.den.hash(state);
        }
    }
}

impl BigRat {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        self.sign.serialize(write)?;
        self.num.serialize(write)?;
        self.den.serialize(write)?;
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(Self {
            sign: Sign::deserialize(read)?,
            num: BigUint::deserialize(read)?,
            den: BigUint::deserialize(read)?,
        })
    }

    pub(crate) fn try_as_usize<I: Interrupt>(mut self, int: &I) -> Result<usize, FendError> {
        if self.sign == Sign::Negative && self.num != 0.into() {
            return Err(FendError::NegativeNumbersNotAllowed);
        }
        self = self.simplify(int)?;
        if self.den != 1.into() {
            return Err(FendError::FractionToInteger);
        }
        self.num.try_as_usize(int)
    }

    #[allow(clippy::float_arithmetic)]
    pub(crate) fn into_f64<I: Interrupt>(mut self, int: &I) -> Result<f64, FendError> {
        self = self.simplify(int)?;
        let positive_result = self.num.as_f64() / self.den.as_f64();
        if self.sign == Sign::Positive {
            Ok(positive_result)
        } else {
            Ok(-positive_result)
        }
    }

    #[allow(
        clippy::as_conversions,
        clippy::float_arithmetic,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub(crate) fn from_f64<I: Interrupt>(mut f: f64, int: &I) -> Result<Self, FendError> {
        let negative = f < 0.0;
        if negative {
            f = -f;
        }
        let i = (f * u64::MAX as f64) as u128;
        let part1 = i as u64;
        let part2 = (i >> 64) as u64;
        Ok(Self {
            sign: if negative {
                Sign::Negative
            } else {
                Sign::Positive
            },
            num: BigUint::from(part1)
                .add(&BigUint::from(part2).mul(&BigUint::from(u64::MAX), int)?),
            den: BigUint::from(u64::MAX),
        })
    }

    // sin works for all real numbers
    pub(crate) fn sin<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, FendError> {
        Ok(if self == 0.into() {
            Exact::new(Self::from(0), true)
        } else {
            Exact::new(Self::from_f64(f64::sin(self.into_f64(int)?), int)?, false)
        })
    }

    // asin, acos and atan only work for values between -1 and 1
    pub(crate) fn asin<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        let one = Self::from(1);
        if self > one || self < -one {
            return Err(out_of_range(self.fm(int)?, Range::open(-1, 1)));
        }
        Self::from_f64(f64::asin(self.into_f64(int)?), int)
    }

    pub(crate) fn acos<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        let one = Self::from(1);
        if self > one || self < -one {
            return Err(out_of_range(self.fm(int)?, Range::open(-1, 1)));
        }
        Self::from_f64(f64::acos(self.into_f64(int)?), int)
    }

    // note that this works for any real number, unlike asin and acos
    pub(crate) fn atan<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Self::from_f64(f64::atan(self.into_f64(int)?), int)
    }

    pub(crate) fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Self::from_f64(f64::sinh(self.into_f64(int)?), int)
    }

    pub(crate) fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Self::from_f64(f64::cosh(self.into_f64(int)?), int)
    }

    pub(crate) fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Self::from_f64(f64::tanh(self.into_f64(int)?), int)
    }

    pub(crate) fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Self::from_f64(f64::asinh(self.into_f64(int)?), int)
    }

    // value must not be less than 1
    pub(crate) fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        if self < 1.into() {
            return Err(out_of_range(
                self.fm(int)?,
                Range {
                    start: RangeBound::Closed(1),
                    end: RangeBound::None,
                },
            ));
        }
        Self::from_f64(f64::acosh(self.into_f64(int)?), int)
    }

    // value must be between -1 and 1.
    pub(crate) fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        let one: Self = 1.into();
        if self >= one || self <= -one {
            return Err(out_of_range(self.fm(int)?, Range::open(-1, 1)));
        }
        Self::from_f64(f64::atanh(self.into_f64(int)?), int)
    }

    // For all logs: value must be greater than 0
    pub(crate) fn ln<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        if self <= 0.into() {
            return Err(out_of_range(
                self.fm(int)?,
                Range {
                    start: RangeBound::Open(0),
                    end: RangeBound::None,
                },
            ));
        }
        Self::from_f64(f64::ln(self.into_f64(int)?), int)
    }

    pub(crate) fn log2<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        if self <= 0.into() {
            return Err(out_of_range(
                self.fm(int)?,
                Range {
                    start: RangeBound::Open(0),
                    end: RangeBound::None,
                },
            ));
        }
        Self::from_f64(f64::log2(self.into_f64(int)?), int)
    }

    pub(crate) fn log10<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        if self <= 0.into() {
            return Err(out_of_range(
                self.fm(int)?,
                Range {
                    start: RangeBound::Open(0),
                    end: RangeBound::None,
                },
            ));
        }
        Self::from_f64(f64::log10(self.into_f64(int)?), int)
    }

    pub(crate) fn factorial<I: Interrupt>(mut self, int: &I) -> Result<Self, FendError> {
        self = self.simplify(int)?;
        if self.den != 1.into() {
            let n = self.fm(int)?;
            return Err(FendError::MustBeAnInteger(Box::new(n)));
        }
        if self.sign == Sign::Negative && self.num != 0.into() {
            return Err(out_of_range(self.fm(int)?, Range::ZERO_OR_GREATER));
        }
        Ok(Self {
            sign: Sign::Positive,
            num: self.num.factorial(int)?,
            den: self.den,
        })
    }

    /// compute a + b
    fn add_internal<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        // a + b == -((-a) + (-b))
        if self.sign == Sign::Negative {
            return Ok(-((-self).add_internal(-rhs, int)?));
        }

        assert_eq!(self.sign, Sign::Positive);

        Ok(if self.den == rhs.den {
            if rhs.sign == Sign::Negative && self.num < rhs.num {
                Self {
                    sign: Sign::Negative,
                    num: rhs.num.sub(&self.num),
                    den: self.den,
                }
            } else {
                Self {
                    sign: Sign::Positive,
                    num: if rhs.sign == Sign::Positive {
                        self.num.add(&rhs.num)
                    } else {
                        self.num.sub(&rhs.num)
                    },
                    den: self.den,
                }
            }
        } else {
            let gcd = BigUint::gcd(self.den.clone(), rhs.den.clone(), int)?;
            let new_denominator = self.den.clone().mul(&rhs.den, int)?.div(&gcd, int)?;
            let a = self.num.mul(&rhs.den, int)?.div(&gcd, int)?;
            let b = rhs.num.mul(&self.den, int)?.div(&gcd, int)?;

            if rhs.sign == Sign::Negative && a < b {
                Self {
                    sign: Sign::Negative,
                    num: b.sub(&a),
                    den: new_denominator,
                }
            } else {
                Self {
                    sign: Sign::Positive,
                    num: if rhs.sign == Sign::Positive {
                        a.add(&b)
                    } else {
                        a.sub(&b)
                    },
                    den: new_denominator,
                }
            }
        })
    }

    fn simplify<I: Interrupt>(mut self, int: &I) -> Result<Self, FendError> {
        if self.den == 1.into() {
            return Ok(self);
        }
        let gcd = BigUint::gcd(self.num.clone(), self.den.clone(), int)?;
        self.num = self.num.div(&gcd, int)?;
        self.den = self.den.div(&gcd, int)?;
        Ok(self)
    }

    pub(crate) fn div<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, FendError> {
        if rhs.num == 0.into() {
            return Err(FendError::DivideByZero);
        }
        Ok(Self {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num.mul(&rhs.den, int)?,
            den: self.den.mul(&rhs.num, int)?,
        })
    }

    pub(crate) fn modulo<I: Interrupt>(
        mut self,
        mut rhs: Self,
        int: &I,
    ) -> Result<Self, FendError> {
        if rhs.num == 0.into() {
            return Err(FendError::ModuloByZero);
        }
        self = self.simplify(int)?;
        rhs = rhs.simplify(int)?;
        if (self.sign == Sign::Negative && self.num != 0.into())
            || rhs.sign == Sign::Negative
            || self.den != 1.into()
            || rhs.den != 1.into()
        {
            return Err(FendError::ModuloForPositiveInts);
        }
        Ok(Self {
            sign: Sign::Positive,
            num: self.num.divmod(&rhs.num, int)?.1,
            den: 1.into(),
        })
    }

    // test if this fraction has a terminating representation
    // e.g. in base 10: 1/4 = 0.25, but not 1/3
    fn terminates_in_base<I: Interrupt>(&self, base: Base, int: &I) -> Result<bool, FendError> {
        let mut x = self.clone();
        let base_as_u64: u64 = base.base_as_u8().into();
        let base = Self {
            sign: Sign::Positive,
            num: base_as_u64.into(),
            den: 1.into(),
        };
        loop {
            let old_den = x.den.clone();
            x = x.mul(&base, int)?.simplify(int)?;
            let new_den = x.den.clone();
            if new_den == old_den {
                break;
            }
        }
        Ok(x.den == 1.into())
    }

    fn format_as_integer<I: Interrupt>(
        num: &BigUint,
        base: Base,
        sign: Sign,
        term: &'static str,
        use_parens_if_product: bool,
        sf_limit: Option<usize>,
        int: &I,
    ) -> Result<Exact<FormattedBigRat>, FendError> {
        let (ty, exact) = if !term.is_empty() && !base.has_prefix() && num == &1.into() {
            (FormattedBigRatType::Integer(None, false, term, false), true)
        } else {
            let formatted_int = num.format(
                &biguint::FormatOptions {
                    base,
                    write_base_prefix: true,
                    sf_limit,
                },
                int,
            )?;
            (
                FormattedBigRatType::Integer(
                    Some(formatted_int.value),
                    !term.is_empty() && base.base_as_u8() > 10,
                    term,
                    // print surrounding parentheses if the number is imaginary
                    use_parens_if_product && !term.is_empty(),
                ),
                formatted_int.exact,
            )
        };
        Ok(Exact::new(FormattedBigRat { sign, ty }, exact))
    }

    fn format_as_fraction<I: Interrupt>(
        &self,
        base: Base,
        sign: Sign,
        term: &'static str,
        mixed: bool,
        use_parens: bool,
        int: &I,
    ) -> Result<Exact<FormattedBigRat>, FendError> {
        let format_options = biguint::FormatOptions {
            base,
            write_base_prefix: true,
            sf_limit: None,
        };
        let formatted_den = self.den.format(&format_options, int)?;
        let (pref, num, prefix_exact) = if mixed {
            let (prefix, num) = self.num.divmod(&self.den, int)?;
            if prefix == 0.into() {
                (None, num, true)
            } else {
                let formatted_prefix = prefix.format(&format_options, int)?;
                (Some(formatted_prefix.value), num, formatted_prefix.exact)
            }
        } else {
            (None, self.num.clone(), true)
        };
        // mixed fractions without a prefix aren't really mixed
        let actually_mixed = pref.is_some();
        let (ty, num_exact) =
            if !term.is_empty() && !actually_mixed && !base.has_prefix() && num == 1.into() {
                (
                    FormattedBigRatType::Fraction(
                        pref,
                        None,
                        false,
                        term,
                        formatted_den.value,
                        "",
                        use_parens,
                    ),
                    true,
                )
            } else {
                let formatted_num = num.format(&format_options, int)?;
                let i_suffix = term;
                let space = !term.is_empty() && (base.base_as_u8() >= 19 || actually_mixed);
                let (isuf1, isuf2) = if actually_mixed {
                    ("", i_suffix)
                } else {
                    (i_suffix, "")
                };
                (
                    FormattedBigRatType::Fraction(
                        pref,
                        Some(formatted_num.value),
                        space,
                        isuf1,
                        formatted_den.value,
                        isuf2,
                        use_parens,
                    ),
                    formatted_num.exact,
                )
            };
        Ok(Exact::new(
            FormattedBigRat { sign, ty },
            formatted_den.exact && prefix_exact && num_exact,
        ))
    }

    fn format_as_decimal<I: Interrupt>(
        &self,
        style: FormattingStyle,
        base: Base,
        sign: Sign,
        term: &'static str,
        mut terminating: impl FnMut() -> Result<bool, FendError>,
        int: &I,
    ) -> Result<Exact<FormattedBigRat>, FendError> {
        let integer_part = self.clone().num.div(&self.den, int)?;
        let sf_limit = if let FormattingStyle::SignificantFigures(sf) = style {
            Some(sf)
        } else {
            None
        };
        let formatted_integer_part = integer_part.format(
            &biguint::FormatOptions {
                base,
                write_base_prefix: true,
                sf_limit,
            },
            int,
        )?;

        let num_trailing_digits_to_print = if style == FormattingStyle::ExactFloat
            || (style == FormattingStyle::Auto && terminating()?)
            || style == FormattingStyle::Exact
        {
            MaxDigitsToPrint::AllDigits
        } else if let FormattingStyle::DecimalPlaces(n) = style {
            MaxDigitsToPrint::DecimalPlaces(n)
        } else if let FormattingStyle::SignificantFigures(sf) = style {
            let num_digits_of_int_part = formatted_integer_part.value.num_digits();
            let dp = if sf > num_digits_of_int_part {
                // we want more significant figures than what was printed
                // in the int component
                sf - num_digits_of_int_part
            } else {
                // no more digits, we already exhausted the number of significant
                // figures
                0
            };
            if integer_part == 0.into() {
                // if the integer part is 0, we don't want leading zeroes
                // after the decimal point to affect the number of non-zero
                // digits printed

                // we add 1 to the number of decimal places in this case because
                // the integer component of '0' shouldn't count against the
                // number of significant figures
                MaxDigitsToPrint::DpButIgnoreLeadingZeroes(dp + 1)
            } else {
                MaxDigitsToPrint::DecimalPlaces(dp)
            }
        } else {
            MaxDigitsToPrint::DecimalPlaces(10)
        };
        let print_integer_part = |ignore_minus_if_zero: bool| {
            let sign =
                if sign == Sign::Negative && (!ignore_minus_if_zero || integer_part != 0.into()) {
                    Sign::Negative
                } else {
                    Sign::Positive
                };
            Ok((sign, formatted_integer_part.value.to_string()))
        };
        let integer_as_rational = Self {
            sign: Sign::Positive,
            num: integer_part.clone(),
            den: 1.into(),
        };
        let remaining_fraction = self.clone().add(-integer_as_rational, int)?;
        let (sign, formatted_trailing_digits) = Self::format_trailing_digits(
            base,
            &remaining_fraction.num,
            &remaining_fraction.den,
            num_trailing_digits_to_print,
            terminating,
            print_integer_part,
            int,
        )?;
        Ok(Exact::new(
            FormattedBigRat {
                sign,
                ty: FormattedBigRatType::Decimal(
                    formatted_trailing_digits.value,
                    !term.is_empty() && base.base_as_u8() > 10,
                    term,
                ),
            },
            formatted_integer_part.exact && formatted_trailing_digits.exact,
        ))
    }

    /// Prints the decimal expansion of num/den, where num < den, in the given base.
    fn format_trailing_digits<I: Interrupt>(
        base: Base,
        numerator: &BigUint,
        denominator: &BigUint,
        max_digits: MaxDigitsToPrint,
        mut terminating: impl FnMut() -> Result<bool, FendError>,
        print_integer_part: impl Fn(bool) -> Result<(Sign, String), FendError>,
        int: &I,
    ) -> Result<(Sign, Exact<String>), FendError> {
        let base_as_u64: u64 = base.base_as_u8().into();
        let b: BigUint = base_as_u64.into();
        let next_digit =
            |i: usize, num: BigUint, base: &BigUint| -> Result<(BigUint, BigUint), NextDigitErr> {
                test_int(int)?;
                if num == 0.into()
                    || max_digits == MaxDigitsToPrint::DecimalPlaces(i)
                    || max_digits == MaxDigitsToPrint::DpButIgnoreLeadingZeroes(i)
                {
                    return Err(NextDigitErr::Terminated);
                }
                // digit = base * numerator / denominator
                // next_numerator = base * numerator - digit * denominator
                let bnum = num.mul(base, int)?;
                let digit = bnum.clone().div(denominator, int)?;
                let next_num = bnum.sub(&digit.clone().mul(denominator, int)?);
                Ok((next_num, digit))
            };
        let fold_digits = |mut s: String, digit: BigUint| -> Result<String, FendError> {
            let digit_str = digit
                .format(
                    &biguint::FormatOptions {
                        base,
                        write_base_prefix: false,
                        sf_limit: None,
                    },
                    int,
                )?
                .value
                .to_string();
            s.push_str(digit_str.as_str());
            Ok(s)
        };
        let skip_cycle_detection = max_digits != MaxDigitsToPrint::AllDigits || terminating()?;
        if skip_cycle_detection {
            let ignore_number_of_leading_zeroes =
                matches!(max_digits, MaxDigitsToPrint::DpButIgnoreLeadingZeroes(_));
            return Self::format_nonrecurring(
                numerator,
                base,
                ignore_number_of_leading_zeroes,
                next_digit,
                print_integer_part,
                int,
            );
        }
        match Self::brents_algorithm(
            next_digit,
            fold_digits,
            numerator.clone(),
            &b,
            String::new(),
        ) {
            Ok((cycle_length, location, output)) => {
                let (ab, _) = output.split_at(location + cycle_length);
                let (a, b) = ab.split_at(location);
                let (sign, formatted_int) = print_integer_part(false)?;
                let mut trailing_digits = String::new();
                trailing_digits.push_str(&formatted_int);
                trailing_digits.push('.');
                trailing_digits.push_str(a);
                trailing_digits.push('(');
                trailing_digits.push_str(b);
                trailing_digits.push(')');
                Ok((sign, Exact::new(trailing_digits, true))) // the recurring decimal is exact
            }
            Err(NextDigitErr::Terminated) => {
                panic!("decimal number terminated unexpectedly");
            }
            Err(NextDigitErr::Error(e)) => Err(e),
        }
    }

    fn format_nonrecurring<I: Interrupt>(
        numerator: &BigUint,
        base: Base,
        ignore_number_of_leading_zeroes: bool,
        mut next_digit: impl FnMut(usize, BigUint, &BigUint) -> Result<(BigUint, BigUint), NextDigitErr>,
        print_integer_part: impl Fn(bool) -> Result<(Sign, String), FendError>,
        int: &I,
    ) -> Result<(Sign, Exact<String>), FendError> {
        let mut current_numerator = numerator.clone();
        let mut i = 0;
        let mut trailing_zeroes = 0;
        // this becomes Some(_) when we write the decimal point
        let mut actual_sign = None;
        let mut trailing_digits = String::new();
        let b: BigUint = u64::from(base.base_as_u8()).into();
        loop {
            match next_digit(i, current_numerator.clone(), &b) {
                Ok((next_n, digit)) => {
                    current_numerator = next_n;
                    if digit == 0.into() {
                        trailing_zeroes += 1;
                        if !(i == 0 && ignore_number_of_leading_zeroes) {
                            i += 1;
                        }
                    } else {
                        if actual_sign.is_none() {
                            // always print leading minus because we have non-zero digits
                            let (sign, formatted_int) = print_integer_part(false)?;
                            actual_sign = Some(sign);
                            trailing_digits.push_str(&formatted_int);
                            trailing_digits.push('.');
                        }
                        for _ in 0..trailing_zeroes {
                            trailing_digits.push('0');
                        }
                        trailing_zeroes = 0;
                        trailing_digits.push_str(
                            &digit
                                .format(
                                    &biguint::FormatOptions {
                                        base,
                                        write_base_prefix: false,
                                        sf_limit: None,
                                    },
                                    int,
                                )?
                                .value
                                .to_string(),
                        );
                        i += 1;
                    }
                }
                Err(NextDigitErr::Terminated) => {
                    let sign = if let Some(actual_sign) = actual_sign {
                        actual_sign
                    } else {
                        // if we reach this point we haven't printed any non-zero digits,
                        // so we can skip the leading minus sign if the integer part is also zero
                        let (sign, formatted_int) = print_integer_part(true)?;
                        trailing_digits.push_str(&formatted_int);
                        sign
                    };
                    // is the number exact, or did we need to truncate?
                    let exact = current_numerator == 0.into();
                    return Ok((sign, Exact::new(trailing_digits, exact)));
                }
                Err(NextDigitErr::Error(e)) => {
                    return Err(e);
                }
            }
        }
    }

    // Brent's cycle detection algorithm (based on pseudocode from Wikipedia)
    // returns (length of cycle, index of first element of cycle, collected result)
    fn brents_algorithm<T: Clone + Eq, R, U, E1: From<E2>, E2>(
        f: impl Fn(usize, T, &T) -> Result<(T, U), E1>,
        g: impl Fn(R, U) -> Result<R, E2>,
        x0: T,
        state: &T,
        r0: R,
    ) -> Result<(usize, usize, R), E1> {
        // main phase: search successive powers of two
        let mut power = 1;
        // lam is the length of the cycle
        let mut lam = 1;
        let mut tortoise = x0.clone();
        let mut depth = 0;
        let (mut hare, _) = f(depth, x0.clone(), state)?;
        depth += 1;
        while tortoise != hare {
            if power == lam {
                tortoise = hare.clone();
                power *= 2;
                lam = 0;
            }
            hare = f(depth, hare, state)?.0;
            depth += 1;
            lam += 1;
        }

        // Find the position of the first repetition of length lam
        tortoise = x0.clone();
        hare = x0;
        let mut collected_res = r0;
        let mut hare_depth = 0;
        for _ in 0..lam {
            let (new_hare, u) = f(hare_depth, hare, state)?;
            hare_depth += 1;
            hare = new_hare;
            collected_res = g(collected_res, u)?;
        }
        // The distance between the hare and tortoise is now lam.

        // Next, the hare and tortoise move at same speed until they agree
        // mu will be the length of the initial sequence, before the cycle
        let mut mu = 0;
        let mut tortoise_depth = 0;
        while tortoise != hare {
            tortoise = f(tortoise_depth, tortoise, state)?.0;
            tortoise_depth += 1;
            let (new_hare, u) = f(hare_depth, hare, state)?;
            hare_depth += 1;
            hare = new_hare;
            collected_res = g(collected_res, u)?;
            mu += 1;
        }
        Ok((lam, mu, collected_res))
    }

    pub(crate) fn pow<I: Interrupt>(
        mut self,
        mut rhs: Self,
        int: &I,
    ) -> Result<Exact<Self>, FendError> {
        self = self.simplify(int)?;
        rhs = rhs.simplify(int)?;
        if self.num != 0.into() && self.sign == Sign::Negative && rhs.den != 1.into() {
            return Err(FendError::RootsOfNegativeNumbers);
        }
        if rhs.sign == Sign::Negative {
            // a^-b => 1/a^b
            rhs.sign = Sign::Positive;
            let inverse_res = self.pow(rhs, int)?;
            return Ok(Exact::new(
                Self::from(1).div(&inverse_res.value, int)?,
                inverse_res.exact,
            ));
        }
        let result_sign = if self.sign == Sign::Positive || rhs.num.is_even(int)? {
            Sign::Positive
        } else {
            Sign::Negative
        };
        let pow_res = Self {
            sign: result_sign,
            num: BigUint::pow(&self.num, &rhs.num, int)?,
            den: BigUint::pow(&self.den, &rhs.num, int)?,
        };
        if rhs.den == 1.into() {
            Ok(Exact::new(pow_res, true))
        } else {
            Ok(pow_res.root_n(
                &Self {
                    sign: Sign::Positive,
                    num: rhs.den,
                    den: 1.into(),
                },
                int,
            )?)
        }
    }

    /// n must be an integer
    fn iter_root_n<I: Interrupt>(
        mut low_bound: Self,
        val: &Self,
        n: &Self,
        int: &I,
    ) -> Result<Self, FendError> {
        let mut high_bound = low_bound.clone().add(1.into(), int)?;
        for _ in 0..30 {
            let guess = low_bound
                .clone()
                .add(high_bound.clone(), int)?
                .div(&2.into(), int)?;
            if &guess.clone().pow(n.clone(), int)?.value < val {
                low_bound = guess;
            } else {
                high_bound = guess;
            }
        }
        low_bound.add(high_bound, int)?.div(&2.into(), int)
    }

    // the boolean indicates whether or not the result is exact
    // n must be an integer
    pub(crate) fn root_n<I: Interrupt>(self, n: &Self, int: &I) -> Result<Exact<Self>, FendError> {
        if self.num != 0.into() && self.sign == Sign::Negative {
            return Err(FendError::RootsOfNegativeNumbers);
        }
        let n = n.clone().simplify(int)?;
        if n.den != 1.into() || n.sign == Sign::Negative {
            return Err(FendError::NonIntegerNegRoots);
        }
        let n = &n.num;
        if self.num == 0.into() {
            return Ok(Exact::new(self, true));
        }
        let num = self.clone().num.root_n(n, int)?;
        let den = self.clone().den.root_n(n, int)?;
        if num.exact && den.exact {
            return Ok(Exact::new(
                Self {
                    sign: Sign::Positive,
                    num: num.value,
                    den: den.value,
                },
                true,
            ));
        }
        // TODO check in which cases this might still be exact
        let num_rat = if num.exact {
            Self::from(num.value)
        } else {
            Self::iter_root_n(
                Self::from(num.value),
                &Self::from(self.num),
                &Self::from(n.clone()),
                int,
            )?
        };
        let den_rat = if den.exact {
            Self::from(den.value)
        } else {
            Self::iter_root_n(
                Self::from(den.value),
                &Self::from(self.den),
                &Self::from(n.clone()),
                int,
            )?
        };
        Ok(Exact::new(num_rat.div(&den_rat, int)?, false))
    }

    pub(crate) fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, FendError> {
        Ok(Self {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num.mul(&rhs.num, int)?,
            den: self.den.mul(&rhs.den, int)?,
        })
    }

    pub(crate) fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        self.add_internal(rhs, int)
    }

    pub(crate) fn is_definitely_zero(&self) -> bool {
        self.num.is_definitely_zero()
    }

    pub(crate) fn is_definitely_one(&self) -> bool {
        self.sign == Sign::Positive && self.num.is_definitely_one() && self.den.is_definitely_one()
    }
}
enum NextDigitErr {
    Error(FendError),
    Terminated,
}

impl From<FendError> for NextDigitErr {
    fn from(e: FendError) -> Self {
        Self::Error(e)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MaxDigitsToPrint {
    /// Print all digits, possibly by writing recurring decimals in parentheses
    AllDigits,
    /// Print only the given number of decimal places, omitting any trailing zeroes
    DecimalPlaces(usize),
    /// Print only the given number of dps, but ignore leading zeroes after the decimal point
    DpButIgnoreLeadingZeroes(usize),
}

impl ops::Neg for BigRat {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            sign: self.sign.flip(),
            ..self
        }
    }
}

impl From<u64> for BigRat {
    fn from(i: u64) -> Self {
        Self {
            sign: Sign::Positive,
            num: i.into(),
            den: 1.into(),
        }
    }
}

impl From<BigUint> for BigRat {
    fn from(n: BigUint) -> Self {
        Self {
            sign: Sign::Positive,
            num: n,
            den: BigUint::from(1),
        }
    }
}

#[derive(Default)]
pub(crate) struct FormatOptions {
    pub(crate) base: Base,
    pub(crate) style: FormattingStyle,
    pub(crate) term: &'static str,
    pub(crate) use_parens_if_fraction: bool,
}

impl Format for BigRat {
    type Params = FormatOptions;
    type Out = FormattedBigRat;

    // Formats as an integer if possible, or a terminating float, otherwise as
    // either a fraction or a potentially approximated floating-point number.
    // The result 'exact' field indicates whether the number was exact or not.
    fn format<I: Interrupt>(
        &self,
        params: &Self::Params,
        int: &I,
    ) -> Result<Exact<Self::Out>, FendError> {
        let base = params.base;
        let style = params.style;
        let term = params.term;
        let use_parens_if_fraction = params.use_parens_if_fraction;

        let mut x = self.clone().simplify(int)?;
        let sign = if x.sign == Sign::Positive || x == 0.into() {
            Sign::Positive
        } else {
            Sign::Negative
        };
        x.sign = Sign::Positive;

        // try as integer if possible
        if x.den == 1.into() {
            let sf_limit = if let FormattingStyle::SignificantFigures(sf) = style {
                Some(sf)
            } else {
                None
            };
            return Self::format_as_integer(
                &x.num,
                base,
                sign,
                term,
                use_parens_if_fraction,
                sf_limit,
                int,
            );
        }

        let mut terminating_res = None;
        let mut terminating = || match terminating_res {
            None => {
                let t = x.terminates_in_base(base, int)?;
                terminating_res = Some(t);
                Ok(t)
            }
            Some(t) => Ok(t),
        };
        let fraction = style == FormattingStyle::ImproperFraction
            || style == FormattingStyle::MixedFraction
            || (style == FormattingStyle::Exact && !terminating()?);
        if fraction {
            let mixed = style == FormattingStyle::MixedFraction || style == FormattingStyle::Exact;
            return x.format_as_fraction(base, sign, term, mixed, use_parens_if_fraction, int);
        }

        // not a fraction, will be printed as a decimal
        x.format_as_decimal(style, base, sign, term, terminating, int)
    }
}

#[derive(Debug)]
enum FormattedBigRatType {
    // optional int,
    // bool whether to add a space before the string
    // followed by a string (empty, "i" or "pi"),
    // followed by whether to wrap the number in parentheses
    Integer(Option<FormattedBigUint>, bool, &'static str, bool),
    // optional int (for mixed fractions)
    // optional int (numerator)
    // space
    // string (empty, "i", "pi", etc.)
    // '/'
    // int (denominator)
    // string (empty, "i", "pi", etc.) (used for mixed fractions, e.g. 1 2/3 i)
    // bool (whether or not to wrap the fraction in parentheses)
    Fraction(
        Option<FormattedBigUint>,
        Option<FormattedBigUint>,
        bool,
        &'static str,
        FormattedBigUint,
        &'static str,
        bool,
    ),
    // string representation of decimal number (may or may not contain recurring digits)
    // space
    // string (empty, "i", "pi", etc.)
    Decimal(String, bool, &'static str),
}

#[must_use]
#[derive(Debug)]
pub(crate) struct FormattedBigRat {
    // whether or not to print a minus sign
    sign: Sign,
    ty: FormattedBigRatType,
}

impl fmt::Display for FormattedBigRat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.sign == Sign::Negative {
            write!(f, "-")?;
        }
        match &self.ty {
            FormattedBigRatType::Integer(int, space, isuf, use_parens) => {
                if *use_parens {
                    write!(f, "(")?;
                }
                if let Some(int) = int {
                    write!(f, "{}", int)?;
                }
                if *space {
                    write!(f, " ")?;
                }
                write!(f, "{}", isuf)?;
                if *use_parens {
                    write!(f, ")")?;
                }
            }
            FormattedBigRatType::Fraction(integer, num, space, isuf, den, isuf2, use_parens) => {
                if *use_parens {
                    write!(f, "(")?;
                }
                if let Some(integer) = integer {
                    write!(f, "{} ", integer)?;
                }
                if let Some(num) = num {
                    write!(f, "{}", num)?;
                }
                if *space && !isuf.is_empty() {
                    write!(f, " ")?;
                }
                write!(f, "{}/{}", isuf, den)?;
                if *space && !isuf2.is_empty() {
                    write!(f, " ")?;
                }
                write!(f, "{}", isuf2)?;
                if *use_parens {
                    write!(f, ")")?;
                }
            }
            FormattedBigRatType::Decimal(s, space, term) => {
                write!(f, "{}", s)?;
                if *space {
                    write!(f, " ")?;
                }
                write!(f, "{}", term)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::sign::Sign;
    use super::BigRat;
    use crate::error::FendError;
    use crate::num::biguint::BigUint;
    use std::mem;

    #[test]
    fn test_bigrat_from() {
        mem::drop(BigRat::from(2));
        mem::drop(BigRat::from(0));
        mem::drop(BigRat::from(u64::MAX));
        mem::drop(BigRat::from(u64::from(u32::MAX)));
    }

    #[test]
    fn test_addition() -> Result<(), FendError> {
        let int = &crate::interrupt::Never::default();
        let two = BigRat::from(2);
        assert_eq!(two.clone().add(two, int)?, BigRat::from(4));
        Ok(())
    }

    #[test]
    fn test_cmp() {
        assert!(
            BigRat {
                sign: Sign::Positive,
                num: BigUint::from(16),
                den: BigUint::from(9)
            } < BigRat::from(2)
        );
    }

    #[test]
    fn test_cmp_2() {
        assert!(
            BigRat {
                sign: Sign::Positive,
                num: BigUint::from(36),
                den: BigUint::from(49)
            } < BigRat {
                sign: Sign::Positive,
                num: BigUint::from(3),
                den: BigUint::from(4)
            }
        );
    }
}
