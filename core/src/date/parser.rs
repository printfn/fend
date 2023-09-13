use crate::{date::Date, error::FendError};

fn parse_yyyymmdd(s: &str) -> Result<Date, ()> {
	let date = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").or(Err(()))?;
	Ok(Date(
		date.and_hms_opt(0, 0, 0)
			.ok_or(())?
			.and_local_timezone(chrono::Local)
			.single()
			.ok_or(())?,
	))
}

pub(crate) fn parse_date(s: &str) -> Result<Date, FendError> {
	let trimmed = s.trim();
	if let Ok(date) = parse_yyyymmdd(trimmed) {
		return Ok(date);
	}
	Err(FendError::ParseDateError(s.to_string()))
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

		parse_date("10000-1-1").unwrap_err();
		parse_date("214748363-1-1").unwrap_err();
		parse_date("2147483647-1-1").unwrap_err();

		parse_date("999-01-01").unwrap();
		parse_date("2021-02-29").unwrap_err();
		parse_date("2100-02-29").unwrap_err();
		parse_date("7453-13-01").unwrap_err();
		parse_date("2147483648-1-1").unwrap_err();
	}
}
