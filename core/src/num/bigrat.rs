use crate::DecimalSeparatorStyle;
use crate::error::{FendError, Interrupt};
use crate::format::Format;
use crate::interrupt::test_int;
use crate::num::biguint::BigUint;
use crate::num::{Base, Exact, FormattingStyle, Range, RangeBound};
use crate::result::FResult;
use crate::serialize::CborValue;
use core::f64;
use num_traits::ToPrimitive;
use std::sync::OnceLock;
use std::{cmp, fmt, hash, ops};

pub(crate) mod sign {
	use crate::{
		error::FendError,
		result::FResult,
		serialize::{Deserialize, Serialize},
	};
	use std::io;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub(crate) enum Sign {
		Negative = 1,
		Positive = 2,
	}

	impl Sign {
		pub(crate) const fn flip(self) -> Self {
			match self {
				Self::Positive => Self::Negative,
				Self::Negative => Self::Positive,
			}
		}

		pub(crate) const fn sign_of_product(a: Self, b: Self) -> Self {
			match (a, b) {
				(Self::Positive, Self::Positive) | (Self::Negative, Self::Negative) => {
					Self::Positive
				}
				(Self::Positive, Self::Negative) | (Self::Negative, Self::Positive) => {
					Self::Negative
				}
			}
		}

		pub(crate) fn serialize(self, write: &mut impl io::Write) -> FResult<()> {
			(self as u8).serialize(write)
		}

		pub(crate) fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
			Ok(match u8::deserialize(read)? {
				1 => Self::Negative,
				2 => Self::Positive,
				_ => return Err(FendError::DeserializationError("sign must be 1 or 2")),
			})
		}
	}
}

use super::biguint::{self, FormattedBigUint};
use super::out_of_range;
use sign::Sign;

#[derive(Clone)]
pub(crate) struct BigRat {
	sign: Sign,
	num: BigUint,
	den: BigUint,
}

impl fmt::Debug for BigRat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.sign == Sign::Negative {
			write!(f, "-")?;
		}
		write!(f, "{:?}", self.num)?;
		if !self.den.is_definitely_one() {
			write!(f, "/{:?}", self.den)?;
		}
		Ok(())
	}
}

impl Ord for BigRat {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		let int = &crate::interrupt::Never;
		let diff = self.clone().add(-other.clone(), int).unwrap();
		if diff.num == 0.into() {
			cmp::Ordering::Equal
		} else if diff.sign == Sign::Positive {
			cmp::Ordering::Greater
		} else {
			cmp::Ordering::Less
		}
	}
}

impl PartialOrd for BigRat {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for BigRat {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other) == cmp::Ordering::Equal
	}
}

impl Eq for BigRat {}

impl hash::Hash for BigRat {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		let int = &crate::interrupt::Never;
		if let Ok(res) = self.clone().simplify(int) {
			// don't hash the sign
			res.num.hash(state);
			res.den.hash(state);
		}
	}
}

impl BigRat {
	const INTERNAL_PRECISION: usize = 200;
	const FIXED_POINT_GUARD_DIGITS: usize = 20;

	pub(crate) fn serialize(&self) -> CborValue {
		let num = self.num.serialize(self.sign);
		if self.den == 1.into() {
			num
		} else {
			let den = self.den.serialize(Sign::Positive);
			CborValue::Tag(30, Box::new(CborValue::Array(vec![num, den])))
		}
	}

	pub(crate) fn deserialize(value: CborValue) -> FResult<Self> {
		Ok(match value {
			CborValue::Tag(30, inner) => {
				if let CborValue::Array(arr) = *inner
					&& arr.len() == 2
				{
					let mut arr = arr.into_iter();
					let (num, sign) = BigUint::deserialize(arr.next().unwrap())?;
					let (den, Sign::Positive) = BigUint::deserialize(arr.next().unwrap())? else {
						return Err(FendError::DeserializationError(
							"tag 30 denominator must be positive",
						));
					};
					Self { sign, num, den }
				} else {
					return Err(FendError::DeserializationError(
						"tag 30 must contain a length-2 array",
					));
				}
			}
			value => {
				let (num, sign) = BigUint::deserialize(value)?;
				Self {
					sign,
					num,
					den: 1.into(),
				}
			}
		})
	}

	pub(crate) fn is_integer(&self) -> bool {
		self.den == 1.into()
	}

	pub(crate) fn try_as_biguint<I: Interrupt>(mut self, int: &I) -> FResult<BigUint> {
		if self.sign == Sign::Negative && self.num != 0.into() {
			return Err(FendError::NegativeNumbersNotAllowed);
		}
		self = self.simplify(int)?;
		if self.den != 1.into() {
			return Err(FendError::FractionToInteger);
		}
		Ok(self.num)
	}

	pub(crate) fn try_as_usize<I: Interrupt>(mut self, int: &I) -> FResult<usize> {
		if self.sign == Sign::Negative && self.num != 0.into() {
			return Err(FendError::NegativeNumbersNotAllowed);
		}
		self = self.simplify(int)?;
		if self.den != 1.into() {
			return Err(FendError::FractionToInteger);
		}
		self.num.try_as_usize(int)
	}

	pub(crate) fn try_as_i64<I: Interrupt>(mut self, int: &I) -> FResult<i64> {
		self = self.simplify(int)?;
		if self.den != 1.into() {
			return Err(FendError::FractionToInteger);
		}
		let res = self.num.try_as_usize(int)?;
		let res: i64 = res.try_into().map_err(|_| FendError::OutOfRange {
			value: Box::new(res),
			range: Range {
				start: RangeBound::None,
				end: RangeBound::Open(Box::new(i64::MAX)),
			},
		})?;
		Ok(match self.sign {
			Sign::Positive => res,
			Sign::Negative => -res,
		})
	}

	pub(crate) fn into_f64<I: Interrupt>(mut self, int: &I) -> FResult<f64> {
		if self.is_definitely_zero() {
			return Ok(0.0);
		}
		self = self.simplify(int)?;
		let positive_result = self.num.as_f64() / self.den.as_f64();
		if self.sign == Sign::Negative {
			Ok(-positive_result)
		} else {
			Ok(positive_result)
		}
	}

	fn high_precision_ln_2() -> Self {
		static LN_2: OnceLock<BigRat> = OnceLock::new();
		LN_2.get_or_init(|| {
			// ln(2) to ~280 digits
			let s = "0.69314718055994530941723212145817656807550013436025525412068000949339362196969471560586332699641868754200148102057068573368552023575813055703267075163507596193072757082837143519030703862389167347112335011536449795523912047517268157493206515552473413952588295045300709532636664265410423915781495204374";
			let split_idx = std::cmp::min(2 + Self::INTERNAL_PRECISION, s.len());
			let slice = &s[..split_idx];
			let int = &crate::interrupt::Never;
			let fraction_str = &slice[2..];
			let mut fraction = BigUint::from(0u64);
			let ten = BigUint::from(10u64);
			for c in fraction_str.chars() {
				let digit = u64::from(c.to_digit(10).unwrap());
				fraction = fraction.mul(&ten, int).unwrap().add(&BigUint::from(digit));
			}
			let den = BigUint::pow(&ten, &BigUint::from(fraction_str.len() as u64), int).unwrap();
			Self {
				sign: Sign::Positive,
				num: fraction,
				den,
			}
		}).clone()
	}

	fn high_precision_ln_10() -> Self {
		static LN_10: OnceLock<BigRat> = OnceLock::new();
		LN_10.get_or_init(|| {
			// ln(10) to ~280 digits
			let s = "2.30258509299404568401799145468436420760110148862877297603332790096757260967735248023599720508959829834196778404228624863340952546508280675666628736909878168948290720832555468084379989482623319852839350530896537773262884616336622228769821988674654366747440424327436515504893431493939147961940440022210";
			let split_idx = std::cmp::min(2 + Self::INTERNAL_PRECISION, s.len());
			let slice = &s[..split_idx];
			let int = &crate::interrupt::Never;
			let integer = BigUint::from(2u64);
			let fraction_str = &slice[2..];
			let mut fraction = BigUint::from(0u64);
			let ten = BigUint::from(10u64);
			for c in fraction_str.chars() {
				let digit = u64::from(c.to_digit(10).unwrap());
				fraction = fraction.mul(&ten, int).unwrap().add(&BigUint::from(digit));
			}
			let den = BigUint::pow(&ten, &BigUint::from(fraction_str.len() as u64), int).unwrap();
			Self {
				sign: Sign::Positive,
				num: integer.mul(&den, int).unwrap().add(&fraction),
				den,
			}
		}).clone()
	}

	// exp(x) = 1 + x + x^2/2! + ...
	fn exp_series<I: Interrupt>(x: &Self, int: &I) -> FResult<Self> {
		if x.is_definitely_zero() {
			return Ok(Self::from(1));
		}

		let scale = Self::fixed_point_scale();
		// We expect x to be small (reduced), so fixed point is fine
		let x_fixed = x.num.clone().mul(&scale.clone(), int)?.div(&x.den, int)?;

		if x_fixed == 0.into() {
			return Ok(Self::from(1));
		}

		let mut result = scale.clone(); // 1.0
		let mut term = scale.clone(); // 1.0

		for i in 1..Self::INTERNAL_PRECISION {
			term = term.mul(&x_fixed, int)?;
			term = term.div(scale, int)?;
			term = term.div(&BigUint::from(i as u64), int)?;

			if term == 0.into() {
				break;
			}

			result = result.add(&term);
		}

		Ok(Self {
			sign: Sign::Positive,
			num: result,
			den: scale.clone(),
		})
	}

	// ln((1+z)/(1-z)) = 2(z + z^3/3 + ...)
	fn ln_series_inv_tanh<I: Interrupt>(z: Self, int: &I) -> FResult<Self> {
		if z.is_definitely_zero() {
			return Ok(z);
		}

		let scale = Self::fixed_point_scale();
		let z_fixed = z.num.clone().mul(&scale.clone(), int)?.div(&z.den, int)?;

		if z_fixed == 0.into() {
			return Ok(z);
		}

		let mut result = z_fixed.clone();
		let mut term = z_fixed.clone();
		let z_sq = z_fixed.clone().mul(&z_fixed, int)?;
		let scale_sq = scale.clone().mul(scale, int)?;

		for i in 1..Self::INTERNAL_PRECISION {
			term = term.mul(&z_sq, int)?;
			term = term.div(&scale_sq, int)?;

			if term == 0.into() {
				break;
			}

			let n = 2 * i + 1;
			let val = term.clone().div(&BigUint::from(n as u64), int)?;

			result = result.add(&val);
		}

		// Multiply by 2
		let result = result.mul(&BigUint::from(2u64), int)?;

		Ok(Self {
			sign: Sign::Positive,
			num: result,
			den: scale.clone(),
		})
	}

	pub(crate) fn exp<I: Interrupt>(self, int: &I) -> FResult<Exact<Self>> {
		if self.is_definitely_zero() {
			return Ok(Exact::new(Self::from(1), true));
		}
		let one = Self::from(1);

		let mut x = self.clone();
		let mut invert = false;
		if x.sign == Sign::Negative {
			x = -x;
			invert = true;
		}

		// Range reduction: Reduce x until x < 0.5
		let mut k = 0;
		// 1/2
		let half = Self {
			sign: Sign::Positive,
			num: 1.into(),
			den: 2.into(),
		};

		while x > half {
			x.den = x.den.mul(&2.into(), int)?;
			k += 1;
		}

		let mut result = Self::exp_series(&x, int)?;
		let scale = Self::fixed_point_scale();

		// Square result k times
		for _ in 0..k {
			// Perform fixed-point squaring (N^2 / scale) instead of rational squaring.
			// This keeps the integer sizes constant (~200 digits) instead of exploding to 400k digits.
			let num_sq = result.num.clone().mul(&result.num, int)?;
			result.num = num_sq.div(scale, int)?;
			// result.den is already 'scale', so we don't need to change it.
		}

		if invert {
			result = one.div(&result, int)?;
		}

		Ok(Exact::new(result, false))
	}

	pub(crate) fn ln<I: Interrupt>(self, int: &I) -> FResult<Exact<Self>> {
		let one = Self::from(1);
		if self <= 0.into() {
			return Err(out_of_range(
				self.fm(int)?,
				Range {
					start: RangeBound::Open(0),
					end: RangeBound::None,
				},
			));
		}
		if self == one {
			return Ok(Exact::new(0.into(), true));
		}

		// Estimate k = floor(log2(x)) to avoid overflow with large inputs (e.g. 2^1024)
		let num_log2 = self.num.log2(int)?;
		let den_log2 = self.den.log2(int)?;
		let k = (num_log2 - den_log2).floor().to_i64().unwrap();

		let pow_2_k = if k >= 0 {
			Self::from(1).mul(
				&Self::from(2).pow(Self::from(k.unsigned_abs()), int)?.value,
				int,
			)?
		} else {
			Self::from(1).div(
				&Self::from(2).pow(Self::from(k.unsigned_abs()), int)?.value,
				int,
			)?
		};

		let y = self.div(&pow_2_k, int)?;

		let num = y.clone().sub(one.clone(), int)?;
		let den = y.clone().add(one.clone(), int)?;
		let z = num.div(&den, int)?;

		let ln_y = Self::ln_series_inv_tanh(z, int)?;
		let k_ln2 = Self::high_precision_ln_2().mul(&Self::from(k.unsigned_abs()), int)?;

		let result = if k >= 0 {
			k_ln2.add(ln_y, int)?
		} else {
			ln_y.sub(k_ln2, int)?
		};

		Ok(Exact::new(result, false))
	}

	pub(crate) fn log2<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let ln_val = self.ln(int)?;
		ln_val.value.div(&Self::high_precision_ln_2(), int)
	}

	pub(crate) fn log10<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let ln_val = self.ln(int)?;
		ln_val.value.div(&Self::high_precision_ln_10(), int)
	}

	pub(crate) fn sinh<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);
		let two = Self::from(2);

		// e^x
		let exp_x = self.clone().exp(int)?;
		// e^-x = 1 / e^x
		let exp_neg_x = one.div(&exp_x.value, int)?;

		// (e^x - e^-x) / 2
		let num = exp_x.value.sub(exp_neg_x, int)?;
		num.div(&two, int)
	}

	pub(crate) fn cosh<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);
		let two = Self::from(2);

		let exp_x = self.clone().exp(int)?;
		let exp_neg_x = one.div(&exp_x.value, int)?;

		let num = exp_x.value.add(exp_neg_x, int)?;
		num.div(&two, int)
	}

	pub(crate) fn tanh<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);
		let two = Self::from(2);

		// e^2x
		let two_x = self.mul(&two, int)?;
		let exp_2x = two_x.exp(int)?;

		let num = exp_2x.value.clone().sub(one.clone(), int)?;
		let den = exp_2x.value.add(one, int)?;

		num.div(&den, int)
	}

	pub(crate) fn asinh<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);
		let two = Self::from(2);

		let x_sq = self.clone().mul(&self, int)?;
		let x_sq_plus_1 = x_sq.add(one, int)?;
		let sqrt = x_sq_plus_1.root_n(&two, int)?;

		let arg = self.add(sqrt.value, int)?;
		Ok(arg.ln(int)?.value)
	}

	pub(crate) fn acosh<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);

		if self < one {
			return Err(out_of_range(
				self.fm(int)?,
				Range {
					start: RangeBound::Closed(1),
					end: RangeBound::None,
				},
			));
		}

		let two = Self::from(2);

		let x_sq = self.clone().mul(&self, int)?;
		let x_sq_minus_1 = x_sq.sub(one, int)?;
		let sqrt = x_sq_minus_1.root_n(&two, int)?;

		let arg = self.add(sqrt.value, int)?;
		Ok(arg.ln(int)?.value)
	}

	pub(crate) fn atanh<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);

		if self >= one || self <= -one.clone() {
			return Err(out_of_range(self.fm(int)?, Range::open(-1, 1)));
		}

		let two = Self::from(2);

		let num = one.clone().add(self.clone(), int)?;
		let den = one.sub(self, int)?;
		let arg = num.div(&den, int)?;

		let ln_val = arg.ln(int)?;
		ln_val.value.div(&two, int)
	}

	fn high_precision_pi() -> Self {
		static PI_CACHE: OnceLock<BigRat> = OnceLock::new();
		PI_CACHE.get_or_init(|| {
			let int = &crate::interrupt::Never;
			let pi_str = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679821480865132823066470938446095505822317253594081284811174502841027019385211055596446229489549303819644288109756659334461284756482337867831652712019091456485669234603486104543266482";

			let split_idx = std::cmp::min(2 + Self::INTERNAL_PRECISION, pi_str.len());
			let slice = &pi_str[..split_idx];

			let integer = BigUint::from(3u64);
			let fraction_str = &slice[2..];

			let mut fraction = BigUint::from(0u64);
			let ten = BigUint::from(10u64);

			for c in fraction_str.chars() {
				let digit = u64::from(c.to_digit(10).unwrap());
				fraction = fraction.mul(&ten, int).unwrap().add(&BigUint::from(digit));
			}

			let denominator = BigUint::pow(
				&ten,
				&BigUint::from(fraction_str.len() as u64),
				int
			).unwrap();

			Self {
				sign: Sign::Positive,
				num: integer.mul(&denominator, int).unwrap().add(&fraction),
				den: denominator,
			}
		}).clone()
	}

	fn high_precision_two_pi() -> Self {
		static TWO_PI: OnceLock<BigRat> = OnceLock::new();
		TWO_PI
			.get_or_init(|| {
				let int = &crate::interrupt::Never;
				Self::high_precision_pi().mul(&Self::from(2), int).unwrap()
			})
			.clone()
	}

	fn high_precision_pi_over_2() -> Self {
		static PI_OVER_2: OnceLock<BigRat> = OnceLock::new();
		PI_OVER_2
			.get_or_init(|| {
				let int = &crate::interrupt::Never;
				Self::high_precision_pi().div(&Self::from(2), int).unwrap()
			})
			.clone()
	}

	fn high_precision_pi_over_4() -> Self {
		static PI_OVER_4: OnceLock<BigRat> = OnceLock::new();
		PI_OVER_4
			.get_or_init(|| {
				let int = &crate::interrupt::Never;
				Self::high_precision_pi().div(&Self::from(4), int).unwrap()
			})
			.clone()
	}

	fn fixed_point_scale() -> &'static BigUint {
		static SCALE_CACHE: OnceLock<BigUint> = OnceLock::new();
		SCALE_CACHE.get_or_init(|| {
			let int = &crate::interrupt::Never;
			let ten = BigUint::from(10u64);
			let exp = (Self::INTERNAL_PRECISION + Self::FIXED_POINT_GUARD_DIGITS) as u64;
			BigUint::pow(&ten, &BigUint::from(exp), int).unwrap()
		})
	}

	fn rem_euclid_rat<I: Interrupt>(self, m: Self, int: &I) -> FResult<Self> {
		let div = self.clone().div(&m, int)?;
		let floor = div.floor(int)?;
		self.sub(m.mul(&floor, int)?, int)
	}

	pub(crate) fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> FResult<Self> {
		self.add(-rhs, int)
	}

	pub(crate) fn abs<I: Interrupt>(self, _int: &I) -> Self {
		Self {
			sign: Sign::Positive,
			num: self.num,
			den: self.den,
		}
	}

	fn sin_series<I: Interrupt>(x: Self, int: &I) -> FResult<Self> {
		if x.is_definitely_zero() {
			return Ok(x);
		}

		// Dynamic Precision:
		// If x is smaller than the default scale (10^-200), we must increase the scale.
		let mut scale = Self::fixed_point_scale().clone();
		let x_mag = x.clone().abs(int);

		if x_mag < 1.into() {
			// Calculate required scale: (1/|x|) * 10^20
			// Use associated function syntax for BigUint::pow
			let guard_digits = BigUint::pow(&BigUint::from(10u64), &BigUint::from(20u64), int)?;

			let required_scale = Self::from(1)
				.div(&x_mag, int)?
				.mul(&Self::from(guard_digits), int)?;

			if let Ok(req_uint) = required_scale.ceil(int)?.try_as_biguint(int)
				&& req_uint > scale {
					scale = req_uint;
				}
		}

		let x_fixed = x.num.clone().mul(&scale.clone(), int)?.div(&x.den, int)?;

		if x_fixed == 0.into() {
			return Ok(x);
		}

		let mut result = x_fixed.clone();
		let mut term = x_fixed.clone();
		let x_sq = x_fixed.clone().mul(&x_fixed, int)?;
		let scale_sq = scale.clone().mul(&scale, int)?;

		for i in 1..Self::INTERNAL_PRECISION * 2 {
			term = term.mul(&x_sq, int)?;
			term = term.div(&scale_sq, int)?;
			let k1 = 2 * i;
			let k2 = 2 * i + 1;
			let divisor = BigUint::from(k1 as u64).mul(&BigUint::from(k2 as u64), int)?;
			term = term.div(&divisor, int)?;

			if term == 0.into() {
				break;
			}

			if i % 2 == 1 {
				result = result.sub(&term);
			} else {
				result = result.add(&term);
			}
		}

		Ok(Self {
			sign: Sign::Positive,
			num: result,
			den: scale,
		})
	}

	fn cos_series<I: Interrupt>(x: &Self, int: &I) -> FResult<Self> {
		let mut scale = Self::fixed_point_scale().clone();
		let x_mag = x.clone().abs(int);

		if x_mag < 1.into() && !x_mag.is_definitely_zero() {
			let guard_digits = BigUint::pow(&BigUint::from(10u64), &BigUint::from(20u64), int)?;

			let required_scale = Self::from(1)
				.div(&x_mag, int)?
				.mul(&Self::from(guard_digits), int)?;

			if let Ok(req_uint) = required_scale.ceil(int)?.try_as_biguint(int)
				&& req_uint > scale {
					scale = req_uint;
				}
		}

		let x_fixed = x.num.clone().mul(&scale.clone(), int)?.div(&x.den, int)?;

		if x_fixed == 0.into() {
			return Ok(Self::from(1));
		}

		let mut result = scale.clone();
		let mut term = scale.clone();
		let x_sq = x_fixed.clone().mul(&x_fixed, int)?;
		let scale_sq = scale.clone().mul(&scale, int)?;

		for i in 1..Self::INTERNAL_PRECISION * 2 {
			term = term.mul(&x_sq, int)?;
			term = term.div(&scale_sq, int)?;
			let k1 = 2 * i - 1;
			let k2 = 2 * i;
			let divisor = BigUint::from(k1 as u64).mul(&BigUint::from(k2 as u64), int)?;
			term = term.div(&divisor, int)?;

			if term == 0.into() {
				break;
			}

			if i % 2 == 1 {
				result = result.sub(&term);
			} else {
				result = result.add(&term);
			}
		}

		Ok(Self {
			sign: Sign::Positive,
			num: result,
			den: scale,
		})
	}

	fn atan_series<I: Interrupt>(x: Self, int: &I) -> FResult<Self> {
		if x.is_definitely_zero() {
			return Ok(x);
		}

		if x.sign == Sign::Negative {
			let pos_x = x.clone().abs(int);
			let res = Self::atan_series(pos_x, int)?;
			return Ok(-res);
		}

		let mut scale = Self::fixed_point_scale().clone();
		let x_mag = x.clone().abs(int);

		if x_mag < 1.into() {
			let guard_digits = BigUint::pow(&BigUint::from(10u64), &BigUint::from(20u64), int)?;

			let required_scale = Self::from(1)
				.div(&x_mag, int)?
				.mul(&Self::from(guard_digits), int)?;

			if let Ok(req_uint) = required_scale.ceil(int)?.try_as_biguint(int)
				&& req_uint > scale {
					scale = req_uint;
				}
		}

		let x_fixed = x.num.clone().mul(&scale.clone(), int)?.div(&x.den, int)?;

		if x_fixed == 0.into() {
			return Ok(x);
		}

		let mut result = x_fixed.clone();
		let mut term = x_fixed.clone();
		let x_sq = x_fixed.clone().mul(&x_fixed, int)?;
		let scale_sq = scale.clone().mul(&scale, int)?;

		for i in 1..Self::INTERNAL_PRECISION * 2 {
			term = term.mul(&x_sq, int)?;
			term = term.div(&scale_sq, int)?;

			if term == 0.into() {
				break;
			}

			let n = 2 * i + 1;
			let val = term.clone().div(&BigUint::from(n as u64), int)?;

			if i % 2 == 1 {
				result = result.sub(&val);
			} else {
				result = result.add(&val);
			}
		}

		Ok(Self {
			sign: Sign::Positive,
			num: result,
			den: scale,
		})
	}

	// sin works for all real numbers
	pub(crate) fn sin<I: Interrupt>(self, int: &I) -> FResult<Exact<Self>> {
		Ok(if self == 0.into() {
			Exact::new(Self::from(0), true)
		} else {
			let pi = Self::high_precision_pi();
			let two_pi = Self::high_precision_two_pi();
			let pi_over_2 = Self::high_precision_pi_over_2();
			let pi_over_4 = Self::high_precision_pi_over_4();

			let mut x = self.rem_euclid_rat(two_pi, int)?;
			let mut sign = Sign::Positive;

			if x >= pi {
				x = x.sub(pi.clone(), int)?;
				sign = sign.flip();
			}
			if x > pi_over_2 {
				x = pi.sub(x, int)?;
			}

			if x > pi_over_4 {
				x = pi_over_2.sub(x, int)?;
				let val = Self::cos_series(&x, int)?;
				Exact::new(if sign == Sign::Negative { -val } else { val }, false)
			} else {
				let val = Self::sin_series(x, int)?;
				Exact::new(if sign == Sign::Negative { -val } else { val }, false)
			}
		})
	}

	// cos works for all real numbers
	pub(crate) fn cos<I: Interrupt>(self, int: &I) -> FResult<Exact<Self>> {
		Ok(if self == 0.into() {
			Exact::new(Self::from(1), true)
		} else {
			let pi = Self::high_precision_pi();
			let two_pi = Self::high_precision_two_pi();
			let pi_over_2 = Self::high_precision_pi_over_2();
			let pi_over_4 = Self::high_precision_pi_over_4();

			let mut x = self.rem_euclid_rat(two_pi.clone(), int)?;
			let mut sign = Sign::Positive;

			if x >= pi {
				x = two_pi.sub(x, int)?;
			}
			if x > pi_over_2 {
				x = pi.sub(x, int)?;
				sign = sign.flip();
			}

			if x > pi_over_4 {
				x = pi_over_2.sub(x, int)?;
				let val = Self::sin_series(x, int)?;
				Exact::new(if sign == Sign::Negative { -val } else { val }, false)
			} else {
				let val = Self::cos_series(&x, int)?;
				Exact::new(if sign == Sign::Negative { -val } else { val }, false)
			}
		})
	}

	// asin, acos and atan only work for values between -1 and 1
	pub(crate) fn atan<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);
		let mut x = self.clone().abs(int);
		let mut invert = false;

		if x > one {
			x = one.clone().div(&x, int)?;
			invert = true;
		}

		let mut shift_pi_4 = false;

		if &x > Self::atan_reduction_threshold() {
			let num = x.clone().sub(one.clone(), int)?;
			let den = one.clone().add(x, int)?;
			x = num.div(&den, int)?;
			shift_pi_4 = true;
		}

		let mut result = Self::atan_series(x, int)?;

		if shift_pi_4 {
			result = result.add(Self::high_precision_pi_over_4(), int)?;
		}

		if invert {
			let pi_over_2 = Self::high_precision_pi_over_2();
			result = pi_over_2.sub(result, int)?;
		}

		if self.sign == Sign::Negative {
			Ok(-result)
		} else {
			Ok(result)
		}
	}

	fn atan_reduction_threshold() -> &'static Self {
		static THRESHOLD: OnceLock<BigRat> = OnceLock::new();
		THRESHOLD.get_or_init(|| {
			// 0.4142 = 2071 / 5000
			Self {
				sign: Sign::Positive,
				num: BigUint::from(2071u64),
				den: BigUint::from(5000u64),
			}
		})
	}

	pub(crate) fn asin<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);

		// Check domain: -1 <= x <= 1
		if self > one || self < -one.clone() {
			return Err(out_of_range(self.fm(int)?, Range::open(-1, 1)));
		}

		// Handle edges x = 1 and x = -1 explicitly to avoid division by zero in the formula
		if self == one {
			return Ok(Self::high_precision_pi_over_2());
		}
		if self == -one.clone() {
			return Ok(-Self::high_precision_pi_over_2());
		}

		// asin(x) = atan(x / sqrt(1 - x^2))
		let x_sq = self.clone().mul(&self, int)?;
		let one_minus_x_sq = one.sub(x_sq, int)?;

		// Compute sqrt(1 - x^2)
		// Using root_n(2) which relies on the high-precision iter_root_n
		let sqrt_den = one_minus_x_sq.root_n(&Self::from(2), int)?;

		let argument = self.div(&sqrt_den.value, int)?;
		argument.atan(int)
	}

	pub(crate) fn acos<I: Interrupt>(self, int: &I) -> FResult<Self> {
		let one = Self::from(1);

		// Check domain: -1 <= x <= 1
		if self > one || self < -one {
			return Err(out_of_range(self.fm(int)?, Range::open(-1, 1)));
		}

		// acos(x) = pi/2 - asin(x)
		let pi_over_2 = Self::high_precision_pi_over_2();
		let asin_val = self.asin(int)?;

		pi_over_2.sub(asin_val, int)
	}

	pub(crate) fn atan2<I: Interrupt>(self, rhs: Self, int: &I) -> FResult<Self> {
		let y = self;
		let x = rhs;

		if x.is_definitely_zero() {
			if y.is_definitely_zero() {
				// atan2(0, 0) is undefined
				return Err(FendError::DivideByZero);
			}
			let pi_over_2 = Self::high_precision_pi_over_2();
			if y.sign == Sign::Positive {
				return Ok(pi_over_2);
			}
			return Ok(-pi_over_2);
		}

		if x.sign == Sign::Positive {
			// Quadrants 1 and 4: atan(y/x)
			let ratio = y.div(&x, int)?;
			return ratio.atan(int);
		}

		// Quadrants 2 and 3: x is negative
		let ratio = y.clone().div(&x, int)?;
		let atan_val = ratio.atan(int)?;
		let pi = Self::high_precision_pi();

		if y.sign == Sign::Positive || y.is_definitely_zero() {
			// Quadrant 2 (y >= 0, x < 0): result is atan(y/x) + pi
			// Note: atan(y/x) will be negative
			atan_val.add(pi, int)
		} else {
			// Quadrant 3 (y < 0, x < 0): result is atan(y/x) - pi
			// Note: atan(y/x) will be positive
			atan_val.sub(pi, int)
		}
	}

	fn apply_uint_op<I: Interrupt, R>(
		mut self,
		f: impl FnOnce(BigUint, &I) -> FResult<R>,
		int: &I,
	) -> FResult<R> {
		self = self.simplify(int)?;
		if self.den != 1.into() {
			let n = self.fm(int)?;
			return Err(FendError::MustBeAnInteger(Box::new(n)));
		}
		if self.sign == Sign::Negative && self.num != 0.into() {
			return Err(out_of_range(self.fm(int)?, Range::ZERO_OR_GREATER));
		}
		f(self.num, int)
	}

	pub(crate) fn factorial<I: Interrupt>(self, int: &I) -> FResult<Self> {
		Ok(self.apply_uint_op(BigUint::factorial, int)?.into())
	}

	pub(crate) fn floor<I: Interrupt>(mut self, int: &I) -> FResult<Self> {
		if self.is_definitely_zero() {
			return Ok(self);
		}
		self = self.simplify(int)?;

		// Use integer division.
		// divmod takes &self.den, so we don't move anything.
		let (q, rem) = self.num.divmod(&self.den, int)?;

		if self.sign == Sign::Positive {
			// Example: 10/3 = 3 rem 1 -> floor is 3
			Ok(Self {
				sign: Sign::Positive,
				num: q,
				den: 1.into(),
			})
		} else {
			// Example: -10/3 = -3 rem 1 -> floor is -4
			// Example: -9/3 = -3 rem 0 -> floor is -3
			let ans = if rem.is_definitely_zero() {
				q
			} else {
				q.add(&1.into())
			};
			Ok(Self {
				sign: Sign::Negative,
				num: ans,
				den: 1.into(),
			})
		}
	}

	pub(crate) fn ceil<I: Interrupt>(mut self, int: &I) -> FResult<Self> {
		if self.is_definitely_zero() {
			return Ok(self);
		}
		self = self.simplify(int)?;

		let (q, rem) = self.num.divmod(&self.den, int)?;

		if self.sign == Sign::Positive {
			// Example: 10/3 = 3 rem 1 -> ceil is 4
			let ans = if rem.is_definitely_zero() {
				q
			} else {
				q.add(&1.into())
			};
			Ok(Self {
				sign: Sign::Positive,
				num: ans,
				den: 1.into(),
			})
		} else {
			// Example: -10/3 = -3 rem 1 -> ceil is -3
			Ok(Self {
				sign: Sign::Negative,
				num: q,
				den: 1.into(),
			})
		}
	}

	pub(crate) fn round<I: Interrupt>(mut self, int: &I) -> FResult<Self> {
		if self.is_definitely_zero() {
			return Ok(self);
		}
		self = self.simplify(int)?;

		let (q, rem) = self.num.divmod(&self.den, int)?;

		// Rounding logic: Round up if remainder >= denominator / 2
		// Equivalent to: 2 * remainder >= denominator
		let rem_2 = rem.mul(&2.into(), int)?;
		let round_up = rem_2 >= self.den;

		let ans = if round_up { q.add(&1.into()) } else { q };

		Ok(Self {
			sign: self.sign,
			num: ans,
			den: 1.into(),
		})
	}

	pub(crate) fn bitwise<I: Interrupt>(
		self,
		rhs: Self,
		op: crate::ast::BitwiseBop,
		int: &I,
	) -> FResult<Self> {
		use crate::ast::BitwiseBop;

		Ok(self
			.apply_uint_op(
				|lhs, int| {
					let rhs = rhs.apply_uint_op(|rhs, _int| Ok(rhs), int)?;
					let result = match op {
						BitwiseBop::And => lhs.bitwise_and(&rhs),
						BitwiseBop::Or => lhs.bitwise_or(&rhs),
						BitwiseBop::Xor => lhs.bitwise_xor(&rhs),
						BitwiseBop::LeftShift => lhs.lshift_n(&rhs, int)?,
						BitwiseBop::RightShift => lhs.rshift_n(&rhs, int)?,
					};
					Ok(result)
				},
				int,
			)?
			.into())
	}

	/// compute a + b
	fn add_internal<I: Interrupt>(self, rhs: Self, int: &I) -> FResult<Self> {
		// a + b == -((-a) + (-b))
		if self.sign == Sign::Negative {
			return Ok(-((-self).add_internal(-rhs, int)?));
		}

		assert_eq!(self.sign, Sign::Positive);

		Ok(if self.den == rhs.den {
			if rhs.sign == Sign::Negative && self.num < rhs.num {
				Self {
					sign: Sign::Negative,
					num: rhs.num.sub(&self.num),
					den: self.den,
				}
			} else {
				Self {
					sign: Sign::Positive,
					num: if rhs.sign == Sign::Positive {
						self.num.add(&rhs.num)
					} else {
						self.num.sub(&rhs.num)
					},
					den: self.den,
				}
			}
		} else {
			let gcd = BigUint::gcd(self.den.clone(), rhs.den.clone(), int)?;
			let new_denominator = self.den.clone().mul(&rhs.den, int)?.div(&gcd, int)?;
			let a = self.num.mul(&rhs.den, int)?.div(&gcd, int)?;
			let b = rhs.num.mul(&self.den, int)?.div(&gcd, int)?;

			if rhs.sign == Sign::Negative && a < b {
				Self {
					sign: Sign::Negative,
					num: b.sub(&a),
					den: new_denominator,
				}
			} else {
				Self {
					sign: Sign::Positive,
					num: if rhs.sign == Sign::Positive {
						a.add(&b)
					} else {
						a.sub(&b)
					},
					den: new_denominator,
				}
			}
		})
	}

	fn simplify<I: Interrupt>(mut self, int: &I) -> FResult<Self> {
		if self.den == 1.into() {
			return Ok(self);
		}
		let gcd = BigUint::gcd(self.num.clone(), self.den.clone(), int)?;
		self.num = self.num.div(&gcd, int)?;
		self.den = self.den.div(&gcd, int)?;
		Ok(self)
	}

	pub(crate) fn div<I: Interrupt>(self, rhs: &Self, int: &I) -> FResult<Self> {
		if rhs.num == 0.into() {
			return Err(FendError::DivideByZero);
		}
		Ok(Self {
			sign: Sign::sign_of_product(self.sign, rhs.sign),
			num: self.num.mul(&rhs.den, int)?,
			den: self.den.mul(&rhs.num, int)?,
		})
	}

	pub(crate) fn modulo<I: Interrupt>(mut self, mut rhs: Self, int: &I) -> FResult<Self> {
		if rhs.num == 0.into() {
			return Err(FendError::ModuloByZero);
		}
		self = self.simplify(int)?;
		rhs = rhs.simplify(int)?;
		if (self.sign == Sign::Negative && self.num != 0.into())
			|| rhs.sign == Sign::Negative
			|| self.den != 1.into()
			|| rhs.den != 1.into()
		{
			return Err(FendError::ModuloForPositiveInts);
		}
		Ok(Self {
			sign: Sign::Positive,
			num: self.num.divmod(&rhs.num, int)?.1,
			den: 1.into(),
		})
	}

	// test if this fraction has a terminating representation
	// e.g. in base 10: 1/4 = 0.25, but not 1/3
	fn terminates_in_base<I: Interrupt>(&self, base: Base, int: &I) -> FResult<bool> {
		let mut x = self.clone();
		let base_as_u64: u64 = base.base_as_u8().into();
		let base = Self {
			sign: Sign::Positive,
			num: base_as_u64.into(),
			den: 1.into(),
		};
		loop {
			let old_den = x.den.clone();
			x = x.mul(&base, int)?.simplify(int)?;
			let new_den = x.den.clone();
			if new_den == old_den {
				break;
			}
		}
		Ok(x.den == 1.into())
	}

	fn format_as_integer<I: Interrupt>(
		num: &BigUint,
		base: Base,
		sign: Sign,
		term: &'static str,
		use_parens_if_product: bool,
		sf_limit: Option<usize>,
		int: &I,
	) -> FResult<Exact<FormattedBigRat>> {
		let (ty, exact) = if !term.is_empty() && !base.has_prefix() && num == &1.into() {
			(FormattedBigRatType::Integer(None, false, term, false), true)
		} else {
			let formatted_int = num.format(
				&biguint::FormatOptions {
					base,
					write_base_prefix: true,
					sf_limit,
				},
				int,
			)?;
			(
				FormattedBigRatType::Integer(
					Some(formatted_int.value),
					!term.is_empty() && base.base_as_u8() > 10,
					term,
					// print surrounding parentheses if the number is imaginary
					use_parens_if_product && !term.is_empty(),
				),
				formatted_int.exact,
			)
		};
		Ok(Exact::new(FormattedBigRat { sign, ty }, exact))
	}

	fn format_as_fraction<I: Interrupt>(
		&self,
		base: Base,
		sign: Sign,
		term: &'static str,
		mixed: bool,
		use_parens: bool,
		int: &I,
	) -> FResult<Exact<FormattedBigRat>> {
		let format_options = biguint::FormatOptions {
			base,
			write_base_prefix: true,
			sf_limit: None,
		};
		let formatted_den = self.den.format(&format_options, int)?;
		let (pref, num, prefix_exact) = if mixed {
			let (prefix, num) = self.num.divmod(&self.den, int)?;
			if prefix == 0.into() {
				(None, num, true)
			} else {
				let formatted_prefix = prefix.format(&format_options, int)?;
				(Some(formatted_prefix.value), num, formatted_prefix.exact)
			}
		} else {
			(None, self.num.clone(), true)
		};
		// mixed fractions without a prefix aren't really mixed
		let actually_mixed = pref.is_some();
		let (ty, num_exact) =
			if !term.is_empty() && !actually_mixed && !base.has_prefix() && num == 1.into() {
				(
					FormattedBigRatType::Fraction(
						pref,
						None,
						false,
						term,
						formatted_den.value,
						"",
						use_parens,
					),
					true,
				)
			} else {
				let formatted_num = num.format(&format_options, int)?;
				let i_suffix = term;
				let space = !term.is_empty() && (base.base_as_u8() >= 19 || actually_mixed);
				let (isuf1, isuf2) = if actually_mixed {
					("", i_suffix)
				} else {
					(i_suffix, "")
				};
				(
					FormattedBigRatType::Fraction(
						pref,
						Some(formatted_num.value),
						space,
						isuf1,
						formatted_den.value,
						isuf2,
						use_parens,
					),
					formatted_num.exact,
				)
			};
		Ok(Exact::new(
			FormattedBigRat { sign, ty },
			formatted_den.exact && prefix_exact && num_exact,
		))
	}

	#[allow(clippy::too_many_arguments)]
	fn format_as_decimal<I: Interrupt>(
		&self,
		style: FormattingStyle,
		base: Base,
		sign: Sign,
		term: &'static str,
		mut terminating: impl FnMut() -> FResult<bool>,
		decimal_separator: DecimalSeparatorStyle,
		int: &I,
	) -> FResult<Exact<FormattedBigRat>> {
		let integer_part = self.clone().num.div(&self.den, int)?;
		let sf_limit = if let FormattingStyle::SignificantFigures(sf) = style {
			Some(sf)
		} else {
			None
		};
		let formatted_integer_part = integer_part.format(
			&biguint::FormatOptions {
				base,
				write_base_prefix: true,
				sf_limit,
			},
			int,
		)?;

		let num_trailing_digits_to_print = if style == FormattingStyle::ExactFloat
			|| (style == FormattingStyle::Auto && terminating()?)
			|| style == FormattingStyle::Exact
		{
			MaxDigitsToPrint::AllDigits
		} else if let FormattingStyle::DecimalPlaces(n) = style {
			MaxDigitsToPrint::DecimalPlaces(n)
		} else if let FormattingStyle::SignificantFigures(sf) = style {
			let num_digits_of_int_part = formatted_integer_part.value.num_digits();
			// reduce decimal places by however many digits we already printed
			// in the integer portion
			//
			// saturate to zero in case we already exhausted all digits and
			// shouldn't print any decimal places
			let dp = sf.saturating_sub(num_digits_of_int_part);
			if integer_part == 0.into() {
				// if the integer part is 0, we don't want leading zeroes
				// after the decimal point to affect the number of non-zero
				// digits printed

				// we add 1 to the number of decimal places in this case because
				// the integer component of '0' shouldn't count against the
				// number of significant figures
				MaxDigitsToPrint::DpButIgnoreLeadingZeroes(dp + 1)
			} else {
				MaxDigitsToPrint::DecimalPlaces(dp)
			}
		} else {
			MaxDigitsToPrint::DecimalPlaces(10)
		};
		let print_integer_part = |ignore_minus_if_zero: bool| {
			let sign =
				if sign == Sign::Negative && (!ignore_minus_if_zero || integer_part != 0.into()) {
					Sign::Negative
				} else {
					Sign::Positive
				};
			Ok((sign, formatted_integer_part.value.to_string()))
		};
		let integer_as_rational = Self {
			sign: Sign::Positive,
			num: integer_part.clone(),
			den: 1.into(),
		};
		let remaining_fraction = self.clone().add(-integer_as_rational, int)?;
		let (sign, formatted_trailing_digits) = Self::format_trailing_digits(
			base,
			&remaining_fraction.num,
			&remaining_fraction.den,
			num_trailing_digits_to_print,
			terminating,
			print_integer_part,
			decimal_separator,
			int,
		)?;
		Ok(Exact::new(
			FormattedBigRat {
				sign,
				ty: FormattedBigRatType::Decimal(
					formatted_trailing_digits.value,
					!term.is_empty() && base.base_as_u8() > 10,
					term,
				),
			},
			formatted_integer_part.exact && formatted_trailing_digits.exact,
		))
	}

	/// Prints the decimal expansion of num/den, where num < den, in the given base.
	#[allow(clippy::too_many_arguments)]
	fn format_trailing_digits<I: Interrupt>(
		base: Base,
		numerator: &BigUint,
		denominator: &BigUint,
		max_digits: MaxDigitsToPrint,
		mut terminating: impl FnMut() -> FResult<bool>,
		print_integer_part: impl Fn(bool) -> FResult<(Sign, String)>,
		decimal_separator: DecimalSeparatorStyle,
		int: &I,
	) -> FResult<(Sign, Exact<String>)> {
		let base_as_u64: u64 = base.base_as_u8().into();
		let b: BigUint = base_as_u64.into();
		let next_digit =
			|i: usize, num: BigUint, base: &BigUint| -> Result<(BigUint, BigUint), NextDigitErr> {
				test_int(int)?;
				if num == 0.into() {
					// reached the end of the number
					return Err(NextDigitErr::Terminated { round_up: false });
				}
				if max_digits == MaxDigitsToPrint::DecimalPlaces(i)
					|| max_digits == MaxDigitsToPrint::DpButIgnoreLeadingZeroes(i)
				{
					// round up if remaining fraction is >1/2
					return Err(NextDigitErr::Terminated {
						round_up: num.mul(&2.into(), int)? >= *denominator,
					});
				}
				// digit = base * numerator / denominator
				// next_numerator = base * numerator - digit * denominator
				let bnum = num.mul(base, int)?;
				let digit = bnum.clone().div(denominator, int)?;
				let next_num = bnum.sub(&digit.clone().mul(denominator, int)?);
				Ok((next_num, digit))
			};
		let fold_digits = |mut s: String, digit: BigUint| -> FResult<String> {
			let digit_str = digit
				.format(
					&biguint::FormatOptions {
						base,
						write_base_prefix: false,
						sf_limit: None,
					},
					int,
				)?
				.value
				.to_string();
			s.push_str(digit_str.as_str());
			Ok(s)
		};
		let skip_cycle_detection = max_digits != MaxDigitsToPrint::AllDigits || terminating()?;
		if skip_cycle_detection {
			let ignore_number_of_leading_zeroes =
				matches!(max_digits, MaxDigitsToPrint::DpButIgnoreLeadingZeroes(_));
			return Self::format_nonrecurring(
				numerator,
				base,
				ignore_number_of_leading_zeroes,
				next_digit,
				print_integer_part,
				decimal_separator,
				int,
			);
		}
		match Self::brents_algorithm(
			next_digit,
			fold_digits,
			numerator.clone(),
			&b,
			String::new(),
		) {
			Ok((cycle_length, location, output)) => {
				let (ab, _) = output.split_at(location + cycle_length);
				let (a, b) = ab.split_at(location);
				let (sign, formatted_int) = print_integer_part(false)?;
				let mut trailing_digits = String::new();
				trailing_digits.push_str(&formatted_int);
				trailing_digits.push(decimal_separator.decimal_separator());
				trailing_digits.push_str(a);
				trailing_digits.push('(');
				trailing_digits.push_str(b);
				trailing_digits.push(')');
				Ok((sign, Exact::new(trailing_digits, true))) // the recurring decimal is exact
			}
			Err(NextDigitErr::Terminated { round_up: _ }) => {
				panic!("decimal number terminated unexpectedly");
			}
			Err(NextDigitErr::Error(e)) => Err(e),
		}
	}

	fn format_nonrecurring<I: Interrupt>(
		numerator: &BigUint,
		base: Base,
		ignore_number_of_leading_zeroes: bool,
		mut next_digit: impl FnMut(usize, BigUint, &BigUint) -> Result<(BigUint, BigUint), NextDigitErr>,
		print_integer_part: impl Fn(bool) -> FResult<(Sign, String)>,
		decimal_separator: DecimalSeparatorStyle,
		int: &I,
	) -> FResult<(Sign, Exact<String>)> {
		let mut current_numerator = numerator.clone();
		let mut i = 0;
		let mut trailing_zeroes = 0;
		// this becomes Some(_) when we write the decimal point
		let mut actual_sign = None;
		let mut trailing_digits = String::new();
		let b: BigUint = u64::from(base.base_as_u8()).into();
		loop {
			match next_digit(i, current_numerator.clone(), &b) {
				Ok((next_n, digit)) => {
					current_numerator = next_n;
					if digit == 0.into() {
						trailing_zeroes += 1;
						if !(i == 0 && ignore_number_of_leading_zeroes) {
							i += 1;
						}
					} else {
						if actual_sign.is_none() {
							// always print leading minus because we have non-zero digits
							let (sign, formatted_int) = print_integer_part(false)?;
							actual_sign = Some(sign);
							trailing_digits.push_str(&formatted_int);
							trailing_digits.push(decimal_separator.decimal_separator());
						}
						for _ in 0..trailing_zeroes {
							trailing_digits.push('0');
						}
						trailing_zeroes = 0;
						trailing_digits.push_str(
							&digit
								.format(
									&biguint::FormatOptions {
										base,
										write_base_prefix: false,
										sf_limit: None,
									},
									int,
								)?
								.value
								.to_string(),
						);
						i += 1;
					}
				}
				Err(NextDigitErr::Terminated { round_up }) => {
					let sign = if let Some(actual_sign) = actual_sign {
						actual_sign
					} else {
						// if we reach this point we haven't printed any non-zero digits,
						// so we can skip the leading minus sign if the integer part is also zero
						let (sign, formatted_int) = print_integer_part(true)?;
						trailing_digits.push_str(&formatted_int);
						sign
					};
					if round_up {
						// todo
					}
					// is the number exact, or did we need to truncate?
					let exact = current_numerator == 0.into();
					return Ok((sign, Exact::new(trailing_digits, exact)));
				}
				Err(NextDigitErr::Error(e)) => {
					return Err(e);
				}
			}
		}
	}

	// Brent's cycle detection algorithm (based on pseudocode from Wikipedia)
	// returns (length of cycle, index of first element of cycle, collected result)
	fn brents_algorithm<T: Clone + Eq, R, U, E1: From<E2>, E2>(
		f: impl Fn(usize, T, &T) -> Result<(T, U), E1>,
		g: impl Fn(R, U) -> Result<R, E2>,
		x0: T,
		state: &T,
		r0: R,
	) -> Result<(usize, usize, R), E1> {
		// main phase: search successive powers of two
		let mut power = 1;
		// lam is the length of the cycle
		let mut lam = 1;
		let mut tortoise = x0.clone();
		let mut depth = 0;
		let (mut hare, _) = f(depth, x0.clone(), state)?;
		depth += 1;
		while tortoise != hare {
			if power == lam {
				tortoise = hare.clone();
				power *= 2;
				lam = 0;
			}
			hare = f(depth, hare, state)?.0;
			depth += 1;
			lam += 1;
		}

		// Find the position of the first repetition of length lam
		tortoise = x0.clone();
		hare = x0;
		let mut collected_res = r0;
		let mut hare_depth = 0;
		for _ in 0..lam {
			let (new_hare, u) = f(hare_depth, hare, state)?;
			hare_depth += 1;
			hare = new_hare;
			collected_res = g(collected_res, u)?;
		}
		// The distance between the hare and tortoise is now lam.

		// Next, the hare and tortoise move at same speed until they agree
		// mu will be the length of the initial sequence, before the cycle
		let mut mu = 0;
		let mut tortoise_depth = 0;
		while tortoise != hare {
			tortoise = f(tortoise_depth, tortoise, state)?.0;
			tortoise_depth += 1;
			let (new_hare, u) = f(hare_depth, hare, state)?;
			hare_depth += 1;
			hare = new_hare;
			collected_res = g(collected_res, u)?;
			mu += 1;
		}
		Ok((lam, mu, collected_res))
	}

	pub(crate) fn pow<I: Interrupt>(mut self, mut rhs: Self, int: &I) -> FResult<Exact<Self>> {
		self = self.simplify(int)?;
		rhs = rhs.simplify(int)?;

		if self.num != 0.into() && self.sign == Sign::Negative && rhs.den != 1.into() {
			return Err(FendError::RootsOfNegativeNumbers);
		}
		if rhs.sign == Sign::Negative {
			rhs.sign = Sign::Positive;
			let inverse_res = self.pow(rhs, int)?;
			return Ok(Exact::new(
				Self::from(1).div(&inverse_res.value, int)?,
				inverse_res.exact,
			));
		}

		// Use exp/ln approximation if:
		// 1. The denominator is large (e.g. ^0.0001), indicating a complex decimal root.
		// 2. The numerator is large AND it's a root (e.g. ^210.1 = ^2101/10).
		let large_root = rhs.den > BigUint::from(1000u64);
		let large_power = rhs.den > BigUint::from(1u64) && rhs.num > BigUint::from(100u64);

		if large_root || large_power {
			// Case 1: Simple fractional root (e.g. ^0.00001) where we might want exactness
			if rhs.num == 1.into() && !large_power {
				let n = rhs.den.clone();
				let num_root = self.num.clone().root_n(&n, int)?;
				let den_root = self.den.clone().root_n(&n, int)?;

				if num_root.exact && den_root.exact {
					return Ok(Exact::new(
						Self {
							sign: Sign::Positive,
							num: num_root.value,
							den: den_root.value,
						},
						true,
					));
				}
			}

			// Case 2: Complex decimal or large hybrid power -> Fast Approximation
			let ln_x = self.ln(int)?;
			let y_ln_x = ln_x.value.mul(&rhs, int)?;
			return y_ln_x.exp(int);
		}

		// Standard Path (Small denominators & Small numerators, e.g. ^0.5, ^2/3)
		let result_sign = if self.sign == Sign::Positive || rhs.num.is_even(int)? {
			Sign::Positive
		} else {
			Sign::Negative
		};

		let pow_res = Self {
			sign: result_sign,
			num: BigUint::pow(&self.num, &rhs.num, int)?,
			den: BigUint::pow(&self.den, &rhs.num, int)?,
		};

		if rhs.den == 1.into() {
			Ok(Exact::new(pow_res, true))
		} else {
			Ok(pow_res.root_n(
				&Self {
					sign: Sign::Positive,
					num: rhs.den,
					den: 1.into(),
				},
				int,
			)?)
		}
	}

	/// n must be an integer
	fn iter_root_n<I: Interrupt>(
		mut low_bound: Self,
		val: &Self,
		n: &Self,
		int: &I,
	) -> FResult<Self> {
		let mut high_bound = low_bound.clone().add(1.into(), int)?;
		// Each iteration adds 1 bit of precision.
		// Since log2(10) ≈ 3.32, we need ~3.32 iterations per decimal digit. Use 3.35 as safety factor.
		let iterations = (Self::INTERNAL_PRECISION * 335) / 100;
		for _ in 0..iterations {
			let guess = low_bound
				.clone()
				.add(high_bound.clone(), int)?
				.div(&2.into(), int)?;
			if &guess.clone().pow(n.clone(), int)?.value < val {
				low_bound = guess;
			} else {
				high_bound = guess;
			}
		}
		low_bound.add(high_bound, int)?.div(&2.into(), int)
	}

	// the boolean indicates whether or not the result is exact
	// n must be an integer
	pub(crate) fn root_n<I: Interrupt>(self, n: &Self, int: &I) -> FResult<Exact<Self>> {
		if self.num != 0.into() && self.sign == Sign::Negative {
			return Err(FendError::RootsOfNegativeNumbers);
		}
		let n = n.clone().simplify(int)?;
		if n.den != 1.into() || n.sign == Sign::Negative {
			return Err(FendError::NonIntegerNegRoots);
		}
		let n = &n.num;
		if self.num == 0.into() {
			return Ok(Exact::new(self, true));
		}
		let num = self.clone().num.root_n(n, int)?;
		let den = self.clone().den.root_n(n, int)?;
		if num.exact && den.exact {
			return Ok(Exact::new(
				Self {
					sign: Sign::Positive,
					num: num.value,
					den: den.value,
				},
				true,
			));
		}
		// TODO check in which cases this might still be exact
		let num_rat = if num.exact {
			Self::from(num.value)
		} else {
			Self::iter_root_n(
				Self::from(num.value),
				&Self::from(self.num),
				&Self::from(n.clone()),
				int,
			)?
		};
		let den_rat = if den.exact {
			Self::from(den.value)
		} else {
			Self::iter_root_n(
				Self::from(den.value),
				&Self::from(self.den),
				&Self::from(n.clone()),
				int,
			)?
		};
		Ok(Exact::new(num_rat.div(&den_rat, int)?, false))
	}

	pub(crate) fn mul<I: Interrupt>(self, rhs: &Self, int: &I) -> FResult<Self> {
		Ok(Self {
			sign: Sign::sign_of_product(self.sign, rhs.sign),
			num: self.num.mul(&rhs.num, int)?,
			den: self.den.mul(&rhs.den, int)?,
		})
	}

	pub(crate) fn add<I: Interrupt>(self, rhs: Self, int: &I) -> FResult<Self> {
		self.add_internal(rhs, int)
	}

	pub(crate) fn is_definitely_zero(&self) -> bool {
		self.num.is_definitely_zero()
	}

	pub(crate) fn is_definitely_one(&self) -> bool {
		self.sign == Sign::Positive && self.num.is_definitely_one() && self.den.is_definitely_one()
	}

	pub(crate) fn combination<I: Interrupt>(self, rhs: Self, int: &I) -> FResult<Self> {
		let n_factorial = self.clone().factorial(int)?;
		let r_factorial = rhs.clone().factorial(int)?;
		let n_minus_r_factorial = self.add(-rhs, int)?.factorial(int)?;
		let denominator = r_factorial.mul(&n_minus_r_factorial, int)?;
		n_factorial.div(&denominator, int)
	}

	pub(crate) fn permutation<I: Interrupt>(self, rhs: Self, int: &I) -> FResult<Self> {
		let n_factorial = self.clone().factorial(int)?;
		let n_minus_r_factorial = self.add(-rhs, int)?.factorial(int)?;
		n_factorial.div(&n_minus_r_factorial, int)
	}
}
enum NextDigitErr {
	Error(FendError),
	/// Stop printing digits because we've reached the end of the number or the
	/// limit of how much we want to print
	Terminated {
		round_up: bool,
	},
}

impl From<FendError> for NextDigitErr {
	fn from(e: FendError) -> Self {
		Self::Error(e)
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MaxDigitsToPrint {
	/// Print all digits, possibly by writing recurring decimals in parentheses
	AllDigits,
	/// Print only the given number of decimal places, omitting any trailing zeroes
	DecimalPlaces(usize),
	/// Print only the given number of dps, but ignore leading zeroes after the decimal point
	DpButIgnoreLeadingZeroes(usize),
}

impl ops::Neg for BigRat {
	type Output = Self;

	fn neg(self) -> Self {
		Self {
			sign: self.sign.flip(),
			..self
		}
	}
}

impl From<u64> for BigRat {
	fn from(i: u64) -> Self {
		Self {
			sign: Sign::Positive,
			num: i.into(),
			den: 1.into(),
		}
	}
}

impl From<BigUint> for BigRat {
	fn from(n: BigUint) -> Self {
		Self {
			sign: Sign::Positive,
			num: n,
			den: BigUint::from(1),
		}
	}
}

#[derive(Default)]
pub(crate) struct FormatOptions {
	pub(crate) base: Base,
	pub(crate) style: FormattingStyle,
	pub(crate) term: &'static str,
	pub(crate) use_parens_if_fraction: bool,
	pub(crate) decimal_separator: DecimalSeparatorStyle,
}

impl Format for BigRat {
	type Params = FormatOptions;
	type Out = FormattedBigRat;

	// Formats as an integer if possible, or a terminating float, otherwise as
	// either a fraction or a potentially approximated floating-point number.
	// The result 'exact' field indicates whether the number was exact or not.
	fn format<I: Interrupt>(&self, params: &Self::Params, int: &I) -> FResult<Exact<Self::Out>> {
		let base = params.base;
		let style = params.style;
		let term = params.term;
		let use_parens_if_fraction = params.use_parens_if_fraction;

		let mut x = self.clone().simplify(int)?;
		let sign = if x.sign == Sign::Positive || x == 0.into() {
			Sign::Positive
		} else {
			Sign::Negative
		};
		x.sign = Sign::Positive;

		// try as integer if possible
		if x.den == 1.into() {
			let sf_limit = if let FormattingStyle::SignificantFigures(sf) = style {
				Some(sf)
			} else {
				None
			};
			return Self::format_as_integer(
				&x.num,
				base,
				sign,
				term,
				use_parens_if_fraction,
				sf_limit,
				int,
			);
		}

		let mut terminating_res = None;
		let mut terminating = || match terminating_res {
			None => {
				let t = x.terminates_in_base(base, int)?;
				terminating_res = Some(t);
				Ok(t)
			}
			Some(t) => Ok(t),
		};
		let fraction = style == FormattingStyle::ImproperFraction
			|| style == FormattingStyle::MixedFraction
			|| (style == FormattingStyle::Exact && !terminating()?);
		if fraction {
			let mixed = style == FormattingStyle::MixedFraction || style == FormattingStyle::Exact;
			return x.format_as_fraction(base, sign, term, mixed, use_parens_if_fraction, int);
		}

		// not a fraction, will be printed as a decimal
		x.format_as_decimal(
			style,
			base,
			sign,
			term,
			terminating,
			params.decimal_separator,
			int,
		)
	}
}

#[derive(Debug)]
enum FormattedBigRatType {
	// optional int,
	// bool whether to add a space before the string
	// followed by a string (empty, "i" or "pi"),
	// followed by whether to wrap the number in parentheses
	Integer(Option<FormattedBigUint>, bool, &'static str, bool),
	// optional int (for mixed fractions)
	// optional int (numerator)
	// space
	// string (empty, "i", "pi", etc.)
	// '/'
	// int (denominator)
	// string (empty, "i", "pi", etc.) (used for mixed fractions, e.g. 1 2/3 i)
	// bool (whether or not to wrap the fraction in parentheses)
	Fraction(
		Option<FormattedBigUint>,
		Option<FormattedBigUint>,
		bool,
		&'static str,
		FormattedBigUint,
		&'static str,
		bool,
	),
	// string representation of decimal number (may or may not contain recurring digits)
	// space
	// string (empty, "i", "pi", etc.)
	Decimal(String, bool, &'static str),
}

#[must_use]
#[derive(Debug)]
pub(crate) struct FormattedBigRat {
	// whether or not to print a minus sign
	sign: Sign,
	ty: FormattedBigRatType,
}

impl fmt::Display for FormattedBigRat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		if self.sign == Sign::Negative {
			write!(f, "-")?;
		}
		match &self.ty {
			FormattedBigRatType::Integer(int, space, isuf, use_parens) => {
				if *use_parens {
					write!(f, "(")?;
				}
				if let Some(int) = int {
					write!(f, "{int}")?;
				}
				if *space {
					write!(f, " ")?;
				}
				write!(f, "{isuf}")?;
				if *use_parens {
					write!(f, ")")?;
				}
			}
			FormattedBigRatType::Fraction(integer, num, space, isuf, den, isuf2, use_parens) => {
				if *use_parens {
					write!(f, "(")?;
				}
				if let Some(integer) = integer {
					write!(f, "{integer} ")?;
				}
				if let Some(num) = num {
					write!(f, "{num}")?;
				}
				if *space && !isuf.is_empty() {
					write!(f, " ")?;
				}
				write!(f, "{isuf}/{den}")?;
				if *space && !isuf2.is_empty() {
					write!(f, " ")?;
				}
				write!(f, "{isuf2}")?;
				if *use_parens {
					write!(f, ")")?;
				}
			}
			FormattedBigRatType::Decimal(s, space, term) => {
				write!(f, "{s}")?;
				if *space {
					write!(f, " ")?;
				}
				write!(f, "{term}")?;
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::BigRat;
	use super::sign::Sign;

	use crate::num::biguint::BigUint;
	use crate::result::FResult;
	use std::mem;

	#[test]
	fn test_bigrat_from() {
		mem::drop(BigRat::from(2));
		mem::drop(BigRat::from(0));
		mem::drop(BigRat::from(u64::MAX));
		mem::drop(BigRat::from(u64::from(u32::MAX)));
	}

	#[test]
	fn test_addition() -> FResult<()> {
		let int = &crate::interrupt::Never;
		let two = BigRat::from(2);
		assert_eq!(two.clone().add(two, int)?, BigRat::from(4));
		Ok(())
	}

	#[test]
	fn test_cmp() {
		assert!(
			BigRat {
				sign: Sign::Positive,
				num: BigUint::from(16),
				den: BigUint::from(9)
			} < BigRat::from(2)
		);
	}

	#[test]
	fn test_cmp_2() {
		assert!(
			BigRat {
				sign: Sign::Positive,
				num: BigUint::from(36),
				den: BigUint::from(49)
			} < BigRat {
				sign: Sign::Positive,
				num: BigUint::from(3),
				den: BigUint::from(4)
			}
		);
	}
}
