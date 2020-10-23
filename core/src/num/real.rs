use crate::err::{IntErr, Interrupt, Never};
use crate::num::bigrat::{BigRat, FormattedBigRat};
use crate::num::Exact;
use crate::num::{Base, DivideByZero, FormattingStyle};
use std::cmp::Ordering;
use std::fmt;
use std::ops::Neg;

#[derive(Clone)]
pub struct Real {
    pattern: Pattern,
}

impl fmt::Debug for Real {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.pattern {
            Pattern::Simple(x) => write!(f, "{:?}", x),
            Pattern::Pi(x) => write!(f, "pi * {:?}", x),
        }
    }
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
        Ok(match self.pattern {
            Pattern::Simple(s) => s.sin(int)?.apply(Self::from),
            Pattern::Pi(n) => {
                if n < 0.into() {
                    let s = Self {
                        pattern: Pattern::Pi(n),
                    };
                    // sin(-x) == -sin(x)
                    return Ok(-Self::sin(-s, int)?);
                }
                if let Ok(integer) = n.clone().mul(&6.into(), int)?.try_as_usize(int) {
                    // values from https://en.wikipedia.org/wiki/Trigonometric_constants_expressed_in_real_radicals#Table_of_some_common_angles
                    if integer % 6 == 0 {
                        return Ok(Exact::new(Self::from(0), true));
                    } else if integer % 12 == 3 {
                        return Ok(Exact::new(Self::from(1), true));
                    } else if integer % 12 == 9 {
                        return Ok(Exact::new(-Self::from(1), true));
                    } else if integer % 12 == 1 || integer % 12 == 5 {
                        return Ok(Exact::new(Self::from(1), true)
                            .div(&Exact::new(2.into(), true), int)
                            .map_err(IntErr::unwrap)?);
                    } else if integer % 12 == 7 || integer % 12 == 11 {
                        return Ok(Exact::new(-Self::from(1), true)
                            .div(&Exact::new(2.into(), true), int)
                            .map_err(IntErr::unwrap)?);
                    }
                }
                let s = Self {
                    pattern: Pattern::Pi(n),
                };
                s.approximate(int)?.sin(int)?.apply(Self::from)
            }
        })
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
    ) -> Result<Exact<FormattedReal>, IntErr<Never, I>> {
        let mut pi = false;
        if style == FormattingStyle::Exact && self != &0.into() {
            if let Pattern::Pi(_) = self.pattern {
                pi = true;
            }
        }

        let term = match (imag, pi) {
            (false, false) => "",
            (false, true) => "π",
            (true, false) => "i",
            (true, true) => "πi",
        };

        let mut override_exact = true;

        let rat = match &self.pattern {
            Pattern::Simple(f) => f.clone(),
            Pattern::Pi(f) => {
                if pi {
                    f.clone()
                } else {
                    override_exact = false;
                    if style == FormattingStyle::Auto {
                        style = FormattingStyle::DecimalPlaces(10);
                    }
                    self.clone().approximate(int)?
                }
            }
        };

        let formatted = rat.format(base, style, term, use_parens_if_fraction, int)?;
        let exact = formatted.exact && override_exact;
        Ok(Exact::new(
            FormattedReal {
                num: formatted.value,
            },
            exact,
        ))
    }

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        // x^1 == x
        if let Pattern::Simple(n) = &rhs.pattern {
            if n == &1.into() {
                return Ok(Exact::new(self, true));
            }
        }

        // 1^x == 1
        if let Pattern::Simple(n) = &self.pattern {
            if n == &1.into() {
                return Ok(Exact::new(1.into(), true));
            }
        }

        if let (Pattern::Simple(a), Pattern::Simple(b)) =
            (self.clone().pattern, rhs.clone().pattern)
        {
            Ok(a.pow(b, int)?.apply(Self::from))
        } else {
            Ok(self
                .approximate(int)?
                .pow(rhs.approximate(int)?, int)?
                .combine(false)
                .apply(Self::from))
        }
    }

    pub fn root_n<I: Interrupt>(self, n: &Self, int: &I) -> Result<Exact<Self>, IntErr<String, I>> {
        // TODO: Combining these match blocks is not currently possible because
        // 'binding by-move and by-ref in the same pattern is unstable'
        // https://github.com/rust-lang/rust/pull/76119
        Ok(match self.pattern {
            Pattern::Simple(a) => match &n.pattern {
                Pattern::Simple(b) => a.root_n(b, int)?.apply(Self::from),
                Pattern::Pi(_) => {
                    let b = n.clone().approximate(int)?;
                    a.root_n(&b, int)?.apply(Self::from).combine(false)
                }
            },
            Pattern::Pi(_) => {
                let a = self.clone().approximate(int)?;
                let b = n.clone().approximate(int)?;
                a.root_n(&b, int)?.apply(Self::from).combine(false)
            }
        })
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
        Ok(
            match (self.clone().value.pattern, rhs.clone().value.pattern) {
                (Pattern::Simple(a), Pattern::Simple(b)) => {
                    Self::new(a.add(b, int)?.into(), args_exact)
                }
                (Pattern::Pi(a), Pattern::Pi(b)) => Self::new(
                    Real {
                        pattern: Pattern::Pi(a.add(b, int)?),
                    },
                    args_exact,
                ),
                _ => {
                    let a = self.value.approximate(int)?;
                    let b = rhs.value.approximate(int)?;
                    Self::new(a.add(b, int)?.into(), false)
                }
            },
        )
    }

    pub fn mul<I: Interrupt>(self, rhs: Exact<&Real>, int: &I) -> Result<Self, IntErr<Never, I>> {
        if self.exact && self.value == 0.into() {
            return Ok(self);
        } else if rhs.exact && rhs.value == &0.into() {
            return Ok(Self::new(rhs.value.clone(), rhs.exact));
        }
        let args_exact = self.exact && rhs.exact;
        Ok(match self.value.pattern {
            Pattern::Simple(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new(a.mul(b, int)?.into(), args_exact),
                Pattern::Pi(b) => Self::new(
                    Real {
                        pattern: Pattern::Pi(a.mul(b, int)?),
                    },
                    args_exact,
                ),
            },
            Pattern::Pi(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new(
                    Real {
                        pattern: Pattern::Pi(a.mul(b, int)?),
                    },
                    args_exact,
                ),
                Pattern::Pi(_) => Self::new(
                    Real {
                        pattern: Pattern::Pi(a.mul(&rhs.value.clone().approximate(int)?, int)?),
                    },
                    false,
                ),
            },
        })
    }

    pub fn div<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
        if rhs.value == 0.into() {
            return Err(DivideByZero {})?;
        }
        if self.exact && self.value == 0.into() {
            return Ok(self);
        }
        Ok(match self.value.pattern {
            Pattern::Simple(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new(a.div(b, int)?.into(), self.exact && rhs.exact),
                Pattern::Pi(_) => Self::new(
                    a.div(&rhs.value.clone().approximate(int)?, int)?.into(),
                    false,
                ),
            },
            Pattern::Pi(a) => match &rhs.value.pattern {
                Pattern::Simple(b) => Self::new(
                    Real {
                        pattern: Pattern::Pi(a.div(b, int)?),
                    },
                    self.exact && rhs.exact,
                ),
                Pattern::Pi(b) => Self::new(a.div(b, int)?.into(), self.exact && rhs.exact),
            },
        })
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

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct FormattedReal {
    num: FormattedBigRat,
}

impl fmt::Display for FormattedReal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.num)
    }
}
