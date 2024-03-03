use crate::error::{FendError, Interrupt};
use crate::interrupt::test_int;
use crate::num::bigrat::BigRat;
use crate::num::complex::{self, Complex};
use crate::result::FResult;
use crate::serialize::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;
use std::ops::Neg;
use std::{fmt, io};

use super::real::Real;
use super::{Base, Exact, FormattingStyle};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Dist {
	// invariant: probabilities must sum to 1
	parts: HashMap<Complex, BigRat>,
}

impl Dist {
	pub(crate) fn serialize(&self, write: &mut impl io::Write) -> FResult<()> {
		self.parts.len().serialize(write)?;
		for (a, b) in &self.parts {
			a.serialize(write)?;
			b.serialize(write)?;
		}
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
		let len = usize::deserialize(read)?;
		let mut hashmap = HashMap::with_capacity(len);
		for _ in 0..len {
			let k = Complex::deserialize(read)?;
			let v = BigRat::deserialize(read)?;
			hashmap.insert(k, v);
		}
		Ok(Self { parts: hashmap })
	}

	pub(crate) fn one_point(self) -> FResult<Complex> {
		if self.parts.len() == 1 {
			Ok(self.parts.into_iter().next().unwrap().0)
		} else {
			Err(FendError::ProbabilityDistributionsNotAllowed)
		}
	}

	pub(crate) fn one_point_ref(&self) -> FResult<&Complex> {
		if self.parts.len() == 1 {
			Ok(self.parts.iter().next().unwrap().0)
		} else {
			Err(FendError::ProbabilityDistributionsNotAllowed)
		}
	}

	pub(crate) fn new_die<I: Interrupt>(count: u32, faces: u32, int: &I) -> FResult<Self> {
		assert!(count != 0);
		assert!(faces != 0);
		if count > 1 {
			let mut result = Self::new_die(1, faces, int)?;
			for _ in 1..count {
				test_int(int)?;
				result = Exact::new(result, true)
					.add(&Exact::new(Self::new_die(1, faces, int)?, true), int)?
					.value;
			}
			return Ok(result);
		}
		let mut hashmap = HashMap::new();
		let probability = BigRat::from(1).div(&BigRat::from(u64::from(faces)), int)?;
		for face in 1..=faces {
			test_int(int)?;
			hashmap.insert(Complex::from(u64::from(face)), probability.clone());
		}
		Ok(Self { parts: hashmap })
	}

	pub(crate) fn equals_int(&self, val: u64) -> bool {
		self.parts.len() == 1 && self.parts.keys().next().unwrap() == &val.into()
	}

	#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	pub(crate) fn sample<I: Interrupt>(self, ctx: &crate::Context, int: &I) -> FResult<Self> {
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

	#[allow(
		clippy::cast_possible_truncation,
		clippy::cast_sign_loss,
		clippy::too_many_arguments
	)]
	pub(crate) fn format<I: Interrupt>(
		&self,
		exact: bool,
		style: FormattingStyle,
		base: Base,
		use_parentheses: complex::UseParentheses,
		out: &mut String,
		ctx: &crate::Context,
		int: &I,
	) -> FResult<Exact<()>> {
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
			let mut max_prob = 0.0;
			for (n, prob) in &self.parts {
				let prob_f64 = prob.clone().into_f64(int)?;
				if prob_f64 > max_prob {
					max_prob = prob_f64;
				}
				ordered_kvs.push((n, prob, prob_f64));
			}
			ordered_kvs.sort_unstable_by(|(a, _, _), (b, _, _)| {
				a.partial_cmp(b).unwrap_or(Ordering::Equal)
			});
			if ctx.output_mode == crate::OutputMode::SimpleText {
				write!(out, "{{ ")?;
			}
			let mut first = true;
			for (num, _prob, prob_f64) in ordered_kvs {
				let num = num
					.format(exact, style, base, use_parentheses, int)?
					.value
					.to_string();
				let prob_percentage = prob_f64 * 100.0;
				if ctx.output_mode == crate::OutputMode::TerminalFixedWidth {
					if !first {
						writeln!(out)?;
					}
					let mut bar = String::new();
					for _ in 0..(prob_f64 / max_prob * 30.0).min(30.0) as u32 {
						bar.push('#');
					}
					write!(out, "{num:>3}: {prob_percentage:>5.2}%  {bar}")?;
				} else {
					if !first {
						write!(out, ", ")?;
					}
					write!(out, "{num}: {prob_percentage:.2}%")?;
				}
				if first {
					first = false;
				}
			}
			if ctx.output_mode == crate::OutputMode::SimpleText {
				write!(out, " }}")?;
			}
			// TODO check exactness
			Ok(Exact::new((), true))
		}
	}

	fn bop<I: Interrupt>(
		self,
		rhs: &Self,
		mut f: impl FnMut(&Complex, &Complex, &I) -> FResult<Complex>,
		int: &I,
	) -> FResult<Self> {
		let mut result = HashMap::<Complex, BigRat>::new();
		for (n1, p1) in &self.parts {
			for (n2, p2) in &rhs.parts {
				let n = f(n1, n2, int)?;
				let p = p1.clone().mul(p2, int)?;
				if let Some(prob) = result.get_mut(&n) {
					*prob = prob.clone().add(p, int)?;
				} else {
					result.insert(n, p);
				}
			}
		}
		Ok(Self { parts: result })
	}
}

impl Exact<Dist> {
	pub(crate) fn add<I: Interrupt>(self, rhs: &Self, int: &I) -> FResult<Self> {
		let self_exact = self.exact;
		let rhs_exact = rhs.exact;
		let mut exact = true;
		Ok(Self::new(
			self.value.bop(
				&rhs.value,
				|a, b, int| {
					let sum = Exact::new(a.clone(), self_exact)
						.add(Exact::new(b.clone(), rhs_exact), int)?;
					exact &= sum.exact;
					Ok(sum.value)
				},
				int,
			)?,
			exact,
		))
	}

	pub(crate) fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> FResult<Self> {
		let self_exact = self.exact;
		let rhs_exact = rhs.exact;
		let mut exact = true;
		Ok(Self::new(
			self.value.bop(
				&rhs.value,
				|a, b, int| {
					let sum = Exact::new(a.clone(), self_exact)
						.mul(&Exact::new(b.clone(), rhs_exact), int)?;
					exact &= sum.exact;
					Ok(sum.value)
				},
				int,
			)?,
			exact,
		))
	}

	pub(crate) fn div<I: Interrupt>(self, rhs: &Self, int: &I) -> FResult<Self> {
		let self_exact = self.exact;
		let rhs_exact = rhs.exact;
		let mut exact = true;
		Ok(Self::new(
			self.value.bop(
				&rhs.value,
				|a, b, int| {
					let sum = Exact::new(a.clone(), self_exact)
						.div(Exact::new(b.clone(), rhs_exact), int)?;
					exact &= sum.exact;
					Ok(sum.value)
				},
				int,
			)?,
			exact,
		))
	}
}

impl From<Complex> for Dist {
	fn from(v: Complex) -> Self {
		let mut parts = HashMap::new();
		parts.insert(v, BigRat::from(1));
		Self { parts }
	}
}

impl From<Real> for Dist {
	fn from(v: Real) -> Self {
		Self::from(Complex::from(v))
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
			Ok(complex) => write!(f, "{complex:?}"),
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
