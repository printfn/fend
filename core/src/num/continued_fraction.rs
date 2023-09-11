use crate::error::FendError;
use crate::num::bigrat::sign::Sign;
use crate::num::biguint::BigUint;
use crate::Interrupt;
use std::hash::Hash;
use std::sync::Arc;
use std::{cmp, fmt, iter, ops};

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

	pub(crate) fn as_f64(&self) -> f64 {
		let mut result = self.integer.as_f64();
		if self.integer_sign == Sign::Negative {
			result = -result;
		}
		let mut denominator = 1.0;
		for term in self.into_iter().take(MAX_ITERATIONS) {
			denominator = 1.0 / (denominator + term.as_f64());
			result = result * denominator + term.as_f64();
		}
		result
	}

	#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	pub(crate) fn from_f64(value: f64) -> Self {
		let integer = value.floor();
		let (sign, bigint) = if integer >= 0.0 {
			(Sign::Positive, BigUint::from(value as u64))
		} else {
			(Sign::Negative, BigUint::from((-value) as u64))
		};
		let mut parts: Vec<BigUint> = vec![];
		let mut f = value - integer;
		while f != 0.0 {
			let recip = f.recip();
			let term = recip.floor();
			parts.push((term as u64).into());
			if parts.len() >= MAX_ITERATIONS {
				break;
			}
			f = recip - term;
		}

		Self {
			integer_sign: sign,
			integer: bigint,
			fraction: Arc::new(move |i| {
				if i >= parts.len() {
					0.into()
				} else {
					parts[i].clone()
				}
			}),
		}
	}

	pub(crate) fn is_zero(&self) -> bool {
		self.integer == 0.into() && (self.fraction)(0) == 0.into()
	}

	pub(crate) fn add<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, FendError> {
		Ok(Self::from_f64(self.as_f64() + other.as_f64()))
	}

	pub(crate) fn mul<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, FendError> {
		Ok(Self::from_f64(self.as_f64() * other.as_f64()))
	}

	pub(crate) fn div<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, FendError> {
		if other.is_zero() {
			return Err(FendError::DivideByZero);
		}
		Ok(Self::from_f64(self.as_f64() / other.as_f64()))
	}

	pub(crate) fn modulo<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, FendError> {
		if other.is_zero() {
			return Err(FendError::ModuloByZero);
		}
		if self.actual_integer_sign() != Sign::Positive
			|| (self.fraction)(0) != 0.into()
			|| other.actual_integer_sign() != Sign::Positive
			|| (other.fraction)(0) != 0.into()
		{
			return Err(FendError::ModuloForPositiveInts);
		}
		Ok(Self::from(self.integer.divmod(&other.integer, int)?.1))
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

impl ops::Neg for ContinuedFraction {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self::from_f64(self.as_f64().neg())
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

impl From<u64> for ContinuedFraction {
	fn from(value: u64) -> Self {
		Self {
			integer_sign: Sign::Positive,
			integer: value.into(),
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

impl Hash for ContinuedFraction {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.actual_integer_sign().hash(state);
		self.integer.hash(state);
	}
}
