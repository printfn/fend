use std::{convert::TryFrom, fmt};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Day(u8);

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    fn number_of_days(self, year: Year) -> u8 {
        match self {
            Self::February => {
                if year.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            Self::April | Self::June | Self::September | Self::November => 30,
            _ => 31,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::January => Self::February,
            Self::February => Self::March,
            Self::March => Self::April,
            Self::April => Self::May,
            Self::May => Self::June,
            Self::June => Self::July,
            Self::July => Self::August,
            Self::August => Self::September,
            Self::September => Self::October,
            Self::October => Self::November,
            Self::November => Self::December,
            Self::December => Self::January,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::January => Self::December,
            Self::February => Self::January,
            Self::March => Self::February,
            Self::April => Self::March,
            Self::May => Self::April,
            Self::June => Self::May,
            Self::July => Self::June,
            Self::August => Self::July,
            Self::September => Self::August,
            Self::October => Self::September,
            Self::November => Self::October,
            Self::December => Self::November,
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::January => "January",
            Self::February => "February",
            Self::March => "March",
            Self::April => "April",
            Self::May => "May",
            Self::June => "June",
            Self::July => "July",
            Self::August => "August",
            Self::September => "September",
            Self::October => "October",
            Self::November => "November",
            Self::December => "December",
        };
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Year(i32);

impl Year {
    fn is_leap_year(self) -> bool {
        if self.0 % 400 == 0 {
            true
        } else if self.0 % 100 == 0 {
            false
        } else {
            self.0 % 4 == 0
        }
    }

    fn number_of_days(self) -> u16 {
        if self.is_leap_year() {
            366
        } else {
            365
        }
    }
}

pub(crate) struct InvalidYearError;

impl TryFrom<i32> for Year {
    type Error = InvalidYearError;

    fn try_from(year: i32) -> Result<Self, Self::Error> {
        if year == 0 {
            Err(InvalidYearError)
        } else {
            Ok(Self(year))
        }
    }
}

impl fmt::Display for Year {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl fmt::Debug for DayOfWeek {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Sunday => "Sunday",
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
        let mut year = Year(1970);
        while days >= year.number_of_days().into() {
            year.0 += 1;
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
            day: Day(days as u8 + 1),
        })
    }

    fn day_of_week(self) -> DayOfWeek {
        let d1 = (1
            + 5 * ((self.year.0 - 1) % 4)
            + 4 * ((self.year.0 - 1) % 100)
            + 6 * ((self.year.0 - 1) % 400))
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
        match (d1 + m + i32::from(self.day.0 - 1)) % 7 {
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
        if self.day.0 < Month::number_of_days(self.month, self.year) {
            Self {
                day: Day(self.day.0 + 1),
                month: self.month,
                year: self.year,
            }
        } else if self.month == Month::December {
            Self {
                day: Day(1),
                month: Month::January,
                year: Year(self.year.0 + 1),
            }
        } else {
            Self {
                day: Day(1),
                month: self.month.next(),
                year: self.year,
            }
        }
    }

    pub(crate) fn prev(self) -> Self {
        if self.day.0 > 1 {
            Self {
                day: Day(self.day.0 - 1),
                month: self.month,
                year: self.year,
            }
        } else if self.month == Month::January {
            Self {
                day: Day(31),
                month: Month::December,
                year: Year(self.year.0 - 1),
            }
        } else {
            let month = self.month.prev();
            Self {
                day: Day(Month::number_of_days(month, self.year)),
                month,
                year: self.year,
            }
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
