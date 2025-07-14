use std::{error, fmt, fs, io::Write, time};

use tokio::runtime::Handle;

use crate::Error;
use crate::config::{self, ExchangeRateSetting, ExchangeRateSource};
use crate::file_paths;

fn get_current_timestamp() -> Result<u64, Error> {
	Ok(time::SystemTime::now()
		.duration_since(time::SystemTime::UNIX_EPOCH)?
		.as_secs())
}

fn get_cache_filename(source: config::ExchangeRateSource) -> &'static str {
	match source {
		ExchangeRateSource::EuropeanUnion => "eurofxref-daily.xml.cache",
		ExchangeRateSource::UnitedNations => "xsql2XML.php.cache",
	}
}

struct RawData {
	exchange_rate_data: String,
	source: ExchangeRateSource,
	cached: bool,
}

fn load_cached_data(source: config::ExchangeRateSource, max_age: u64) -> Result<String, Error> {
	let mut cache_file = file_paths::get_cache_dir(file_paths::DirMode::DontCreate)?;
	cache_file.push(get_cache_filename(source));
	let cache_contents = fs::read_to_string(cache_file)?;
	let (timestamp, cache_xml) =
		cache_contents.split_at(cache_contents.find(';').ok_or("invalid cache file")?);
	let timestamp = timestamp.parse::<u64>()?;
	let current_timestamp = get_current_timestamp()?;
	let age = current_timestamp
		.checked_sub(timestamp)
		.ok_or("invalid cache timestamp")?;
	if age > max_age {
		return Err("cache expired".into());
	}
	Ok(cache_xml.to_string())
}

fn store_cached_data(source: config::ExchangeRateSource, xml: &str) -> Result<(), Error> {
	let mut cache_file = file_paths::get_cache_dir(file_paths::DirMode::Create)?;
	cache_file.push(get_cache_filename(source));
	let mut file = fs::File::create(cache_file)?;
	write!(file, "{};{xml}", get_current_timestamp()?)?;
	Ok(())
}

#[cfg(any(feature = "native-tls", feature = "rustls"))]
async fn http_get(url: &str) -> Result<String, Error> {
	let response = reqwest::get(url).await?.text().await?;
	Ok(response)
}

#[cfg(not(any(feature = "native-tls", feature = "rustls")))]
async fn http_get(_url: &str) -> Result<String, Error> {
	Err("internet access has been disabled in this build of fend".into())
}

async fn load_exchange_rate_xml(source: config::ExchangeRateSource) -> Result<RawData, Error> {
	let url = match source {
		ExchangeRateSource::EuropeanUnion => {
			"https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml"
		}
		ExchangeRateSource::UnitedNations => {
			"https://treasury.un.org/operationalrates/xsql2XML.php"
		}
	};
	Ok(RawData {
		exchange_rate_data: http_get(url).await?,
		source,
		cached: false,
	})
}

#[derive(Debug)]
struct MultiError(&'static str, Vec<Error>);
impl fmt::Display for MultiError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)?;
		if !self.1.is_empty() {
			write!(f, ": [")?;
			for (i, e) in self.1.iter().enumerate() {
				if i != 0 {
					write!(f, ", ")?;
				}
				write!(f, "{e}")?;
			}
			write!(f, "]")?;
		}
		Ok(())
	}
}
impl error::Error for MultiError {}

async fn load_exchange_rates_auto(
	sources: &[config::ExchangeRateSource],
) -> Result<RawData, Error> {
	let use_delay = sources.len() > 1;
	let mut tasks = tokio::task::JoinSet::new();
	for &source in sources {
		tasks.spawn(async move {
			if use_delay {
				tokio::time::sleep(source.get_delay()).await;
			}
			load_exchange_rate_xml(source).await
		});
	}
	let mut errors = vec![];
	while let Some(result) = tasks.join_next().await {
		match result {
			Ok(Ok(res)) => return Ok(res),
			Ok(Err(e)) => errors.push(e),
			_ => (),
		}
	}
	if errors.len() == 1 {
		return Err(errors.into_iter().next().unwrap());
	}
	Err(MultiError("failed to load exchange rate data", errors).into())
}

fn parse_exchange_rates(data: &RawData) -> Result<Vec<(String, f64)>, Error> {
	match data.source {
		ExchangeRateSource::EuropeanUnion => parse_exchange_rates_eu(&data.exchange_rate_data),
		ExchangeRateSource::UnitedNations => parse_exchange_rates_un(&data.exchange_rate_data),
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
				break;
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

fn get_exchange_rates(
	setting: config::ExchangeRateSetting,
	max_age: u64,
	options: &fend_core::ExchangeRateFnV2Options,
) -> Result<Vec<(String, f64)>, Error> {
	if options.is_preview() {
		return Err(ExchangeRateSourceDisabledError.into());
	}
	let sources = setting.get_sources();
	if sources.is_empty() {
		return Err(ExchangeRateSourceDisabledError.into());
	}
	let rt = Handle::current();
	rt.block_on(async {
		let mut data = None;
		for &source in sources {
			if let Ok(xml) = load_cached_data(source, max_age) {
				data = Some(RawData {
					exchange_rate_data: xml,
					source,
					cached: true,
				});
				break;
			}
		}
		let data = match data {
			Some(data) => data,
			None => load_exchange_rates_auto(sources).await?,
		};
		let parsed_data = parse_exchange_rates(&data)?;
		if !data.cached {
			store_cached_data(data.source, &data.exchange_rate_data)?;
		}
		Ok(parsed_data)
	})
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
	pub source: ExchangeRateSetting,
	pub max_age: u64,
}

impl fend_core::ExchangeRateFnV2 for ExchangeRateHandler {
	fn relative_to_base_currency(
		&self,
		currency: &str,
		options: &fend_core::ExchangeRateFnV2Options,
	) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>> {
		if !self.enable_internet_access {
			return Err(InternetAccessDisabledError.into());
		}
		let exchange_rates = get_exchange_rates(self.source, self.max_age, options)?;
		for (c, rate) in exchange_rates {
			if currency == c {
				return Ok(rate);
			}
		}
		Err(UnknownExchangeRate(currency.to_string()).into())
	}
}
