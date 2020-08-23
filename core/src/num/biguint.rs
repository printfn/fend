use crate::num::Base;
use std::cmp::{max, Ordering};
use std::fmt::{Debug, Error, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Rem, Sub};

#[derive(Clone)]
pub enum BigUint {
    Small(u64),
    // little-endian, len >= 1
    Large(Vec<u64>),
}

use BigUint::{Large, Small};

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
                } else if new_value != 0 {
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
}

impl Ord for BigUint {
    fn cmp(&self, other: &BigUint) -> Ordering {
        match (self, other) {
            (Small(a), Small(b)) => return a.cmp(b),
            _ => (),
        }
        let mut i = std::cmp::max(self.value_len(), other.value_len());
        while i != 0 {
            let v1 = self.get(i - 1);
            let v2 = other.get(i - 1);
            if v1 < v2 {
                return Ordering::Less;
            } else if v1 > v2 {
                return Ordering::Greater;
            }
            i -= 1;
        }

        Ordering::Equal
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &BigUint) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BigUint {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for BigUint {}

impl From<u64> for BigUint {
    fn from(val: u64) -> BigUint {
        Small(val)
    }
}

impl BigUint {
    /// computes self += (other * mul_digit) << (64 * shift)
    fn add_assign_internal(&mut self, other: &BigUint, mul_digit: u64, shift: usize) {
        let mut carry = 0;
        for i in 0..max(self.value_len(), other.value_len() + shift) {
            let a = self.get(i);
            let b = if i >= shift { other.get(i - shift) } else { 0 };
            let sum = a as u128 + (b as u128 * mul_digit as u128) + carry as u128;
            self.set(i, sum as u64);
            carry = (sum >> 64) as u64;
        }
        if carry != 0 {
            self.value_push(carry);
        }
    }
}

impl AddAssign<&BigUint> for BigUint {
    fn add_assign(&mut self, other: &BigUint) {
        self.add_assign_internal(other, 1, 0);
    }
}

impl BigUint {
    fn pow_internal(&self, mut exponent: u64) -> BigUint {
        let mut result = BigUint::from(1);
        let mut base = self.clone();
        while exponent > 0 {
            if exponent % 2 == 1 {
                result = &result * &base;
            }
            exponent >>= 1;
            base = &base * &base;
        }
        result
    }

    fn lshift(&mut self) {
        match self {
            Small(n) => {
                if *n & 0xc000_0000_0000_0000 != 0 {
                    self.make_large();
                    self.lshift();
                } else {
                    *n <<= 1;
                }
            }
            Large(value) => {
                if value[value.len() - 1] & (1u64 << 63) != 0 {
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

    fn divmod(&self, other: &BigUint) -> (BigUint, BigUint) {
        match (self, other) {
            (Small(a), Small(b)) => return (Small(*a / *b), Small(*a % *b)),
            _ => (),
        }
        if other.is_zero() {
            panic!("Can't divide by 0");
        }
        if other == &BigUint::from(1) {
            return (self.clone(), BigUint::from(0));
        }
        if self.is_zero() {
            return (BigUint::from(0), BigUint::from(0));
        }
        if self < other {
            return (BigUint::from(0), self.clone());
        }
        if self == other {
            return (BigUint::from(1), BigUint::from(0));
        }
        if other == &BigUint::from(2) {
            let mut div_result = self.clone();
            div_result.rshift();
            let modulo = self.get(0) & 1;
            return (div_result, BigUint::from(modulo));
        }
        let mut remaining_dividend = self.clone();
        let mut quotient = BigUint::from(0);
        let mut step_size = BigUint::from(1);
        let mut step_size_times_other = &step_size * other;
        while &remaining_dividend >= other {
            while step_size_times_other < remaining_dividend {
                step_size.lshift();
                step_size_times_other.lshift();
            }
            while step_size_times_other > remaining_dividend {
                step_size.rshift();
                step_size_times_other.rshift();
            }
            remaining_dividend = remaining_dividend.clone() - step_size_times_other.clone();
            quotient += &step_size;
        }
        (quotient, remaining_dividend)
    }

    /// computes self *= other
    fn mul_internal(&mut self, other: BigUint) {
        if self.is_zero() || other.is_zero() {
            *self = BigUint::from(0);
            return;
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
            self.add_assign_internal(&self_clone, other.get(i), i);
        }
    }
}

impl Add for BigUint {
    type Output = BigUint;

    fn add(mut self, other: BigUint) -> BigUint {
        self += &other;
        self
    }
}

impl Sub for BigUint {
    type Output = BigUint;

    fn sub(self, other: BigUint) -> BigUint {
        match (&self, &other) {
            (Small(a), Small(b)) => return BigUint::from(a - b),
            _ => (),
        }
        if self < other {
            panic!("Number would be less than 0");
        }
        if self == other {
            return BigUint::from(0);
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
                res.push((a as u128 + ((1 as u128) << 64) - b as u128 - carry as u128) as u64);
                carry = 1;
            }
        }
        assert_eq!(carry, 0);
        BigUint::Large(res)
    }
}

impl Mul for &BigUint {
    type Output = BigUint;

    fn mul(self, other: &BigUint) -> BigUint {
        match (self, other) {
            (Small(a), Small(b)) => {
                if let Some(res) = a.checked_mul(*b) {
                    return BigUint::from(res);
                }
            }
            _ => (),
        }
        let mut res = self.clone();
        res.mul_internal(other.clone());
        res
    }
}

impl Mul for BigUint {
    type Output = BigUint;

    fn mul(mut self, other: BigUint) -> BigUint {
        match (&self, &other) {
            (Small(a), Small(b)) => {
                if let Some(res) = a.checked_mul(*b) {
                    return BigUint::from(res);
                }
            }
            _ => (),
        }
        self.mul_internal(other);
        self
    }
}

impl Div for BigUint {
    type Output = BigUint;

    fn div(self, other: BigUint) -> BigUint {
        self.divmod(&other).0
    }
}

impl Div for &BigUint {
    type Output = BigUint;

    fn div(self, other: &BigUint) -> BigUint {
        self.divmod(other).0
    }
}

impl Rem for BigUint {
    type Output = BigUint;

    fn rem(self, other: BigUint) -> BigUint {
        self.divmod(&other).1
    }
}

impl Rem for &BigUint {
    type Output = BigUint;

    fn rem(self, other: &BigUint) -> BigUint {
        self.divmod(other).1
    }
}

impl BigUint {
    pub fn format(
        &self,
        f: &mut Formatter,
        base: Base,
        write_base_prefix: bool,
    ) -> Result<(), Error> {
        use std::convert::TryFrom;

        if write_base_prefix {
            base.write_prefix(f)?;
        }

        if self.is_zero() {
            write!(f, "0")?;
            return Ok(());
        }

        let mut num = self.clone();
        if num.value_len() == 1 && base.base_as_u8() == 10 {
            write!(f, "{}", num.get(0))?;
        } else {
            let mut output = String::new();
            while !num.is_zero() {
                let base_as_u64: u64 = base.base_as_u8().into();
                let divmod_res = num.divmod(&BigUint::from(base_as_u64));
                let digit_value = divmod_res.1.get(0) as u8;
                let ch = if digit_value < 10 {
                    char::try_from(digit_value + 48).unwrap()
                } else {
                    char::try_from(digit_value + 96 - 9).unwrap()
                };
                output.insert(0, ch);
                num = divmod_res.0;
            }
            write!(f, "{}", output)?;
        }
        Ok(())
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

impl BigUint {
    pub fn gcd(mut a: BigUint, mut b: BigUint) -> BigUint {
        while b >= 1.into() {
            let r = a.clone() % b.clone();
            a = b;
            b = r;
        }

        a
    }

    pub fn lcm(a: BigUint, b: BigUint) -> BigUint {
        a.clone() * b.clone() / BigUint::gcd(a, b)
    }

    pub fn pow(a: BigUint, b: BigUint) -> Result<BigUint, String> {
        if a.is_zero() && b.is_zero() {
            return Err("Zero to the power of zero is undefined".to_string());
        }
        if b.is_zero() {
            return Ok(BigUint::from(1));
        }
        if b.value_len() > 1 {
            return Err("Exponent too large".to_string());
        }
        Ok(a.pow_internal(b.get(0)))
    }

    // computes the exact square root if possible, otherwise the next lower integer
    pub fn root_n(self, n: &BigUint) -> Result<(BigUint, bool), String> {
        if self == 0.into() || self == 1.into() {
            return Ok((self, true));
        }
        let mut low_guess = BigUint::from(1);
        let mut high_guess = self.clone();
        while high_guess.clone() - low_guess.clone() > 1.into() {
            let mut guess = low_guess.clone() + high_guess.clone();
            guess.rshift();

            let res = Self::pow(guess.clone(), n.clone())?;
            if res == self {
                return Ok((guess, true));
            } else if res > self {
                high_guess = guess;
            } else if res < self {
                low_guess = guess;
            }
        }
        Ok((low_guess, false))
    }
}

#[cfg(test)]
mod tests {
    use super::BigUint;

    #[test]
    fn test_sqrt() {
        let two = &BigUint::from(2);
        let test_sqrt_inner = |n, expected_root, exact| {
            assert_eq!(
                BigUint::from(n).root_n(two).unwrap(),
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
                .root_n(two)
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
        assert_eq!(BigUint::from(20) * BigUint::from(3), BigUint::from(60));
    }

    #[test]
    fn test_small_division_by_two() {
        assert_eq!(BigUint::from(0) / BigUint::from(2), BigUint::from(0));
        assert_eq!(BigUint::from(1) / BigUint::from(2), BigUint::from(0));
        assert_eq!(BigUint::from(2) / BigUint::from(2), BigUint::from(1));
        assert_eq!(BigUint::from(3) / BigUint::from(2), BigUint::from(1));
        assert_eq!(BigUint::from(4) / BigUint::from(2), BigUint::from(2));
        assert_eq!(BigUint::from(5) / BigUint::from(2), BigUint::from(2));
        assert_eq!(BigUint::from(6) / BigUint::from(2), BigUint::from(3));
        assert_eq!(BigUint::from(7) / BigUint::from(2), BigUint::from(3));
        assert_eq!(BigUint::from(8) / BigUint::from(2), BigUint::from(4));
    }

    #[test]
    fn test_rem() {
        assert_eq!(BigUint::from(20) % BigUint::from(3), BigUint::from(2));
        assert_eq!(BigUint::from(21) % BigUint::from(3), BigUint::from(0));
        assert_eq!(BigUint::from(22) % BigUint::from(3), BigUint::from(1));
        assert_eq!(BigUint::from(23) % BigUint::from(3), BigUint::from(2));
        assert_eq!(BigUint::from(24) % BigUint::from(3), BigUint::from(0));
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
        assert_eq!(BigUint::gcd(2.into(), 4.into()), 2.into());
        assert_eq!(BigUint::gcd(4.into(), 2.into()), 2.into());
        assert_eq!(BigUint::gcd(37.into(), 43.into()), 1.into());
        assert_eq!(BigUint::gcd(43.into(), 37.into()), 1.into());
        assert_eq!(BigUint::gcd(215.into(), 86.into()), 43.into());
        assert_eq!(BigUint::gcd(86.into(), 215.into()), 43.into());
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
        assert_eq!(
            BigUint::from(1) * BigUint::Large(vec![0, 1]),
            BigUint::Large(vec![0, 1])
        );
    }
}
