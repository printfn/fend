use crate::err::{IntErr, Interrupt, Never};
use crate::num::bigrat::BigRat;
use crate::num::Exact;
use crate::num::{Base, DivideByZero, FormattingStyle};
use std::cmp::Ordering;
use std::fmt;
use std::ops::Neg;

#[derive(Clone, Debug)]
pub struct Real {
    pattern: Pattern,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    /// a simple fraction
    Simple(BigRat),
    // n * pi
    Pi(BigRat),
}

impl Ord for Real {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.pattern, &other.pattern) {
            (Pattern::Simple(a), Pattern::Simple(b)) | (Pattern::Pi(a), Pattern::Pi(b)) => a.cmp(b),
            _ => {
                let int = &crate::interrupt::Never::default();
                let a = self.clone().approximate(int).unwrap();
                let b = other.clone().approximate(int).unwrap();
                a.cmp(&b)
            }
        }
    }
}

impl PartialOrd for Real {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Real {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Real {}

impl Real {
    fn approximate<I: Interrupt>(self, int: &I) -> Result<BigRat, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(s),
            Pattern::Pi(n) => {
                let num = BigRat::from(3_141_592_653_589_793_238);
                let den = BigRat::from(1_000_000_000_000_000_000);
                let pi = num.div(&den, int).map_err(IntErr::unwrap)?;
                Ok(n.mul(&pi, int)?)
            }
        }
    }

    pub fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => s.try_as_usize(int),
            Pattern::Pi(n) => {
                if n == 0.into() {
                    Ok(0)
                } else {
                    Err("Number cannot be converted to an integer".to_string())?
                }
            }
        }
    }

    pub fn into_f64<I: Interrupt>(self, int: &I) -> Result<f64, IntErr<Never, I>> {
        self.approximate(int)?.into_f64(int)
    }

    pub fn from_f64<I: Interrupt>(f: f64, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from(BigRat::from_f64(f, int)?))
    }

    // sin works for all real numbers
    pub fn sin<I: Interrupt>(self, int: &I) -> Result<Exact<Self>, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => {
                let (res, exact) = s.sin(int)?;
                Exact::new_ok(Self::from(res), exact)
            }
            Pattern::Pi(n) => {
                if n < 0.into() {
                    let s = Self {
                        pattern: Pattern::Pi(n),
                    };
                    return Ok(-Self::sin(-s, int)?);
                }
                if let Ok(integer) = n.clone().mul(&2.into(), int)?.try_as_usize(int) {
                    if integer % 2 == 0 {
                        Exact::new_ok(Self::from(0), true)
                    } else if integer % 4 == 1 {
                        Exact::new_ok(Self::from(1), true)
                    } else {
                        Exact::new_ok(-Self::from(1), true)
                    }
                } else {
                    let (res, _) = Self {
                        pattern: Pattern::Pi(n),
                    }
                    .approximate(int)?
                    .sin(int)?;
                    Exact::new_ok(Self::from(res), false)
                }
            }
        }
    }

    pub fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.asin(int)?))
    }

    pub fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.acos(int)?))
    }

    // note that this works for any real number, unlike asin and acos
    pub fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from(self.approximate(int)?.atan(int)?))
    }

    pub fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from(self.approximate(int)?.sinh(int)?))
    }

    pub fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from(self.approximate(int)?.cosh(int)?))
    }

    pub fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from(self.approximate(int)?.tanh(int)?))
    }

    pub fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from(self.approximate(int)?.asinh(int)?))
    }

    pub fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.acosh(int)?))
    }

    // value must be between -1 and 1.
    pub fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.atanh(int)?))
    }

    // For all logs: value must be greater than 0
    pub fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.ln(int)?))
    }

    pub fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.log2(int)?))
    }

    pub fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.log10(int)?))
    }

    pub fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self::from(self.approximate(int)?.factorial(int)?))
    }

    pub fn format<I: Interrupt>(
        &self,
        base: Base,
        mut style: FormattingStyle,
        imag: bool,
        use_parens_if_fraction: bool,
        int: &I,
    ) -> Result<(String, bool), IntErr<fmt::Error, I>> {
        if style == FormattingStyle::Auto {
            if let Pattern::Pi(_) = self.pattern {
                style = FormattingStyle::ApproxFloat(10);
            } else {
                style = FormattingStyle::ExactFloatWithFractionFallback;
            }
        }

        let s = self.clone().approximate(int)?;
        let (string, x) = crate::num::to_string(|f| {
            let x = s.format(f, base, style, imag, use_parens_if_fraction, int)?;
            write!(f, "{}", x)?;
            Ok(x)
        })?;
        Ok((string, x.exact))
    }

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<(Self, bool), IntErr<String, I>> {
        if let (Pattern::Simple(a), Pattern::Simple(b)) =
            (self.clone().pattern, rhs.clone().pattern)
        {
            let (result, exact) = a.pow(b, int)?;
            Ok((Self::from(result), exact))
        } else {
            let (result, _) = self.approximate(int)?.pow(rhs.approximate(int)?, int)?;
            Ok((Self::from(result), false))
        }
    }

    pub fn root_n<I: Interrupt>(
        self,
        n: &Self,
        int: &I,
    ) -> Result<(Self, bool), IntErr<String, I>> {
        // TODO: Combining these match blocks is not currently possible because
        // 'binding by-move and by-ref in the same pattern is unstable'
        // https://github.com/rust-lang/rust/pull/76119
        match self.pattern {
            Pattern::Simple(a) => match &n.pattern {
                Pattern::Simple(b) => {
                    let (res, exact) = a.root_n(b, int)?;
                    Ok((Self::from(res), exact))
                }
                Pattern::Pi(_) => {
                    let b = n.clone().approximate(int)?;
                    let (res, _) = a.root_n(&b, int)?;
                    Ok((Self::from(res), false))
                }
            },
            Pattern::Pi(_) => {
                let a = self.clone().approximate(int)?;
                let b = n.clone().approximate(int)?;
                let (res, _) = a.root_n(&b, int)?;
                Ok((Self::from(res), false))
            }
        }
    }

    pub fn pi() -> Self {
        Self {
            pattern: Pattern::Pi(1.into()),
        }
    }
}

#[allow(clippy::use_self)]
impl Exact<Real> {
    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        if self.exact && self.value == 0.into() {
            return Ok(rhs);
        } else if rhs.exact && rhs.value == 0.into() {
            return Ok(self);
        }
        let args_exact = self.exact && rhs.exact;
        match (self.clone().value.pattern, rhs.clone().value.pattern) {
            (Pattern::Simple(a), Pattern::Simple(b)) => Self::new_ok(a.add(b, int)?, args_exact),
            (Pattern::Pi(a), Pattern::Pi(b)) => Self::new_ok(
                Real {
                    pattern: Pattern::Pi(a.add(b, int)?),
                },
                args_exact,
            ),
            _ => {
                let a = self.value.approximate(int)?;
                let b = rhs.value.approximate(int)?;
                Self::new_ok(a.add(b, int)?, false)
            }
        }
    }

    pub fn mul<I: Interrupt>(self, rhs: Exact<&Real>, int: &I) -> Result<Self, IntErr<Never, I>> {
        let args_exact = self.exact && rhs.exact;
        match self.value.pattern {
            Pattern::Simple(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new_ok(a.mul(b, int)?, args_exact),
                Pattern::Pi(b) => Self::new_ok(
                    Real {
                        pattern: Pattern::Pi(a.mul(b, int)?),
                    },
                    args_exact,
                ),
            },
            Pattern::Pi(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new_ok(
                    Real {
                        pattern: Pattern::Pi(a.mul(b, int)?),
                    },
                    args_exact,
                ),
                Pattern::Pi(_) => Self::new_ok(
                    Real {
                        pattern: Pattern::Pi(a.mul(&rhs.value.clone().approximate(int)?, int)?),
                    },
                    false,
                ),
            },
        }
    }

    pub fn div<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
        if self.exact && self.value == 0.into() {
            return Ok(self);
        }
        match self.value.pattern {
            Pattern::Simple(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new_ok(a.div(b, int)?, self.exact && rhs.exact),
                Pattern::Pi(_) => {
                    Self::new_ok(a.div(&rhs.value.clone().approximate(int)?, int)?, false)
                }
            },
            Pattern::Pi(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new_ok(
                    Real {
                        pattern: Pattern::Pi(a.div(b, int)?),
                    },
                    self.exact && rhs.exact,
                ),
                Pattern::Pi(b) => Self::new_ok(a.div(b, int)?, self.exact && rhs.exact),
            },
        }
    }
}

impl Neg for Real {
    type Output = Self;

    fn neg(self) -> Self {
        match self.pattern {
            Pattern::Simple(s) => Self::from(-s),
            Pattern::Pi(n) => Self {
                pattern: Pattern::Pi(-n),
            },
        }
    }
}

impl From<u64> for Real {
    fn from(i: u64) -> Self {
        Self {
            pattern: Pattern::Simple(i.into()),
        }
    }
}

impl From<BigRat> for Real {
    fn from(n: BigRat) -> Self {
        Self {
            pattern: Pattern::Simple(n),
        }
    }
}
