use crate::err::{IntErr, Interrupt, Never};
use crate::num::bigrat::BigRat;
use crate::num::{Base, FormattingStyle};
use std::cmp::Ordering;
use std::fmt::{Error, Formatter};
use std::ops::Neg;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Complex {
    real: BigRat,
    imag: BigRat,
}

impl Complex {
    pub fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, IntErr<String, I>> {
        if self.imag != 0.into() {
            return Err("Cannot convert complex number to integer".to_string())?;
        }
        Ok(self.real.try_as_usize(int)?)
    }

    pub fn conjugate(self) -> Self {
        Self {
            real: self.real,
            imag: -self.imag,
        }
    }

    pub fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        if self.imag != 0.into() {
            return Err("Factorial is not supported for complex numbers".to_string())?;
        }
        Ok(Self {
            real: self.real.factorial(int)?,
            imag: self.imag,
        })
    }

    pub fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        // (u + vi) / (x + yi) = (1/(x^2 + y^2)) * ((ux + vy) + (vx - uy)i)
        let u = self.real;
        let v = self.imag;
        let x = rhs.real;
        let y = rhs.imag;
        let sum = x.clone().mul(&x, int)?.add(y.clone().mul(&y, int)?, int)?;
        Ok(Self {
            real: BigRat::from(1).div(&sum, int)?,
            imag: 0.into(),
        }
        .mul(
            &Self {
                real: u.clone().mul(&x, int)?.add(v.clone().mul(&y, int)?, int)?,
                imag: v.mul(&x, int)?.sub(u.mul(&y, int)?, int)?,
            },
            int,
        )?)
    }

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<(Self, bool), IntErr<String, I>> {
        if self.imag != 0.into() || rhs.imag != 0.into() {
            return Err("Exponentiation is currently unsupported for complex numbers".to_string())?;
        }
        let (real, exact) = self.real.pow(rhs.real, int)?;
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
    pub fn add_digit_in_base<I: Interrupt>(
        &mut self,
        digit: u64,
        base: u8,
        int: &I,
    ) -> Result<(), IntErr<Never, I>> {
        self.real.add_digit_in_base(digit, base, int)
    }

    pub fn i() -> Self {
        Self {
            real: 0.into(),
            imag: 1.into(),
        }
    }

    pub fn abs<I: Interrupt>(self, int: &I) -> Result<(Self, bool), IntErr<String, I>> {
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
                real: self
                    .real
                    .pow(2.into(), int)?
                    .0
                    .add(self.imag.pow(2.into(), int)?.0, int)?,
                imag: 0.into(),
            };
            res_squared.root_n(&Self::from(2), int)?
        })
    }

    pub fn format<I: Interrupt>(
        &self,
        f: &mut Formatter,
        exact: bool,
        style: FormattingStyle,
        base: Base,
        use_parentheses_if_complex: bool,
        int: &I,
    ) -> Result<(), IntErr<Error, I>> {
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
            self.real.format(f, base, style, false, int)?;
            return Ok(());
        }

        if self.real == 0.into() {
            self.imag.format(f, base, style, true, int)?;
        } else {
            if use_parentheses_if_complex {
                write!(f, "(")?;
            }
            self.real.format(f, base, style, false, int)?;
            if self.imag > 0.into() {
                write!(f, " + ")?;
                self.imag.format(f, base, style, true, int)?;
            } else {
                write!(f, " - ")?;
                (-self.imag.clone()).format(f, base, style, true, int)?;
            }
            if use_parentheses_if_complex {
                write!(f, ")")?;
            }
        }

        Ok(())
    }

    pub fn root_n<I: Interrupt>(
        self,
        n: &Self,
        int: &I,
    ) -> Result<(Self, bool), IntErr<String, I>> {
        if self.imag != 0.into() || n.imag != 0.into() {
            return Err("Roots are currently unsupported for complex numbers".to_string())?;
        }
        let (real_root, real_root_exact) = self.real.root_n(&n.real, int)?;
        Ok((
            Self {
                real: real_root,
                imag: 0.into(),
            },
            real_root_exact,
        ))
    }

    fn expect_real<I: Interrupt>(self) -> Result<BigRat, IntErr<String, I>> {
        if self.imag == 0.into() {
            Ok(self.real)
        } else {
            Err("Expected a real number".to_string())?
        }
    }

    pub fn sin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.sin(int)?))
    }

    pub fn cos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.cos(int)?))
    }

    pub fn tan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.tan(int)?))
    }

    pub fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.asin(int)?))
    }

    pub fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.acos(int)?))
    }

    pub fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.atan(int)?))
    }

    pub fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.sinh(int)?))
    }

    pub fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.cosh(int)?))
    }

    pub fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.tanh(int)?))
    }

    pub fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.asinh(int)?))
    }

    pub fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.acosh(int)?))
    }

    pub fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.atanh(int)?))
    }

    pub fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.ln(int)?))
    }

    pub fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.log2(int)?))
    }

    pub fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.log10(int)?))
    }

    pub fn exp<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.expect_real()?.exp(int)?))
    }

    pub fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        // (a + bi) * (c + di)
        //     => ac + bci + adi - bd
        //     => (ac - bd) + (bc + ad)i
        Ok(Self {
            real: self
                .real
                .clone()
                .mul(&rhs.real, int)?
                .sub(self.imag.clone().mul(&rhs.imag, int)?, int)?,
            imag: self
                .real
                .mul(&rhs.imag, int)?
                .add(self.imag.mul(&rhs.real, int)?, int)?,
        })
    }

    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self {
            real: self.real.add(rhs.real, int)?,
            imag: self.imag.add(rhs.imag, int)?,
        })
    }

    pub fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        self.add(-rhs, int)
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
