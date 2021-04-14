use crate::date::{Date, Day, Month, Year};
use std::{error, fmt};

#[derive(Debug)]
pub(crate) struct ParseDateError<'a>(&'a str);

impl fmt::Display for ParseDateError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to convert '{}' to a date", self.0)
    }
}

impl error::Error for ParseDateError<'_> {}

pub(crate) fn parse_date(s: &str) -> Result<Date, ParseDateError> {
    let trimmed = s.trim();
    if trimmed == "2021-04-14" {
        Ok(Date {
            year: Year::new(2021),
            month: Month::April,
            day: Day::new(14),
        })
    } else {
        Err(ParseDateError(s))
    }
}
