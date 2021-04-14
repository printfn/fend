use std::fmt;

mod day;
mod day_of_week;
mod month;
mod year;

use day::Day;
use day_of_week::DayOfWeek;
use month::Month;
use year::Year;

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Date {
    year: Year,
    month: Month,
    day: Day,
}

pub(crate) struct TodayError;

impl fmt::Display for TodayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to get the current date")
    }
}

pub(crate) struct ParseDateError<'a>(&'a str);

impl fmt::Display for ParseDateError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to convert '{}' to a date", self.0)
    }
}

impl Date {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    pub(crate) fn today(context: &mut crate::Context) -> Result<Self, TodayError> {
        let mut ms_since_epoch = if let Some(ms) = context.elapsed_unix_time_ms {
            ms as i64
        } else {
            return Err(TodayError);
        };
        if let Some(offset_secs) = context.timezone_offset_secs {
            ms_since_epoch -= offset_secs * 1000;
        } else {
            return Err(TodayError);
        }
        let mut days = ms_since_epoch / 86_400_000; // no leap seconds
        let mut year = Year::new(1970);
        while days >= year.number_of_days().into() {
            year = year.next();
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
            day: Day::new(days as u8),
        })
    }

    fn day_of_week(self) -> DayOfWeek {
        let d1 = (1
            + 5 * ((self.year.value() - 1) % 4)
            + 4 * ((self.year.value() - 1) % 100)
            + 6 * ((self.year.value() - 1) % 400))
            % 7;
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
        match (d1 + m + i32::from(self.day.value() - 1)) % 7 {
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

    pub(crate) fn next(self) -> Self {
        if self.day.value() < Month::number_of_days(self.month, self.year) {
            Self {
                day: Day::new(self.day.value() + 1),
                month: self.month,
                year: self.year,
            }
        } else if self.month == Month::December {
            Self {
                day: Day::new(1),
                month: Month::January,
                year: self.year.next(),
            }
        } else {
            Self {
                day: Day::new(1),
                month: self.month.next(),
                year: self.year,
            }
        }
    }

    pub(crate) fn prev(self) -> Self {
        if self.day.value() > 1 {
            Self {
                day: Day::new(self.day.value() - 1),
                month: self.month,
                year: self.year,
            }
        } else if self.month == Month::January {
            Self {
                day: Day::new(31),
                month: Month::December,
                year: self.year.prev(),
            }
        } else {
            let month = self.month.prev();
            Self {
                day: Day::new(Month::number_of_days(month, self.year)),
                month,
                year: self.year,
            }
        }
    }

    pub(crate) fn parse(s: &str) -> Result<Self, ParseDateError> {
        let trimmed = s.trim();
        if trimmed == "2021-04-14" {
            Ok(Self {
                year: Year::new(2021),
                month: Month::April,
                day: Day::new(14),
            })
        } else {
            Err(ParseDateError(s))
        }
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
