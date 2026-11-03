use std::{convert, fmt, io};

use crate::format::DisplayDebug;
use crate::num::RangeBound;
use crate::{
	error::FendError,
	result::FResult,
	serialize::{Deserialize, Serialize},
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Year(i32);

impl Year {
	pub(crate) const MAX: Self = Self::new(i32::MAX);
	pub(crate) const MIN: Self = Self::new(i32::MIN);

	pub(crate) const fn new(year: i32) -> Self {
		assert!(year != 0, "year 0 is invalid");
		Self(year)
	}

	#[inline]
	pub(crate) const fn value(self) -> i32 {
		self.0
	}

	pub(crate) fn out_of_range_error(value: impl DisplayDebug + 'static) -> FendError {
		FendError::OutOfRange {
			value: Box::new(value),
			range: crate::num::Range {
				start: RangeBound::Closed(Box::new(Self::MIN)),
				end: RangeBound::Closed(Box::new(Self::MAX)),
			},
		}
	}

	pub(crate) fn add(
		self,
		value: impl TryInto<u32> + Copy + DisplayDebug + 'static,
	) -> FResult<Self> {
		let value = value.try_into().map_err(|_| FendError::ValueTooLarge)?;

		let new_year = self
			.value()
			.checked_add_unsigned(value)
			.ok_or(FendError::ValueTooLarge)?;

		Ok(if new_year == 0 {
			Self::new(1)
		} else {
			match (self.value().is_positive(), new_year.is_positive()) {
				(true, true) | (false, false) => Self::new(new_year),
				(false, true) => Self::new(new_year).next()?, // add one year because 0 isn't valid.
				(true, false) => unreachable!("Year can't have become negative"),
			}
		})
	}

	pub(crate) fn next(self) -> FResult<Self> {
		Ok(if self.value() == -1 {
			Self::new(1)
		} else {
			Self::new(
				self.value().checked_add(1).ok_or_else(|| {
					Self::out_of_range_error(const { Self::MAX.value() as i64 + 1 })
				})?,
			)
		})
	}

	pub(crate) fn sub(self, value: impl TryInto<u32> + Copy + 'static) -> FResult<Self> {
		let value = value.try_into().map_err(|_| FendError::ValueTooLarge)?;

		let new_year = self
			.value()
			.checked_sub_unsigned(value)
			.ok_or(FendError::ValueTooLarge)?;

		Ok(if new_year == 0 {
			Self::new(-1)
		} else {
			match (self.value().is_positive(), new_year.is_positive()) {
				(true, true) | (false, false) => Self::new(new_year),
				(true, false) => Self::new(new_year).prev()?, // sub one year because 0 isn't valid.
				(false, true) => unreachable!("Year can't have become positive"),
			}
		})
	}

	pub(crate) fn prev(self) -> FResult<Self> {
		Ok(if self.value() == 1 {
			Self::new(-1)
		} else {
			Self::new(
				self.value().checked_sub(1).ok_or_else(|| {
					Self::out_of_range_error(const { Self::MIN.value() as i64 - 1 })
				})?,
			)
		})
	}

	pub(crate) fn is_leap_year(self) -> bool {
		if self.value() % 400 == 0 {
			true
		} else if self.value() % 100 == 0 {
			false
		} else {
			self.value() % 4 == 0
		}
	}

	pub(crate) fn number_of_days(self) -> u16 {
		if self.is_leap_year() { 366 } else { 365 }
	}

	pub(crate) fn serialize(self, write: &mut impl io::Write) -> FResult<()> {
		self.value().serialize(write)?;
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
		Self::try_from(i32::deserialize(read)?)
			.map_err(|_| FendError::DeserializationError("year is out of range"))
	}
}

pub(crate) struct InvalidYearError;

impl convert::TryFrom<i32> for Year {
	type Error = InvalidYearError;

	fn try_from(year: i32) -> Result<Self, Self::Error> {
		if year == 0 {
			Err(InvalidYearError)
		} else {
			Ok(Self(year))
		}
	}
}

impl fmt::Debug for Year {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self, f)
	}
}

impl fmt::Display for Year {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.value() < 0 {
			// cast to bigger int to fix this for Self::MIN
			write!(f, "{} BC", -i64::from(self.0))
		} else {
			write!(f, "{}", self.0)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[should_panic(expected = "year 0 is invalid")]
	fn year_0() {
		Year::new(0);
	}

	#[test]
	fn negative_year_string() {
		assert_eq!(Year::new(-823).to_string(), "823 BC");
		assert_eq!(Year::MIN.to_string(), "2147483648 BC");
	}
}
