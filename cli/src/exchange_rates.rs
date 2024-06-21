use crate::config::{self, ExchangeRateSource};
use crate::file_paths;
use crate::Error;
use std::{error, fmt, fs, io::Write, time};

const MAX_AGE: u64 = 86400 * 3;

fn get_current_timestamp() -> Result<u64, Error> {
	Ok(time::SystemTime::now()
		.duration_since(time::SystemTime::UNIX_EPOCH)?
		.as_secs())
}

fn get_cache_filename(source: config::ExchangeRateSource) -> Result<&'static str, Error> {
	Ok(match source {
		ExchangeRateSource::Disabled => return Err(ExchangeRateSourceDisabledError.into()),
		ExchangeRateSource::EuropeanUnion => "eurofxref-daily.xml.cache",
		ExchangeRateSource::UnitedNations => "xsql2XML.php.cache",
	})
}

fn load_cached_data(source: config::ExchangeRateSource) -> Result<String, Error> {
	let mut cache_file = file_paths::get_cache_dir(file_paths::DirMode::DontCreate)?;
	cache_file.push(get_cache_filename(source)?);
	let cache_contents = fs::read_to_string(cache_file)?;
	let (timestamp, cache_xml) =
		cache_contents.split_at(cache_contents.find(';').ok_or("invalid cache file")?);
	let timestamp = timestamp.parse::<u64>()?;
	let current_timestamp = get_current_timestamp()?;
	let age = current_timestamp
		.checked_sub(timestamp)
		.ok_or("invalid cache timestamp")?;
	if age > MAX_AGE {
		return Err("cache expired".into());
	}
	Ok(cache_xml.to_string())
}

fn store_cached_data(source: config::ExchangeRateSource, xml: &str) -> Result<(), Error> {
	let mut cache_file = file_paths::get_cache_dir(file_paths::DirMode::Create)?;
	cache_file.push(get_cache_filename(source)?);
	let mut file = fs::File::create(cache_file)?;
	write!(file, "{};{xml}", get_current_timestamp()?)?;
	Ok(())
}

#[cfg(any(feature = "native-tls", feature = "rustls"))]
fn http_get(url: &str) -> Result<String, Error> {
	let response = minreq::get(url).send()?;
	Ok(response.as_str()?.to_string())
}

#[cfg(not(any(feature = "native-tls", feature = "rustls")))]
fn http_get(_url: &str) -> Result<String, Error> {
	Err("internet access has been disabled in this build of fend".into())
}

fn load_exchange_rate_xml(source: config::ExchangeRateSource) -> Result<(String, bool), Error> {
	match load_cached_data(source) {
		Ok(xml) => return Ok((xml, true)),
		Err(_e) => {
			// failed to load cached data
		}
	}
	let url = match source {
		ExchangeRateSource::Disabled => return Err(ExchangeRateSourceDisabledError.into()),
		ExchangeRateSource::EuropeanUnion => {
			"https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml"
		}
		ExchangeRateSource::UnitedNations => {
			"https://treasury.un.org/operationalrates/xsql2XML.php"
		}
	};
	let xml = http_get(url)?;
	Ok((xml, false))
}

fn parse_exchange_rates(
	source: config::ExchangeRateSource,
	exchange_rates: &str,
) -> Result<Vec<(String, f64)>, Error> {
	match source {
		ExchangeRateSource::Disabled => Err(ExchangeRateSourceDisabledError.into()),
		ExchangeRateSource::EuropeanUnion => parse_exchange_rates_eu(exchange_rates),
		ExchangeRateSource::UnitedNations => parse_exchange_rates_un(exchange_rates),
	}
}

fn parse_exchange_rates_eu(exchange_rates: &str) -> Result<Vec<(String, f64)>, Error> {
	let err = "failed to load exchange rates";
	let mut result = vec![("EUR".to_string(), 1.0)];
	for l in exchange_rates.lines() {
		let l = l.trim();
		if !l.starts_with("<Cube currency=") {
			continue;
		}
		let l = l.strip_prefix("<Cube currency='").ok_or(err)?;
		let (currency, l) = l.split_at(3);
		let l = l.trim_start_matches("' rate='");
		let exchange_rate_eur = l.split_at(l.find('\'').ok_or(err)?).0;
		let exchange_rate_eur = exchange_rate_eur.parse::<f64>()?;
		if !exchange_rate_eur.is_normal() {
			return Err(err.into());
		}
		result.push((currency.to_string(), exchange_rate_eur));
	}
	if result.len() < 10 {
		return Err(err.into());
	}
	Ok(result)
}

fn parse_exchange_rates_un(exchange_rates: &str) -> Result<Vec<(String, f64)>, Error> {
	const F_CURR_LEN: usize = "<f_curr_code>".len();
	const RATE_LEN: usize = "<rate>".len();

	let err = "failed to load exchange rates";
	let mut result = vec![("USD".to_string(), 1.0)];
	let mut exchange_rates = &exchange_rates[exchange_rates
		.find("<UN_OPERATIONAL_RATES>")
		.ok_or("op rates")?..];

	while !exchange_rates.is_empty() {
		let start = match exchange_rates.find("<f_curr_code>") {
			Some(s) => s,
			None if exchange_rates
				== "\r\n\t</UN_OPERATIONAL_RATES>\r\n</UN_OPERATIONAL_RATES_DATASET>" =>
			{
				break
			}
			None => return Err(err.into()),
		};
		exchange_rates = &exchange_rates[start + F_CURR_LEN..];
		let end = exchange_rates.find("</f_curr_code>").ok_or(err)?;
		let currency = &exchange_rates[..end];
		exchange_rates = &exchange_rates[end + F_CURR_LEN + 1..];

		let start = exchange_rates.find("<rate>").ok_or(err)?;
		exchange_rates = &exchange_rates[start + RATE_LEN..];
		let end = exchange_rates.find("</rate>").ok_or(err)?;
		let exchange_rate_usd = &exchange_rates[..end];
		let exchange_rate_usd = exchange_rate_usd.parse::<f64>()?;
		exchange_rates = &exchange_rates[end + RATE_LEN + 1..];

		if !exchange_rate_usd.is_normal() {
			return Err(err.into());
		}

		result.push((currency.to_string(), exchange_rate_usd));
	}

	Ok(result)
}

fn get_exchange_rates(source: config::ExchangeRateSource) -> Result<Vec<(String, f64)>, Error> {
	let (xml, cached) = load_exchange_rate_xml(source)?;
	let parsed_data = parse_exchange_rates(source, &xml)?;
	if !cached {
		store_cached_data(source, &xml)?;
	}
	Ok(parsed_data)
}

#[derive(Debug, Clone)]
struct UnknownExchangeRate(String);

impl fmt::Display for UnknownExchangeRate {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "currency exchange rate for {} is unknown", self.0)
	}
}

impl error::Error for UnknownExchangeRate {}

#[derive(Copy, Clone, Debug)]
pub struct InternetAccessDisabledError;
impl fmt::Display for InternetAccessDisabledError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "internet access is disabled by fend configuration")
	}
}

impl error::Error for InternetAccessDisabledError {}

#[derive(Copy, Clone, Debug)]
pub struct ExchangeRateSourceDisabledError;
impl fmt::Display for ExchangeRateSourceDisabledError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "exchange rate source is set to `disabled`")
	}
}

impl error::Error for ExchangeRateSourceDisabledError {}

pub struct ExchangeRateHandler {
	pub enable_internet_access: bool,
	pub source: ExchangeRateSource,
}

impl fend_core::ExchangeRateFn for ExchangeRateHandler {
	fn relative_to_base_currency(
		&self,
		currency: &str,
	) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>> {
		if !self.enable_internet_access {
			return Err(InternetAccessDisabledError.into());
		}
		let exchange_rates = get_exchange_rates(self.source)?;
		for (c, rate) in exchange_rates {
			if currency == c {
				return Ok(rate);
			}
		}
		Err(UnknownExchangeRate(currency.to_string()).into())
	}
}
