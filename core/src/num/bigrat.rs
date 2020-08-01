use crate::num::bigint::BigInt;
use std::fmt::Display;
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone)]
pub struct BigRat {
    num: BigInt,
    den: BigInt,
}

impl Add for BigRat {
    type Output = BigRat;

    fn add(self, rhs: BigRat) -> BigRat {
        if self.den == rhs.den {
            BigRat {
                num: self.num + rhs.num,
                den: self.den,
            }
        } else {
            let new_denominator = BigInt::lcm(self.den.clone(), rhs.den.clone());
            let a = self.num * new_denominator.clone() / self.den;
            let b = rhs.num * new_denominator.clone() / rhs.den;
            BigRat {
                num: a + b,
                den: new_denominator,
            }
        }
    }
}

impl Neg for BigRat {
    type Output = BigRat;

    fn neg(self) -> BigRat {
        BigRat {
            num: -self.num,
            den: self.den,
        }
    }
}

impl Sub for BigRat {
    type Output = BigRat;

    fn sub(self, rhs: BigRat) -> BigRat {
        self + -rhs
    }
}

impl PartialEq for BigRat {
    fn eq(&self, other: &BigRat) -> bool {
        if self.den == other.den {
            self.num == other.num
        } else {
            (self.clone() - other.clone()).num == 0.into()
        }
    }
}

impl Mul for BigRat {
    type Output = BigRat;

    fn mul(self, rhs: BigRat) -> BigRat {
        BigRat {
            num: self.num * rhs.num,
            den: self.den * rhs.den,
        }
    }
}

impl From<i32> for BigRat {
    fn from(i: i32) -> Self {
        BigRat {
            num: i.into(),
            den: 1.into(),
        }
    }
}

impl BigRat {
    fn simplify(mut self) -> BigRat {
        if self.den < 0.into() {
            self.num = -self.num;
            self.den = -self.den;
        }
        if self.den == 1.into() {
            return self;
        }
        let gcd = BigInt::gcd(self.num.clone(), self.den.clone());
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
            num: self.num * rhs.den,
            den: self.den * rhs.num,
        })
    }

    // test if this fraction has a terminating representation
    // e.g. in base 10: 1/4 = 0.25, but not 1/3
    fn terminates_in_base(&self, base: BigInt) -> bool {
        let mut x = self.clone();
        let base = BigRat {
            num: base,
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

    fn is_negative(&self) -> bool {
        self.num < 0.into()
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_decimal_digit(&mut self, digit: i32) {
        self.num = self.num.clone() * 10.into() + digit.into();
        self.den = self.den.clone() * 10.into();
    }
}

impl Display for BigRat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut x = self.clone().simplify();
        let negative = x.is_negative();
        if negative {
            write!(f, "-")?;
            x.num = -x.num;
        };
        if x.den == 1.into() {
            write!(f, "{}", x.num)?;
        } else {
            let terminating = x.terminates_in_base(10.into());
            if !terminating {
                write!(f, "{}/{}, approx. ", x.num, x.den)?;
                if negative {
                    write!(f, "-")?;
                }
            }
            let num_trailing_digits_to_print = if terminating { std::usize::MAX } else { 10 };
            let integer_part = x.num.clone() / x.den.clone();
            write!(f, "{}.", integer_part)?;
            let integer_as_rational = BigRat {
                num: integer_part,
                den: 1.into(),
            };
            let mut remaining_fraction = x - integer_as_rational;
            let mut i = 0;
            while remaining_fraction.num > 0.into() && i < num_trailing_digits_to_print {
                remaining_fraction = (remaining_fraction * 10.into()).simplify();
                let digit = remaining_fraction.num.clone() / remaining_fraction.den.clone();
                write!(f, "{}", digit)?;
                remaining_fraction = remaining_fraction
                    - BigRat {
                        num: digit,
                        den: 1.into(),
                    };
                i += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BigRat;

    #[test]
    fn test_addition() {
        assert_eq!(BigRat::from(2) + BigRat::from(2), BigRat::from(4));
    }
}
