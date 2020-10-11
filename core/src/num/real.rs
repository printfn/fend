use crate::err::{IntErr, Interrupt, Never};
use crate::num::bigrat::BigRat;
use crate::num::{Base, DivideByZero, FormattingStyle};
use std::cmp::Ordering;
use std::fmt::{Debug, Error};
use std::ops::Neg;

#[derive(Clone, Debug)]
pub enum Real {
    Simple(BigRat),
    //Pi,
}

impl Ord for Real {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Simple(a), Self::Simple(b)) => a.cmp(b),
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
        match self {
            Self::Simple(s) => s.try_as_usize(int),
        }
    }

    pub fn into_f64<I: Interrupt>(self, int: &I) -> Result<f64, IntErr<Never, I>> {
        match self {
            Self::Simple(s) => s.into_f64(int),
        }
    }

    pub fn from_f64<I: Interrupt>(f: f64, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::Simple(BigRat::from_f64(f, int)?))
    }

    // sin and cos work for all real numbers
    pub fn sin<I: Interrupt>(self, int: &I) -> Result<(Self, bool), IntErr<Never, I>> {
        match self {
            Self::Simple(s) => {
                let (res, exact) = s.sin(int)?;
                Ok((Self::Simple(res), exact))
            }
        }
    }

    pub fn cos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self::from_f64(f64::cos(self.into_f64(int)?), int)?)
    }

    pub fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.asin(int)?)),
        }
    }

    pub fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.acos(int)?)),
        }
    }

    // note that this works for any real number, unlike asin and acos
    pub fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.atan(int)?)),
        }
    }

    pub fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.sinh(int)?)),
        }
    }

    pub fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.cosh(int)?)),
        }
    }

    pub fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.tanh(int)?)),
        }
    }

    pub fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.asinh(int)?)),
        }
    }

    pub fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.acosh(int)?)),
        }
    }

    // value must be between -1 and 1.
    pub fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.atanh(int)?)),
        }
    }

    // For all logs: value must be greater than 0
    pub fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.ln(int)?)),
        }
    }

    pub fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.log2(int)?)),
        }
    }

    pub fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.log10(int)?)),
        }
    }

    pub fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Simple(s) => Ok(Self::Simple(s.factorial(int)?)),
        }
    }

    pub fn div<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
        match self {
            Self::Simple(a) => match rhs {
                Self::Simple(b) => Ok(Self::Simple(a.div(b, int)?)),
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
        match self {
            Self::Simple(s) => s.add_digit_in_base(digit, base, rec, int),
        }
    }

    pub fn format<I: Interrupt>(
        &self,
        base: Base,
        style: FormattingStyle,
        imag: bool,
        int: &I,
    ) -> Result<(String, bool), IntErr<Error, I>> {
        match self {
            Self::Simple(s) => {
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
        match (self, rhs) {
            (Self::Simple(a), Self::Simple(b)) => {
                let (result, exact) = a.pow(b, int)?;
                Ok((Self::Simple(result), exact))
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
        match self {
            Self::Simple(a) => match n {
                Self::Simple(b) => {
                    let (res, exact) = a.root_n(b, int)?;
                    Ok((Self::Simple(res), exact))
                }
            },
        }
    }

    pub fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match self {
            Self::Simple(a) => match rhs {
                Self::Simple(b) => Ok(Self::Simple(a.mul(b, int)?)),
            },
        }
    }

    pub fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match (self, rhs) {
            (Self::Simple(a), Self::Simple(b)) => Ok(Self::Simple(a.sub(b, int)?)),
        }
    }

    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        match (self, rhs) {
            (Self::Simple(a), Self::Simple(b)) => Ok(Self::Simple(a.add(b, int)?)),
        }
    }

    pub fn pi<I: Interrupt>(int: &I) -> Result<Self, IntErr<Never, I>> {
        let num = BigRat::from(3_141_592_653_589_793_238);
        let den = BigRat::from(1_000_000_000_000_000_000);
        Ok(Self::Simple(num.div(&den, int).map_err(IntErr::unwrap)?))
    }
}

impl Neg for Real {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Self::Simple(s) => Self::Simple(-s),
        }
    }
}

impl From<u64> for Real {
    fn from(i: u64) -> Self {
        Self::Simple(i.into())
    }
}

impl From<BigRat> for Real {
    fn from(n: BigRat) -> Self {
        Self::Simple(n)
    }
}
