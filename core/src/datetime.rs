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
            let current_time = std::time::SystemTime::now();
            let epoch = std::time::SystemTime::UNIX_EPOCH;
            let time_since_epoch = current_time.duration_since(epoch).map_err(|_| TodayError)?;
            time_since_epoch.as_millis() as i64
        };
        if let Some(offset_secs) = context.timezone_offset_secs {
            ms_since_epoch -= offset_secs * 1000;
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
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.day, self.month, self.year)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.day, self.month, self.year)
    }
}
