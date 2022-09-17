use std::cell::RefCell;
use std::{error, fmt, time};

use crate::config;

pub struct HintInterrupt {
    start: time::Instant,
    duration: time::Duration,
}

impl fend_core::Interrupt for HintInterrupt {
    fn should_interrupt(&self) -> bool {
        time::Instant::now().duration_since(self.start) >= self.duration
    }
}

impl Default for HintInterrupt {
    fn default() -> Self {
        Self {
            start: time::Instant::now(),
            duration: time::Duration::from_millis(20),
        }
    }
}

pub struct InnerCtx {
    core_ctx: fend_core::Context,

    // true if the user typed some partial input, false otherwise
    input_typed: bool,
}

impl InnerCtx {
    pub fn new(config: &config::Config) -> Self {
        let mut res = Self {
            core_ctx: fend_core::Context::new(),
            input_typed: false,
        };
        if config.coulomb_and_farad {
            res.core_ctx.use_coulomb_and_farad();
        }
        res
    }
}

#[derive(Clone)]
pub struct Context<'a> {
    ctx: &'a RefCell<InnerCtx>,
}

impl<'a> Context<'a> {
    pub fn new(ctx: &'a RefCell<InnerCtx>) -> Self {
        Self { ctx }
    }

    pub fn eval(
        &self,
        line: &str,
        int: &impl fend_core::Interrupt,
    ) -> Result<fend_core::FendResult, String> {
        let mut ctx_borrow = self.ctx.borrow_mut();
        ctx_borrow.core_ctx.set_random_u32_fn(random_u32);
        ctx_borrow.core_ctx.set_output_mode_terminal();
        ctx_borrow
            .core_ctx
            .set_exchange_rate_handler_v1(exchange_rate);
        ctx_borrow.input_typed = false;
        fend_core::evaluate_with_interrupt(line, &mut ctx_borrow.core_ctx, int)
    }

    pub fn eval_hint(&self, line: &str) -> fend_core::FendResult {
        let mut ctx_borrow = self.ctx.borrow_mut();
        ctx_borrow.core_ctx.set_output_mode_terminal();
        ctx_borrow.input_typed = !line.is_empty();
        let int = HintInterrupt::default();
        fend_core::evaluate_preview_with_interrupt(line, &mut ctx_borrow.core_ctx, &int)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut result = vec![];
        self.ctx
            .borrow()
            .core_ctx
            .serialize_variables(&mut result)?;
        Ok(result)
    }

    pub fn get_input_typed(&self) -> bool {
        self.ctx.borrow().input_typed
    }
}

fn random_u32() -> u32 {
    let mut rng = nanorand::WyRand::new();
    nanorand::Rng::generate(&mut rng)
}

type Error = Box<dyn error::Error + Send + Sync + 'static>;

fn download_exchange_rates() -> Result<Vec<(String, f64)>, Error> {
    let exchange_rates = ureq::get("https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml")
        .call()?
        .into_string()?;
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

fn exchange_rate(currency: &str) -> Result<f64, Error> {
    let exchange_rates = download_exchange_rates()?;
    for (c, rate) in exchange_rates {
        if currency == c {
            return Ok(rate);
        }
    }
    Err(UnknownExchangeRate(currency.to_string()))?
}
