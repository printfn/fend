use crate::num::bigint::BigInt;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub};

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

impl Div for BigRat {
    type Output = BigRat;

    fn div(self, rhs: BigRat) -> BigRat {
        #[allow(clippy::suspicious_arithmetic_impl)]
        BigRat {
            num: self.num * rhs.den,
            den: self.den * rhs.num,
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
}

impl Display for BigRat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut x = self.clone().simplify();
        if x.num < 0.into() {
            write!(f, "-")?;
            if x.num < 0.into() {
                x.num = -x.num;
            }
        }
        if x.den == 1.into() {
            write!(f, "{}", x.num)?;
        } else {
            write!(f, "{}/{}", x.num, x.den)?;
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
