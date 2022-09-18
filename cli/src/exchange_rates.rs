use std::{error, fmt, fs, io::Write, time};

type Error = Box<dyn error::Error + Send + Sync + 'static>;

const MAX_AGE: u64 = 86400 * 3;

fn get_current_timestamp() -> Result<u64, Error> {
    Ok(time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)?
        .as_secs())
}

fn load_cached_data() -> Result<String, Error> {
    let mut cache_file = crate::file_paths::get_cache_dir()?;
    cache_file.push("eurofxref-daily.xml.cache");
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

fn store_cached_data(xml: &str) -> Result<(), Error> {
    let mut cache_file = crate::file_paths::create_cache_dir()?;
    cache_file.push("eurofxref-daily.xml.cache");
    let mut file = fs::File::create(cache_file)?;
    write!(file, "{};{xml}", get_current_timestamp()?)?;
    Ok(())
}

fn load_exchange_rate_xml() -> Result<String, Error> {
    match load_cached_data() {
        Ok(xml) => return Ok(xml),
        Err(_e) => {
            //eprintln!("failed to load cached data: {e}");
        }
    }
    let xml = ureq::get("https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml")
        .call()?
        .into_string()?;
    store_cached_data(&xml)?;
    Ok(xml)
}

fn get_exchange_rates() -> Result<Vec<(String, f64)>, Error> {
    let exchange_rates = load_exchange_rate_xml()?;
    let err = "failed to load exchange rates";
    let mut result = vec![("EUR".to_string(), 1.0)];
    let mut one_eur_in_usd = None;
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
        result.push((currency.to_string(), exchange_rate_eur));
        if currency == "USD" {
            one_eur_in_usd = Some(exchange_rate_eur);
        }
    }
    let one_eur_in_usd = one_eur_in_usd.ok_or(err)?;
    for (_, exchange_rate) in &mut result {
        // exchange rate currently represents 1 EUR, but we want it to be 1 USD instead
        *exchange_rate /= one_eur_in_usd;
    }
    Ok(result)
}

#[derive(Debug, Clone)]
struct UnknownExchangeRate(String);

impl fmt::Display for UnknownExchangeRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "currency exchange rate for {} is unknown", self.0)
    }
}

impl error::Error for UnknownExchangeRate {}

pub fn exchange_rate_handler(currency: &str) -> Result<f64, Error> {
    let exchange_rates = get_exchange_rates()?;
    for (c, rate) in exchange_rates {
        if currency == c {
            return Ok(rate);
        }
    }
    Err(UnknownExchangeRate(currency.to_string()))?
}
