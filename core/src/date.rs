use chrono::Datelike;
use num_traits::FromPrimitive;
use std::{fmt, io};

mod parser;

use crate::{
	error::FendError,
	ident::Ident,
	serialize::{
		deserialize_i32, deserialize_u32, deserialize_u8, serialize_i32, serialize_u32,
		serialize_u8,
	},
	value::Value,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Date(pub(crate) chrono::DateTime<chrono::Local>);

impl Date {
	pub(crate) fn today() -> Self {
		Self(chrono::Local::now())
	}

	fn day_of_week(self) -> DayOfWeek {
		DayOfWeek(self.0.weekday())
	}

	pub(crate) fn next(self) -> Self {
		Self(self.0 + chrono::Duration::days(1))
	}

	pub(crate) fn prev(self) -> Self {
		Self(self.0 - chrono::Duration::days(1))
	}

	pub(crate) fn parse(s: &str) -> Result<Self, FendError> {
		parser::parse_date(s)
	}

	fn day(self) -> u32 {
		self.0.day()
	}

	fn month(self) -> Month {
		let month = chrono::Month::try_from(u8::try_from(self.0.month()).unwrap()).unwrap();
		Month(month)
	}

	fn year(self) -> i32 {
		self.0.year()
	}

	pub(crate) fn serialize(self, write: &mut impl io::Write) -> Result<(), FendError> {
		serialize_i32(self.year(), write)?;
		serialize_u32(self.month().number_from_month(), write)?;
		serialize_u32(self.day(), write)?;
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
		let year = deserialize_i32(read)?;
		let month = deserialize_u32(read)?;
		let day = deserialize_u32(read)?;
		let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
			.ok_or(FendError::DeserializationError)?
			.and_hms_opt(0, 0, 0)
			.ok_or(FendError::DeserializationError)?
			.and_local_timezone(chrono::Local)
			.unwrap();
		Ok(Self(date))
	}

	pub(crate) fn get_object_member(self, key: &Ident) -> Result<crate::value::Value, FendError> {
		Ok(match key.as_str() {
			"month" => Value::Month(self.month()),
			"day_of_week" => Value::DayOfWeek(self.day_of_week()),
			_ => return Err(FendError::CouldNotFindKey(key.to_string())),
		})
	}

	pub(crate) fn add(self, rhs: Value) -> Result<Value, FendError> {
		let rhs = rhs.expect_num()?;
		let int = &crate::interrupt::Never::default();
		if rhs.unit_equal_to("day") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 + chrono::Duration::days(num as i64);
			Ok(Value::Date(Self(result)))
		} else if rhs.unit_equal_to("week") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 + chrono::Duration::weeks(num as i64);
			Ok(Value::Date(Self(result)))
		} else if rhs.unit_equal_to("month") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 + chrono::Months::new(num as u32);
			Ok(Value::Date(Self(result)))
		} else if rhs.unit_equal_to("year") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 + chrono::Months::new(12 * num as u32);
			Ok(Value::Date(Self(result)))
		} else {
			Err(FendError::ExpectedANumber)
		}
	}

	pub(crate) fn sub(self, rhs: Value) -> Result<Value, FendError> {
		let int = &crate::interrupt::Never::default();
		let rhs = rhs.expect_num()?;
		if rhs.unit_equal_to("day") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 - chrono::Duration::days(num as i64);
			Ok(Value::Date(Self(result)))
		} else if rhs.unit_equal_to("week") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 - chrono::Duration::weeks(num as i64);
			Ok(Value::Date(Self(result)))
		} else if rhs.unit_equal_to("month") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 - chrono::Months::new(num as u32);
			Ok(Value::Date(Self(result)))
		} else if rhs.unit_equal_to("year") {
			let num = rhs.try_as_usize_unit(int)?;
			let result = self.0 - chrono::Months::new(12 * num as u32);
			Ok(Value::Date(Self(result)))
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
			self.day(),
			self.month(),
			self.year()
		)
	}
}

impl fmt::Display for Date {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}, {} {}",
			self.day_of_week(),
			self.day(),
			self.0.format("%B %Y")
		)
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Month(chrono::Month);

impl Month {
	pub(crate) fn serialize(self, write: &mut impl io::Write) -> Result<(), FendError> {
		let month = self.0.number_from_month() as u8;
		serialize_u8(month, write)?;
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
		let number = deserialize_u8(read).or(Err(FendError::DeserializationError))?;
		let month = chrono::Month::from_u8(number).ok_or(FendError::DeserializationError)?;
		Ok(Self(month))
	}

	fn number_from_month(self) -> u32 {
		self.0.number_from_month()
	}
}

impl fmt::Display for Month {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0.name())
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct DayOfWeek(chrono::Weekday);

impl DayOfWeek {
	pub(crate) fn serialize(self, write: &mut impl io::Write) -> Result<(), FendError> {
		let month = self.0.number_from_sunday() as u8;
		serialize_u8(month, write)?;
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
		let number = deserialize_u8(read).or(Err(FendError::DeserializationError))?;
		let weekday = chrono::Weekday::from_u8(number).ok_or(FendError::DeserializationError)?;
		Ok(Self(weekday))
	}
}

impl fmt::Display for DayOfWeek {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self.0 {
			chrono::Weekday::Mon => "Monday",
			chrono::Weekday::Tue => "Tuesday",
			chrono::Weekday::Wed => "Wednesday",
			chrono::Weekday::Thu => "Thursday",
			chrono::Weekday::Fri => "Friday",
			chrono::Weekday::Sat => "Saturday",
			chrono::Weekday::Sun => "Sunday",
		};
		write!(f, "{name}")
	}
}
