use crate::num::complex::Complex;
use std::cmp::Ordering;
use std::fmt::{Display, Error, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExactBase {
    value: Complex,
    exact: bool,
    base: u8,
}

impl PartialOrd for ExactBase {
    fn partial_cmp(&self, other: &ExactBase) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Add for ExactBase {
    type Output = ExactBase;

    fn add(self, rhs: ExactBase) -> ExactBase {
        ExactBase {
            value: self.value + rhs.value,
            exact: self.exact && rhs.exact,
            base: self.base,
        }
    }
}

impl Neg for ExactBase {
    type Output = ExactBase;

    fn neg(self) -> ExactBase {
        ExactBase {
            value: -self.value,
            exact: self.exact,
            base: self.base,
        }
    }
}

impl Neg for &ExactBase {
    type Output = ExactBase;

    fn neg(self) -> ExactBase {
        -self.clone()
    }
}

impl Sub for ExactBase {
    type Output = ExactBase;

    fn sub(self, rhs: ExactBase) -> ExactBase {
        self + -rhs
    }
}

impl Sub for &ExactBase {
    type Output = ExactBase;

    fn sub(self, rhs: &ExactBase) -> ExactBase {
        self.clone() + -rhs.clone()
    }
}

impl Mul for ExactBase {
    type Output = ExactBase;

    fn mul(self, rhs: ExactBase) -> ExactBase {
        ExactBase {
            value: self.value * rhs.value,
            exact: self.exact && rhs.exact,
            base: self.base,
        }
    }
}

impl From<u64> for ExactBase {
    fn from(i: u64) -> Self {
        ExactBase {
            value: i.into(),
            exact: true,
            base: 10,
        }
    }
}

impl From<i32> for ExactBase {
    fn from(i: i32) -> Self {
        ExactBase {
            value: i.into(),
            exact: true,
            base: 10,
        }
    }
}

impl ExactBase {
    pub fn conjugate(self) -> Self {
        ExactBase {
            value: self.value.conjugate(),
            exact: self.exact,
            base: self.base,
        }
    }

    pub fn div(self, rhs: ExactBase) -> Result<ExactBase, String> {
        Ok(ExactBase {
            value: self.value.div(rhs.value)?,
            exact: self.exact && rhs.exact,
            base: self.base,
        })
    }

    pub fn pow(self, rhs: ExactBase) -> Result<ExactBase, String> {
        Ok(ExactBase {
            value: self.value.pow(rhs.value)?,
            exact: self.exact && rhs.exact,
            base: self.base,
        })
    }

    pub fn zero_with_base(base: u8) -> Self {
        Self {
            value: 0.into(),
            exact: true,
            base,
        }
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_digit_in_base(&mut self, digit: u64, base: u8) -> Result<(), String> {
        if base != self.base {
            return Err(format!("Base does not match: {} != {}", base, self.base));
        }
        self.value.add_digit_in_base(digit, base);
        Ok(())
    }

    pub fn i() -> ExactBase {
        ExactBase {
            value: Complex::i(),
            exact: true,
            base: 10,
        }
    }
}

impl Display for ExactBase {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        self.value.format(f, self.exact, self.base)?;
        Ok(())
    }
}

impl ExactBase {
    pub fn root_n(self, n: &ExactBase) -> Result<ExactBase, String> {
        let (root, root_exact) = self.value.root_n(&n.value)?;
        Ok(ExactBase {
            value: root,
            exact: self.exact && n.exact && root_exact,
            base: self.base,
        })
    }

    pub fn approx_pi() -> ExactBase {
        ExactBase {
            value: Complex::approx_pi(),
            exact: false,
            base: 10,
        }
    }
}
