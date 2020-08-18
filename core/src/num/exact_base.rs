use crate::num::complex::Complex;
use crate::num::Base;
use std::cmp::Ordering;
use std::fmt::{Error, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExactBase {
    value: Complex,
    exact: bool,
    base: Base,
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
            base: Base::Decimal,
        }
    }
}

impl From<i32> for ExactBase {
    fn from(i: i32) -> Self {
        ExactBase {
            value: i.into(),
            exact: true,
            base: Base::Decimal,
        }
    }
}

impl ExactBase {
    pub fn make_approximate(self) -> Self {
        ExactBase {
            value: self.value,
            exact: false,
            base: self.base,
        }
    }

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

    pub fn zero_with_base(base: Base) -> Self {
        Self {
            value: 0.into(),
            exact: true,
            base,
        }
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_digit_in_base(&mut self, digit: u64, base: Base) -> Result<(), String> {
        if base != self.base {
            return Err(format!(
                "Base does not match: {} != {}",
                base.base_as_u8(),
                self.base.base_as_u8()
            ));
        }
        self.value.add_digit_in_base(digit, base.base_as_u8());
        Ok(())
    }

    pub fn i() -> ExactBase {
        ExactBase {
            value: Complex::i(),
            exact: true,
            base: Base::Decimal,
        }
    }

    pub fn format(&self, f: &mut Formatter, use_parentheses_if_complex: bool) -> Result<(), Error> {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        self.value.format(f, self.exact, self.base, use_parentheses_if_complex)?;
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
            base: Base::Decimal,
        }
    }
}
