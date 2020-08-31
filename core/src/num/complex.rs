use crate::num::bigrat::BigRat;
use crate::num::{Base, FormattingStyle};
use std::cmp::Ordering;
use std::fmt::{Error, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Complex {
    real: BigRat,
    imag: BigRat,
}

impl Complex {
    pub fn try_as_usize(self) -> Result<usize, String> {
        if self.imag != 0.into() {
            return Err("Cannot convert complex number to integer".to_string());
        }
        Ok(self.real.try_as_usize()?)
    }

    pub fn conjugate(self) -> Self {
        Self {
            real: self.real,
            imag: -self.imag,
        }
    }

    pub fn factorial(self) -> Result<Self, String> {
        if self.imag != 0.into() {
            return Err("Factorial is not supported for complex numbers".to_string());
        }
        Ok(Self {
            real: self.real.factorial()?,
            imag: self.imag,
        })
    }

    pub fn div(self, rhs: Self) -> Result<Self, String> {
        // (u + vi) / (x + yi) = (1/(x^2 + y^2)) * ((ux + vy) + (vx - uy)i)
        let u = self.real;
        let v = self.imag;
        let x = rhs.real;
        let y = rhs.imag;
        Ok(Self {
            real: BigRat::from(1).div(x.clone() * x.clone() + y.clone() * y.clone())?,
            imag: 0.into(),
        } * Self {
            real: u.clone() * x.clone() + v.clone() * y.clone(),
            imag: v * x - u * y,
        })
    }

    pub fn pow(self, rhs: Self) -> Result<(Self, bool), String> {
        if self.imag != 0.into() || rhs.imag != 0.into() {
            return Err("Exponentiation is currently unsupported for complex numbers".to_string());
        }
        let (real, exact) = self.real.pow(rhs.real)?;
        Ok((
            Self {
                real,
                imag: 0.into(),
            },
            exact,
        ))
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_digit_in_base(&mut self, digit: u64, base: u8) {
        self.real.add_digit_in_base(digit, base)
    }

    pub fn i() -> Self {
        Self {
            real: 0.into(),
            imag: 1.into(),
        }
    }

    pub fn abs(self) -> Result<(Self, bool), String> {
        Ok(if self.imag == 0.into() {
            if self.real < 0.into() {
                (
                    Self {
                        real: -self.real,
                        imag: 0.into(),
                    },
                    true,
                )
            } else {
                (self, true)
            }
        } else if self.real == 0.into() {
            if self.imag < 0.into() {
                (
                    Self {
                        real: -self.imag,
                        imag: 0.into(),
                    },
                    true,
                )
            } else {
                (
                    Self {
                        real: self.imag,
                        imag: 0.into(),
                    },
                    true,
                )
            }
        } else {
            let res_squared = Self {
                // we can ignore the 'exact' bool because integer powers are always exact
                real: self.real.pow(2.into())?.0 + self.imag.pow(2.into())?.0,
                imag: 0.into(),
            };
            res_squared.root_n(&Self::from(2))?
        })
    }

    pub fn format(
        &self,
        f: &mut Formatter,
        exact: bool,
        style: FormattingStyle,
        base: Base,
        use_parentheses_if_complex: bool,
    ) -> Result<(), Error> {
        let style = if style == FormattingStyle::Auto {
            if exact {
                FormattingStyle::ExactFloatWithFractionFallback
            } else {
                FormattingStyle::ApproxFloat(10)
            }
        } else {
            style
        };
        if self.imag == 0.into() {
            self.real
                .format(f, base, style, false)?;
            return Ok(());
        }

        if self.real == 0.into() {
            self.imag
                .format(f, base, style, true)?;
        } else {
            if use_parentheses_if_complex {
                write!(f, "(")?;
            }
            self.real.format(f, base, style, false)?;
            if self.imag > 0.into() {
                write!(f, " + ")?;
                self.imag.format(f, base, style, true)?;
            } else {
                write!(f, " - ")?;
                (-self.imag.clone()).format(f, base, style, true)?;
            }
            if use_parentheses_if_complex {
                write!(f, ")")?;
            }
        }

        Ok(())
    }

    pub fn root_n(self, n: &Self) -> Result<(Self, bool), String> {
        if self.imag != 0.into() || n.imag != 0.into() {
            return Err("Roots are currently unsupported for complex numbers".to_string());
        }
        let (real_root, real_root_exact) = self.real.root_n(&n.real)?;
        Ok((
            Self {
                real: real_root,
                imag: 0.into(),
            },
            real_root_exact,
        ))
    }

    pub fn approx_pi() -> Self {
        Self {
            real: BigRat::approx_pi(),
            imag: 0.into(),
        }
    }

    pub fn approx_e() -> Self {
        Self {
            real: BigRat::approx_e(),
            imag: 0.into(),
        }
    }

    fn expect_real(self) -> Result<BigRat, String> {
        if self.imag == 0.into() {
            Ok(self.real)
        } else {
            Err("Expected a real number".to_string())
        }
    }

    pub fn sin(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.sin()))
    }

    pub fn cos(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.cos()))
    }

    pub fn tan(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.tan()))
    }

    pub fn asin(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.asin()?))
    }

    pub fn acos(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.acos()?))
    }

    pub fn atan(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.atan()))
    }

    pub fn sinh(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.sinh()))
    }

    pub fn cosh(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.cosh()))
    }

    pub fn tanh(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.tanh()))
    }

    pub fn asinh(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.asinh()))
    }

    pub fn acosh(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.acosh()?))
    }

    pub fn atanh(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.atanh()?))
    }

    pub fn ln(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.ln()?))
    }

    pub fn log2(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.log2()?))
    }

    pub fn log10(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.log10()?))
    }

    pub fn exp(self) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.exp()))
    }
}

impl PartialOrd for Complex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.real <= other.real && self.imag <= other.imag {
            Some(Ordering::Less)
        } else if self.real >= other.real && self.imag >= other.imag {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            real: self.real + rhs.real,
            imag: self.imag + rhs.imag,
        }
    }
}

impl Neg for Complex {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            real: -self.real,
            imag: -self.imag,
        }
    }
}

impl Neg for &Complex {
    type Output = Complex;

    fn neg(self) -> Complex {
        -self.clone()
    }
}

impl Sub for Complex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + -rhs
    }
}

impl Sub for &Complex {
    type Output = Complex;

    fn sub(self, rhs: Self) -> Complex {
        self.clone() + -rhs.clone()
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // (a + bi) * (c + di)
        //     => ac + bci + adi - bd
        //     => (ac - bd) + (bc + ad)i
        Self {
            real: self.real.clone() * rhs.real.clone() - self.imag.clone() * rhs.imag.clone(),
            imag: self.real * rhs.imag + self.imag * rhs.real,
        }
    }
}

impl From<u64> for Complex {
    fn from(i: u64) -> Self {
        Self {
            real: i.into(),
            imag: 0.into(),
        }
    }
}

impl From<BigRat> for Complex {
    fn from(i: BigRat) -> Self {
        Self {
            real: i,
            imag: 0.into(),
        }
    }
}
