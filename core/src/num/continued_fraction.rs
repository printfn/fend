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
		args: [impl Into<BigUint>; 4],
		int: &I,
	) -> Result<Self, FendError> {
		if self.actual_integer_sign() == Sign::Negative {
			return Err(FendError::NegativeNumbersNotAllowed);
		}
		let args = {
			let [a, b, c, d] = args;
			[b.into(), a.into(), d.into(), c.into()]
		};
		let mut result_iterator = HomographicIterator {
			heading: self.integer.clone(),
			fraction_iter: (self.fraction)(),
			args: args.clone(),
			state: HomographicState::Initial,
		};
		let Some(integer) = result_iterator.next() else {
			return Err(FendError::Unknown);
		};
		Ok(Self {
			integer_sign: Sign::Positive,
			integer,
			fraction: Arc::new(move || {
				Box::new(
					HomographicIterator {
						heading: self.integer.clone(),
						fraction_iter: (self.fraction)(),
						args: args.clone(),
						state: HomographicState::Initial,
					}
					.skip(1),
				)
			}),
		})
	}

	// (axy + bx + cy + d) / (exy + fx + gy + h)
	pub(crate) fn bihomographic(
		x: Self,
		y: Self,
		args: [impl Into<BigUint>; 8],
	) -> Result<Self, FendError> {
		let args = {
			let [g, c, e, a, h, d, f, b] = args;
			[
				a.into(),
				b.into(),
				c.into(),
				d.into(),
				e.into(),
				f.into(),
				g.into(),
				h.into(),
			]
		};
		let mut result_iterator = BihomographicIterator {
			args: args.clone(),
			x_iter: Box::new(iter::once(x.integer.clone()).chain((x.fraction)())),
			y_iter: Box::new(iter::once(y.integer.clone()).chain((y.fraction)())),
			state: BihomographicState::Initial,
		};
		let Some(integer) = result_iterator.next() else {
			return Err(FendError::Unknown);
		};
		Ok(Self {
			integer_sign: Sign::Positive,
			integer,
			fraction: Arc::new(move || {
				Box::new(
					BihomographicIterator {
						args: args.clone(),
						x_iter: Box::new(iter::once(x.integer.clone()).chain((x.fraction)())),
						y_iter: Box::new(iter::once(y.integer.clone()).chain((y.fraction)())),
						state: BihomographicState::Initial,
					}
					.skip(1),
				)
			}),
		})
	}

	pub(crate) fn add<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, FendError> {
		Self::bihomographic(self.clone(), other.clone(), [0, 1, 1, 0, 0, 0, 0, 1])
	}

	pub(crate) fn mul<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, FendError> {
		Self::bihomographic(self.clone(), other.clone(), [1, 0, 0, 0, 0, 0, 0, 1])
	}

	pub(crate) fn div<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, FendError> {
		if other.is_zero() {
			return Err(FendError::DivideByZero);
		}
		Ok(self.clone().invert()?.mul(&other.clone().invert()?, int)?)
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
	args: [BigUint; 4],
	fraction_iter: Box<dyn Iterator<Item = BigUint>>,
	state: HomographicState,
}

impl Iterator for HomographicIterator {
	type Item = BigUint;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match mem::replace(&mut self.state, HomographicState::Initial) {
				HomographicState::Initial => {
					let m = self.args[1]
						.clone()
						.mul(&self.heading, &Never)
						.unwrap()
						.add(&self.args[0]);
					let n = self.args[3]
						.clone()
						.mul(&self.heading, &Never)
						.unwrap()
						.add(&self.args[2]);
					// check if b/d and m/n floor to the same value
					let (q1, r1) = self.args[1].divmod(&self.args[3], &Never).unwrap();
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
					self.args[1] = mem::replace(&mut self.args[3], r1);
					self.state = HomographicState::A { m: n, n: r2 };
				}
				HomographicState::A { m, n } => {
					let Some(f) = self.fraction_iter.next() else {
						self.state = HomographicState::Terminated;
						return None;
					};
					self.heading = f;
					self.args[0] = mem::replace(&mut self.args[1], m);
					self.args[2] = mem::replace(&mut self.args[3], n);
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

#[derive(Clone)]
enum BihomographicState {
	Initial,
	ShiftDown,
	ShiftRight,
	Terminated,
}

struct BihomographicIterator {
	args: [BigUint; 8],
	state: BihomographicState,
	x_iter: Box<dyn Iterator<Item = BigUint>>,
	y_iter: Box<dyn Iterator<Item = BigUint>>,
}

impl Iterator for BihomographicIterator {
	type Item = BigUint;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match mem::replace(&mut self.state, BihomographicState::Initial) {
				BihomographicState::Initial => {
					if self.args[1] == 0.into()
						|| self.args[3] == 0.into()
						|| self.args[5] == 0.into()
						|| self.args[7] == 0.into()
					{
						if self.args[3] == 0.into() {
							self.state = BihomographicState::ShiftDown;
						} else {
							self.state = BihomographicState::ShiftRight;
						}
						continue;
					}
					let (q1, r1) = self.args[0].divmod(&self.args[1], &Never).unwrap();
					let (q2, r2) = self.args[2].divmod(&self.args[3], &Never).unwrap();
					let (q3, r3) = self.args[4].divmod(&self.args[5], &Never).unwrap();
					let (q4, r4) = self.args[6].divmod(&self.args[7], &Never).unwrap();
					if q1 == q2 && q1 == q3 && q1 == q4 {
						// all quotients equal, yield a value
						self.args[0] = mem::replace(&mut self.args[1], r1);
						self.args[2] = mem::replace(&mut self.args[3], r2);
						self.args[4] = mem::replace(&mut self.args[5], r3);
						self.args[6] = mem::replace(&mut self.args[7], r4);
						self.state = BihomographicState::Initial;
						return Some(q1);
					} else {
						if q2 == q4 {
							self.state = BihomographicState::ShiftRight;
						} else {
							self.state = BihomographicState::ShiftDown;
						}
					}
				}
				#[rustfmt::skip]
				BihomographicState::ShiftRight => {
					if let Some(x_next) = self.x_iter.next() {
						let n1 = self.args[2].clone().mul(&x_next, &Never).unwrap().add(&self.args[0]);
						let n2 = self.args[3].clone().mul(&x_next, &Never).unwrap().add(&self.args[1]);
						let n3 = self.args[6].clone().mul(&x_next, &Never).unwrap().add(&self.args[4]);
						let n4 = self.args[7].clone().mul(&x_next, &Never).unwrap().add(&self.args[5]);
						self.args[0] = mem::replace(&mut self.args[2], n1);
						self.args[1] = mem::replace(&mut self.args[3], n2);
						self.args[4] = mem::replace(&mut self.args[6], n3);
						self.args[5] = mem::replace(&mut self.args[7], n4);
						self.state = BihomographicState::Initial;
					} else {
						self.state = BihomographicState::Terminated;
					}
				},
				#[rustfmt::skip]
				BihomographicState::ShiftDown => {
					if let Some(y_next) = self.y_iter.next() {
						let n1 = self.args[4].clone().mul(&y_next, &Never).unwrap().add(&self.args[0]);
						let n2 = self.args[5].clone().mul(&y_next, &Never).unwrap().add(&self.args[1]);
						let n3 = self.args[6].clone().mul(&y_next, &Never).unwrap().add(&self.args[2]);
						let n4 = self.args[7].clone().mul(&y_next, &Never).unwrap().add(&self.args[3]);
						self.args[0] = mem::replace(&mut self.args[4], n1);
						self.args[1] = mem::replace(&mut self.args[5], n2);
						self.args[2] = mem::replace(&mut self.args[6], n3);
						self.args[3] = mem::replace(&mut self.args[7], n4);
						self.state = BihomographicState::Initial;
					} else {
						self.state = BihomographicState::Terminated;
					}
				},
				BihomographicState::Terminated => {
					self.state = BihomographicState::Terminated;
					return None;
				}
			}
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

	fn coth_1() -> ContinuedFraction {
		ContinuedFraction {
			integer_sign: Sign::Positive,
			integer: 1.into(),
			fraction: Arc::new(|| {
				let mut current = 1;
				Box::new(iter::repeat_with(move || {
					current += 2;
					current.into()
				}))
			}),
		}
	}

	fn sqrt_6() -> ContinuedFraction {
		ContinuedFraction {
			integer_sign: Sign::Positive,
			integer: 2.into(),
			fraction: Arc::new(|| Box::new([2, 4].into_iter().map(BigUint::from).cycle())),
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
		let res = sqrt_2().homographic([2, 3, 5, 1], &Never).unwrap();
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

	#[test]
	fn bihomographic() {
		let res =
			ContinuedFraction::bihomographic(coth_1(), sqrt_6(), [2, 1, 0, 0, 1, 0, 1, 0]).unwrap();
		assert_eq!(res.integer, 1.into());
		let v = (res.fraction)()
			.take(880)
			.map(|b| b.try_as_usize(&Never).unwrap())
			.collect::<Vec<_>>();
		assert_eq!(
			format!("{v:?}"),
			"[2, 1, 2, 1, 1, 1, 2, 39, 1, 7, 4, 1, 65, 6, 2, 2, 4, 5, 2, 1, 1, 1, 2, 7, 3, 1, 1, 3, 3, 3, 2, 47, 2, 1, 1, 1, 1, 1, 9, 2, 42, 2, 4, 1, 92, 1, 2, 1, 4, 1, 41, 1, 1, 2, 1, 16, 3, 3, 117, 1, 1, 1, 1, 1044, 1, 2, 5, 2, 1, 1, 18, 1, 1, 1, 2, 43, 1, 14, 2, 1, 6, 4, 1, 13, 3, 10, 1, 29, 1, 1, 10, 2, 1, 1, 1, 5, 3, 1, 14, 8, 3, 1, 2, 1, 8, 1, 15, 1, 14, 34, 2, 1, 4, 1, 1, 4, 1, 3, 4, 1, 4, 1, 1, 1, 4, 1, 2, 6, 1, 108, 1, 34, 2, 5, 17, 6, 1, 1, 1, 5, 2, 1, 1, 42, 6, 5, 8, 1, 8, 1, 1, 1, 5, 3, 58, 1, 14, 4, 1, 14, 5, 1, 1, 15, 2, 3, 2, 1, 10, 2, 1, 1, 1, 1, 1, 22, 1, 1, 2, 1, 1, 2, 3, 24, 5, 1, 1, 1, 1, 1, 6, 20, 1, 13, 1, 9, 3, 2, 2, 2, 1, 4, 2, 3, 3, 5, 52, 1, 1, 11, 1, 2, 2, 2, 1, 6, 540, 20, 2, 3, 4, 46, 3, 18, 1, 1, 2, 2, 1, 1, 10, 1, 1, 6, 4, 2, 1, 1, 8, 1, 1, 1, 3, 1, 14, 1, 4, 1, 14, 3, 4, 12, 5, 1, 2, 1, 3, 1, 1, 1, 3, 1, 1, 1, 2, 6, 1, 6, 32, 1, 21, 2, 1, 2, 4, 1, 4, 4, 1, 1, 14, 2, 1, 4, 5, 38, 1, 23, 2, 10, 1, 20, 15, 4, 1, 12, 1, 3, 1, 8, 5, 4, 1, 3, 4, 6, 2, 8, 1, 1, 4, 3, 1, 1, 2, 13, 4, 1, 2, 1, 1, 2, 1, 2, 3, 1, 1, 5, 2, 7, 1, 7, 2, 3, 3, 1, 1, 1, 1, 3, 1, 1, 1, 8, 4, 1, 1, 4, 2, 3, 1, 1, 2, 1, 1, 1, 2, 1, 2, 7, 1, 1, 2, 3, 2, 1, 7, 1, 2, 2, 3, 1, 10, 1, 3, 1, 2, 8, 1, 16, 1, 9, 1, 3, 5, 1, 1, 1, 1, 1, 6, 2, 3, 1, 1, 3, 1, 2, 1, 14, 1, 41, 1, 1, 2, 1, 22, 1, 1, 2, 1, 1, 1, 1, 1, 1, 83, 2, 4, 1, 4, 1, 3, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 16, 1, 1, 4, 1, 2, 4, 3, 1, 1, 1, 2, 10, 2, 4, 2, 4, 1, 1, 1, 4, 2, 5, 1, 38, 1, 1, 4, 3, 3, 1, 1, 1, 3, 5, 1, 3, 11, 1, 2, 1, 2, 1, 1, 7, 1, 1, 62, 1, 1, 1, 2, 11, 6, 73, 1, 13, 1, 1, 1, 1, 7, 1, 5, 2, 1, 28, 1, 2, 3, 1, 1, 2, 1, 4, 11, 1, 2, 1, 15, 1, 3, 2, 1, 3, 1, 1, 2, 4, 1, 12, 4, 1, 1, 4, 1, 1, 3, 3, 1, 10, 12, 2, 6, 2, 1, 1, 1, 3, 3, 2, 1, 1, 5, 2, 1, 1, 3, 9, 21, 14, 1, 1, 1, 1, 1, 15, 3, 1, 2, 17, 12, 1, 1, 6, 3, 1, 2, 1, 4, 2, 3, 1, 3, 532, 6, 1, 11, 1, 1, 98, 1, 2, 3, 1, 1, 1, 1, 1, 1, 3, 3, 8, 1, 3, 16, 1, 6, 11, 1, 181, 43, 1, 1, 34, 11, 1, 1, 7, 1, 9, 4, 2, 3, 1, 1, 1, 42, 1, 5, 17, 2, 302, 1, 2, 1, 2, 2, 2, 11, 51, 2, 7, 1, 2, 1, 1, 27, 7, 3, 24, 9, 1, 3, 1, 12, 1, 2, 2, 2, 1, 25, 1, 1, 1, 104, 1, 1, 17, 1, 2, 9, 1, 53, 3, 1, 33, 2, 4, 3, 1, 4, 2, 3, 2, 83, 4, 1, 4, 14, 1, 8, 1, 1, 2, 1, 5, 1, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 1, 1, 3, 2, 1, 1, 4, 21, 3, 6, 2, 9, 1, 2, 4, 1, 3, 1, 1, 1, 1, 1, 9, 1, 14, 1, 3, 5, 3, 1, 4, 2, 1, 1, 1, 1, 1, 27, 2, 74, 42, 10, 1, 1, 1, 2, 2, 2, 1, 15, 1, 2, 3, 2, 16, 1, 4, 2, 2, 2, 4, 1, 2, 3, 2, 5, 6, 6, 14, 2, 4, 1, 1, 5, 1, 1, 4, 22, 3, 2, 33, 1, 2, 3, 17, 3, 2, 426, 10, 3, 8, 1, 4, 1, 1, 2, 8, 32, 2, 2, 2, 1, 1, 1, 23, 2, 3, 4, 1, 402, 1, 7, 10, 1, 7, 3, 2, 1, 1, 1, 2, 1, 1, 1, 6, 7, 1, 1, 1, 2, 4, 2, 2, 1, 2, 46, 8, 1, 1034, 1, 1, 1, 1, 13, 4, 5, 1, 5, 7, 9, 3, 7, 1, 1, 13, 1, 3, 43, 2, 2, 1, 1, 2, 1, 371, 1, 6, 1, 1, 53, 1, 5, 1, 1, 2, 2, 1, 1]",
		);
	}
}
