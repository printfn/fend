use super::{
	Fixed, WORK_BITS, exp_bits, exponent, is_negative, ln_bits, log2_estimate, pi, pow_bits, shl,
	sin,
};
use crate::error::Interrupt;
use crate::interrupt::test_int;
use crate::num::Exact;
use crate::num::bigrat::BigRat;
use crate::num::biguint::BigUint;
use crate::result::FResult;

const BITS: u64 = WORK_BITS + 16;

const BERNOULLI: [(i128, u64); 29] = [
	(1, 6),
	(-1, 30),
	(1, 42),
	(-1, 30),
	(5, 66),
	(-691, 2730),
	(7, 6),
	(-3617, 510),
	(43_867, 798),
	(-174_611, 330),
	(854_513, 138),
	(-236_364_091, 2730),
	(8_553_103, 6),
	(-23_749_461_029, 870),
	(8_615_841_276_005, 14322),
	(-7_709_321_041_217, 510),
	(2_577_687_858_367, 6),
	(-26_315_271_553_053_477_373, 1_919_190),
	(2_929_993_913_841_559, 6),
	(-261_082_718_496_449_122_051, 13530),
	(1_520_097_643_918_070_802_691, 1806),
	(-27_833_269_579_301_024_235_023, 690),
	(596_451_111_593_912_163_277_961, 282),
	(-5_609_403_368_997_817_686_249_127_547, 46410),
	(495_057_205_241_079_648_212_477_525, 66),
	(-801_165_718_135_489_957_347_924_991_853, 1590),
	(29_149_963_634_884_862_421_418_123_812_691, 798),
	(-2_479_392_929_313_226_753_685_415_739_663_229, 870),
	(84_483_613_348_880_041_862_046_775_994_036_021, 354),
];

const STIRLING_MIN: u64 = 60;

fn biguint(n: u128) -> BigUint {
	let high = u64::try_from(n >> 64).unwrap();
	let low = u64::try_from(n & u128::from(u64::MAX)).unwrap();
	shl(high.into(), 64).add(&low.into())
}

fn bernoulli(k: usize) -> BigRat {
	let (num, den) = BERNOULLI[k - 1];
	let magnitude = BigRat::from(biguint(num.unsigned_abs()));
	let value = BigRat {
		den: den.into(),
		..magnitude
	};
	if num < 0 { -value } else { value }
}

fn index(n: u64) -> usize {
	usize::try_from(n).unwrap()
}

fn pi_bits<I: Interrupt>(bits: u64, int: &I) -> FResult<BigRat> {
	Ok(pi(bits, int)?.into_rat(bits))
}

fn sin_pi<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	let n = x.clone().round(int)?;
	let f = x.clone().add(-n.clone(), int)?;
	let s = sin(&pi_bits(BITS, int)?.mul(&f, int)?, int)?;
	Ok(if n.num.is_even(int)? { s } else { -s })
}

fn ln_gamma<I: Interrupt>(z: &BigRat, int: &I) -> FResult<BigRat> {
	let size = exponent(log2_estimate(z))?.unsigned_abs() + 2;
	let ln_z = ln_bits(z, BITS + size, int)?;
	let half = BigRat::from(1).div(&2.into(), int)?;
	let main = z
		.clone()
		.add(-half.clone(), int)?
		.mul(&ln_z, int)?
		.add(-z.clone(), int)?;
	let two_pi = pi_bits(BITS + 8, int)?.mul(&2.into(), int)?;
	let constant = ln_bits(&two_pi, BITS, int)?.mul(&half, int)?;
	let inverse = Fixed::from_rat(&BigRat::from(1).div(z, int)?, BITS, int)?;
	let inverse2 = inverse.mul(&inverse, BITS, int)?;
	let mut sum = Fixed::new(false, 0.into());
	for (i, &(num, den)) in BERNOULLI.iter().enumerate().rev() {
		test_int(int)?;
		let k = u64::try_from(i).unwrap() + 1;
		let coefficient = shl(biguint(num.unsigned_abs()), BITS)
			.div(&(den * (2 * k) * (2 * k - 1)).into(), int)?;
		sum = Fixed::new(num < 0, coefficient).add(&sum.mul(&inverse2, BITS, int)?);
	}
	let sum = sum.mul(&inverse, BITS, int)?;
	main.add(constant, int)?.add(sum.into_rat(BITS), int)
}

pub(in crate::num::bigrat) fn gamma<I: Interrupt>(x: &BigRat, int: &I) -> FResult<BigRat> {
	let one = BigRat::from(1);
	if is_negative(x) {
		let rest = gamma(&one.add(-x.clone(), int)?, int)?;
		return pi_bits(BITS, int)?.div(&sin_pi(x, int)?.mul(&rest, int)?, int);
	}
	if x < &one {
		return gamma(&x.clone().add(one, int)?, int)?.div(x, int);
	}
	let mut z = x.clone();
	let mut product = Fixed::one(BITS);
	while z < BigRat::from(STIRLING_MIN) {
		test_int(int)?;
		product = product.mul(&Fixed::from_rat(&z, BITS, int)?, BITS, int)?;
		z = z.add(one.clone(), int)?;
	}
	exp_bits(&ln_gamma(&z, int)?, WORK_BITS, int)?.div(&product.into_rat(BITS), int)
}

fn inverse_powers<I: Interrupt>(s: &BigRat, n: u64, int: &I) -> FResult<Vec<Fixed>> {
	let mut powers = vec![Fixed::one(BITS); index(n) + 1];
	for m in 2..=n {
		test_int(int)?;
		let p = (2..=m).find(|p| m % p == 0).unwrap();
		powers[index(m)] = if p == m {
			let power = if s.is_integer() {
				let exact = BigRat::from(m).pow(s.clone(), int)?.value;
				BigRat::from(1).div(&exact, int)?
			} else {
				pow_bits(&m.into(), &-s.clone(), BITS, int)?
			};
			Fixed::from_rat(&power, BITS, int)?
		} else {
			powers[index(p)].mul(&powers[index(m / p)], BITS, int)?
		};
	}
	Ok(powers)
}

fn zeta_positive<I: Interrupt>(s: &BigRat, int: &I) -> FResult<BigRat> {
	let one = BigRat::from(1);
	if s > &BigRat::from(BITS) {
		return Ok(one);
	}
	let n = (2 * BITS + 10) / 5;
	let mut d = Vec::with_capacity(index(n) + 1);
	let mut term = BigUint::from(1);
	let mut sum = term.clone();
	d.push(sum.clone());
	for i in 0..n {
		test_int(int)?;
		term = term
			.mul(&(2 * (n + i) * (n - i)).into(), int)?
			.div(&((2 * i + 1) * (i + 1)).into(), int)?;
		sum = sum.add(&term);
		d.push(sum.clone());
	}
	let powers = inverse_powers(s, n, int)?;
	let d_n = &d[index(n)];
	let mut eta = Fixed::new(false, 0.into());
	for k in 0..n {
		test_int(int)?;
		let weight = d_n.clone().sub(&d[index(k)]);
		let term = Fixed::new(false, powers[index(k + 1)].mag.clone().mul(&weight, int)?);
		eta = if k % 2 == 0 {
			eta.add(&term)
		} else {
			eta.sub(&term)
		};
	}
	let eta = Fixed::new(eta.neg, eta.mag.div(d_n, int)?).into_rat(BITS);
	let half_t = one
		.add(-s.clone(), int)?
		.mul(&super::ln_2(BITS + 8, int)?.into_rat(BITS + 8), int)?
		.div(&2.into(), int)?;
	let factor = super::sinh(&half_t, int)?
		.mul(&exp_bits(&half_t, BITS, int)?, int)?
		.mul(&2.into(), int)?;
	eta.div(&-factor, int)
}

pub(in crate::num::bigrat) fn zeta<I: Interrupt>(s: &BigRat, int: &I) -> FResult<Exact<BigRat>> {
	if !is_negative(s) && !s.num.is_zero() {
		return Ok(Exact::new(zeta_positive(s, int)?, false));
	}
	if s.is_integer() {
		if s.num.is_zero() {
			return Ok(Exact::new(
				BigRat::from(1).div(&(-BigRat::from(2)), int)?,
				true,
			));
		}
		if s.num.is_even(int)? {
			return Ok(Exact::new(0.into(), true));
		}
		let k = s.num.clone().add(&1.into()).div(&2.into(), int)?;
		if k <= BigUint::from(u64::try_from(BERNOULLI.len()).unwrap()) {
			let k = k.try_as_usize(int)?;
			let value = bernoulli(k).div(&-BigRat::from(u64::try_from(2 * k).unwrap()), int)?;
			return Ok(Exact::new(value, true));
		}
	}
	let one_minus = BigRat::from(1).add(-s.clone(), int)?;
	let two_pi = pi_bits(BITS + 64, int)?.mul(&2.into(), int)?;
	let power = pow_bits(&two_pi, &-one_minus.clone(), BITS, int)?;
	let sin = sin_pi(&s.clone().div(&2.into(), int)?, int)?;
	let rest = gamma(&one_minus, int)?;
	let value = power
		.mul(&2.into(), int)?
		.mul(&sin, int)?
		.mul(&rest, int)?
		.mul(&zeta_positive(&one_minus, int)?, int)?;
	Ok(Exact::new(value, false))
}
