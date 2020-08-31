use crate::num::biguint::BigUint;
use crate::num::{Base, FormattingStyle};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter};
use std::{
    collections::HashMap,
    ops::{Add, Mul, Neg, Sub},
};

mod sign {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Sign {
        Positive,
        Negative,
    }

    impl Sign {
        pub const fn flip(self) -> Self {
            match self {
                Self::Positive => Self::Negative,
                Self::Negative => Self::Positive,
            }
        }

        pub const fn sign_of_product(a: Self, b: Self) -> Self {
            match (a, b) {
                (Self::Positive, Self::Positive) | (Self::Negative, Self::Negative) => {
                    Self::Positive
                }
                (Self::Positive, Self::Negative) | (Self::Negative, Self::Positive) => {
                    Self::Negative
                }
            }
        }
    }
}

use sign::Sign;

#[derive(Clone, Debug)]
pub struct BigRat {
    sign: Sign,
    num: BigUint,
    den: BigUint,
}

impl Ord for BigRat {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = self - other;
        if diff.num == 0.into() {
            Ordering::Equal
        } else if diff.sign == Sign::Positive {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for BigRat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BigRat {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for BigRat {}

impl BigRat {
    pub fn try_as_usize(mut self) -> Result<usize, String> {
        self = self.simplify();
        if self.den != 1.into() {
            return Err("Cannot convert fraction to integer".to_string());
        }
        Ok(self.num.try_as_usize()?)
    }

    #[allow(clippy::float_arithmetic)]
    pub fn into_f64(mut self) -> f64 {
        self = self.simplify();
        self.num.as_f64() / self.den.as_f64()
    }

    #[allow(
        clippy::as_conversions,
        clippy::float_arithmetic,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn from_f64(mut f: f64) -> Self {
        let negative = f < 0.0;
        if negative {
            f = -f;
        }
        let i = (f * f64::from(u32::MAX)) as u64;
        Self {
            sign: if negative {
                Sign::Negative
            } else {
                Sign::Positive
            },
            num: BigUint::from(i),
            den: BigUint::from(u64::from(u32::MAX)),
        }
    }

    pub fn sin(self) -> Self {
        Self::from_f64(f64::sin(self.into_f64()))
    }

    pub fn cos(self) -> Self {
        Self::from_f64(f64::cos(self.into_f64()))
    }

    pub fn tan(self) -> Self {
        Self::from_f64(f64::tan(self.into_f64()))
    }

    pub fn asin(self) -> Self {
        Self::from_f64(f64::asin(self.into_f64()))
    }

    pub fn acos(self) -> Self {
        Self::from_f64(f64::acos(self.into_f64()))
    }

    pub fn atan(self) -> Self {
        Self::from_f64(f64::atan(self.into_f64()))
    }

    pub fn sinh(self) -> Self {
        Self::from_f64(f64::sinh(self.into_f64()))
    }

    pub fn cosh(self) -> Self {
        Self::from_f64(f64::cosh(self.into_f64()))
    }

    pub fn tanh(self) -> Self {
        Self::from_f64(f64::tanh(self.into_f64()))
    }

    pub fn asinh(self) -> Self {
        Self::from_f64(f64::asinh(self.into_f64()))
    }

    pub fn acosh(self) -> Self {
        Self::from_f64(f64::acosh(self.into_f64()))
    }

    pub fn atanh(self) -> Self {
        Self::from_f64(f64::atanh(self.into_f64()))
    }

    pub fn ln(self) -> Self {
        Self::from_f64(f64::ln(self.into_f64()))
    }

    pub fn log2(self) -> Self {
        Self::from_f64(f64::log2(self.into_f64()))
    }

    pub fn log10(self) -> Self {
        Self::from_f64(f64::log10(self.into_f64()))
    }

    pub fn exp(self) -> Self {
        Self::from_f64(f64::exp(self.into_f64()))
    }

    /// compute a + b
    fn add_internal(self, rhs: Self) -> Self {
        // a + b == -((-a) + (-b))
        if self.sign == Sign::Negative {
            return -((-self).add_internal(-rhs));
        }

        assert_eq!(self.sign, Sign::Positive);

        if self.den == rhs.den {
            if rhs.sign == Sign::Negative && self.num < rhs.num {
                Self {
                    sign: Sign::Negative,
                    num: rhs.num - self.num,
                    den: self.den,
                }
            } else {
                Self {
                    sign: Sign::Positive,
                    num: if rhs.sign == Sign::Positive {
                        self.num + rhs.num
                    } else {
                        self.num - rhs.num
                    },
                    den: self.den,
                }
            }
        } else {
            let gcd = BigUint::gcd(self.den.clone(), rhs.den.clone());
            let new_denominator = self.den.clone() * rhs.den.clone() / gcd.clone();
            let a = self.num * rhs.den / gcd.clone();
            let b = rhs.num * self.den / gcd;

            if rhs.sign == Sign::Negative && a < b {
                Self {
                    sign: Sign::Negative,
                    num: b - a,
                    den: new_denominator,
                }
            } else {
                Self {
                    sign: Sign::Positive,
                    num: if rhs.sign == Sign::Positive {
                        a + b
                    } else {
                        a - b
                    },
                    den: new_denominator,
                }
            }
        }
    }

    fn simplify(mut self) -> Self {
        if self.den == 1.into() {
            return self;
        }
        let gcd = BigUint::gcd(self.num.clone(), self.den.clone());
        self.num = self.num / gcd.clone();
        self.den = self.den / gcd;
        self
    }

    pub fn div(self, rhs: Self) -> Result<Self, String> {
        if rhs.num == 0.into() {
            return Err("Attempt to divide by zero".to_string());
        }
        Ok(Self {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num * rhs.den,
            den: self.den * rhs.num,
        })
    }

    // test if this fraction has a terminating representation
    // e.g. in base 10: 1/4 = 0.25, but not 1/3
    fn terminates_in_base(&self, base: Base) -> bool {
        let mut x = self.clone();
        let base_as_u64: u64 = base.base_as_u8().into();
        let base = Self {
            sign: Sign::Positive,
            num: base_as_u64.into(),
            den: 1.into(),
        };
        loop {
            let old_den = x.den.clone();
            x = (x * base.clone()).simplify();
            let new_den = x.den.clone();
            if new_den == old_den {
                break;
            }
        }
        x.den == 1.into()
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_digit_in_base(&mut self, digit: u64, base: u8) {
        let base_as_u64: u64 = base.into();
        self.num = self.num.clone() * base_as_u64.into() + digit.into();
        self.den = self.den.clone() * base_as_u64.into();
    }

    pub fn approx_pi() -> Self {
        Self {
            sign: Sign::Positive,
            num: BigUint::from(3_141_592_653_589_793_238_u64),
            den: BigUint::from(1_000_000_000_000_000_000_u64),
        }
    }

    // Formats as an integer if possible, or a terminating float, otherwise as
    // either a fraction or a potentially approximated floating-point number.
    // The result bool indicates whether the number had to be approximated or not.
    pub fn format(
        &self,
        f: &mut Formatter,
        base: Base,
        style: FormattingStyle,
        imag: bool,
    ) -> Result<bool, Error> {
        let mut x = self.clone().simplify();
        let negative = x.sign == Sign::Negative && x != 0.into();
        if negative {
            x.sign = Sign::Positive;
        };

        // try as integer if possible
        if x.den == 1.into() {
            if negative {
                write!(f, "-")?;
            }
            if imag && base == Base::Decimal && x.num == 1.into() {
                write!(f, "i")?;
            } else {
                x.num.format(f, base, true)?;
                if imag {
                    if base.base_as_u8() >= 19 {
                        // at this point 'i' could be a digit, so we need a space to disambiguate
                        write!(f, " ")?;
                    }
                    write!(f, "i")?;
                }
            }
            return Ok(false);
        }

        let terminating = x.terminates_in_base(base);
        let fraction = style == FormattingStyle::ExactFraction
            || (style == FormattingStyle::ExactFloatWithFractionFallback && !terminating);
        if fraction {
            if negative {
                write!(f, "-")?;
            }
            if imag && base == Base::Decimal && x.num == 1.into() {
                write!(f, "i")?;
            } else {
                x.num.format(f, base, true)?;
                if imag {
                    if base.base_as_u8() >= 19 {
                        write!(f, " ")?;
                    }
                    write!(f, "i")?;
                }
            }
            write!(f, "/")?;
            x.den.format(f, base, true)?;
            return Ok(false);
        }

        // not a fraction, will be printed as a decimal
        if negative {
            write!(f, "-")?;
        }
        let num_trailing_digits_to_print = if style == FormattingStyle::ExactFloat
            || (style == FormattingStyle::ExactFloatWithFractionFallback && terminating)
        {
            None
        } else if let FormattingStyle::ApproxFloat(n) = style {
            Some(n)
        } else {
            Some(10)
        };
        let integer_part = x.num.clone() / x.den.clone();
        integer_part.format(f, base, true)?;
        write!(f, ".")?;
        let integer_as_rational = Self {
            sign: Sign::Positive,
            num: integer_part,
            den: 1.into(),
        };
        let remaining_fraction = x - integer_as_rational;
        Self::format_trailing_digits(
            f,
            base,
            remaining_fraction.num,
            &remaining_fraction.den,
            num_trailing_digits_to_print,
        )?;
        if imag {
            if base.base_as_u8() >= 19 {
                write!(f, " ")?;
            }
            write!(f, "i")?;
        }
        Ok(!terminating)
    }

    /// Prints the decimal expansion of num/den, where num < den, in the given base.
    /// If `max_digits` is given, only up to that many digits are printed, and recurring
    /// digits are not printed in parentheses.
    fn format_trailing_digits(
        f: &mut Formatter,
        base: Base,
        mut numerator: BigUint,
        denominator: &BigUint,
        max_digits: Option<usize>,
    ) -> Result<(), Error> {
        let mut output = String::new();
        let mut pos = 0;
        let mut remainder_occurs_at_pos: HashMap<BigUint, usize> = HashMap::new();
        let base_as_u64: u64 = base.base_as_u8().into();
        let b: BigUint = base_as_u64.into();
        while max_digits.is_some() || remainder_occurs_at_pos.get(&numerator) == None {
            remainder_occurs_at_pos.insert(numerator.clone(), pos);
            let bnum = b.clone() * numerator;
            let digit = bnum.clone() / denominator.clone();
            numerator = bnum - digit.clone() * denominator.clone();
            output.push_str(Fmt(|f| digit.format(f, base, false)).to_string().as_str());
            pos += 1;
            if numerator == 0.into() || max_digits == Some(pos) {
                // terminates here
                write!(f, "{}", output)?;
                return Ok(());
            }
        }
        // todo: this may panic if numerator is not found
        let location = remainder_occurs_at_pos[&numerator];
        let (a, b) = output.split_at(location);
        write!(f, "{}({})", a, b)?;
        Ok(())
    }

    pub fn pow(mut self, mut rhs: Self) -> Result<(Self, bool), String> {
        self = self.simplify();
        rhs = rhs.simplify();
        if rhs.sign == Sign::Negative {
            // a^-b => 1/a^b
            rhs.sign = Sign::Positive;
            let (inverse_res, exact) = self.pow(rhs)?;
            return Ok((Self::from(1).div(inverse_res)?, exact));
        }
        let pow_res = Self {
            sign: Sign::Positive,
            num: BigUint::pow(&self.num, &rhs.num)?,
            den: BigUint::pow(&self.den, &rhs.num)?,
        };
        if rhs.den == 1.into() {
            Ok((pow_res, true))
        } else {
            Ok(pow_res.root_n(&Self {
                sign: Sign::Positive,
                num: rhs.den,
                den: 1.into(),
            })?)
        }
    }

    /// n must be an integer
    fn iter_root_n(mut low_bound: Self, val: &Self, n: &Self) -> Result<Self, String> {
        let mut high_bound = low_bound.clone() + 1.into();
        for _ in 0..30 {
            let guess = (low_bound.clone() + high_bound.clone()).div(2.into())?;
            if &guess.clone().pow(n.clone())?.0 < val {
                low_bound = guess;
            } else {
                high_bound = guess;
            }
        }
        Ok((low_bound + high_bound).div(2.into())?)
    }

    // the boolean indicates whether or not the result is exact
    // n must be an integer
    pub fn root_n(self, n: &Self) -> Result<(Self, bool), String> {
        if self.sign == Sign::Negative {
            return Err("Can't compute roots of negative numbers".to_string());
        }
        let n = n.clone().simplify();
        if n.den != 1.into() || n.sign == Sign::Negative {
            return Err("Can't compute non-integer or negative roots".to_string());
        }
        let n = &n.num;
        if self.num == 0.into() {
            return Ok((self, true));
        }
        let (num, num_exact) = self.clone().num.root_n(n)?;
        let (den, den_exact) = self.clone().den.root_n(n)?;
        if num_exact && den_exact {
            return Ok((
                Self {
                    sign: Sign::Positive,
                    num,
                    den,
                },
                true,
            ));
        }
        let num_rat = if num_exact {
            Self::from(num)
        } else {
            Self::iter_root_n(
                Self::from(num),
                &Self::from(self.num),
                &Self::from(n.clone()),
            )?
        };
        let den_rat = if den_exact {
            Self::from(den)
        } else {
            Self::iter_root_n(
                Self::from(den),
                &Self::from(self.den),
                &Self::from(n.clone()),
            )?
        };
        Ok((num_rat.div(den_rat)?, false))
    }
}

// Small formatter helper
struct Fmt<F>(F)
where
    F: Fn(&mut Formatter) -> Result<(), Error>;

impl<F> Display for Fmt<F>
where
    F: Fn(&mut Formatter) -> Result<(), Error>,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        (self.0)(f)
    }
}

impl Add for BigRat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.add_internal(rhs)
    }
}

impl Neg for BigRat {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            sign: self.sign.flip(),
            num: self.num,
            den: self.den,
        }
    }
}

impl Neg for &BigRat {
    type Output = BigRat;

    fn neg(self) -> BigRat {
        -self.clone()
    }
}

impl Sub for BigRat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.add_internal(-rhs)
    }
}

impl Sub for &BigRat {
    type Output = BigRat;

    fn sub(self, rhs: Self) -> BigRat {
        self.clone().add_internal(-rhs.clone())
    }
}

impl Mul for BigRat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num * rhs.num,
            den: self.den * rhs.den,
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

#[cfg(test)]
mod tests {
    use super::sign::Sign;
    use super::BigRat;
    use crate::num::biguint::BigUint;

    #[test]
    fn test_bigrat_from() {
        BigRat::from(2);
        BigRat::from(0);
        BigRat::from(u64::MAX);
        BigRat::from(u64::from(u32::MAX));
    }

    #[test]
    fn test_addition() {
        assert_eq!(BigRat::from(2) + BigRat::from(2), BigRat::from(4));
    }

    #[test]
    fn test_cmp() {
        assert!(
            BigRat {
                sign: Sign::Positive,
                num: BigUint::from(16),
                den: BigUint::from(9)
            } < BigRat::from(2)
        )
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
        )
    }
}
