use std::{fmt, io};

mod day;
mod day_of_week;
mod month;
mod parser;
mod year;

use day::Day;
pub(crate) use day_of_week::DayOfWeek;
pub(crate) use month::Month;
use year::Year;

use crate::interrupt::test_int;
use crate::{Interrupt, error::FendError, ident::Ident, result::FResult, value::Value};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Date {
	year: Year,
	month: Month,
	day: Day,
}

impl Date {
	pub(crate) fn today(context: &crate::Context) -> FResult<Self> {
		let Some(current_time_info) = &context.current_time else {
			return Err(FendError::UnableToGetCurrentDate);
		};
		let mut ms_since_epoch: i64 = current_time_info.elapsed_unix_time_ms.try_into().unwrap();
		ms_since_epoch -= current_time_info.timezone_offset_secs * 1000;
		let mut days = ms_since_epoch / 86_400_000; // no leap seconds
		let mut year = Year::new(1970);
		while days >= year.number_of_days().into() {
			year = year.next()?;
			days -= i64::from(year.number_of_days());
		}
		let mut month = Month::January;
		while days >= month.number_of_days(year).into() {
			month = month.next();
			days -= i64::from(month.number_of_days(year));
		}
		Ok(Self {
			year,
			month,
			day: Day::new(days.try_into().unwrap()),
		})
	}

	fn day_of_week(self) -> DayOfWeek {
		let d1 = ((1
			+ 5 * ((i64::from(self.year.value()) - 1) % 4)
			+ 4 * ((i64::from(self.year.value()) - 1) % 100)
			+ 6 * ((i64::from(self.year.value()) - 1) % 400))
			% 7) as i32;
		let ms = match self.month {
			Month::January => (0, 0),
			Month::February => (3, 3),
			Month::March | Month::November => (3, 4),
			Month::April | Month::July => (6, 0),
			Month::May => (1, 2),
			Month::June => (4, 5),
			Month::August => (2, 3),
			Month::September | Month::December => (5, 6),
			Month::October => (0, 1),
		};
		let m = if self.year.is_leap_year() { ms.1 } else { ms.0 };
		match ((d1 + m + i32::from(self.day.value() - 1)) % 7 + 7) % 7 {
			0 => DayOfWeek::Sunday,
			1 => DayOfWeek::Monday,
			2 => DayOfWeek::Tuesday,
			3 => DayOfWeek::Wednesday,
			4 => DayOfWeek::Thursday,
			5 => DayOfWeek::Friday,
			6 => DayOfWeek::Saturday,
			_ => unreachable!(),
		}
	}

	pub(crate) fn next(self) -> FResult<Self> {
		if self.day.value() < Month::number_of_days(self.month, self.year) {
			Ok(Self {
				day: Day::new(self.day.value() + 1),
				month: self.month,
				year: self.year,
			})
		} else if self.month == Month::December {
			Ok(Self {
				day: Day::new(1),
				month: Month::January,
				year: self.year.next()?,
			})
		} else {
			Ok(Self {
				day: Day::new(1),
				month: self.month.next(),
				year: self.year,
			})
		}
	}

	pub(crate) fn prev(self) -> FResult<Self> {
		if self.day.value() > 1 {
			Ok(Self {
				day: Day::new(self.day.value() - 1),
				month: self.month,
				year: self.year,
			})
		} else if self.month == Month::January {
			Ok(Self {
				day: Day::new(31),
				month: Month::December,
				year: self.year.prev()?,
			})
		} else {
			let month = self.month.prev();
			Ok(Self {
				day: Day::new(Month::number_of_days(month, self.year)),
				month,
				year: self.year,
			})
		}
	}

	pub(crate) fn diff_months(self, mut months: i64) -> FResult<Self> {
		let mut result = self;
		if months >= 12 {
			let years = months / 12;
			months %= 12;
			result.year = result.year.add(years)?;
		} else if months <= -12 {
			let years = (months / 12).unsigned_abs();
			months %= 12;
			result.year = result.year.sub(years)?;
		}
		while months > 0 {
			if result.month == Month::December {
				result.month = Month::January;
				result.year = result.year.next()?;
			} else {
				result.month = result.month.next();
			}
			months -= 1;
		}
		while months < 0 {
			if result.month == Month::January {
				result.month = Month::December;
				result.year = result.year.prev()?;
			} else {
				result.month = result.month.prev();
			}
			months += 1;
		}
		if result.day.value() > Month::number_of_days(result.month, result.year) {
			let mut before = result;
			before.day = Day::new(Month::number_of_days(before.month, before.year));
			let mut after = result;
			if after.month == Month::December {
				after.month = Month::January;
				after.year = after.year.next()?;
			} else {
				after.month = after.month.next();
			}
			after.day = Day::new(1);
			return Err(FendError::NonExistentDate {
				year: result.year.value(),
				month: result.month,
				expected_day: result.day.value(),
				before,
				after,
			});
		}
		Ok(result)
	}

	pub(crate) fn parse(s: &str) -> FResult<Self> {
		parser::parse_date(s)
	}

	pub(crate) fn serialize(self, write: &mut impl io::Write) -> FResult<()> {
		self.year.serialize(write)?;
		self.month.serialize(write)?;
		self.day.serialize(write)?;
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
		Ok(Self {
			year: Year::deserialize(read)?,
			month: Month::deserialize(read)?,
			day: Day::deserialize(read)?,
		})
	}

	pub(crate) fn get_object_member(self, key: &Ident) -> FResult<crate::value::Value> {
		Ok(match key.as_str() {
			"month" => Value::Month(self.month),
			"day_of_week" => Value::DayOfWeek(self.day_of_week()),
			_ => return Err(FendError::CouldNotFindKey(key.to_string())),
		})
	}

	fn add_days<I: Interrupt>(self, mut num_days: usize, int: &I) -> FResult<Self> {
		let mut result = self;

		// make sure to be before 29th February to make skipping years work
		while result.month != Month::January && num_days > 0 {
			result = result.next()?;
			num_days -= 1;
		}

		while let days_in_year = result.year.number_of_days().into()
			&& num_days >= days_in_year
		{
			num_days -= days_in_year;

			result.year = result.year.next()?;

			test_int(int)?;
		}

		for _ in 0..num_days {
			test_int(int)?;
			result = result.next()?;
		}

		Ok(result)
	}

	pub(crate) fn add<I: Interrupt>(self, rhs: Value, int: &I) -> FResult<Value> {
		let rhs = rhs.expect_num()?;
		if rhs.unit_equal_to("day", int)? {
			let num_days = rhs.try_as_usize_unit(int)?;

			let result = self.add_days(num_days, int)?;

			Ok(Value::Date(result))
		} else if rhs.unit_equal_to("week", int)? {
			let num_weeks = rhs.try_as_usize_unit(int)?;

			let num_days = num_weeks.checked_mul(7).ok_or(FendError::ValueTooLarge)?;

			let result = self.add_days(num_days, int)?;

			debug_assert_eq!(self.day_of_week(), result.day_of_week());

			Ok(Value::Date(result))
		} else if rhs.unit_equal_to("month", int)? {
			let num_months = rhs.try_as_usize_unit(int)?;
			let result =
				self.diff_months(i64::try_from(num_months).map_err(|_| FendError::ValueTooLarge)?)?;
			Ok(Value::Date(result))
		} else if rhs.unit_equal_to("year", int)? {
			let num_years = rhs.try_as_usize_unit(int)?;
			let num_months = num_years.checked_mul(12).ok_or(FendError::ValueTooLarge)?;
			let result =
				self.diff_months(i64::try_from(num_months).map_err(|_| FendError::ValueTooLarge)?)?;
			Ok(Value::Date(result))
		} else {
			Err(FendError::ExpectedANumber)
		}
	}

	fn sub_days<I: Interrupt>(self, mut num_days: usize, int: &I) -> FResult<Self> {
		let mut result = self;

		// make sure to be after 29th February to make skipping years work
		while result.month.as_u8() < Month::March.as_u8() && num_days > 0 {
			result = result.prev()?;
			num_days -= 1;
		}

		while let days_in_year = result.year.number_of_days().into()
			&& num_days >= days_in_year
		{
			num_days -= days_in_year;

			result.year = result.year.prev()?;

			test_int(int)?;
		}

		for _ in 0..num_days {
			result = result.prev()?;
			test_int(int)?;
		}

		Ok(result)
	}

	pub(crate) fn sub<I: Interrupt>(self, rhs: Value, int: &I) -> FResult<Value> {
		let rhs = rhs.expect_num()?;

		if rhs.unit_equal_to("day", int)? {
			let num_days = rhs.try_as_usize_unit(int)?;

			let result = self.sub_days(num_days, int)?;

			Ok(Value::Date(result))
		} else if rhs.unit_equal_to("week", int)? {
			let num_weeks = rhs.try_as_usize_unit(int)?;

			let num_days = num_weeks.checked_mul(7).ok_or(FendError::ValueTooLarge)?;

			let result = self.sub_days(num_days, int)?;

			Ok(Value::Date(result))
		} else if rhs.unit_equal_to("month", int)? {
			let num_months = rhs.try_as_usize_unit(int)?;
			let result = self
				.diff_months(-i64::try_from(num_months).map_err(|_| FendError::ValueTooLarge)?)?;
			Ok(Value::Date(result))
		} else if rhs.unit_equal_to("year", int)? {
			let num_years = rhs.try_as_usize_unit(int)?;
			let num_months = num_years.checked_mul(12).ok_or(FendError::ValueTooLarge)?;
			let result = self
				.diff_months(-i64::try_from(num_months).map_err(|_| FendError::ValueTooLarge)?)?;
			Ok(Value::Date(result))
		} else {
			Err(FendError::ExpectedANumber)
		}
	}
}

impl fmt::Debug for Date {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}, {} {} {}",
			self.day_of_week(),
			self.day,
			self.month,
			self.year
		)
	}
}

impl fmt::Display for Date {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{self:?}")
	}
}
