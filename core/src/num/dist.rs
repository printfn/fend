use crate::error::{FendError, Interrupt};
use crate::format::Format;
use crate::num::bigrat::{self, BigRat};
use crate::num::complex::{self, Complex};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::ops::Neg;

use super::{Base, Exact, FormattingStyle};

#[derive(Clone)]
pub(crate) struct Dist {
    // invariant: probabilities must sum to 1
    parts: HashMap<Complex, BigRat>,
}

impl Dist {
    pub(crate) fn one_point(self) -> Result<Complex, FendError> {
        if self.parts.len() == 1 {
            Ok(self.parts.into_iter().next().unwrap().0)
        } else {
            Err(FendError::ProbabilityDistributionsNotAllowed)
        }
    }

    pub(crate) fn one_point_ref(&self) -> Result<&Complex, FendError> {
        if self.parts.len() == 1 {
            Ok(self.parts.iter().next().unwrap().0)
        } else {
            Err(FendError::ProbabilityDistributionsNotAllowed)
        }
    }

    pub(crate) fn new_die<I: Interrupt>(n: u32, int: &I) -> Result<Self, FendError> {
        assert!(n != 0);
        let mut hashmap = HashMap::new();
        let probability = BigRat::from(1).div(&BigRat::from(u64::from(n)), int)?;
        for i in 1..=n {
            hashmap.insert(Complex::from(u64::from(i)), probability.clone());
        }
        Ok(Self { parts: hashmap })
    }

    pub(crate) fn equals_int(&self, val: u64) -> bool {
        self.parts.len() == 1 && self.parts.keys().next().unwrap() == &val.into()
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub(crate) fn sample<I: Interrupt>(
        self,
        ctx: &crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        if self.parts.len() == 1 {
            return Ok(self);
        }
        let mut random = ctx.random_u32.ok_or(FendError::RandomNumbersNotAvailable)?();
        let mut res = None;
        for (k, v) in self.parts {
            random = random.saturating_sub((v.into_f64(int)? * f64::from(u32::MAX)) as u32);
            if random == 0 {
                return Ok(Self::from(k));
            }
            res = Some(Self::from(k));
        }
        Ok(res.expect("there must be at least one part in a dist"))
    }

    pub(crate) fn format<I: Interrupt>(
        &self,
        exact: bool,
        style: FormattingStyle,
        base: Base,
        use_parentheses: complex::UseParentheses,
        out: &mut String,
        int: &I,
    ) -> Result<Exact<()>, FendError> {
        if self.parts.len() == 1 {
            let res = self.parts.iter().next().unwrap().0.format(
                exact,
                style,
                base,
                use_parentheses,
                int,
            )?;
            write!(out, "{}", res.value)?;
            Ok(Exact::new((), res.exact))
        } else {
            let mut ordered_kvs = vec![];
            for kv in &self.parts {
                ordered_kvs.push(kv);
            }
            ordered_kvs
                .sort_unstable_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let mut first = true;
            for (num, prob) in &ordered_kvs {
                if first {
                    first = false;
                } else {
                    writeln!(out)?;
                }
                write!(
                    out,
                    "{}: {}",
                    num.format(exact, style, base, use_parentheses, int)?.value,
                    prob.format(
                        &bigrat::FormatOptions {
                            base,
                            style: FormattingStyle::ImproperFraction,
                            term: "",
                            use_parens_if_fraction: false
                        },
                        int
                    )?
                    .value
                )?;
            }
            // TODO check exactness
            Ok(Exact::new((), true))
        }
    }
}

impl From<Complex> for Dist {
    fn from(v: Complex) -> Self {
        let mut parts = HashMap::new();
        parts.insert(v, BigRat::from(1));
        Self { parts }
    }
}

impl From<u64> for Dist {
    fn from(i: u64) -> Self {
        Self::from(Complex::from(i))
    }
}

impl fmt::Debug for Dist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.one_point_ref() {
            Ok(complex) => write!(f, "{:?}", complex),
            Err(_) => write!(f, "dist {:?}", self.parts),
        }
    }
}

impl Neg for Dist {
    type Output = Self;
    fn neg(self) -> Self {
        let mut res = HashMap::new();
        for (k, v) in self.parts {
            res.insert(-k, v);
        }
        Self { parts: res }
    }
}
