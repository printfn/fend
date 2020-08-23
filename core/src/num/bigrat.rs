use crate::num::biguint::BigUint;
use crate::num::Base;
use std::cmp::Ordering;
use std::fmt::{Debug, Error, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

mod sign {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Sign {
        Positive,
        Negative,
    }

    impl Sign {
        pub fn flip(self) -> Self {
            match self {
                Self::Positive => Self::Negative,
                Self::Negative => Self::Positive,
            }
        }

        pub fn sign_of_product(a: Self, b: Self) -> Self {
            match (a, b) {
                (Sign::Positive, Sign::Positive) => Sign::Positive,
                (Sign::Positive, Sign::Negative) => Sign::Negative,
                (Sign::Negative, Sign::Positive) => Sign::Negative,
                (Sign::Negative, Sign::Negative) => Sign::Positive,
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
    fn cmp(&self, other: &BigRat) -> Ordering {
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
    fn partial_cmp(&self, other: &BigRat) -> Option<Ordering> {
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
    /// compute a + b
    fn add_internal(self, rhs: BigRat) -> BigRat {
        // a + b == -((-a) + (-b))
        if self.sign == Sign::Negative {
            return -((-self).add_internal(-rhs));
        }

        assert_eq!(self.sign, Sign::Positive);

        if self.den == rhs.den {
            if rhs.sign == Sign::Negative && self.num < rhs.num {
                BigRat {
                    sign: Sign::Negative,
                    num: rhs.num - self.num,
                    den: self.den,
                }
            } else {
                BigRat {
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
                BigRat {
                    sign: Sign::Negative,
                    num: b - a,
                    den: new_denominator,
                }
            } else {
                BigRat {
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
}

impl Add for BigRat {
    type Output = BigRat;

    fn add(self, rhs: BigRat) -> BigRat {
        self.add_internal(rhs)
    }
}

impl Neg for BigRat {
    type Output = BigRat;

    fn neg(self) -> BigRat {
        BigRat {
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
    type Output = BigRat;

    fn sub(self, rhs: BigRat) -> BigRat {
        self.add_internal(-rhs)
    }
}

impl Sub for &BigRat {
    type Output = BigRat;

    fn sub(self, rhs: &BigRat) -> BigRat {
        self.clone().add_internal(-rhs.clone())
    }
}

impl Mul for BigRat {
    type Output = BigRat;

    fn mul(self, rhs: BigRat) -> BigRat {
        #[allow(clippy::suspicious_arithmetic_impl)]
        BigRat {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num * rhs.num,
            den: self.den * rhs.den,
        }
    }
}

impl From<u64> for BigRat {
    fn from(i: u64) -> Self {
        BigRat {
            sign: Sign::Positive,
            num: i.into(),
            den: 1.into(),
        }
    }
}

impl From<i32> for BigRat {
    fn from(i: i32) -> Self {
        use std::convert::TryInto;

        if let Ok(j) = TryInto::<u64>::try_into(i) {
            BigRat {
                sign: Sign::Positive,
                num: j.into(),
                den: 1.into(),
            }
        } else {
            let j: u64 = (-i).try_into().unwrap();
            BigRat {
                sign: Sign::Negative,
                num: j.into(),
                den: 1.into(),
            }
        }
    }
}

impl BigRat {
    fn simplify(mut self) -> BigRat {
        if self.den == 1.into() {
            return self;
        }
        let gcd = BigUint::gcd(self.num.clone(), self.den.clone());
        self.num = self.num / gcd.clone();
        self.den = self.den / gcd;
        self
    }

    pub fn div(self, rhs: BigRat) -> Result<BigRat, String> {
        if rhs.num == 0.into() {
            return Err("Attempt to divide by zero".to_string());
        }
        #[allow(clippy::suspicious_arithmetic_impl)]
        Ok(BigRat {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num * rhs.den,
            den: self.den * rhs.num,
        })
    }

    pub fn pow(mut self, mut rhs: BigRat) -> Result<BigRat, String> {
        self = self.simplify();
        rhs = rhs.simplify();
        if rhs.den != 1.into() {
            return Err("Non-integer exponents not currently supported".to_string());
        }
        if rhs.sign == Sign::Negative {
            // a^-b => 1/a^b
            rhs.sign = Sign::Positive;
            return Ok(BigRat::from(1).div(self.pow(rhs)?)?);
        }
        Ok(BigRat {
            sign: Sign::Positive,
            num: BigUint::pow(self.num, rhs.num.clone())?,
            den: BigUint::pow(self.den, rhs.num)?,
        })
    }

    // test if this fraction has a terminating representation
    // e.g. in base 10: 1/4 = 0.25, but not 1/3
    fn terminates_in_base(&self, base: Base) -> bool {
        let mut x = self.clone();
        let base_as_u64: u64 = base.base_as_u8().into();
        let base = BigRat {
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

    pub fn approx_pi() -> BigRat {
        BigRat {
            sign: Sign::Positive,
            num: BigUint::from(3141592653589793238_u64),
            den: BigUint::from(1000000000000000000_u64),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum FormattingStyle {
    /// Print value as an exact fraction
    ExactFraction,
    /// If possible, print as an exact float, but fall back to an exact fraction
    ExactFloatWithFractionFallback,
    /// Print as an approximate float, with up to 10 decimal places
    ApproxFloat,
}

impl BigRat {
    // Formats as an integer if possible, or a terminating float, otherwise as
    // either a fraction or a potentially approximated floating-point number.
    // The result bool indicates whether the number had to be approximated or not.
    pub fn format(
        &self,
        f: &mut Formatter,
        base: Base,
        style: FormattingStyle,
        imag: bool,
        use_parentheses_if_rational: bool,
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
            if use_parentheses_if_rational {
                write!(f, "(")?;
            }
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
            if use_parentheses_if_rational {
                write!(f, ")")?;
            }
            return Ok(false);
        }

        // not a fraction, will be printed as a decimal
        if negative {
            write!(f, "-")?;
        }
        let num_trailing_digits_to_print =
            if style == FormattingStyle::ExactFloatWithFractionFallback && terminating {
                std::usize::MAX
            } else {
                10
            };
        let integer_part = x.num.clone() / x.den.clone();
        integer_part.format(f, base, true)?;
        write!(f, ".")?;
        let integer_as_rational = BigRat {
            sign: Sign::Positive,
            num: integer_part,
            den: 1.into(),
        };
        let mut remaining_fraction = x.clone() - integer_as_rational;
        let mut i = 0;
        while remaining_fraction.num > 0.into() && i < num_trailing_digits_to_print {
            let base_as_u64: u64 = base.base_as_u8().into();
            remaining_fraction = (remaining_fraction * base_as_u64.into()).simplify();
            let digit = remaining_fraction.num.clone() / remaining_fraction.den.clone();
            digit.format(f, base, false)?;
            remaining_fraction = remaining_fraction
                - BigRat {
                    sign: Sign::Positive,
                    num: digit,
                    den: 1.into(),
                };
            i += 1;
        }
        if imag {
            if base.base_as_u8() >= 19 {
                write!(f, " ")?;
            }
            write!(f, "i")?;
        }
        Ok(!terminating)
    }
}

impl From<BigUint> for BigRat {
    fn from(n: BigUint) -> Self {
        BigRat {
            sign: Sign::Positive,
            num: n,
            den: BigUint::from(1),
        }
    }
}

impl BigRat {
    fn iter_root_n(mut low_bound: BigRat, val: &BigRat, n: &BigRat) -> Result<BigRat, String> {
        let mut high_bound = low_bound.clone() + 1.into();
        for _ in 0..30 {
            let guess = (low_bound.clone() + high_bound.clone()).div(2.into())?;
            if &guess.clone().pow(n.clone())? < val {
                low_bound = guess;
            } else {
                high_bound = guess;
            }
        }
        Ok((low_bound + high_bound).div(2.into())?)
    }

    // the boolean indicates whether or not the result is exact
    pub fn root_n(self, n: &BigRat) -> Result<(BigRat, bool), String> {
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
                BigRat {
                    sign: Sign::Positive,
                    num,
                    den,
                },
                true,
            ));
        }
        let num_rat = if num_exact {
            BigRat::from(num)
        } else {
            Self::iter_root_n(
                BigRat::from(num),
                &BigRat::from(self.num),
                &BigRat::from(n.clone()),
            )?
        };
        let den_rat = if den_exact {
            BigRat::from(den)
        } else {
            Self::iter_root_n(
                BigRat::from(den),
                &BigRat::from(self.den),
                &BigRat::from(n.clone()),
            )?
        };
        Ok((num_rat.div(den_rat)?, false))
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
        BigRat::from(-5);
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
