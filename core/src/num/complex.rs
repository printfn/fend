use crate::err::{IntErr, Interrupt, Never};
use crate::num::real::Real;
use crate::num::Exact;
use crate::num::{Base, DivideByZero, FormattingStyle};
use std::cmp::Ordering;
use std::fmt;
use std::ops::Neg;

#[derive(Clone, PartialEq, Eq)]
pub struct Complex {
    real: Real,
    imag: Real,
}

impl fmt::Debug for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} + {:?}i", self.real, self.imag)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum UseParentheses {
    No,
    IfComplex,
    IfComplexOrFraction,
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

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        if self.imag != 0.into() || rhs.imag != 0.into() {
            return Err("Exponentiation is currently unsupported for complex numbers".to_string())?;
        }
        let real = self.real.pow(rhs.real, int)?;
        Ok(Exact::new(
            Self {
                real: real.value,
                imag: 0.into(),
            },
            real.exact,
        ))
    }

    pub fn i() -> Self {
        Self {
            real: 0.into(),
            imag: 1.into(),
        }
    }

    pub fn pi() -> Self {
        Self {
            real: Real::pi(),
            imag: 0.into(),
        }
    }

    pub fn abs<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        Ok(if self.imag == 0.into() {
            if self.real < 0.into() {
                Exact::new(
                    Self {
                        real: -self.real,
                        imag: 0.into(),
                    },
                    true,
                )
            } else {
                Exact::new(self, true)
            }
        } else if self.real == 0.into() {
            if self.imag < 0.into() {
                Exact::new(
                    Self {
                        real: -self.imag,
                        imag: 0.into(),
                    },
                    true,
                )
            } else {
                Exact::new(
                    Self {
                        real: self.imag,
                        imag: 0.into(),
                    },
                    true,
                )
            }
        } else {
            let power = self.real.pow(2.into(), int)?;
            let power2 = self.imag.pow(2.into(), int)?;
            let real = power.add(power2, int)?;
            let res_squared = Self {
                real: real.value,
                imag: 0.into(),
            };
            let result = res_squared.root_n(&Self::from(2), int)?;
            result.combine(real.exact)
        })
    }

    pub fn format<I: Interrupt>(
        &self,
        f: &mut fmt::Formatter,
        exact: bool,
        style: FormattingStyle,
        base: Base,
        use_parentheses: UseParentheses,
        int: &I,
    ) -> Result<(), IntErr<fmt::Error, I>> {
        let style = if !exact && style == FormattingStyle::Auto {
            FormattingStyle::ApproxFloat(10)
        } else {
            style
        };

        if self.imag == 0.into() {
            let use_parens = use_parentheses == UseParentheses::IfComplexOrFraction;
            let x = self.real.format(base, style, false, use_parens, int)?;
            if !exact || !x.exact {
                write!(f, "approx. ")?;
            }
            write!(f, "{}", x.value)?;
            return Ok(());
        }

        if self.real == 0.into() {
            let use_parens = use_parentheses == UseParentheses::IfComplexOrFraction;
            let x = self.imag.format(base, style, true, use_parens, int)?;
            if !exact || !x.exact {
                write!(f, "approx. ")?;
            }
            write!(f, "{}", x.value)?;
        } else {
            let mut exact = exact;
            let real_part = self.real.format(base, style, false, false, int)?;
            exact = exact && real_part.exact;
            let (positive, imag_part) = if self.imag > 0.into() {
                (true, self.imag.format(base, style, true, false, int)?)
            } else {
                (
                    false,
                    (-self.imag.clone()).format(base, style, true, false, int)?,
                )
            };
            exact = exact && imag_part.exact;
            if !exact {
                write!(f, "approx. ")?;
            }
            if use_parentheses == UseParentheses::IfComplex
                || use_parentheses == UseParentheses::IfComplexOrFraction
            {
                write!(f, "(")?;
            }
            write!(f, "{}", real_part.value)?;
            if positive {
                write!(f, " + ")?;
            } else {
                write!(f, " - ")?;
            }
            write!(f, "{}", imag_part.value)?;
            if use_parentheses == UseParentheses::IfComplex
                || use_parentheses == UseParentheses::IfComplexOrFraction
            {
                write!(f, ")")?;
            }
        }

        Ok(())
    }

    pub fn root_n<I: Interrupt>(self, n: &Self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        if self.imag != 0.into() || n.imag != 0.into() {
            return Err("Roots are currently unsupported for complex numbers".to_string())?;
        }
        let real_root = self.real.root_n(&n.real, int)?;
        Ok(Exact::new(
            Self {
                real: real_root.value,
                imag: 0.into(),
            },
            real_root.exact,
        ))
    }

    fn expect_real<I: Interrupt>(self) -> Result<Real, IntErr<String, I>> {
        if self.imag == 0.into() {
            Ok(self.real)
        } else {
            Err("Expected a real number".to_string())?
        }
    }

    pub fn sin<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        Ok(self.expect_real()?.sin(int)?.apply(Self::from))
    }

    pub fn cos<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        // cos(self) == sin(pi/2 - self)
        let pi = Exact::new(Self::pi(), true);
        let half_pi = pi
            .div(Exact::new(2.into(), true), int)
            .map_err(IntErr::into_string)?;
        let sin_arg = half_pi.add(-Exact::new(self, true), int)?;
        Ok(sin_arg
            .value
            .expect_real()?
            .sin(int)?
            .combine(sin_arg.exact)
            .apply(Self::from))
    }

    pub fn tan<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        let num = self.clone().sin(int)?;
        let den = self.cos(int)?;
        num.div(den, int).map_err(IntErr::into_string)
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
}

#[allow(clippy::use_self)]
impl Exact<Complex> {
    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        let (self_real, self_imag) = self.apply(|x| (x.real, x.imag)).pair();
        let (rhs_real, rhs_imag) = rhs.apply(|x| (x.real, x.imag)).pair();
        let real = self_real.add(rhs_real, int)?;
        let imag = self_imag.add(rhs_imag, int)?;
        Ok(Self::new(
            Complex {
                real: real.value,
                imag: imag.value,
            },
            real.exact && imag.exact,
        ))
    }

    pub fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        // (a + bi) * (c + di)
        //     => ac + bci + adi - bd
        //     => (ac - bd) + (bc + ad)i
        let (self_real, self_imag) = self.apply(|x| (x.real, x.imag)).pair();
        let (rhs_real, rhs_imag) = rhs.clone().apply(|x| (x.real, x.imag)).pair();

        let prod1 = self_real.clone().mul(rhs_real.re(), int)?;
        let prod2 = self_imag.clone().mul(rhs_imag.re(), int)?;
        let real_part = prod1.add(-prod2, int)?;
        let prod3 = self_real.mul(rhs_imag.re(), int)?;
        let prod4 = self_imag.mul(rhs_real.re(), int)?;
        let imag_part = prod3.add(prod4, int)?;
        Ok(Self::new(
            Complex {
                real: real_part.value,
                imag: imag_part.value,
            },
            real_part.exact && imag_part.exact,
        ))
    }

    pub fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
        // (u + vi) / (x + yi) = (1/(x^2 + y^2)) * ((ux + vy) + (vx - uy)i)
        let (u, v) = self.apply(|x| (x.real, x.imag)).pair();
        let (x, y) = rhs.apply(|x| (x.real, x.imag)).pair();
        // if both numbers are real, use this simplified algorithm
        if v.exact && v.value == 0.into() && y.exact && y.value == 0.into() {
            return Ok(u.div(&x, int)?.apply(|real| Complex {
                real,
                imag: 0.into(),
            }));
        }
        let prod1 = x.clone().mul(x.re(), int)?;
        let prod2 = y.clone().mul(y.re(), int)?;
        let sum = prod1.add(prod2, int)?;
        let real_part = Exact::new(Real::from(1), true).div(&sum, int)?;
        let prod3 = u.clone().mul(x.re(), int)?;
        let prod4 = v.clone().mul(y.re(), int)?;
        let real2 = prod3.add(prod4, int)?;
        let prod5 = v.mul(x.re(), int)?;
        let prod6 = u.mul(y.re(), int)?;
        let imag2 = prod5.add(-prod6, int)?;
        let multiplicand = Self::new(
            Complex {
                real: real2.value,
                imag: imag2.value,
            },
            real2.exact && imag2.exact,
        );
        let result = Self::new(
            Complex {
                real: real_part.value,
                imag: 0.into(),
            },
            real_part.exact,
        )
        .mul(&multiplicand, int)?;
        Ok(result)
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

impl From<Real> for Complex {
    fn from(i: Real) -> Self {
        Self {
            real: i,
            imag: 0.into(),
        }
    }
}
