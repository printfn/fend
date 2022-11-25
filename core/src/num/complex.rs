use crate::error::{FendError, Interrupt};
use crate::num::real::{self, Real};
use crate::num::Exact;
use crate::num::{Base, FormattingStyle};
use std::cmp::Ordering;
use std::ops::Neg;
use std::{fmt, io};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Complex {
    real: Real,
    imag: Real,
}

impl fmt::Debug for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.real)?;
        if !self.imag.is_definitely_zero() {
            write!(f, " + {:?}i", self.imag)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum UseParentheses {
    No,
    IfComplex,
    IfComplexOrFraction,
}

impl Complex {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        self.real.serialize(write)?;
        self.imag.serialize(write)?;
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(Self {
            real: Real::deserialize(read)?,
            imag: Real::deserialize(read)?,
        })
    }

    pub(crate) fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, FendError> {
        if self.imag != 0.into() {
            return Err(FendError::ComplexToInteger);
        }
        self.real.try_as_usize(int)
    }

    pub(crate) fn conjugate(self) -> Self {
        Self {
            real: self.real,
            imag: -self.imag,
        }
    }

    pub(crate) fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        if self.imag != 0.into() {
            return Err(FendError::FactorialComplex);
        }
        Ok(Self {
            real: self.real.factorial(int)?,
            imag: self.imag,
        })
    }

    pub(crate) fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Exact<Self>, FendError> {
        if self.imag != 0.into() || rhs.imag != 0.into() {
            return Err(FendError::ExpComplex);
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

    pub(crate) fn i() -> Self {
        Self {
            real: 0.into(),
            imag: 1.into(),
        }
    }

    pub(crate) fn pi() -> Self {
        Self {
            real: Real::pi(),
            imag: 0.into(),
        }
    }

    pub(crate) fn abs<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, FendError> {
        Ok(if self.imag.is_zero() {
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
        } else if self.real.is_zero() {
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

    pub(crate) fn format<I: Interrupt>(
        &self,
        exact: bool,
        style: FormattingStyle,
        base: Base,
        use_parentheses: UseParentheses,
        int: &I,
    ) -> Result<Exact<Formatted>, FendError> {
        let style = if !exact && style == FormattingStyle::Auto {
            FormattingStyle::DecimalPlaces(10)
        } else if self.imag != 0.into() && style == FormattingStyle::Auto {
            FormattingStyle::Exact
        } else {
            style
        };

        if self.imag.is_zero() {
            let use_parens = use_parentheses == UseParentheses::IfComplexOrFraction;
            let x = self.real.format(base, style, false, use_parens, int)?;
            return Ok(Exact::new(
                Formatted {
                    first_component: x.value,
                    separator: "",
                    second_component: None,
                    use_parentheses: false,
                },
                exact && x.exact,
            ));
        }

        Ok(if self.real.is_zero() {
            let use_parens = use_parentheses == UseParentheses::IfComplexOrFraction;
            let x = self.imag.format(base, style, true, use_parens, int)?;
            Exact::new(
                Formatted {
                    first_component: x.value,
                    separator: "",
                    second_component: None,
                    use_parentheses: false,
                },
                exact && x.exact,
            )
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
            let separator = if positive { " + " } else { " - " };
            Exact::new(
                Formatted {
                    first_component: real_part.value,
                    separator,
                    second_component: Some(imag_part.value),
                    use_parentheses: use_parentheses == UseParentheses::IfComplex
                        || use_parentheses == UseParentheses::IfComplexOrFraction,
                },
                exact,
            )
        })
    }

    pub(crate) fn root_n<I: Interrupt>(self, n: &Self, int: &I) -> Result<Exact<Self>, FendError> {
        if self.imag != 0.into() || n.imag != 0.into() {
            return Err(FendError::RootsComplex);
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

    fn expect_real(self) -> Result<Real, FendError> {
        if self.imag.is_zero() {
            Ok(self.real)
        } else {
            Err(FendError::ExpectedARealNumber)
        }
    }

    pub(crate) fn sin<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, FendError> {
        Ok(self.expect_real()?.sin(int)?.apply(Self::from))
    }

    pub(crate) fn cos<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, FendError> {
        // cos(self) == sin(pi/2 - self)
        let pi = Exact::new(Self::pi(), true);
        let half_pi = pi.div(Exact::new(2.into(), true), int)?;
        let sin_arg = half_pi.add(-Exact::new(self, true), int)?;
        Ok(sin_arg
            .value
            .expect_real()?
            .sin(int)?
            .combine(sin_arg.exact)
            .apply(Self::from))
    }

    pub(crate) fn tan<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, FendError> {
        let num = self.clone().sin(int)?;
        let den = self.cos(int)?;
        num.div(den, int)
    }

    pub(crate) fn asin<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.asin(int)?))
    }

    pub(crate) fn acos<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.acos(int)?))
    }

    pub(crate) fn atan<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.atan(int)?))
    }

    pub(crate) fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.sinh(int)?))
    }

    pub(crate) fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.cosh(int)?))
    }

    pub(crate) fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.tanh(int)?))
    }

    pub(crate) fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.asinh(int)?))
    }

    pub(crate) fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.acosh(int)?))
    }

    pub(crate) fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.atanh(int)?))
    }

    pub(crate) fn ln<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.ln(int)?))
    }

    pub(crate) fn log2<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.log2(int)?))
    }

    pub(crate) fn log10<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.log10(int)?))
    }

    pub(crate) fn is_definitely_one(&self) -> bool {
        self.real.is_definitely_one() && self.imag.is_definitely_zero()
    }

    pub(crate) fn modulo<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(
            self.expect_real()?.modulo(rhs.expect_real()?, int)?,
        ))
    }

    pub(crate) fn bitwise<I: Interrupt>(
        self,
        rhs: Self,
        op: crate::ast::BitwiseBop,
        int: &I,
    ) -> Result<Self, FendError> {
        Ok(Self::from(self.expect_real()?.bitwise(
            rhs.expect_real()?,
            op,
            int,
        )?))
    }

    pub(crate) fn combination<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(
            self.expect_real()?.combination(rhs.expect_real()?, int)?,
        ))
    }

    pub(crate) fn permutation<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        Ok(Self::from(
            self.expect_real()?.permutation(rhs.expect_real()?, int)?,
        ))
    }
}

impl Exact<Complex> {
    pub(crate) fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
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

    pub(crate) fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, FendError> {
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

    pub(crate) fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        // (u + vi) / (x + yi) = (1/(x^2 + y^2)) * ((ux + vy) + (vx - uy)i)
        let (u, v) = self.apply(|x| (x.real, x.imag)).pair();
        let (x, y) = rhs.apply(|x| (x.real, x.imag)).pair();
        // if both numbers are real, use this simplified algorithm
        if v.exact && v.value.is_zero() && y.exact && y.value.is_zero() {
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

#[derive(Debug)]
pub(crate) struct Formatted {
    first_component: real::Formatted,
    separator: &'static str,
    second_component: Option<real::Formatted>,
    use_parentheses: bool,
}

impl fmt::Display for Formatted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.use_parentheses {
            write!(f, "(")?;
        }
        write!(f, "{}{}", self.first_component, self.separator)?;
        if let Some(second_component) = &self.second_component {
            write!(f, "{second_component}")?;
        }
        if self.use_parentheses {
            write!(f, ")")?;
        }
        Ok(())
    }
}
