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

impl ExactBase {
    #[allow(clippy::missing_const_for_fn)]
    pub fn make_approximate(self) -> Self {
        Self {
            value: self.value,
            exact: false,
            base: self.base,
        }
    }

    pub fn conjugate(self) -> Self {
        Self {
            value: self.value.conjugate(),
            exact: self.exact,
            base: self.base,
        }
    }

    pub fn div(self, rhs: Self) -> Result<Self, String> {
        Ok(Self {
            value: self.value.div(rhs.value)?,
            exact: require_both_exact(self.exact, rhs.exact),
            base: self.base,
        })
    }

    pub fn pow(self, rhs: Self) -> Result<Self, String> {
        Ok(Self {
            value: self.value.pow(rhs.value)?,
            exact: require_both_exact(self.exact, rhs.exact),
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

    pub fn i() -> Self {
        Self {
            value: Complex::i(),
            exact: true,
            base: Base::Decimal,
        }
    }

    pub fn abs(self) -> Result<Self, String> {
        let (new_value, res_exact) = self.value.abs()?;
        Ok(Self {
            value: new_value,
            exact: require_both_exact(self.exact, res_exact),
            base: self.base,
        })
    }

    pub fn format(&self, f: &mut Formatter, use_parentheses_if_complex: bool) -> Result<(), Error> {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        self.value
            .format(f, self.exact, self.base, use_parentheses_if_complex)?;
        Ok(())
    }

    pub fn root_n(self, n: &Self) -> Result<Self, String> {
        let (root, root_exact) = self.value.root_n(&n.value)?;
        Ok(Self {
            value: root,
            exact: self.exact && n.exact && root_exact,
            base: self.base,
        })
    }

    pub fn approx_pi() -> Self {
        Self {
            value: Complex::approx_pi(),
            exact: false,
            base: Base::Decimal,
        }
    }
}

impl PartialOrd for ExactBase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Add for ExactBase {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            value: self.value + rhs.value,
            exact: require_both_exact(self.exact, rhs.exact),
            base: self.base,
        }
    }
}

impl Neg for ExactBase {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
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
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + -rhs
    }
}

impl Sub for &ExactBase {
    type Output = ExactBase;

    fn sub(self, rhs: Self) -> ExactBase {
        self.clone() + -rhs.clone()
    }
}

impl Mul for ExactBase {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            value: self.value * rhs.value,
            exact: require_both_exact(self.exact, rhs.exact),
            base: self.base,
        }
    }
}

impl From<u64> for ExactBase {
    fn from(i: u64) -> Self {
        Self {
            value: i.into(),
            exact: true,
            base: Base::Decimal,
        }
    }
}

fn require_both_exact(a_exact: bool, b_exact: bool) -> bool {
    a_exact && b_exact
}
