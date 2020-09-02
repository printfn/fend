use crate::interrupt::Interrupt;
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
    pub fn try_as_usize(self, int: &impl Interrupt) -> Result<usize, String> {
        if self.imag != 0.into() {
            return Err("Cannot convert complex number to integer".to_string());
        }
        Ok(self.real.try_as_usize(int)?)
    }

    pub fn conjugate(self) -> Self {
        Self {
            real: self.real,
            imag: -self.imag,
        }
    }

    pub fn factorial(self, int: &impl Interrupt) -> Result<Self, String> {
        if self.imag != 0.into() {
            return Err("Factorial is not supported for complex numbers".to_string());
        }
        Ok(Self {
            real: self.real.factorial(int)?,
            imag: self.imag,
        })
    }

    pub fn div(self, rhs: Self, int: &impl Interrupt) -> Result<Self, String> {
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

    pub fn pow(self, rhs: Self, int: &impl Interrupt) -> Result<(Self, bool), String> {
        if self.imag != 0.into() || rhs.imag != 0.into() {
            return Err("Exponentiation is currently unsupported for complex numbers".to_string());
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
    pub fn add_digit_in_base(
        &mut self,
        digit: u64,
        base: u8,
        int: &impl Interrupt,
    ) -> Result<(), crate::err::Interrupt> {
        self.real.add_digit_in_base(digit, base, int)
    }

    pub fn i() -> Self {
        Self {
            real: 0.into(),
            imag: 1.into(),
        }
    }

    pub fn abs(self, int: &impl Interrupt) -> Result<(Self, bool), String> {
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

    pub fn format(
        &self,
        f: &mut Formatter,
        exact: bool,
        style: FormattingStyle,
        base: Base,
        use_parentheses_if_complex: bool,
        int: &impl Interrupt,
    ) -> Result<Result<(), Error>, crate::err::Interrupt> {
        macro_rules! try_i {
            ($e:expr) => {
                if let Err(e) = $e {
                    return Ok(Err(e));
                }
            };
        }
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
            try_i!(self.real.format(f, base, style, false, int)?);
            return Ok(Ok(()));
        }

        if self.real == 0.into() {
            try_i!(self.imag.format(f, base, style, true, int)?);
        } else {
            if use_parentheses_if_complex {
                try_i!(write!(f, "("));
            }
            try_i!(self.real.format(f, base, style, false, int)?);
            if self.imag > 0.into() {
                try_i!(write!(f, " + "));
                try_i!(self.imag.format(f, base, style, true, int)?);
            } else {
                try_i!(write!(f, " - "));
                try_i!((-self.imag.clone()).format(f, base, style, true, int)?);
            }
            if use_parentheses_if_complex {
                try_i!(write!(f, ")"));
            }
        }

        Ok(Ok(()))
    }

    pub fn root_n(self, n: &Self, int: &impl Interrupt) -> Result<(Self, bool), String> {
        if self.imag != 0.into() || n.imag != 0.into() {
            return Err("Roots are currently unsupported for complex numbers".to_string());
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

    pub fn sin(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.sin(int)?))
    }

    pub fn cos(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.cos(int)?))
    }

    pub fn tan(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.tan(int)?))
    }

    pub fn asin(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.asin(int)?))
    }

    pub fn acos(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.acos(int)?))
    }

    pub fn atan(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.atan(int)?))
    }

    pub fn sinh(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.sinh(int)?))
    }

    pub fn cosh(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.cosh(int)?))
    }

    pub fn tanh(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.tanh(int)?))
    }

    pub fn asinh(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.asinh(int)?))
    }

    pub fn acosh(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.acosh(int)?))
    }

    pub fn atanh(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.atanh(int)?))
    }

    pub fn ln(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.ln(int)?))
    }

    pub fn log2(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.log2(int)?))
    }

    pub fn log10(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.log10(int)?))
    }

    pub fn exp(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self::from(self.expect_real()?.exp(int)?))
    }

    pub fn mul(self, rhs: &Self, int: &impl Interrupt) -> Result<Self, crate::err::Interrupt> {
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

    pub fn add(self, rhs: Self, int: &impl Interrupt) -> Result<Self, crate::err::Interrupt> {
        Ok(Self {
            real: self.real.add(rhs.real, int)?,
            imag: self.imag.add(rhs.imag, int)?,
        })
    }

    pub fn sub(self, rhs: Self, int: &impl Interrupt) -> Result<Self, crate::err::Interrupt> {
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
