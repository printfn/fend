use crate::date::Year;
use std::fmt;

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
    pub(crate) fn number_of_days(self, year: Year) -> u8 {
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

    pub(crate) fn next(self) -> Self {
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

    pub(crate) fn prev(self) -> Self {
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

    fn as_str(self) -> &'static str {
        match self {
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
        }
    }
}

impl fmt::Debug for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
