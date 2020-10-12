use crate::err::{IntErr, Interrupt, Never};
use crate::num::bigrat::BigRat;
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
    //Pi(BigRat),
}

impl Ord for Real {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.pattern, &other.pattern) {
            (Pattern::Simple(a), Pattern::Simple(b)) => a.cmp(b),
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
    pub fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => s.try_as_usize(int),
        }
    }

    pub fn into_f64<I: Interrupt>(self, int: &I) -> Result<f64, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => s.into_f64(int),
        }
    }

    pub fn from_f64<I: Interrupt>(f: f64, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from(BigRat::from_f64(f, int)?))
    }

    // sin works for all real numbers
    pub fn sin<I: Interrupt>(self, int: &I) -> Result<(Self, bool), IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => {
                let (res, exact) = s.sin(int)?;
                Ok((Self::from(res), exact))
            }
        }
    }

    pub fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.asin(int)?)),
        }
    }

    pub fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.acos(int)?)),
        }
    }

    // note that this works for any real number, unlike asin and acos
    pub fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.atan(int)?)),
        }
    }

    pub fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.sinh(int)?)),
        }
    }

    pub fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.cosh(int)?)),
        }
    }

    pub fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.tanh(int)?)),
        }
    }

    pub fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.asinh(int)?)),
        }
    }

    pub fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.acosh(int)?)),
        }
    }

    // value must be between -1 and 1.
    pub fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.atanh(int)?)),
        }
    }

    // For all logs: value must be greater than 0
    pub fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.ln(int)?)),
        }
    }

    pub fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.log2(int)?)),
        }
    }

    pub fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.log10(int)?)),
        }
    }

    pub fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self.pattern {
            Pattern::Simple(s) => Ok(Self::from(s.factorial(int)?)),
        }
    }

    pub fn div<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
        match self.pattern {
            Pattern::Simple(a) => match &rhs.pattern {
                Pattern::Simple(b) => Ok(Self::from(a.div(b, int)?)),
            },
        }
    }

    pub fn add_digit_in_base<I: Interrupt>(
        &mut self,
        digit: u64,
        base: u8,
        rec: bool,
        int: &I,
    ) -> Result<(), IntErr<Never, I>> {
        match &mut self.pattern {
            Pattern::Simple(s) => s.add_digit_in_base(digit, base, rec, int),
        }
    }

    pub fn format<I: Interrupt>(
        &self,
        base: Base,
        style: FormattingStyle,
        imag: bool,
        int: &I,
    ) -> Result<(String, bool), IntErr<fmt::Error, I>> {
        match &self.pattern {
            Pattern::Simple(s) => {
                let (string, x) = crate::num::to_string(|f| {
                    let x = s.format(f, base, style, imag, int)?;
                    write!(f, "{}", x)?;
                    Ok(x)
                })?;
                Ok((string, x.exact))
            }
        }
    }

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<(Self, bool), IntErr<String, I>> {
        match (self.pattern, rhs.pattern) {
            (Pattern::Simple(a), Pattern::Simple(b)) => {
                let (result, exact) = a.pow(b, int)?;
                Ok((Self::from(result), exact))
            }
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
            },
        }
    }

    pub fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self.pattern {
            Pattern::Simple(a) => match &rhs.pattern {
                Pattern::Simple(b) => Ok(Self::from(a.mul(b, int)?)),
            },
        }
    }

    pub fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match (self.pattern, rhs.pattern) {
            (Pattern::Simple(a), Pattern::Simple(b)) => Ok(Self::from(a.sub(b, int)?)),
        }
    }

    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match (self.pattern, rhs.pattern) {
            (Pattern::Simple(a), Pattern::Simple(b)) => Ok(Self::from(a.add(b, int)?)),
        }
    }

    pub fn pi<I: Interrupt>(int: &I) -> Result<Self, IntErr<Never, I>> {
        let num = BigRat::from(3_141_592_653_589_793_238);
        let den = BigRat::from(1_000_000_000_000_000_000);
        Ok(Self::from(num.div(&den, int).map_err(IntErr::unwrap)?))
    }
}

impl Neg for Real {
    type Output = Self;

    fn neg(self) -> Self {
        match self.pattern {
            Pattern::Simple(s) => Self::from(-s),
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
