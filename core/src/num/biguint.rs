use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::{Base, DivideByZero, Exact, IntegerPowerError, ValueOutOfRange};
use std::cmp::{max, Ordering};
use std::fmt;

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
                for v in value.iter() {
                    if *v != 0 {
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

    pub fn try_as_usize(&self) -> Result<usize, ValueOutOfRange<usize>> {
        use std::convert::TryFrom;
        // todo: include `self` in the error message
        // This requires rewriting the BigUint format code to use a separate
        // struct that implements Display
        let error = ValueOutOfRange::MustBeLessThanOrEqualTo(usize::MAX);

        Ok(match self {
            Small(n) => {
                if let Ok(res) = usize::try_from(*n) {
                    res
                } else {
                    return Err(error);
                }
            }
            Large(v) => {
                // todo use correct method to get actual length excluding leading zeroes
                if v.len() == 1 {
                    if let Ok(res) = usize::try_from(v[0]) {
                        res
                    } else {
                        return Err(error);
                    }
                } else {
                    return Err(error);
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

    pub fn gcd<I: Interrupt>(mut a: Self, mut b: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        while b >= 1.into() {
            let r = a
                .rem(&b, int)
                .map_err(|e| e.expect("Unexpected division by zero"))?;
            a = b;
            b = r;
        }

        Ok(a)
    }

    pub fn pow<I: Interrupt>(
        a: &Self,
        b: &Self,
        int: &I,
    ) -> Result<Self, IntErr<IntegerPowerError, I>> {
        if a.is_zero() && b.is_zero() {
            return Err(IntegerPowerError::ZeroToThePowerOfZero)?;
        }
        if b.is_zero() {
            return Ok(Self::from(1));
        }
        if b.value_len() > 1 {
            return Err(IntegerPowerError::ExponentTooLarge)?;
        }
        Ok(a.pow_internal(b.get(0), int)?)
    }

    // computes the exact square root if possible, otherwise the next lower integer
    pub fn root_n<I: Interrupt>(
        self,
        n: &Self,
        int: &I,
    ) -> Result<Exact<Self>, IntErr<IntegerPowerError, I>> {
        if self == 0.into() || self == 1.into() || n == &Self::from(1) {
            return Ok(Exact::new(self, true));
        }
        let mut low_guess = Self::from(1);
        let mut high_guess = self.clone();
        while high_guess.clone().sub(&low_guess) > 1.into() {
            test_int(int)?;
            let mut guess = low_guess.clone().add(&high_guess);
            guess.rshift(int)?;

            let res = Self::pow(&guess, n, int)?;
            match res.cmp(&self) {
                Ordering::Equal => return Ok(Exact::new(guess, true)),
                Ordering::Greater => high_guess = guess,
                Ordering::Less => low_guess = guess,
            }
        }
        Ok(Exact::new(low_guess, false))
    }

    fn pow_internal<I: Interrupt>(
        &self,
        mut exponent: u64,
        int: &I,
    ) -> Result<Self, IntErr<Never, I>> {
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

    fn lshift<I: Interrupt>(&mut self, int: &I) -> Result<(), IntErr<Never, I>> {
        match self {
            Small(n) => {
                if *n & 0xc000_0000_0000_0000 == 0 {
                    *n <<= 1;
                } else {
                    *self = Large(vec![*n << 1, *n >> 63]);
                }
            }
            Large(value) => {
                if value[value.len() - 1] & (1_u64 << 63) != 0 {
                    value.push(0);
                }
                for i in (0..value.len()).rev() {
                    test_int(int)?;
                    value[i] <<= 1;
                    if i != 0 {
                        value[i] |= value[i - 1] >> 63;
                    }
                }
            }
        }
        Ok(())
    }

    fn rshift<I: Interrupt>(&mut self, int: &I) -> Result<(), IntErr<Never, I>> {
        match self {
            Small(n) => *n >>= 1,
            Large(value) => {
                for i in 0..value.len() {
                    test_int(int)?;
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
        Ok(())
    }

    pub fn divmod<I: Interrupt>(
        &self,
        other: &Self,
        int: &I,
    ) -> Result<(Self, Self), IntErr<DivideByZero, I>> {
        if let (Small(a), Small(b)) = (self, other) {
            if let (Some(div_res), Some(mod_res)) = (a.checked_div(*b), a.checked_rem(*b)) {
                return Ok((Small(div_res), Small(mod_res)));
            }
            return Err(DivideByZero {})?;
        }
        if other.is_zero() {
            return Err(DivideByZero {})?;
        }
        if other == &Self::from(1) {
            return Ok((self.clone(), Self::from(0)));
        }
        if self.is_zero() {
            return Ok((Self::from(0), Self::from(0)));
        }
        if self < other {
            return Ok((Self::from(0), self.clone()));
        }
        if self == other {
            return Ok((Self::from(1), Self::from(0)));
        }
        if other == &Self::from(2) {
            let mut div_result = self.clone();
            div_result.rshift(int)?;
            let modulo = self.get(0) & 1;
            return Ok((div_result, Self::from(modulo)));
        }
        // binary long division
        let mut q = Self::from(0);
        let mut r = Self::from(0);
        for i in (0..self.value_len()).rev() {
            test_int(int)?;
            for j in (0..64).rev() {
                r.lshift(int)?;
                let bit_of_self = if (self.get(i) & (1 << j)) == 0 { 0 } else { 1 };
                r.set(0, r.get(0) | bit_of_self);
                if &r >= other {
                    r = r.sub(other);
                    q.set(i, q.get(i) | (1 << j));
                }
            }
        }
        Ok((q, r))
    }

    /// computes self *= other
    fn mul_internal<I: Interrupt>(
        &mut self,
        other: &Self,
        int: &I,
    ) -> Result<(), IntErr<Never, I>> {
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

    pub fn format<I: Interrupt>(
        &self,
        base: Base,
        write_base_prefix: bool,
        sf_limit: Option<usize>,
        int: &I,
    ) -> Result<Exact<FormattedBigUint>, IntErr<Never, I>> {
        let base_prefix = if write_base_prefix { Some(base) } else { None };

        if self.is_zero() {
            return Ok(Exact::new(
                FormattedBigUint {
                    base: base_prefix,
                    ty: FormattedBigUintType::Zero,
                },
                true,
            ));
        }

        let mut num = self.clone();
        Ok(
            if num.value_len() == 1 && base.base_as_u8() == 10 && sf_limit.is_none() {
                Exact::new(
                    FormattedBigUint {
                        base: base_prefix,
                        ty: FormattedBigUintType::Simple(num.get(0)),
                    },
                    true,
                )
            } else {
                let base_as_u128: u128 = base.base_as_u8().into();
                let mut divisor = base_as_u128;
                let mut rounds = 1;
                // note that the string is reversed: this is the number of trailing zeroes while
                // printing, but actually the number of leading zeroes in the final number
                let mut num_trailing_zeroes = 0;
                let mut num_leading_zeroes = 0;
                let mut finished_counting_leading_zeroes = false;
                while divisor
                    < u128::MAX
                        .checked_div(base_as_u128)
                        .expect("Base appears to be 0")
                {
                    divisor *= base_as_u128;
                    rounds += 1;
                }
                let divisor = Self::Large(vec![truncate(divisor), truncate(divisor >> 64)]);
                let mut output = String::with_capacity(rounds);
                while !num.is_zero() {
                    test_int(int)?;
                    let divmod_res = num
                        .divmod(&divisor, int)
                        .map_err(|e| e.expect("Division by zero is not allowed"))?;
                    let mut digit_group_value =
                        u128::from(divmod_res.1.get(1)) << 64 | u128::from(divmod_res.1.get(0));
                    for _ in 0..rounds {
                        let digit_value = digit_group_value % base_as_u128;
                        digit_group_value /= base_as_u128;
                        let ch = Base::digit_as_char(truncate(digit_value)).unwrap();
                        if ch == '0' {
                            num_trailing_zeroes += 1;
                        } else {
                            for _ in 0..num_trailing_zeroes {
                                output.push('0');
                                if !finished_counting_leading_zeroes {
                                    num_leading_zeroes += 1;
                                }
                            }
                            finished_counting_leading_zeroes = true;
                            num_trailing_zeroes = 0;
                            output.push(ch);
                        }
                    }
                    num = divmod_res.0;
                }
                let exact = sf_limit.map_or(true, |sf| sf >= output.len() - num_leading_zeroes);
                Exact::new(
                    FormattedBigUint {
                        base: base_prefix,
                        ty: FormattedBigUintType::Complex(output, sf_limit),
                    },
                    exact,
                )
            },
        )
    }

    // Note: 0! = 1, 1! = 1
    pub fn factorial<I: Interrupt>(mut self, int: &I) -> Result<Self, IntErr<Never, I>> {
        let mut res = Self::from(1);
        while self > 1.into() {
            test_int(int)?;
            res = res.mul(&self, int)?;
            self = self.sub(&1.into());
        }
        Ok(res)
    }

    pub fn mul<I: Interrupt>(mut self, other: &Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        if let (Small(a), Small(b)) = (&self, &other) {
            if let Some(res) = a.checked_mul(*b) {
                return Ok(Self::from(res));
            }
        }
        self.mul_internal(&other, int)?;
        Ok(self)
    }

    fn rem<I: Interrupt>(&self, other: &Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
        Ok(self.divmod(other, int)?.1)
    }

    pub fn is_even<I: Interrupt>(&self, int: &I) -> Result<bool, IntErr<DivideByZero, I>> {
        Ok(self.divmod(&Self::from(2), int)?.1 == 0.into())
    }

    pub fn div<I: Interrupt>(self, other: &Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
        Ok(self.divmod(other, int)?.0)
    }

    pub fn add(mut self, other: &Self) -> Self {
        self.add_assign_internal(other, 1, 0);
        self
    }

    pub fn sub(self, other: &Self) -> Self {
        if let (Small(a), Small(b)) = (&self, &other) {
            return Self::from(a - b);
        }
        match self.cmp(other) {
            Ordering::Equal => return Self::from(0),
            Ordering::Less => unreachable!("Number would be less than 0"),
            Ordering::Greater => (),
        };
        if other.is_zero() {
            return self;
        }
        let mut carry = 0; // 0 or 1
        let mut res = match self {
            Large(x) => x,
            Small(v) => vec![v],
        };
        if res.len() < other.value_len() {
            res.resize(other.value_len(), 0);
        }
        for (i, a) in res.iter_mut().enumerate() {
            let b = other.get(i);
            if !(b == std::u64::MAX && carry == 1) && *a >= b + carry {
                *a = *a - b - carry;
                carry = 0;
            } else {
                let next_digit =
                    u128::from(*a) + ((1_u128) << 64) - u128::from(b) - u128::from(carry);
                *a = truncate(next_digit);
                carry = 1;
            }
        }
        assert_eq!(carry, 0);
        Large(res)
    }

    pub fn is_definitely_zero(&self) -> bool {
        match self {
            Small(x) => *x == 0,
            Large(_) => false,
        }
    }

    pub fn is_definitely_one(&self) -> bool {
        match self {
            Small(x) => *x == 1,
            Large(_) => false,
        }
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

impl From<u64> for BigUint {
    fn from(val: u64) -> Self {
        Small(val)
    }
}

impl fmt::Debug for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Small(n) => write!(f, "{}", n)?,
            Large(value) => {
                write!(f, "[")?;
                let mut first = true;
                for v in value.iter().rev() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                    first = false;
                }
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum FormattedBigUintType {
    Zero,
    Simple(u64),
    Complex(String, Option<usize>),
}

#[must_use]
#[derive(Debug)]
pub struct FormattedBigUint {
    base: Option<Base>,
    ty: FormattedBigUintType,
}

impl fmt::Display for FormattedBigUint {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(base) = self.base {
            base.write_prefix(f)?;
        }
        match &self.ty {
            FormattedBigUintType::Zero => write!(f, "0")?,
            FormattedBigUintType::Simple(i) => write!(f, "{}", i)?,
            FormattedBigUintType::Complex(s, sf_limit) => {
                for (i, ch) in s.chars().rev().enumerate() {
                    if sf_limit.is_some() && i >= sf_limit.unwrap() {
                        write!(f, "0")?;
                    } else {
                        write!(f, "{}", ch)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl FormattedBigUint {
    pub fn num_digits(&self) -> usize {
        match &self.ty {
            FormattedBigUintType::Zero => 1,
            FormattedBigUintType::Simple(i) => i.to_string().len(),
            FormattedBigUintType::Complex(s, _) => s.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BigUint;
    use crate::err::{IntErr, Never};
    type Res<E = Never> = Result<(), IntErr<E, crate::interrupt::Never>>;

    #[test]
    fn test_sqrt() -> Res<crate::num::IntegerPowerError> {
        let two = &BigUint::from(2);
        let int = crate::interrupt::Never::default();
        let test_sqrt_inner = |n, expected_root, exact| -> Res<crate::num::IntegerPowerError> {
            let actual = BigUint::from(n).root_n(two, &int)?;
            assert_eq!(actual.value, BigUint::from(expected_root));
            assert_eq!(actual.exact, exact);
            Ok(())
        };
        test_sqrt_inner(0, 0, true)?;
        test_sqrt_inner(1, 1, true)?;
        test_sqrt_inner(2, 1, false)?;
        test_sqrt_inner(3, 1, false)?;
        test_sqrt_inner(4, 2, true)?;
        test_sqrt_inner(5, 2, false)?;
        test_sqrt_inner(6, 2, false)?;
        test_sqrt_inner(7, 2, false)?;
        test_sqrt_inner(8, 2, false)?;
        test_sqrt_inner(9, 3, true)?;
        test_sqrt_inner(10, 3, false)?;
        test_sqrt_inner(11, 3, false)?;
        test_sqrt_inner(12, 3, false)?;
        test_sqrt_inner(13, 3, false)?;
        test_sqrt_inner(14, 3, false)?;
        test_sqrt_inner(15, 3, false)?;
        test_sqrt_inner(16, 4, true)?;
        test_sqrt_inner(17, 4, false)?;
        test_sqrt_inner(18, 4, false)?;
        test_sqrt_inner(19, 4, false)?;
        test_sqrt_inner(20, 4, false)?;
        test_sqrt_inner(200_000, 447, false)?;
        test_sqrt_inner(1_740_123_984_719_364_372, 1_319_137_591, false)?;
        let val = BigUint::Large(vec![0, 3_260_954_456_333_195_555]).root_n(two, &int)?;
        assert_eq!(val.value, BigUint::from(7_755_900_482_342_532_476));
        assert_eq!(val.exact, false);
        Ok(())
    }

    #[test]
    fn test_cmp() {
        assert_eq!(BigUint::from(0), BigUint::from(0));
        assert!(BigUint::from(0) < BigUint::from(1));
        assert!(BigUint::from(100) > BigUint::from(1));
        assert!(BigUint::from(10_000_000) > BigUint::from(1));
        assert!(BigUint::from(10_000_000) > BigUint::from(9_999_999));
    }

    #[test]
    fn test_addition() {
        assert_eq!(BigUint::from(2).add(&BigUint::from(2)), BigUint::from(4));
        assert_eq!(BigUint::from(5).add(&BigUint::from(3)), BigUint::from(8));
        assert_eq!(
            BigUint::from(0).add(&BigUint::Large(vec![0, 9_223_372_036_854_775_808, 0])),
            BigUint::Large(vec![0, 9_223_372_036_854_775_808, 0])
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(BigUint::from(5).sub(&BigUint::from(3)), BigUint::from(2));
        assert_eq!(BigUint::from(0).sub(&BigUint::from(0)), BigUint::from(0));
    }

    #[test]
    fn test_multiplication() -> Res {
        let int = &crate::interrupt::Never::default();
        assert_eq!(
            BigUint::from(20).mul(&BigUint::from(3), int)?,
            BigUint::from(60)
        );
        Ok(())
    }

    #[test]
    fn test_small_division_by_two() -> Res<crate::num::DivideByZero> {
        let int = &crate::interrupt::Never::default();
        let two = BigUint::from(2);
        assert_eq!(BigUint::from(0).div(&two, int)?, BigUint::from(0));
        assert_eq!(BigUint::from(1).div(&two, int)?, BigUint::from(0));
        assert_eq!(BigUint::from(2).div(&two, int)?, BigUint::from(1));
        assert_eq!(BigUint::from(3).div(&two, int)?, BigUint::from(1));
        assert_eq!(BigUint::from(4).div(&two, int)?, BigUint::from(2));
        assert_eq!(BigUint::from(5).div(&two, int)?, BigUint::from(2));
        assert_eq!(BigUint::from(6).div(&two, int)?, BigUint::from(3));
        assert_eq!(BigUint::from(7).div(&two, int)?, BigUint::from(3));
        assert_eq!(BigUint::from(8).div(&two, int)?, BigUint::from(4));
        Ok(())
    }

    #[test]
    fn test_rem() -> Res<crate::num::DivideByZero> {
        let int = &crate::interrupt::Never::default();
        let three = BigUint::from(3);
        assert_eq!(BigUint::from(20).rem(&three, int)?, BigUint::from(2));
        assert_eq!(BigUint::from(21).rem(&three, int)?, BigUint::from(0));
        assert_eq!(BigUint::from(22).rem(&three, int)?, BigUint::from(1));
        assert_eq!(BigUint::from(23).rem(&three, int)?, BigUint::from(2));
        assert_eq!(BigUint::from(24).rem(&three, int)?, BigUint::from(0));
        Ok(())
    }

    #[test]
    fn test_lshift() -> Res {
        let int = &crate::interrupt::Never::default();
        let mut n = BigUint::from(1);
        for _ in 0..100 {
            n.lshift(int)?;
            assert_eq!(n.get(0) & 1, 0);
        }
        Ok(())
    }

    #[test]
    fn test_gcd() -> Res {
        let int = &crate::interrupt::Never::default();
        assert_eq!(BigUint::gcd(2.into(), 4.into(), int)?, 2.into());
        assert_eq!(BigUint::gcd(4.into(), 2.into(), int)?, 2.into());
        assert_eq!(BigUint::gcd(37.into(), 43.into(), int)?, 1.into());
        assert_eq!(BigUint::gcd(43.into(), 37.into(), int)?, 1.into());
        assert_eq!(BigUint::gcd(215.into(), 86.into(), int)?, 43.into());
        assert_eq!(BigUint::gcd(86.into(), 215.into(), int)?, 43.into());
        Ok(())
    }

    #[test]
    fn test_add_assign_internal() {
        // 0 += (1 * 1) << (64 * 1)
        let mut x = BigUint::from(0);
        x.add_assign_internal(&BigUint::from(1), 1, 1);
        assert_eq!(x, BigUint::Large(vec![0, 1]));
    }

    #[test]
    fn test_large_lshift() -> Res {
        let int = &crate::interrupt::Never::default();
        let mut a = BigUint::from(9_223_372_036_854_775_808);
        a.lshift(int)?;
        assert!(!a.is_zero());
        Ok(())
    }

    #[test]
    fn test_big_multiplication() -> Res {
        let int = &crate::interrupt::Never::default();
        assert_eq!(
            BigUint::from(1).mul(&BigUint::Large(vec![0, 1]), int)?,
            BigUint::Large(vec![0, 1])
        );
        Ok(())
    }
}
