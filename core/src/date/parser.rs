use crate::date::{Date, Day, Month, Year};
use std::{convert, error, fmt};

#[derive(Debug)]
pub(crate) struct ParseDateError<'a>(&'a str);

impl fmt::Display for ParseDateError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to convert '{}' to a date", self.0)
    }
}

impl error::Error for ParseDateError<'_> {}

fn parse_char(s: &str) -> Result<(char, &str), ()> {
    let ch = s.chars().next().ok_or(())?;
    let (_, b) = s.split_at(ch.len_utf8());
    Ok((ch, b))
}

fn parse_specific_char(s: &str, c: char) -> Result<&str, ()> {
    let (ch, s) = parse_char(s)?;
    if ch == c {
        Ok(s)
    } else {
        Err(())
    }
}

fn parse_digit(s: &str) -> Result<(i32, &str), ()> {
    let (ch, b) = parse_char(s)?;
    let digit = ch.to_digit(10).ok_or(())?;
    let digit_i32: i32 = convert::TryInto::try_into(digit).map_err(|_| ())?;
    Ok((digit_i32, b))
}

fn parse_num(s: &str, leading_zeroes: bool) -> Result<(i32, &str), ()> {
    let (mut num, mut s) = parse_digit(s)?;
    if !leading_zeroes && num == 0 {
        return Err(());
    }
    while let Ok((digit, remaining)) = parse_digit(s) {
        num = num.checked_mul(10).ok_or(())?;
        num = num.checked_add(digit).ok_or(())?;
        s = remaining;
    }
    Ok((num, s))
}

fn parse_yyyymmdd(s: &str) -> Result<(Date, &str), ()> {
    let (year, s) = parse_num(s, false)?;
    let s = parse_specific_char(s, '-')?;
    if year < 1000 {
        return Err(());
    }
    let year = Year::new(year);
    let (month, s) = parse_num(s, true)?;
    let s = parse_specific_char(s, '-')?;
    let month: Month = convert::TryInto::try_into(month).map_err(|_| ())?;
    let (day, s) = parse_num(s, true)?;
    if day < 1 || day > i32::from(month.number_of_days(year)) {
        return Err(());
    }
    let day: u8 = convert::TryInto::try_into(day).map_err(|_| ())?;
    let day = Day::new(day);
    Ok((Date { year, month, day }, s))
}

pub(crate) fn parse_date(s: &str) -> Result<Date, ParseDateError> {
    let trimmed = s.trim();
    if let Ok((date, remaining)) = parse_yyyymmdd(trimmed) {
        if remaining.is_empty() {
            Ok(date)
        } else {
            Err(ParseDateError(s))
        }
    } else {
        Err(ParseDateError(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date_tests() {
        parse_date("2021-04-14").unwrap();
        parse_date("2021-4-14").unwrap();
        parse_date("9999-12-31").unwrap();
        parse_date("1000-01-01").unwrap();
        parse_date("1000-1-1").unwrap();
        parse_date("10000-1-1").unwrap();
        parse_date("214748363-1-1").unwrap();
        parse_date("2147483647-1-1").unwrap();

        parse_date("999-01-01").unwrap_err();
        parse_date("2021-02-29").unwrap_err();
        parse_date("2100-02-29").unwrap_err();
        parse_date("7453-13-01").unwrap_err();
        parse_date("2147483648-1-1").unwrap_err();
    }
}
