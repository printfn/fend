use crate::err::{
    err, DivideByZero, ExponentTooLarge, IntegerPowerError, ValueTooLarge,
    ZeroToThePowerOfZero,
};
use crate::interrupt::{test_int, Interrupt};
use crate::num::Base;
use std::cmp::{max, Ordering};
use std::fmt::{Debug, Error, Formatter};
use std::{
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Sub},
};

#[derive(Clone)]
pub enum BigUint {
    Small(u64),
    // little-endian, len >= 1
    Large(Vec<u64>),
}

use BigUint::{Large, Small};

#[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
const fn truncate(n: u128) -> u64 {
    n as u64
}

impl BigUint {
    fn is_zero(&self) -> bool {
        match self {
            Small(n) => *n == 0,
            Large(value) => {
                for v in value.iter().copied() {
                    if v != 0 {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn get(&self, idx: usize) -> u64 {
        match self {
            Small(n) => {
                if idx == 0 {
                    *n
                } else {
                    0
                }
            }
            Large(value) => {
                if idx < value.len() {
                    value[idx]
                } else {
                    0
                }
            }
        }
    }

    pub fn try_as_usize(&self) -> Result<usize, ValueTooLarge> {
        use std::convert::TryFrom;

        Ok(match self {
            Small(n) => {
                if let Ok(res) = usize::try_from(*n) {
                    res
                } else {
                    return err();
                }
            }
            Large(v) => {
                if v.len() == 1 {
                    if let Ok(res) = usize::try_from(v[0]) {
                        res
                    } else {
                        return err();
                    }
                } else {
                    return err();
                }
            }
        })
    }

    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        clippy::float_arithmetic
    )]
    pub fn as_f64(&self) -> f64 {
        match self {
            Small(n) => *n as f64,
            Large(v) => {
                let mut res = 0.0;
                for &n in v.iter().rev() {
                    res *= u64::MAX as f64;
                    res += n as f64;
                }
                res
            }
        }
    }

    fn make_large(&mut self) {
        match self {
            Small(n) => {
                *self = Large(vec![*n]);
            }
            Large(_) => (),
        }
    }

    fn set(&mut self, idx: usize, new_value: u64) {
        match self {
            Small(n) => {
                if idx == 0 {
                    *n = new_value;
                } else if new_value == 0 {
                    // no need to do anything
                } else {
                    self.make_large();
                    self.set(idx, new_value)
                }
            }
            Large(value) => {
                while idx >= value.len() {
                    value.push(0);
                }
                value[idx] = new_value;
            }
        }
    }

    fn value_len(&self) -> usize {
        match self {
            Small(_) => 1,
            Large(value) => value.len(),
        }
    }

    fn value_push(&mut self, new: u64) {
        if new == 0 {
            return;
        }
        self.make_large();
        match self {
            Small(_) => unreachable!(),
            Large(v) => v.push(new),
        }
    }

    pub fn gcd(
        mut a: Self,
        mut b: Self,
        int: &impl Interrupt,
    ) -> Result<Self, crate::err::Interrupt> {
        while b >= 1.into() {
            let r = a.rem(&b, int)?;
            a = b;
            b = r;
        }

        Ok(a)
    }

    pub fn pow<I: Interrupt>(
        a: &Self,
        b: &Self,
        int: &I,
    ) -> Result<Result<Self, IntegerPowerError>, crate::err::Interrupt> {
        if a.is_zero() && b.is_zero() {
            return ZeroToThePowerOfZero::err();
        }
        if b.is_zero() {
            return Ok(Ok(Self::from(1)));
        }
        if b.value_len() > 1 {
            return ExponentTooLarge::err();
        }
        Ok(Ok(a.pow_internal(b.get(0), int)?))
    }

    // computes the exact square root if possible, otherwise the next lower integer
    pub fn root_n(self, n: &Self, int: &impl Interrupt) -> Result<(Self, bool), String> {
        if self == 0.into() || self == 1.into() || n == &Self::from(1) {
            return Ok((self, true));
        }
        let mut low_guess = Self::from(1);
        let mut high_guess = self.clone();
        while high_guess.clone() - low_guess.clone() > 1.into() {
            test_int(int)?;
            let mut guess = low_guess.clone() + high_guess.clone();
            guess.rshift();

            let res = Self::pow(&guess, n, int)??;
            match res.cmp(&self) {
                Ordering::Equal => return Ok((guess, true)),
                Ordering::Greater => high_guess = guess,
                Ordering::Less => low_guess = guess,
            }
        }
        Ok((low_guess, false))
    }

    fn pow_internal<I: Interrupt>(
        &self,
        mut exponent: u64,
        int: &I,
    ) -> Result<Self, crate::err::Interrupt> {
        let mut result = Self::from(1);
        let mut base = self.clone();
        while exponent > 0 {
            test_int(int)?;
            if exponent % 2 == 1 {
                result = result.mul(&base, int)?;
            }
            exponent >>= 1;
            base = base.clone().mul(&base, int)?;
        }
        Ok(result)
    }

    fn lshift(&mut self) {
        match self {
            Small(n) => {
                if *n & 0xc000_0000_0000_0000 == 0 {
                    *n <<= 1;
                } else {
                    self.make_large();
                    self.lshift();
                }
            }
            Large(value) => {
                if value[value.len() - 1] & (1_u64 << 63) != 0 {
                    value.push(0);
                }
                for i in (0..value.len()).rev() {
                    value[i] <<= 1;
                    if i != 0 {
                        value[i] |= value[i - 1] >> 63;
                    }
                }
            }
        }
    }

    fn rshift(&mut self) {
        match self {
            Small(n) => *n >>= 1,
            Large(value) => {
                for i in 0..value.len() {
                    value[i] >>= 1;
                    let next = if i + 1 >= value.len() {
                        0
                    } else {
                        value[i + 1]
                    };
                    value[i] |= next << 63;
                }
            }
        }
    }

    fn divmod(
        &self,
        other: &Self,
        int: &impl Interrupt,
    ) -> Result<Result<(Self, Self), DivideByZero>, crate::err::Interrupt> {
        if let (Small(a), Small(b)) = (self, other) {
            if let (Some(div_res), Some(mod_res)) = (a.checked_div(*b), a.checked_rem(*b)) {
                return Ok(Ok((Small(div_res), Small(mod_res))));
            }
            return err();
        }
        if other.is_zero() {
            return err();
        }
        if other == &Self::from(1) {
            return Ok(Ok((self.clone(), Self::from(0))));
        }
        if self.is_zero() {
            return Ok(Ok((Self::from(0), Self::from(0))));
        }
        if self < other {
            return Ok(Ok((Self::from(0), self.clone())));
        }
        if self == other {
            return Ok(Ok((Self::from(1), Self::from(0))));
        }
        if other == &Self::from(2) {
            let mut div_result = self.clone();
            div_result.rshift();
            let modulo = self.get(0) & 1;
            return Ok(Ok((div_result, Self::from(modulo))));
        }
        // binary long division
        let mut q = Self::from(0);
        let mut r = Self::from(0);
        for i in (0..self.value_len()).rev() {
            test_int(int)?;
            for j in (0..64).rev() {
                r.lshift();
                let bit_of_self = if (self.get(i) & (1 << j)) == 0 { 0 } else { 1 };
                r.set(0, r.get(0) | bit_of_self);
                if &r >= other {
                    r = r - other.clone();
                    q.set(i, q.get(i) | (1 << j));
                }
            }
        }
        Ok(Ok((q, r)))
    }

    /// computes self *= other
    fn mul_internal(
        &mut self,
        other: &Self,
        int: &impl Interrupt,
    ) -> Result<(), crate::err::Interrupt> {
        if self.is_zero() || other.is_zero() {
            *self = Self::from(0);
            return Ok(());
        }
        let self_clone = self.clone();
        self.make_large();
        match self {
            Small(_) => unreachable!(),
            Large(v) => {
                v.clear();
                v.push(0);
            }
        }
        for i in 0..other.value_len() {
            test_int(int)?;
            self.add_assign_internal(&self_clone, other.get(i), i);
        }
        Ok(())
    }

    /// computes `self += (other * mul_digit) << (64 * shift)`
    fn add_assign_internal(&mut self, other: &Self, mul_digit: u64, shift: usize) {
        let mut carry = 0;
        for i in 0..max(self.value_len(), other.value_len() + shift) {
            let a = self.get(i);
            let b = if i >= shift { other.get(i - shift) } else { 0 };
            let sum = u128::from(a) + (u128::from(b) * u128::from(mul_digit)) + u128::from(carry);
            self.set(i, truncate(sum));
            carry = truncate(sum >> 64);
        }
        if carry != 0 {
            self.value_push(carry);
        }
    }

    pub fn format(
        &self,
        f: &mut Formatter,
        base: Base,
        write_base_prefix: bool,
        int: &impl Interrupt,
    ) -> Result<Result<(), Error>, crate::err::Interrupt> {
        macro_rules! try_i {
            ($e:expr) => {
                if let Err(e) = $e {
                    return Ok(Err(e));
                }
            };
        }

        if write_base_prefix {
            try_i!(base.write_prefix(f));
        }

        if self.is_zero() {
            try_i!(write!(f, "0"));
            return Ok(Ok(()));
        }

        let mut num = self.clone();
        if num.value_len() == 1 && base.base_as_u8() == 10 {
            try_i!(write!(f, "{}", num.get(0)));
        } else {
            let mut output = String::new();
            let base_as_u128: u128 = base.base_as_u8().into();
            let mut divisor = base_as_u128;
            let mut rounds = 1;
            while divisor
                < u128::MAX
                    .checked_div(base_as_u128)
                    .expect("Base appears to be 0")
            {
                divisor *= base_as_u128;
                rounds += 1;
            }
            while !num.is_zero() {
                test_int(int)?;
                let divmod_res = num
                    .divmod(
                        &Self::Large(vec![truncate(divisor), truncate(divisor >> 64)]),
                        int,
                    )?
                    .expect("Division by zero is not allowed");
                let mut digit_group_value =
                    u128::from(divmod_res.1.get(1)) << 64 | u128::from(divmod_res.1.get(0));
                for _ in 0..rounds {
                    let digit_value = digit_group_value % base_as_u128;
                    digit_group_value /= base_as_u128;
                    let ch = Base::digit_as_char(truncate(digit_value)).unwrap();
                    output.insert(0, ch);
                }
                num = divmod_res.0;
            }
            output = output.trim_start_matches('0').to_string();
            try_i!(write!(f, "{}", output));
        }
        Ok(Ok(()))
    }

    // Note: 0! = 1, 1! = 1
    pub fn factorial(mut self, int: &impl Interrupt) -> Result<Self, String> {
        let mut res = Self::from(1);
        while self > 1.into() {
            test_int(int)?;
            res = res.mul(&self, int)?;
            self = self - 1.into();
        }
        Ok(res)
    }

    pub fn mul(
        mut self,
        other: &Self,
        int: &impl Interrupt,
    ) -> Result<Self, crate::err::Interrupt> {
        if let (Small(a), Small(b)) = (&self, &other) {
            if let Some(res) = a.checked_mul(*b) {
                return Ok(Self::from(res));
            }
        }
        self.mul_internal(&other, int)?;
        Ok(self)
    }

    fn rem(&self, other: &Self, int: &impl Interrupt) -> Result<BigUint, crate::err::Interrupt> {
        Ok(self.divmod(other, int)?.unwrap().1)
    }

    pub fn div(self, other: &Self, int: &impl Interrupt) -> Result<BigUint, crate::err::Interrupt> {
        Ok(self.divmod(other, int)?.unwrap().0)
    }
}

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        if let (Small(a), Small(b)) = (self, other) {
            return a.cmp(b);
        }
        let mut i = std::cmp::max(self.value_len(), other.value_len());
        while i != 0 {
            let v1 = self.get(i - 1);
            let v2 = other.get(i - 1);
            match v1.cmp(&v2) {
                Ordering::Less => return Ordering::Less,
                Ordering::Greater => return Ordering::Greater,
                Ordering::Equal => (),
            }
            i -= 1;
        }

        Ordering::Equal
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BigUint {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for BigUint {}

impl Hash for BigUint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Small(n) => n.hash(state),
            Large(v) => {
                v[0].hash(state);
                let mut zeroes = true;
                for &x in v.iter().skip(1) {
                    if !zeroes || x != 0 {
                        x.hash(state);
                    }
                    if x != 0 {
                        zeroes = false;
                    }
                }
            }
        }
    }
}

impl From<u64> for BigUint {
    fn from(val: u64) -> Self {
        Small(val)
    }
}

impl AddAssign<&BigUint> for BigUint {
    fn add_assign(&mut self, other: &Self) {
        self.add_assign_internal(other, 1, 0);
    }
}

impl Add for BigUint {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self += &other;
        self
    }
}

impl Sub for BigUint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if let (Small(a), Small(b)) = (&self, &other) {
            return Self::from(a - b);
        }
        if self < other {
            unreachable!("Number would be less than 0");
        }
        if self == other {
            return Self::from(0);
        }
        if other == 0.into() {
            return self;
        }
        let mut carry = 0; // 0 or 1
        let mut res = vec![];
        for i in 0..max(self.value_len(), other.value_len()) {
            let a = self.get(i);
            let b = other.get(i);
            if !(b == std::u64::MAX && carry == 1) && a >= b + carry {
                res.push(a - b - carry);
                carry = 0;
            } else {
                let next_digit =
                    u128::from(a) + ((1_u128) << 64) - u128::from(b) - u128::from(carry);
                res.push(truncate(next_digit));
                carry = 1;
            }
        }
        assert_eq!(carry, 0);
        Large(res)
    }
}

impl Debug for BigUint {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Small(n) => write!(f, "{}", n),
            Large(value) => write!(f, "{:?}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BigUint;

    #[test]
    fn test_sqrt() {
        let two = &BigUint::from(2);
        let int = crate::interrupt::Never::default();
        let test_sqrt_inner = |n, expected_root, exact| {
            assert_eq!(
                BigUint::from(n).root_n(two, &int).unwrap(),
                (BigUint::from(expected_root), exact)
            );
        };
        test_sqrt_inner(0, 0, true);
        test_sqrt_inner(1, 1, true);
        test_sqrt_inner(2, 1, false);
        test_sqrt_inner(3, 1, false);
        test_sqrt_inner(4, 2, true);
        test_sqrt_inner(5, 2, false);
        test_sqrt_inner(6, 2, false);
        test_sqrt_inner(7, 2, false);
        test_sqrt_inner(8, 2, false);
        test_sqrt_inner(9, 3, true);
        test_sqrt_inner(10, 3, false);
        test_sqrt_inner(11, 3, false);
        test_sqrt_inner(12, 3, false);
        test_sqrt_inner(13, 3, false);
        test_sqrt_inner(14, 3, false);
        test_sqrt_inner(15, 3, false);
        test_sqrt_inner(16, 4, true);
        test_sqrt_inner(17, 4, false);
        test_sqrt_inner(18, 4, false);
        test_sqrt_inner(19, 4, false);
        test_sqrt_inner(20, 4, false);
        test_sqrt_inner(200000, 447, false);
        test_sqrt_inner(1740123984719364372, 1319137591, false);
        assert_eq!(
            BigUint::Large(vec![0, 3260954456333195555])
                .root_n(two, &int)
                .unwrap(),
            (BigUint::from(7755900482342532476), false)
        );
    }

    #[test]
    fn test_cmp() {
        assert_eq!(BigUint::from(0), BigUint::from(0));
        assert!(BigUint::from(0) < BigUint::from(1));
        assert!(BigUint::from(100) > BigUint::from(1));
        assert!(BigUint::from(10000000) > BigUint::from(1));
        assert!(BigUint::from(10000000) > BigUint::from(9999999));
    }

    #[test]
    fn test_addition() {
        assert_eq!(BigUint::from(2) + BigUint::from(2), BigUint::from(4));
        assert_eq!(BigUint::from(5) + BigUint::from(3), BigUint::from(8));
        assert_eq!(
            BigUint::from(0) + BigUint::Large(vec![0, 9223372036854775808, 0]),
            BigUint::Large(vec![0, 9223372036854775808, 0])
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(BigUint::from(5) - BigUint::from(3), BigUint::from(2));
        assert_eq!(BigUint::from(0) - BigUint::from(0), BigUint::from(0));
    }

    #[test]
    fn test_multiplication() {
        let int = &crate::interrupt::Never::default();
        assert_eq!(
            BigUint::from(20).mul(&BigUint::from(3), int).unwrap(),
            BigUint::from(60)
        );
    }

    #[test]
    fn test_small_division_by_two() {
        let int = &crate::interrupt::Never::default();
        assert_eq!(
            BigUint::from(0).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(0)
        );
        assert_eq!(
            BigUint::from(1).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(0)
        );
        assert_eq!(
            BigUint::from(2).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(1)
        );
        assert_eq!(
            BigUint::from(3).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(1)
        );
        assert_eq!(
            BigUint::from(4).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(2)
        );
        assert_eq!(
            BigUint::from(5).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(2)
        );
        assert_eq!(
            BigUint::from(6).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(3)
        );
        assert_eq!(
            BigUint::from(7).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(3)
        );
        assert_eq!(
            BigUint::from(8).div(&BigUint::from(2), int).unwrap(),
            BigUint::from(4)
        );
    }

    #[test]
    fn test_rem() {
        let int = &crate::interrupt::Never::default();
        assert_eq!(
            BigUint::from(20).rem(&BigUint::from(3), int).unwrap(),
            BigUint::from(2)
        );
        assert_eq!(
            BigUint::from(21).rem(&BigUint::from(3), int).unwrap(),
            BigUint::from(0)
        );
        assert_eq!(
            BigUint::from(22).rem(&BigUint::from(3), int).unwrap(),
            BigUint::from(1)
        );
        assert_eq!(
            BigUint::from(23).rem(&BigUint::from(3), int).unwrap(),
            BigUint::from(2)
        );
        assert_eq!(
            BigUint::from(24).rem(&BigUint::from(3), int).unwrap(),
            BigUint::from(0)
        );
    }

    #[test]
    fn test_lshift() {
        let mut n = BigUint::from(1);
        for _ in 0..100 {
            n.lshift();
            eprintln!("{:?}", &n);
            assert_eq!(n.get(0) & 1, 0);
        }
    }

    #[test]
    fn test_gcd() {
        let int = &crate::interrupt::Never::default();
        assert_eq!(BigUint::gcd(2.into(), 4.into(), int).unwrap(), 2.into());
        assert_eq!(BigUint::gcd(4.into(), 2.into(), int).unwrap(), 2.into());
        assert_eq!(BigUint::gcd(37.into(), 43.into(), int).unwrap(), 1.into());
        assert_eq!(BigUint::gcd(43.into(), 37.into(), int).unwrap(), 1.into());
        assert_eq!(BigUint::gcd(215.into(), 86.into(), int).unwrap(), 43.into());
        assert_eq!(BigUint::gcd(86.into(), 215.into(), int).unwrap(), 43.into());
    }

    #[test]
    fn test_add_assign_internal() {
        // 0 += (1 * 1) << (64 * 1)
        let mut x = BigUint::from(0);
        x.add_assign_internal(&BigUint::from(1), 1, 1);
        assert_eq!(x, BigUint::Large(vec![0, 1]));
    }

    #[test]
    fn test_large_lshift() {
        let mut a = BigUint::from(9223372036854775808);
        a.lshift();
        assert!(!a.is_zero());
    }

    #[test]
    fn test_big_multiplication() {
        let int = &crate::interrupt::Never::default();
        assert_eq!(
            BigUint::from(1)
                .mul(&BigUint::Large(vec![0, 1]), int)
                .unwrap(),
            BigUint::Large(vec![0, 1])
        );
    }
}
