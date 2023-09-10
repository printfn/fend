use crate::Interrupt;
use crate::error::FendError;
use crate::num::bigrat::sign::Sign;
use crate::num::biguint::BigUint;
use std::sync::Arc;
use std::{cmp, fmt, iter};

#[derive(Clone)]
pub(crate) struct ContinuedFraction {
	integer_sign: Sign,
	integer: BigUint,
	fraction: Arc<dyn Fn(usize) -> BigUint>, // returning zero indicates the end of the fraction
}

const MAX_ITERATIONS: usize = 50;

impl ContinuedFraction {
	fn actual_integer_sign(&self) -> Sign {
		match self.integer_sign {
			Sign::Positive => Sign::Positive,
			Sign::Negative => {
				if self.integer == 0.into() {
					Sign::Positive
				} else {
					Sign::Negative
				}
			}
		}
	}

	pub(crate) fn try_as_usize<I: Interrupt>(&self, int: &I) -> Result<usize, FendError> {
		if self.actual_integer_sign() == Sign::Negative {
			return Err(FendError::NegativeNumbersNotAllowed);
		}
		if (self.fraction)(0) != 0.into() {
			return Err(FendError::FractionToInteger);
		}
		self.integer.try_as_usize(int)
	}
}

pub(crate) struct CFIterator<'a> {
	cf: &'a ContinuedFraction,
	i: usize,
}

impl Iterator for CFIterator<'_> {
	type Item = BigUint;

	fn next(&mut self) -> Option<Self::Item> {
		let result = (self.cf.fraction)(self.i);
		if result == 0.into() {
			None
		} else {
			self.i += 1;
			Some(result)
		}
	}
}

impl iter::FusedIterator for CFIterator<'_> {}

impl<'a> IntoIterator for &'a ContinuedFraction {
	type Item = BigUint;

	type IntoIter = CFIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		CFIterator { cf: self, i: 0 }
	}
}

impl fmt::Debug for ContinuedFraction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[")?;
		if matches!(self.integer_sign, Sign::Negative) {
			write!(f, "-")?;
		}
		write!(f, "{:?}", self.integer)?;
		for (i, term) in self.into_iter().enumerate() {
			if i == 0 {
				write!(f, "; {term:?}")?;
			} else {
				write!(f, ", {term:?}")?;
			}
		}
		write!(f, "]")?;
		Ok(())
	}
}

impl From<BigUint> for ContinuedFraction {
	fn from(value: BigUint) -> Self {
		Self {
			integer_sign: Sign::Positive,
			integer: value,
			fraction: Arc::new(|_| 0.into()),
		}
	}
}

impl PartialOrd for ContinuedFraction {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for ContinuedFraction {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		let s = self.actual_integer_sign().cmp(&other.actual_integer_sign());
		if s != cmp::Ordering::Equal {
			return s;
		}
		let s = self.integer.cmp(&other.integer);
		if s != cmp::Ordering::Equal {
			return s;
		}
		if std::sync::Arc::ptr_eq(&self.fraction, &other.fraction) {
			return cmp::Ordering::Equal;
		}
		let mut reversed = true;
		let iter1 = self.into_iter().map(Some).chain(iter::repeat(None));
		let iter2 = other.into_iter().map(Some).chain(iter::repeat(None));
		for (a, b) in iter1.zip(iter2).take(MAX_ITERATIONS) {
			if a.is_none() && b.is_none() {
				break;
			}
			let s = a.cmp(&b);
			if s != cmp::Ordering::Equal {
				return if reversed { s.reverse() } else { s };
			}
			reversed = !reversed;
		}
		cmp::Ordering::Equal
	}
}

impl PartialEq for ContinuedFraction {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other) == cmp::Ordering::Equal
	}
}

impl Eq for ContinuedFraction {}
