use crate::error::{FendError, Interrupt};
use crate::num::bigrat::BigRat;
use crate::num::complex::Complex;
use std::collections::HashMap;
use std::fmt;
use std::ops::Neg;

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
            Err(_) => write!(f, "dist {{ {:?} }}", self.parts),
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
