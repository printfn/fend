use crate::err::{IntErr, Interrupt, Never};
use crate::num::complex::Complex;
use crate::num::{Base, FormattingStyle};
use std::cmp::Ordering;
use std::fmt::{Error, Formatter};
use std::ops::Neg;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExactBase {
    value: Complex,
    exact: bool,
    base: Base,
    format: FormattingStyle,
}

impl ExactBase {
    pub fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, IntErr<String, I>> {
        if !self.exact {
            return Err("Cannot convert inexact number to integer".to_string())?;
        }
        Ok(self.value.try_as_usize(int)?)
    }

    pub fn make_approximate(self) -> Self {
        Self {
            exact: false,
            ..self
        }
    }

    pub fn conjugate(self) -> Self {
        Self {
            value: self.value.conjugate(),
            ..self
        }
    }

    pub fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self {
            value: self.value.factorial(int)?,
            ..self
        })
    }

    pub fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self {
            value: self.value.div(rhs.value, int)?,
            exact: require_both_exact(self.exact, rhs.exact),
            ..self
        })
    }

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (value, exact_root) = self.value.pow(rhs.value, int)?;
        Ok(Self {
            value,
            exact: require_both_exact(require_both_exact(self.exact, rhs.exact), exact_root),
            ..self
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
    pub fn add_digit_in_base<I: Interrupt>(
        &mut self,
        digit: u64,
        base: Base,
        int: &I,
    ) -> Result<(), IntErr<String, I>> {
        if base != self.base {
            return Err(format!(
                "Base does not match: {} != {}",
                base.base_as_u8(),
                self.base.base_as_u8()
            ))?;
        }
        self.value
            .add_digit_in_base(digit, base.base_as_u8(), int)?;
        Ok(())
    }

    pub fn i() -> Self {
        Self {
            value: Complex::i(),
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }

    pub fn abs<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (new_value, res_exact) = self.value.abs(int)?;
        Ok(Self {
            value: new_value,
            exact: require_both_exact(self.exact, res_exact),
            ..self
        })
    }

    pub fn format<I: Interrupt>(
        &self,
        f: &mut Formatter,
        use_parentheses_if_complex: bool,
        int: &I,
    ) -> Result<(), IntErr<Error, I>> {
        if !self.exact {
            write!(f, "approx. ")?;
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
        Self { format, ..self }
    }

    pub fn with_base(self, base: Base) -> Self {
        Self { base, ..self }
    }

    pub const fn get_format(&self) -> FormattingStyle {
        self.format
    }

    pub fn root_n<I: Interrupt>(self, n: &Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (root, root_exact) = self.value.root_n(&n.value, int)?;
        Ok(Self {
            value: root,
            exact: self.exact && n.exact && root_exact,
            ..self
        })
    }

    fn apply_approx_fn<I: Interrupt>(
        self,
        f: impl FnOnce(Complex, &I) -> Result<Complex, IntErr<String, I>>,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(Self {
            value: f(self.value, int)?,
            exact: false,
            ..self
        })
    }

    pub fn sin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::sin, int)
    }

    pub fn cos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::cos, int)
    }

    pub fn tan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::tan, int)
    }

    pub fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::asin, int)
    }

    pub fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::acos, int)
    }

    pub fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::atan, int)
    }

    pub fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::sinh, int)
    }

    pub fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::cosh, int)
    }

    pub fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::tanh, int)
    }

    pub fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::asinh, int)
    }

    pub fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::acosh, int)
    }

    pub fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::atanh, int)
    }

    pub fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::ln, int)
    }

    pub fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::log2, int)
    }

    pub fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::log10, int)
    }

    pub fn exp<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_approx_fn(Complex::exp, int)
    }

    pub fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self {
            value: self.value.mul(&rhs.value, int)?,
            exact: require_both_exact(self.exact, rhs.exact),
            ..self
        })
    }

    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self {
            value: self.value.add(rhs.value, int)?,
            exact: require_both_exact(self.exact, rhs.exact),
            ..self
        })
    }

    pub fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        self.add(-rhs, int)
    }
}

impl PartialOrd for ExactBase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Neg for ExactBase {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            value: -self.value,
            ..self
        }
    }
}

impl Neg for &ExactBase {
    type Output = ExactBase;

    fn neg(self) -> ExactBase {
        -self.clone()
    }
}

impl From<u64> for ExactBase {
    fn from(i: u64) -> Self {
        Self {
            value: i.into(),
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }
}

const fn require_both_exact(a_exact: bool, b_exact: bool) -> bool {
    a_exact && b_exact
}
