use crate::num::bigrat::sign::Sign;
use crate::num::biguint::BigUint;
use std::sync::Arc;
use std::{cmp, fmt};

#[derive(Clone)]
pub(crate) struct ContinuedFraction {
	integer_sign: Sign,
	integer: BigUint,
	fraction: Arc<dyn Fn(usize) -> BigUint>, // returning zero indicates the end of the fraction
}

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
}

impl fmt::Debug for ContinuedFraction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[")?;
		if matches!(self.integer_sign, Sign::Negative) {
			write!(f, "-")?;
		}
		write!(f, "{:?}", self.integer)?;
		let first_term = (self.fraction)(0);
		if first_term != 0.into() {
			write!(f, "; {first_term:?}")?;
			for i in 1.. {
				let term = (self.fraction)(i);
				if term == 0.into() {
					break;
				};
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
		for i in 0..50 {
			let a = (self.fraction)(i);
			let b = (other.fraction)(i);
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
