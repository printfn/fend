use crate::error::FendError;
use crate::interrupt::Never;
use crate::num::bigrat::sign::Sign;
use crate::num::biguint::BigUint;
use crate::Interrupt;
use std::hash::Hash;
use std::sync::Arc;
use std::{cmp, fmt, iter, mem, ops, result};

#[derive(Clone)]
pub(crate) struct ContinuedFraction {
	integer_sign: Sign,
	integer: BigUint,
	fraction: Arc<dyn Fn() -> Box<dyn Iterator<Item = BigUint>>>, // must never return a zero
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
		if (self.fraction)().next().is_some() {
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
			fraction: Arc::new(move || Box::new(parts.clone().into_iter())),
		}
	}

	pub(crate) fn is_zero(&self) -> bool {
		self.integer == 0.into() && (self.fraction)().next().is_none()
	}

	pub(crate) fn invert(self) -> Result<Self, FendError> {
		if self.actual_integer_sign() == Sign::Negative {
			return Err(FendError::NegativeNumbersNotAllowed);
		}
		if self.integer == 0.into() {
			let Some(integer) = (self.fraction)().next() else {
				return Err(FendError::DivideByZero);
			};
			Ok(Self {
				integer,
				integer_sign: self.integer_sign,
				fraction: Arc::new(move || Box::new((self.fraction)().skip(1))),
			})
		} else {
			Ok(Self {
				integer: 0.into(),
				integer_sign: self.integer_sign,
				fraction: Arc::new(move || {
					Box::new(iter::once(self.integer.clone()).chain((self.fraction)()))
				}),
			})
		}
	}

	// (ax+b)/(cx+d)
	pub(crate) fn homographic<I: Interrupt>(
		self,
		mut a: BigUint,
		mut b: BigUint,
		mut c: BigUint,
		mut d: BigUint,
		int: &I,
	) -> Result<Self, FendError> {
		if self.actual_integer_sign() == Sign::Negative {
			return Err(FendError::NegativeNumbersNotAllowed);
		}
		mem::swap(&mut a, &mut b);
		mem::swap(&mut c, &mut d);
		let mut result_iterator = HomographicIterator {
			heading: self.integer.clone(),
			fraction_iter: (self.fraction)(),
			a,
			b,
			c,
			d,
			state: HomographicState::Initial,
		};
		let Some(integer) = result_iterator.next() else {
			return Err(FendError::Unknown);
		};
		Ok(Self {
			integer_sign: Sign::Positive,
			integer,
			fraction: Arc::new(move || {
				Box::new(HomographicIterator {
					heading: result_iterator.heading.clone(),
					a: result_iterator.a.clone(),
					b: result_iterator.b.clone(),
					c: result_iterator.c.clone(),
					d: result_iterator.d.clone(),
					state: result_iterator.state.clone(),
					fraction_iter: Box::new((self.fraction)().skip(1)),
				})
			}),
		})
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
			|| (self.fraction)().next().is_some()
			|| other.actual_integer_sign() != Sign::Positive
			|| (other.fraction)().next().is_some()
		{
			return Err(FendError::ModuloForPositiveInts);
		}
		Ok(Self::from(self.integer.divmod(&other.integer, int)?.1))
	}
}

#[derive(Clone)]
enum HomographicState {
	Initial,
	Yielded {
		r1: BigUint,
		r2: BigUint,
		n: BigUint,
	},
	A {
		m: BigUint,
		n: BigUint,
	},
	Terminated,
}

struct HomographicIterator {
	heading: BigUint,
	a: BigUint,
	b: BigUint,
	c: BigUint,
	d: BigUint,
	fraction_iter: Box<dyn Iterator<Item = BigUint>>,
	state: HomographicState,
}

impl Iterator for HomographicIterator {
	type Item = BigUint;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match mem::replace(&mut self.state, HomographicState::Initial) {
				HomographicState::Initial => {
					let m = self
						.b
						.clone()
						.mul(&self.heading, &Never)
						.unwrap()
						.add(&self.a);
					let n = self
						.d
						.clone()
						.mul(&self.heading, &Never)
						.unwrap()
						.add(&self.c);
					// check if b/d and m/n floor to the same value
					let (q1, r1) = self.b.divmod(&self.d, &Never).unwrap();
					let (q2, r2) = m.divmod(&n, &Never).unwrap();
					if q1 == q2 {
						// same value!
						// we can now yield that value
						self.state = HomographicState::Yielded { r1, r2, n };
						return Some(q1);
					} else {
						self.state = HomographicState::A { m, n };
					}
				}
				HomographicState::Yielded { r1, r2, n } => {
					// now take reciprocals and subtract remainders
					self.b = mem::replace(&mut self.d, r1);
					self.state = HomographicState::A { m: n, n: r2 };
				}
				HomographicState::A { m, n } => {
					let Some(f) = self.fraction_iter.next() else {
						self.state = HomographicState::Terminated;
						return None;
					};
					self.heading = f;
					self.a = mem::replace(&mut self.b, m);
					self.c = mem::replace(&mut self.d, n);
					self.state = HomographicState::Initial;
				}
				HomographicState::Terminated => {
					self.state = HomographicState::Terminated;
					return None;
				}
			};
		}
	}
}

impl ops::Neg for ContinuedFraction {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self::from_f64(self.as_f64().neg())
	}
}

impl<'a> IntoIterator for &'a ContinuedFraction {
	type Item = BigUint;

	type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

	fn into_iter(self) -> Self::IntoIter {
		(self.fraction)()
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
			fraction: Arc::new(|| Box::new(iter::empty())),
		}
	}
}

impl From<u64> for ContinuedFraction {
	fn from(value: u64) -> Self {
		Self {
			integer_sign: Sign::Positive,
			integer: value.into(),
			fraction: Arc::new(|| Box::new(iter::empty())),
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
		let (i1, i2) = match self.integer_sign {
			Sign::Positive => (&self.integer, &other.integer),
			Sign::Negative => (&other.integer, &self.integer),
		};
		let s = i1.cmp(&i2);
		if s != cmp::Ordering::Equal {
			return s;
		}
		if Arc::ptr_eq(&self.fraction, &other.fraction) {
			return cmp::Ordering::Equal;
		}
		let iter1 = self.into_iter().map(Ok).chain(iter::repeat(Err(())));
		let iter2 = other.into_iter().map(Ok).chain(iter::repeat(Err(())));
		iter1
			.zip(iter2)
			.take_while(|x| x != &(Err(()), Err(())))
			.enumerate()
			.map(|(i, (a, b))| if i % 2 == 0 { (b, a) } else { (a, b) })
			.map(|(a, b)| a.cmp(&b))
			.take(MAX_ITERATIONS)
			.try_for_each(|o| match o {
				cmp::Ordering::Equal => Ok(()),
				_ => Err(o),
			})
			.err()
			.unwrap_or(cmp::Ordering::Equal)
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
		Arc::as_ptr(&self.fraction).hash(state);
	}
}

macro_rules! cf {
	($a:literal $( ; $( $b:literal ),+ )? ) => {
		{
			let i: i32 = $a.into();
			let parts: Vec<$crate::num::continued_fraction::BigUint> = vec![ $( $( $b.into() ),+ )? ];
			$crate::num::continued_fraction::ContinuedFraction {
				integer_sign: if i >= 0 {
					$crate::num::continued_fraction::Sign::Positive
				} else {
					$crate::num::continued_fraction::Sign::Negative
				},
				integer: (i.abs() as u64).into(),
				fraction: ::std::sync::Arc::new(move || {
					Box::new(parts.clone().into_iter())
				}),
			}
		}
	};
}

#[cfg(test)]
mod tests {
	use crate::interrupt::Never;

	use super::*;

	fn sqrt_2() -> ContinuedFraction {
		ContinuedFraction {
			integer_sign: Sign::Positive,
			integer: 1.into(),
			fraction: Arc::new(|| Box::new(iter::repeat_with(|| 2.into()))),
		}
	}

	#[test]
	fn comparisons() {
		assert_eq!(cf!(3; 1), cf!(3; 1));
		assert!(cf!(3; 1) > cf!(3; 2));
		assert!(cf!(4) > cf!(3; 2));
		assert!(cf!(3; 2, 1) < cf!(3; 2));
		assert!(cf!(3; 2, 1) < cf!(3; 2, 2));
		assert!(cf!(3; 2, 1) < cf!(3; 2, 20000));
		assert!(cf!(3) < cf!(3; 2, 20000));
		assert!(cf!(3) < cf!(4));
		assert!(cf!(-3) < cf!(4));
		assert!(cf!(-3) > cf!(-4));
		assert_eq!(cf!(-3), cf!(-3));
		assert!(cf!(-3; 2, 1) < cf!(-3; 2));
	}

	#[test]
	fn invert() {
		assert_eq!(cf!(3; 2, 6, 4).invert().unwrap(), cf!(0; 3, 2, 6, 4));
		assert_eq!(cf!(0; 3, 2, 6, 4).invert().unwrap(), cf!(3; 2, 6, 4));
	}

	#[test]
	fn homographic() {
		let res = sqrt_2()
			.homographic(2.into(), 3.into(), 5.into(), 1.into(), &Never)
			.unwrap();
		assert_eq!(res.integer, 0.into());
		assert_eq!(
			(res.fraction)()
				.take(1000)
				.map(|b| b.try_as_usize(&Never).unwrap())
				.collect::<Vec<_>>(),
			Some(1)
				.into_iter()
				.chain([2, 1, 1, 2, 36].into_iter().cycle())
				.take(1000)
				.collect::<Vec<_>>(),
		);
	}
}
