use crate::interrupt::Interrupt;
use crate::num::complex::Complex;
use crate::num::{Base, FormattingStyle};
use std::cmp::Ordering;
use std::fmt::{Error, Formatter};
use std::ops::{Add, Neg, Sub};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExactBase {
    value: Complex,
    exact: bool,
    base: Base,
    format: FormattingStyle,
}

impl ExactBase {
    pub fn try_as_usize(self, int: &impl Interrupt) -> Result<usize, String> {
        if !self.exact {
            return Err("Cannot convert inexact number to integer".to_string());
        }
        Ok(self.value.try_as_usize(int)?)
    }

    pub fn make_approximate(self) -> Self {
        Self {
            value: self.value,
            exact: false,
            base: self.base,
            format: self.format,
        }
    }

    pub fn conjugate(self) -> Self {
        Self {
            value: self.value.conjugate(),
            exact: self.exact,
            base: self.base,
            format: self.format,
        }
    }

    pub fn factorial(self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self {
            value: self.value.factorial(int)?,
            exact: self.exact,
            base: self.base,
            format: self.format,
        })
    }

    pub fn div(self, rhs: Self, int: &impl Interrupt) -> Result<Self, String> {
        Ok(Self {
            value: self.value.div(rhs.value, int)?,
            exact: require_both_exact(self.exact, rhs.exact),
            base: self.base,
            format: self.format,
        })
    }

    pub fn pow(self, rhs: Self, int: &impl Interrupt) -> Result<Self, String> {
        let (value, exact_root) = self.value.pow(rhs.value, int)?;
        Ok(Self {
            value,
            exact: require_both_exact(require_both_exact(self.exact, rhs.exact), exact_root),
            base: self.base,
            format: self.format,
        })
    }

    pub fn zero_with_base(base: Base) -> Self {
        Self {
            value: 0.into(),
            exact: true,
            base,
            format: FormattingStyle::default(),
        }
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_digit_in_base(
        &mut self,
        digit: u64,
        base: Base,
        int: &impl Interrupt,
    ) -> Result<(), String> {
        if base != self.base {
            return Err(format!(
                "Base does not match: {} != {}",
                base.base_as_u8(),
                self.base.base_as_u8()
            ));
        }
        self.value
            .add_digit_in_base(digit, base.base_as_u8(), int)?;
        Ok(())
    }

    pub fn i() -> Self {
        Self {
            value: Complex::i(),
            exact: true,
            base: Base::Decimal,
            format: FormattingStyle::default(),
        }
    }

    pub fn abs(self, int: &impl Interrupt) -> Result<Self, String> {
        let (new_value, res_exact) = self.value.abs(int)?;
        Ok(Self {
            value: new_value,
            exact: require_both_exact(self.exact, res_exact),
            base: self.base,
            format: self.format,
        })
    }

    pub fn format(
        &self,
        f: &mut Formatter,
        use_parentheses_if_complex: bool,
        int: &impl Interrupt,
    ) -> Result<Result<(), Error>, crate::err::Interrupt> {
        if !self.exact {
            if let Err(e) = write!(f, "approx. ") {
                return Ok(Err(e));
            }
        }
        self.value.format(
            f,
            self.exact,
            self.format,
            self.base,
            use_parentheses_if_complex,
            int,
        )
    }

    pub fn with_format(self, format: FormattingStyle) -> Self {
        Self {
            value: self.value,
            exact: self.exact,
            base: self.base,
            format,
        }
    }

    pub const fn get_format(&self) -> FormattingStyle {
        self.format
    }

    pub fn root_n(self, n: &Self, int: &impl Interrupt) -> Result<Self, String> {
        let (root, root_exact) = self.value.root_n(&n.value, int)?;
        Ok(Self {
            value: root,
            exact: self.exact && n.exact && root_exact,
            base: self.base,
            format: self.format,
        })
    }

    pub fn approx_pi() -> Self {
        Self {
            value: Complex::approx_pi(),
            exact: false,
            base: Base::Decimal,
            format: FormattingStyle::default(),
        }
    }

    pub fn approx_e() -> Self {
        Self {
            value: Complex::approx_e(),
            exact: false,
            base: Base::Decimal,
            format: FormattingStyle::default(),
        }
    }

    fn apply_approx_fn<I: Interrupt>(
        self,
        f: impl FnOnce(Complex, &I) -> Result<Complex, String>,
        int: &I,
    ) -> Result<Self, String> {
        Ok(Self {
            value: f(self.value, int)?,
            exact: false,
            base: self.base,
            format: self.format,
        })
    }

    pub fn sin(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::sin, int)
    }

    pub fn cos(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::cos, int)
    }

    pub fn tan(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::tan, int)
    }

    pub fn asin(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::asin, int)
    }

    pub fn acos(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::acos, int)
    }

    pub fn atan(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::atan, int)
    }

    pub fn sinh(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::sinh, int)
    }

    pub fn cosh(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::cosh, int)
    }

    pub fn tanh(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::tanh, int)
    }

    pub fn asinh(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::asinh, int)
    }

    pub fn acosh(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::acosh, int)
    }

    pub fn atanh(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::atanh, int)
    }

    pub fn ln(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::ln, int)
    }

    pub fn log2(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::log2, int)
    }

    pub fn log10(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::log10, int)
    }

    pub fn exp(self, int: &impl Interrupt) -> Result<Self, String> {
        self.apply_approx_fn(Complex::exp, int)
    }

    pub fn mul(self, rhs: &Self, int: &impl Interrupt) -> Result<Self, crate::err::Interrupt> {
        Ok(Self {
            value: self.value.mul(&rhs.value, int)?,
            exact: require_both_exact(self.exact, rhs.exact),
            base: self.base,
            format: self.format,
        })
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
            format: self.format,
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
            format: self.format,
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

impl From<u64> for ExactBase {
    fn from(i: u64) -> Self {
        Self {
            value: i.into(),
            exact: true,
            base: Base::Decimal,
            format: FormattingStyle::default(),
        }
    }
}

fn require_both_exact(a_exact: bool, b_exact: bool) -> bool {
    a_exact && b_exact
}
