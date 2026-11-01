use super::BigRat;
use super::sign::Sign;
use crate::error::{FendError, Interrupt};
use crate::interrupt::test_int;
use crate::num::biguint::BigUint;
use crate::result::FResult;

mod special;

pub(super) use special::{gamma, zeta};

pub(super) const WORK_BITS: u64 = super::ROOT_BITS + 32;

const MAX_EXPONENT: u64 = 1 << 24;

const CONST_BITS: u64 = 512;

const PI: [u64; 9] = [
	0x3f84_d5b5_b547_0917,
	0xc0ac_29b7_c97c_50dd,
	0xbe54_66cf_34e9_0c6c,
	0x4528_21e6_38d0_1377,
	0x082e_fa98_ec4e_6c89,
	0xa409_3822_299f_31d0,
	0x1319_8a2e_0370_7344,
	0x243f_6a88_85a3_08d3,
	0x0000_0000_0000_0003,
];

const MAX_TRIG_SIZE: u64 = 1 << 16;

const LN_2: [u64; 8] = [
	0x2757_3b29_1169_b825,
	0xed2e_ae35_c138_2144,
	0x5595_52fb_4afa_1b10,
	0xe7b8_7620_6deb_ac98,
	0x8a0d_175b_8baa_fa2b,
	0x40f3_4326_7298_b62d,
	0xc9e3_b398_03f2_f6af,
	0xb172_17f7_d1cf_79ab,
];

fn shl(mut x: BigUint, n: u64) -> BigUint {
	x.lshift_u64(n);
	x
}

fn shr(mut x: BigUint, n: u64) -> BigUint {
	x.rshift_u64(n);
	x
}

fn pow2(n: u64) -> BigUint {
	shl(1.into(), n)
}

#[derive(Clone)]
struct Fixed {
	neg: bool,
	mag: BigUint,
}

impl Fixed {
	fn new(neg: bool, mag: BigUint) -> Self {
		let neg = neg && !mag.is_zero();
		Self { neg, mag }
	}

	fn one(bits: u64) -> Self {
		Self::new(false, pow2(bits))
	}

	fn from_rat<I: Interrupt>(x: &BigRat, bits: u64, int: &I) -> FResult<Self> {
		let mag = shl(x.num.clone(), bits).div(&x.den, int)?;
		Ok(Self::new(x.sign == Sign::Negative, mag))
	}

	fn into_rat(self, bits: u64) -> BigRat {
		BigRat {
			sign: if self.neg {
				Sign::Negative
			} else {
				Sign::Positive
			},
			num: self.mag,
			den: pow2(bits),
		}
	}

	fn add(self, other: &Self) -> Self {
		if self.neg == other.neg {
			Self::new(self.neg, self.mag.add(&other.mag))
		} else if self.mag >= other.mag {
			Self::new(self.neg, self.mag.sub(&other.mag))
		} else {
			Self::new(other.neg, other.mag.clone().sub(&self.mag))
		}
	}

	fn sub(self, other: &Self) -> Self {
		self.add(&Self::new(!other.neg, other.mag.clone()))
	}

	fn mul<I: Interrupt>(&self, other: &Self, bits: u64, int: &I) -> FResult<Self> {
		let mag = shr(self.mag.clone().mul(&other.mag, int)?, bits);
		Ok(Self::new(self.neg != other.neg, mag))
	}

	fn div<I: Interrupt>(&self, other: &Self, bits: u64, int: &I) -> FResult<Self> {
		let mag = shl(self.mag.clone(), bits).div(&other.mag, int)?;
		Ok(Self::new(self.neg != other.neg, mag))
	}

	fn mul_int<I: Interrupt>(&self, n: u64, int: &I) -> FResult<Self> {
		Ok(Self::new(self.neg, self.mag.clone().mul(&n.into(), int)?))
	}

	fn div_int<I: Interrupt>(&self, n: u64, int: &I) -> FResult<Self> {
		Ok(Self::new(self.neg, self.mag.clone().div(&n.into(), int)?))
	}

	fn times_pow2(self, n: u64) -> Self {
		Self::new(self.neg, shl(self.mag, n))
	}

	fn over_pow2(self, n: u64) -> Self {
		Self::new(self.neg, shr(self.mag, n))
	}

	fn negate_if(self, neg: bool) -> Self {
		Self::new(self.neg != neg, self.mag)
	}

	fn is_zero(&self) -> bool {
		self.mag.is_zero()
	}
}

fn ln_2<I: Interrupt>(bits: u64, int: &I) -> FResult<Fixed> {
	if bits <= CONST_BITS {
		let constant = BigUint::Large(LN_2.to_vec());
		return Ok(Fixed::new(false, shr(constant, CONST_BITS - bits)));
	}
	let third = BigRat::from(1).div(&3.into(), int)?;
	let half = atanh_small(&third, bits + 8, int)?;
	Fixed::from_rat(&half.mul(&2.into(), int)?, bits, int)
}

fn atan_inv<I: Interrupt>(n: u64, bits: u64, int: &I) -> FResult<Fixed> {
	let mut power = Fixed::one(bits).div_int(n, int)?;
	let mut sum = power.clone();
	for k in 1..=u64::MAX {
		test_int(int)?;
		power = power.div_int(n * n, int)?;
		let term = power.div_int(2 * k + 1, int)?;
		if term.is_zero() {
			break;
		}
		sum = if k % 2 == 1 {
			sum.sub(&term)
		} else {
			sum.add(&term)
		};
	}
	Ok(sum)
}

fn pi<I: Interrupt>(bits: u64, int: &I) -> FResult<Fixed> {
	if bits <= CONST_BITS {
		let constant = BigUint::Large(PI.to_vec());
		return Ok(Fixed::new(false, shr(constant, CONST_BITS - bits)));
	}
	let work = bits + 16;
	let a = atan_inv(5, work, int)?.mul_int(16, int)?;
	let b = atan_inv(239, work, int)?.mul_int(4, int)?;
	Ok(a.sub(&b).over_pow2(16))
}

pub(super) fn pi_rat<I: Interrupt>(int: &I) -> FResult<BigRat> {
	Ok(pi(WORK_BITS, int)?.into_rat(WORK_BITS))
}

fn scale_pow2(mut x: BigRat, k: i64) -> BigRat {
	if k >= 0 {
		x.num = shl(x.num, k.unsigned_abs());
	} else {
		x.den = shl(x.den, k.unsigned_abs());
	}
	x
}

fn times<I: Interrupt>(x: &BigRat, f: &Fixed, bits: u64, int: &I) -> FResult<BigRat> {
	Ok(BigRat {
		sign: if (x.sign == Sign::Negative) == f.neg {
			Sign::Positive
		} else {
			Sign::Negative
		},
		num: x.num.clone().mul(&f.mag, int)?,
		den: shl(x.den.clone(), bits),
	})
}

fn log2_estimate(x: &BigRat) -> f64 {
	x.num.log2() - x.den.log2()
}

fn f64_estimate(x: &BigRat) -> f64 {
	if x.num.is_zero() {
		0.0
	} else {
		log2_estimate(x).exp2()
	}
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn exponent(value: f64) -> FResult<i64> {
	let rounded = value.round();
	if !rounded.is_finite() || rounded.abs() > MAX_EXPONENT as f64 {
		return Err(FendError::ValueTooLarge);
	}
	Ok(rounded as i64)
}

fn bit_width(n: u64) -> u64 {
	u64::from(n.bit_width())
}

fn is_negative(x: &BigRat) -> bool {
	x.sign == Sign::Negative && !x.num.is_zero()
}

fn abs(x: &BigRat) -> BigRat {
	BigRat {
		sign: Sign::Positive,
		num: x.num.clone(),
		den: x.den.clone(),
	}
}

fn is_small<I: Interrupt>(x: &BigRat, int: &I) -> FResult<bool> {
	Ok(x.num.clone().mul(&2.into(), int)? < x.den)
}

fn exp_fixed<I: Interrupt>(r: &Fixed, bits: u64, int: &I) -> FResult<Fixed> {
	const HALVINGS: u64 = 8;
	let work = bits + HALVINGS + 8;
	let r = r.clone().times_pow2(work - bits - HALVINGS);
	let mut sum = Fixed::one(work);
	let mut term = Fixed::one(work);
	for n in 1..=u64::MAX {
		test_int(int)?;
		term = term.mul(&r, work, int)?.div_int(n, int)?;
		if term.is_zero() {
			break;
		}
		sum = sum.add(&term);
	}
	for _ in 0..HALVINGS {
		sum = sum.mul(&sum, work, int)?;
	}
	Ok(sum.over_pow2(work - bits))
}

fn exp_bits<I: Interrupt>(x: &BigRat, bits: u64, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(1.into());
	}
	let mut log2 = log2_estimate(x).exp2() * std::f64::consts::LOG2_E;
	if is_negative(x) {
		log2 = -log2;
	}
	let k = exponent(log2)?;
	let extra = bit_width(k.unsigned_abs()) + 2;
	let r = Fixed::from_rat(x, bits + extra, int)?
		.sub(
			&ln_2(bits + extra, int)?
				.mul_int(k.unsigned_abs(), int)?
				.negate_if(k < 0),
		)
		.over_pow2(extra);
	let m = exp_fixed(&r, bits, int)?;
	Ok(scale_pow2(m.into_rat(bits), k))
}

pub(super) fn exp<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	exp_bits(x, WORK_BITS, int)
}

fn atanh_small<I: Interrupt>(z: &BigRat, bits: u64, int: &I) -> FResult<BigRat> {
	let z2 = Fixed::from_rat(&z.clone().mul(z, int)?, bits, int)?;
	let mut sum = Fixed::one(bits);
	let mut power = Fixed::one(bits);
	for k in 1..=u64::MAX {
		test_int(int)?;
		power = power.mul(&z2, bits, int)?;
		let term = power.div_int(2 * k + 1, int)?;
		if term.is_zero() {
			break;
		}
		sum = sum.add(&term);
	}
	times(z, &sum, bits, int)
}

fn ln_bits<I: Interrupt>(x: &BigRat, bits: u64, int: &I) -> FResult<BigRat> {
	let e = exponent(log2_estimate(x))?;
	let m = scale_pow2(abs(x), -e);
	let (sign, num) = if m.num >= m.den {
		(Sign::Positive, m.num.clone().sub(&m.den))
	} else {
		(Sign::Negative, m.den.clone().sub(&m.num))
	};
	let z = BigRat {
		sign,
		num,
		den: m.num.add(&m.den),
	};
	let ln_m = atanh_small(&z, bits, int)?.mul(&2.into(), int)?;
	if e == 0 {
		return Ok(ln_m);
	}
	let extra = bit_width(e.unsigned_abs()) + 2;
	let sum = ln_2(bits + extra, int)?
		.mul_int(e.unsigned_abs(), int)?
		.negate_if(e < 0)
		.add(&Fixed::from_rat(&ln_m, bits + extra, int)?);
	Ok(sum.into_rat(bits + extra))
}

pub(super) fn ln<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	ln_bits(x, WORK_BITS, int)
}

fn pow_bits<I: Interrupt>(base: &BigRat, exponent: &BigRat, bits: u64, int: &I) -> FResult<BigRat> {
	let size = log2_estimate(exponent) + log2_estimate(base).abs().max(1.0).log2();
	let extra = self::exponent(size.max(0.0))?.unsigned_abs() + 2;
	let t = exponent
		.clone()
		.mul(&ln_bits(base, bits + extra, int)?, int)?;
	exp_bits(&t, bits, int)
}

pub(super) fn pow<I: Interrupt>(base: &BigRat, exponent: &BigRat, int: &I) -> FResult<BigRat> {
	pow_bits(base, exponent, WORK_BITS, int)
}

pub(super) fn sqrt<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	let x = x.clone().simplify(int)?;
	let num = x.num.clone().root_n(&2.into(), int)?;
	let den = x.den.clone().root_n(&2.into(), int)?;
	if num.exact && den.exact {
		return Ok(BigRat {
			sign: Sign::Positive,
			num: num.value,
			den: den.value,
		});
	}
	let size = i64::try_from(x.num.bits()).unwrap() - i64::try_from(x.den.bits()).unwrap();
	let s = i64::try_from(WORK_BITS).unwrap() + 1 - size.div_euclid(2);
	let scaled = if s >= 0 {
		shl(x.num, 2 * s.unsigned_abs()).div(&x.den, int)?
	} else {
		x.num.div(&shl(x.den, 2 * s.unsigned_abs()), int)?
	};
	let root = scaled.root_n(&2.into(), int)?.value;
	Ok(scale_pow2(root.into(), -s))
}

fn sinh_small<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	let bits = WORK_BITS;
	let x2 = Fixed::from_rat(&x.clone().mul(x, int)?, bits, int)?;
	let mut sum = Fixed::one(bits);
	let mut term = Fixed::one(bits);
	for k in 1..=u64::MAX {
		test_int(int)?;
		term = term
			.mul(&x2, bits, int)?
			.div_int((2 * k) * (2 * k + 1), int)?;
		if term.is_zero() {
			break;
		}
		sum = sum.add(&term);
	}
	times(x, &sum, bits, int)
}

fn exp_dominates(x: &BigRat) -> bool {
	#[allow(clippy::cast_precision_loss)]
	let bits = (WORK_BITS + 8) as f64;
	f64_estimate(x) > bits
}

pub(super) fn sinh<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(0.into());
	}
	if is_small(x, int)? {
		return sinh_small(x, int);
	}
	if exp_dominates(x) {
		let half = scale_pow2(exp(&abs(x), int)?, -1);
		return Ok(if is_negative(x) { -half } else { half });
	}
	let e = exp(x, int)?;
	let inverse = BigRat::from(1).div(&e, int)?;
	e.add(-inverse, int)?.div(&2.into(), int)
}

pub(super) fn cosh<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(1.into());
	}
	if exp_dominates(x) {
		return Ok(scale_pow2(exp(&abs(x), int)?, -1));
	}
	let e = exp(x, int)?;
	let inverse = BigRat::from(1).div(&e, int)?;
	e.add(inverse, int)?.div(&2.into(), int)
}

pub(super) fn tanh<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(0.into());
	}
	if is_small(x, int)? {
		return sinh_small(x, int)?.div(&cosh(x, int)?, int);
	}
	let t = if exp_dominates(x) {
		BigRat::from(1)
	} else {
		let e = exp(&abs(x).mul(&2.into(), int)?, int)?;
		e.clone()
			.add(-BigRat::from(1), int)?
			.div(&e.add(1.into(), int)?, int)?
	};
	Ok(if is_negative(x) { -t } else { t })
}

pub(super) fn asinh<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(0.into());
	}
	let root = sqrt(&x.clone().mul(x, int)?.add(1.into(), int)?, int)?;
	if is_small(x, int)? {
		return atanh_small(&x.clone().div(&root, int)?, WORK_BITS, int);
	}
	let t = ln(&abs(x).add(root, int)?, int)?;
	Ok(if is_negative(x) { -t } else { t })
}

pub(super) fn acosh<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	let t = x.clone().add(-BigRat::from(1), int)?.div(&2.into(), int)?;
	if t.num.is_zero() {
		return Ok(0.into());
	}
	asinh(&sqrt(&t, int)?, int)?.mul(&2.into(), int)
}

pub(super) fn atanh<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(0.into());
	}
	if is_small(x, int)? {
		return atanh_small(x, WORK_BITS, int);
	}
	let one = BigRat::from(1);
	let ratio = one
		.clone()
		.add(x.clone(), int)?
		.div(&one.add(-x.clone(), int)?, int)?;
	ln(&ratio, int)?.div(&2.into(), int)
}

fn sqrt_fixed<I: Interrupt>(v: &Fixed, bits: u64, int: &I) -> FResult<Fixed> {
	let root = shl(v.mag.clone(), bits).root_n(&2.into(), int)?.value;
	Ok(Fixed::new(false, root))
}

fn reduce_half_pi<I: Interrupt>(x: &BigRat, int: &I) -> FResult<(BigRat, u64)> {
	let size = log2_estimate(x);
	if size < -1.0 {
		return Ok((x.clone(), 0));
	}
	let size = exponent(size)?.unsigned_abs();
	if size > MAX_TRIG_SIZE {
		return Err(FendError::ValueTooLarge);
	}
	let mut extra = size + 32;
	loop {
		let bits = WORK_BITS + extra;
		let half_pi = pi(bits, int)?.over_pow2(1);
		let xf = Fixed::from_rat(x, bits, int)?;
		let k = xf
			.mag
			.clone()
			.mul(&2.into(), int)?
			.add(&half_pi.mag)
			.div(&half_pi.mag.clone().mul(&2.into(), int)?, int)?;
		let multiple = Fixed::new(xf.neg, half_pi.mag.mul(&k, int)?);
		let r = xf.clone().sub(&multiple);
		let lost = (WORK_BITS + 16).saturating_sub(r.mag.bits());
		if lost == 0 || extra > size + MAX_TRIG_SIZE {
			let q = k.divmod(&4.into(), int)?.1.try_as_usize(int)? as u64;
			let q = if xf.neg { (4 - q) % 4 } else { q };
			return Ok((r.into_rat(bits), q));
		}
		extra += lost;
	}
}

fn sin_cos_series<I: Interrupt>(r2: &Fixed, bits: u64, int: &I) -> FResult<(Fixed, Fixed)> {
	let mut sinc = Fixed::one(bits);
	let mut cos = Fixed::one(bits);
	let mut term = Fixed::one(bits);
	for k in 1..=u64::MAX {
		test_int(int)?;
		term = term
			.mul(r2, bits, int)?
			.div_int((2 * k - 1) * (2 * k), int)?;
		if term.is_zero() {
			break;
		}
		let sin_term = term.div_int(2 * k + 1, int)?;
		if k % 2 == 1 {
			cos = cos.sub(&term);
			sinc = sinc.sub(&sin_term);
		} else {
			cos = cos.add(&term);
			sinc = sinc.add(&sin_term);
		}
	}
	Ok((sinc, cos))
}

fn sin_cos<I: Interrupt>(x: &BigRat, int: &I) -> FResult<(BigRat, BigRat)> {
	let (r, q) = reduce_half_pi(x, int)?;
	let r2 = Fixed::from_rat(&r.clone().mul(&r, int)?, WORK_BITS, int)?;
	let (sinc, cos) = sin_cos_series(&r2, WORK_BITS, int)?;
	let sin = times(&r, &sinc, WORK_BITS, int)?;
	let cos = cos.into_rat(WORK_BITS);
	Ok(match q {
		0 => (sin, cos),
		1 => (cos, -sin),
		2 => (-sin, -cos),
		_ => (-cos, sin),
	})
}

pub(super) fn sin<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	Ok(sin_cos(x, int)?.0)
}

pub(super) fn cos<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	Ok(sin_cos(x, int)?.1)
}

fn atan_series<I: Interrupt>(t: &BigRat, int: &I) -> FResult<BigRat> {
	let bits = WORK_BITS;
	let t2 = Fixed::from_rat(&t.clone().mul(t, int)?, bits, int)?;
	let mut sum = Fixed::one(bits);
	let mut power = Fixed::one(bits);
	for k in 1..=u64::MAX {
		test_int(int)?;
		power = power.mul(&t2, bits, int)?;
		let term = power.div_int(2 * k + 1, int)?;
		if term.is_zero() {
			break;
		}
		sum = if k % 2 == 1 {
			sum.sub(&term)
		} else {
			sum.add(&term)
		};
	}
	times(t, &sum, bits, int)
}

fn atan_unit<I: Interrupt>(a: &BigRat, int: &I) -> FResult<BigRat> {
	if a.num.clone().mul(&8.into(), int)? <= a.den {
		return atan_series(a, int);
	}
	let bits = WORK_BITS + 8;
	let one = Fixed::one(bits);
	let eighth = one.clone().over_pow2(3);
	let mut t = Fixed::from_rat(a, bits, int)?;
	let mut halvings = 0;
	while t.mag > eighth.mag {
		test_int(int)?;
		let root = sqrt_fixed(&one.clone().add(&t.mul(&t, bits, int)?), bits, int)?;
		t = t.div(&one.clone().add(&root), bits, int)?;
		halvings += 1;
	}
	Ok(scale_pow2(atan_series(&t.into_rat(bits), int)?, halvings))
}

pub(super) fn atan<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(0.into());
	}
	let t = if x.num > x.den {
		let inverse = BigRat {
			sign: Sign::Positive,
			num: x.den.clone(),
			den: x.num.clone(),
		};
		pi_rat(int)?
			.div(&2.into(), int)?
			.add(-atan_unit(&inverse, int)?, int)?
	} else {
		atan_unit(&abs(x), int)?
	};
	Ok(if is_negative(x) { -t } else { t })
}

pub(super) fn asin<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	if x.num.is_zero() {
		return Ok(0.into());
	}
	let one = BigRat::from(1);
	let rest = one.add(-x.clone().mul(x, int)?, int)?;
	if rest.num.is_zero() {
		let half_pi = pi_rat(int)?.div(&2.into(), int)?;
		return Ok(if is_negative(x) { -half_pi } else { half_pi });
	}
	atan(&x.clone().div(&sqrt(&rest, int)?, int)?, int)
}

pub(super) fn acos<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	let one = BigRat::from(1);
	let above = one.clone().add(x.clone(), int)?;
	if above.num.is_zero() {
		return pi_rat(int);
	}
	let ratio = one.add(-x.clone(), int)?.div(&above, int)?;
	atan(&sqrt(&ratio, int)?, int)?.mul(&2.into(), int)
}

pub(super) fn atan2<I: Interrupt>(y: &BigRat, x: &BigRat, int: &I) -> FResult<BigRat> {
	let base = atan(&y.clone().div(x, int)?, int)?;
	if !is_negative(x) {
		return Ok(base);
	}
	let pi = pi_rat(int)?;
	if is_negative(y) {
		base.add(-pi, int)
	} else {
		base.add(pi, int)
	}
}

pub(super) fn ln_2_rat<I: Interrupt>(int: &I) -> FResult<BigRat> {
	Ok(ln_2(WORK_BITS, int)?.into_rat(WORK_BITS))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn pi_constant_matches_machin() {
		let int = &crate::interrupt::Never;
		let series = pi(CONST_BITS + 64, int).unwrap().over_pow2(64);
		let constant = pi(CONST_BITS, int).unwrap();
		let diff = if series.mag >= constant.mag {
			series.mag.sub(&constant.mag)
		} else {
			constant.mag.sub(&series.mag)
		};
		assert!(diff <= 2.into());
	}

	#[test]
	fn ln_2_constant_matches_its_series() {
		let int = &crate::interrupt::Never;
		let bits = CONST_BITS + 64;
		let series = ln_2(bits, int).unwrap().over_pow2(64);
		let constant = ln_2(CONST_BITS, int).unwrap();
		let diff = if series.mag >= constant.mag {
			series.mag.sub(&constant.mag)
		} else {
			constant.mag.sub(&series.mag)
		};
		assert!(diff <= 2.into());
	}
}
