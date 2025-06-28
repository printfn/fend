#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::use_self)]
#![forbid(clippy::needless_borrow)]
#![forbid(unreachable_pub)]
#![forbid(elided_lifetimes_in_paths)]
#![allow(clippy::tabs_in_doc_comments)]

//! This library implements most of the features of [fend](https://github.com/printfn/fend).
//!
//! ## Example
//!
//! ```rust
//! extern crate fend_core;
//!
//! fn main() {
//!     let mut context = fend_core::Context::new();
//!     let result = fend_core::evaluate("1 + 1", &mut context).unwrap();
//!     assert_eq!(result.get_main_result(), "2");
//! }
//! ```

mod ast;
mod date;
mod error;
mod eval;
mod format;
mod ident;
mod inline_substitutions;
mod interrupt;
/// This module is not meant to be used by other crates. It may change or be removed at any point.
pub mod json;
mod lexer;
mod num;
mod parser;
mod random;
mod result;
mod scope;
mod serialize;
mod units;
mod value;

use std::error::Error;
use std::fmt::Write;
use std::mem;
use std::sync::Arc;
use std::{collections::HashMap, fmt, io};

use error::FendError;
pub(crate) use eval::Attrs;
pub use interrupt::Interrupt;
pub use random::Random;
use result::FResult;
use serialize::{Deserialize, Serialize};

/// This contains the result of a computation.
#[derive(PartialEq, Eq, Debug)]
pub struct FendResult {
	plain_result: String,
	span_result: Vec<Span>,
	attrs: eval::Attrs,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum SpanKind {
	Number,
	BuiltInFunction,
	Keyword,
	String,
	Date,
	Whitespace,
	Ident,
	Boolean,
	Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Span {
	string: String,
	kind: SpanKind,
}

impl Span {
	fn from_string(s: String) -> Self {
		Self {
			string: s,
			kind: SpanKind::Other,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanRef<'a> {
	string: &'a str,
	kind: SpanKind,
}

impl<'a> SpanRef<'a> {
	#[must_use]
	pub fn kind(self) -> SpanKind {
		self.kind
	}

	#[must_use]
	pub fn string(self) -> &'a str {
		self.string
	}
}

impl FendResult {
	/// This retrieves the main result of the computation.
	#[must_use]
	pub fn get_main_result(&self) -> &str {
		self.plain_result.as_str()
	}

	/// This retrieves the main result as a list of spans, which is useful
	/// for colored output.
	pub fn get_main_result_spans(&self) -> impl Iterator<Item = SpanRef<'_>> {
		self.span_result.iter().map(|span| SpanRef {
			string: &span.string,
			kind: span.kind,
		})
	}

	#[must_use]
	#[deprecated(note = "use `output_is_empty()` instead")]
	pub fn is_unit_type(&self) -> bool {
		self.output_is_empty()
	}

	/// Returns whether the output should be hidden by default. This is set
	/// if the output would just be the unit `()` type. It's useful to hide this
	/// by default.
	#[must_use]
	pub fn output_is_empty(&self) -> bool {
		self.span_result.is_empty()
	}

	fn empty() -> Self {
		Self {
			plain_result: String::new(),
			span_result: vec![],
			attrs: Attrs::default(),
		}
	}

	/// Returns whether or not the result should be outputted with a
	/// trailing newline. This is controlled by the `@no_trailing_newline`
	/// attribute.
	#[must_use]
	pub fn has_trailing_newline(&self) -> bool {
		self.attrs.trailing_newline
	}
}

#[derive(Clone, Debug)]
struct CurrentTimeInfo {
	elapsed_unix_time_ms: u64,
	timezone_offset_secs: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FCMode {
	CelsiusFahrenheit,
	CoulombFarad,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum OutputMode {
	SimpleText,
	TerminalFixedWidth,
}

/// An exchange rate handler.
pub trait ExchangeRateFn {
	/// Returns the value of a currency relative to the base currency.
	/// The base currency depends on your implementation. fend-core can work
	/// with any base currency as long as it is consistent.
	///
	/// # Errors
	/// This function errors out if the currency was not found or the
	/// conversion is impossible for any reason (HTTP request failed, etc.)
	fn relative_to_base_currency(
		&self,
		currency: &str,
	) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

impl<T> ExchangeRateFn for T
where
	T: Fn(&str) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>>,
{
	fn relative_to_base_currency(
		&self,
		currency: &str,
	) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>> {
		self(currency)
	}
}

/// Options passed to an exchange rate handler function
pub struct ExchangeRateFnV2Options {
	is_preview: bool,
}

impl ExchangeRateFnV2Options {
	/// Whether or not this is a preview rather than a standard calculation.
	#[must_use]
	pub fn is_preview(&self) -> bool {
		self.is_preview
	}
}

/// An exchange rate handler.
pub trait ExchangeRateFnV2 {
	/// Returns the value of a currency relative to the base currency.
	/// The base currency depends on your implementation. fend-core can work
	/// with any base currency as long as it is consistent.
	///
	/// If `options.is_preview()` returns true, implementors are
	/// encouraged to only return cached or pre-fetched results. Blocking the
	/// thread for an extended period of time is not ideal if this calculation
	/// is merely for a preview.
	///
	/// # Errors
	/// This function errors out if the currency was not found or the
	/// conversion is impossible for any reason (HTTP request failed, etc.)
	fn relative_to_base_currency(
		&self,
		currency: &str,
		options: &ExchangeRateFnV2Options,
	) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

struct ExchangeRateV1CompatWrapper {
	get_exchange_rate_v1: Arc<dyn ExchangeRateFn + Send + Sync>,
}

impl ExchangeRateFnV2 for ExchangeRateV1CompatWrapper {
	fn relative_to_base_currency(
		&self,
		currency: &str,
		options: &ExchangeRateFnV2Options,
	) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>> {
		if options.is_preview {
			return Err(FendError::NoExchangeRatesAvailable.into());
		}
		self.get_exchange_rate_v1
			.relative_to_base_currency(currency)
	}
}

/// This controls decimal and thousands separators.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DecimalSeparatorStyle {
	/// Use `.` as the decimal separator and `,` as the thousands separator. This is common in English.
	#[default]
	Dot,
	/// Use `,` as the decimal separator and `.` as the thousands separator. This is common in European languages.
	Comma,
}

impl DecimalSeparatorStyle {
	fn decimal_separator(self) -> char {
		match self {
			Self::Dot => '.',
			Self::Comma => ',',
		}
	}

	fn thousands_separator(self) -> char {
		match self {
			Self::Dot => ',',
			Self::Comma => '.',
		}
	}
}

/// This struct contains fend's current context, including some settings
/// as well as stored variables.
///
/// If you're writing an interpreter it's recommended to only
/// instantiate this struct once so that variables and settings are
/// preserved, but you can also manually serialise all variables
/// and recreate the context for every calculation, depending on
/// which is easier.
pub struct Context {
	current_time: Option<CurrentTimeInfo>,
	variables: HashMap<String, value::Value>,
	fc_mode: FCMode,
	rng: Option<Box<dyn Random>>,
	output_mode: OutputMode,
	echo_result: bool, // whether to automatically print the result
	get_exchange_rate_v2: Option<Arc<dyn ExchangeRateFnV2 + Send + Sync>>,
	custom_units: Vec<(String, String, String)>,
	decimal_separator: DecimalSeparatorStyle,
	is_preview: bool,
}

impl fmt::Debug for Context {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// we can't derive Debug because of the get_exchange_rate field
		f.debug_struct("Context")
			.field("current_time", &self.current_time)
			.field("variables", &self.variables)
			.field("fc_mode", &self.fc_mode)
			.field("rng", &self.rng)
			.field("output_mode", &self.output_mode)
			.field("echo_result", &self.echo_result)
			.field("custom_units", &self.custom_units)
			.field("decimal_separator", &self.decimal_separator)
			.field("is_preview", &self.is_preview)
			.finish_non_exhaustive()
	}
}

impl Clone for Context {
	fn clone(&self) -> Self {
		// we can't derive Clone because of the rng field
		Self {
			current_time: self.current_time.clone(),
			variables: self.variables.clone(),
			fc_mode: self.fc_mode.clone(),
			rng: None,
			output_mode: self.output_mode.clone(),
			echo_result: self.echo_result,
			get_exchange_rate_v2: self.get_exchange_rate_v2.clone(),
			custom_units: self.custom_units.clone(),
			decimal_separator: self.decimal_separator,
			is_preview: self.is_preview,
		}
	}
}

impl Default for Context {
	fn default() -> Self {
		Self::new()
	}
}

impl Context {
	/// Create a new context instance.
	#[must_use]
	pub fn new() -> Self {
		Self {
			current_time: None,
			variables: HashMap::new(),
			fc_mode: FCMode::CelsiusFahrenheit,
			rng: None,
			output_mode: OutputMode::SimpleText,
			echo_result: true,
			get_exchange_rate_v2: None,
			custom_units: vec![],
			decimal_separator: DecimalSeparatorStyle::default(),
			is_preview: false,
		}
	}

	/// This method currently has no effect!
	///
	/// Set the current time. This API will likely change in the future!
	///
	/// The first argument (`ms_since_1970`) must be the number of elapsed milliseconds
	/// since January 1, 1970 at midnight UTC, ignoring leap seconds in the same way
	/// as Unix time.
	///
	/// The second argument (`tz_offset_secs`) is the current time zone
	/// offset to UTC, in seconds.
	pub fn set_current_time_v1(&mut self, _ms_since_1970: u64, _tz_offset_secs: i64) {
		// self.current_time = Some(CurrentTimeInfo {
		//     elapsed_unix_time_ms: ms_since_1970,
		//     timezone_offset_secs: tz_offset_secs,
		// });
		self.current_time = None;
	}

	/// Define the units `C` and `F` as coulomb and farad instead of degrees
	/// celsius and degrees fahrenheit.
	pub fn use_coulomb_and_farad(&mut self) {
		self.fc_mode = FCMode::CoulombFarad;
	}

	/// Set a random number generator
	pub fn set_rng(&mut self, rng: impl Random + 'static) {
		self.rng = Some(Box::new(rng));
	}

	/// Clear the random number generator after setting it with via [`Self::set_rng`]
	pub fn disable_rng(&mut self) {
		self.rng = None;
	}

	/// Whether to automatically print the resulting value of this calculation.
	/// This option is enabled by default. If disabled, only values printed
	/// with the `print` or `println` functions will be shown in the output.
	pub fn set_echo_result(&mut self, echo_result: bool) {
		self.echo_result = echo_result;
	}

	/// Change the output mode to fixed-width terminal style. This enables ASCII
	/// graphs in the output.
	pub fn set_output_mode_terminal(&mut self) {
		self.output_mode = OutputMode::TerminalFixedWidth;
	}

	fn serialize_variables_internal(&self, write: &mut impl io::Write) -> FResult<()> {
		self.variables.len().serialize(write)?;
		for (k, v) in &self.variables {
			k.as_str().serialize(write)?;
			v.serialize(write)?;
		}
		Ok(())
	}

	/// Serializes all variables defined in this context to a stream of bytes.
	/// Note that the specific format is NOT stable, and can change with any
	/// minor update.
	///
	/// # Errors
	/// This function returns an error if the input cannot be serialized.
	pub fn serialize_variables(&self, write: &mut impl io::Write) -> Result<(), String> {
		match self.serialize_variables_internal(write) {
			Ok(()) => Ok(()),
			Err(e) => Err(e.to_string()),
		}
	}

	fn deserialize_variables_internal(&mut self, read: &mut impl io::Read) -> FResult<()> {
		let len = usize::deserialize(read)?;
		self.variables.clear();
		self.variables.reserve(len);
		for _ in 0..len {
			let s = String::deserialize(read)?;
			let v = value::Value::deserialize(read)?;
			self.variables.insert(s, v);
		}
		Ok(())
	}

	/// Deserializes the given variables, replacing all prior variables in
	/// the given context.
	///
	/// # Errors
	/// Returns an error if the input byte stream is invalid and cannot be
	/// deserialized.
	pub fn deserialize_variables(&mut self, read: &mut impl io::Read) -> Result<(), String> {
		match self.deserialize_variables_internal(read) {
			Ok(()) => Ok(()),
			Err(e) => Err(e.to_string()),
		}
	}

	/// Set a handler function for loading exchange rates.
	#[deprecated(note = "Use `set_exchange_rate_handler_v2` instead")]
	pub fn set_exchange_rate_handler_v1<T: ExchangeRateFn + 'static + Send + Sync>(
		&mut self,
		get_exchange_rate: T,
	) {
		self.get_exchange_rate_v2 = Some(Arc::new(ExchangeRateV1CompatWrapper {
			get_exchange_rate_v1: Arc::new(get_exchange_rate),
		}));
	}

	/// Set a handler function for loading exchange rates.
	pub fn set_exchange_rate_handler_v2<T: ExchangeRateFnV2 + 'static + Send + Sync>(
		&mut self,
		get_exchange_rate: T,
	) {
		self.get_exchange_rate_v2 = Some(Arc::new(get_exchange_rate));
	}

	pub fn define_custom_unit_v1(
		&mut self,
		singular: &str,
		plural: &str,
		definition: &str,
		attribute: &CustomUnitAttribute,
	) {
		let definition_prefix = match attribute {
			CustomUnitAttribute::None => "",
			CustomUnitAttribute::AllowLongPrefix => "l@",
			CustomUnitAttribute::AllowShortPrefix => "s@",
			CustomUnitAttribute::IsLongPrefix => "lp@",
			CustomUnitAttribute::Alias => "=",
		};
		self.custom_units.push((
			singular.to_string(),
			plural.to_string(),
			format!("{definition_prefix}{definition}"),
		));
	}

	/// Sets the decimal separator style for this context. This can be used to
	/// change the number format from e.g. `1,234.00` to `1.234,00`.
	pub fn set_decimal_separator_style(&mut self, style: DecimalSeparatorStyle) {
		self.decimal_separator = style;
	}
}

/// These attributes make is possible to change the behaviour of custom units
#[non_exhaustive]
pub enum CustomUnitAttribute {
	/// Don't allow using prefixes with this custom unit
	None,
	/// Support long prefixes (e.g. `milli-`, `giga-`) with this unit
	AllowLongPrefix,
	/// Support short prefixes (e.g. `k` for `kilo`) with this unit
	AllowShortPrefix,
	/// Allow using this unit as a long prefix with another unit
	IsLongPrefix,
	/// This unit definition is an alias and will always be replaced with its definition.
	Alias,
}

/// This function evaluates a string using the given context. Any evaluation using this
/// function cannot be interrupted.
///
/// For example, passing in the string `"1 + 1"` will return a result of `"2"`.
///
/// # Errors
/// It returns an error if the given string is invalid.
/// This may be due to parser or runtime errors.
pub fn evaluate(input: &str, context: &mut Context) -> Result<FendResult, String> {
	evaluate_with_interrupt(input, context, &interrupt::Never)
}

fn evaluate_with_interrupt_internal(
	input: &str,
	context: &mut Context,
	int: &impl Interrupt,
) -> Result<FendResult, String> {
	if input.is_empty() {
		// no or blank input: return no output
		return Ok(FendResult::empty());
	}
	let (result, attrs) = match eval::evaluate_to_spans(input, None, context, int) {
		Ok(value) => value,
		Err(e) => {
			let mut error: &dyn Error = &e;
			let mut s = error.to_string();
			while let Some(inner) = error.source() {
				write!(&mut s, ": {inner}").unwrap();
				error = inner;
			}
			return Err(s);
		}
	};
	let mut plain_result = String::new();
	for s in &result {
		plain_result.push_str(&s.string);
	}
	Ok(FendResult {
		plain_result,
		span_result: result,
		attrs,
	})
}

/// This function evaluates a string using the given context and the provided
/// Interrupt object.
///
/// For example, passing in the string `"1 + 1"` will return a result of `"2"`.
///
/// # Errors
/// It returns an error if the given string is invalid.
/// This may be due to parser or runtime errors.
pub fn evaluate_with_interrupt(
	input: &str,
	context: &mut Context,
	int: &impl Interrupt,
) -> Result<FendResult, String> {
	evaluate_with_interrupt_internal(input, context, int)
}

/// Evaluate the given string to use as a live preview.
///
/// Unlike the normal evaluation functions, `evaluate_preview_with_interrupt`
/// does not mutate the passed-in context, and only returns results suitable
/// for displaying as a live preview: overly long output, multi-line output,
/// unit types etc. are all filtered out. RNG functions (e.g. `roll d6`) are
/// also disabled.
pub fn evaluate_preview_with_interrupt(
	input: &str,
	context: &Context,
	int: &impl Interrupt,
) -> FendResult {
	let empty = FendResult::empty();
	// unfortunately making a complete copy of the context is necessary
	// because we want variables to still work in multi-statement inputs
	// like `a = 2; 5a`.
	let mut context_clone = context.clone();
	context_clone.is_preview = true;
	let result = evaluate_with_interrupt_internal(input, &mut context_clone, int);
	mem::drop(context_clone);
	let Ok(result) = result else {
		return empty;
	};
	let s = result.get_main_result();
	if s.is_empty()
		|| result.output_is_empty()
		|| s.len() > 50
		|| s.trim() == input.trim()
		|| s.contains(|c| c < ' ')
	{
		return empty;
	}
	result
}

#[derive(Debug)]
pub struct Completion {
	display: String,
	insert: String,
}

impl Completion {
	#[must_use]
	pub fn display(&self) -> &str {
		&self.display
	}

	#[must_use]
	pub fn insert(&self) -> &str {
		&self.insert
	}
}

static GREEK_LOWERCASE_LETTERS: [(&str, &str); 24] = [
	("alpha", "α"),
	("beta", "β"),
	("gamma", "γ"),
	("delta", "δ"),
	("epsilon", "ε"),
	("zeta", "ζ"),
	("eta", "η"),
	("theta", "θ"),
	("iota", "ι"),
	("kappa", "κ"),
	("lambda", "λ"),
	("mu", "μ"),
	("nu", "ν"),
	("xi", "ξ"),
	("omicron", "ο"),
	("pi", "π"),
	("rho", "ρ"),
	("sigma", "σ"),
	("tau", "τ"),
	("upsilon", "υ"),
	("phi", "φ"),
	("chi", "χ"),
	("psi", "ψ"),
	("omega", "ω"),
];
static GREEK_UPPERCASE_LETTERS: [(&str, &str); 24] = [
	("Alpha", "Α"),
	("Beta", "Β"),
	("Gamma", "Γ"),
	("Delta", "Δ"),
	("Epsilon", "Ε"),
	("Zeta", "Ζ"),
	("Eta", "Η"),
	("Theta", "Θ"),
	("Iota", "Ι"),
	("Kappa", "Κ"),
	("Lambda", "Λ"),
	("Mu", "Μ"),
	("Nu", "Ν"),
	("Xi", "Ξ"),
	("Omicron", "Ο"),
	("Pi", "Π"),
	("Rho", "Ρ"),
	("Sigma", "Σ"),
	("Tau", "Τ"),
	("Upsilon", "Υ"),
	("Phi", "Φ"),
	("Chi", "Χ"),
	("Psi", "Ψ"),
	("Omega", "Ω"),
];

#[must_use]
pub fn get_completions_for_prefix(mut prefix: &str) -> (usize, Vec<Completion>) {
	if let Some((prefix, letter)) = prefix.rsplit_once('\\')
		&& letter.starts_with(|c: char| c.is_ascii_alphabetic())
		&& letter.len() <= 7
	{
		return if letter.starts_with(|c: char| c.is_ascii_uppercase()) {
			GREEK_UPPERCASE_LETTERS
		} else {
			GREEK_LOWERCASE_LETTERS
		}
		.iter()
		.find(|l| l.0 == letter)
		.map_or((0, vec![]), |l| {
			(
				prefix.len(),
				vec![Completion {
					display: prefix.to_string(),
					insert: l.1.to_string(),
				}],
			)
		});
	}

	let mut prepend = "";
	let position = prefix.len();
	if let Some((a, b)) = prefix.rsplit_once(' ') {
		prepend = a;
		prefix = b;
	}

	if prefix.is_empty() {
		return (0, vec![]);
	}
	let mut res = units::get_completions_for_prefix(prefix);
	for c in &mut res {
		c.display.insert_str(0, prepend);
	}
	(position, res)
}

pub use inline_substitutions::substitute_inline_fend_expressions;

const fn get_version_as_str() -> &'static str {
	env!("CARGO_PKG_VERSION")
}

/// Returns the current version of `fend-core`.
#[must_use]
pub fn get_version() -> String {
	get_version_as_str().to_string()
}

/// Used by unit and integration tests
#[doc(hidden)]
pub mod test_utils {
	use crate::ExchangeRateFnV2;

	/// A simple currency handler used in unit and integration tests. Not intended
	/// to be used outside of `fend_core`.
	///
	/// # Panics
	/// Panics on unknown currencies
	///
	/// # Errors
	/// Panics on error, so it never needs to return Err(_)
	#[doc(hidden)]
	pub struct DummyCurrencyHandler;

	impl ExchangeRateFnV2 for DummyCurrencyHandler {
		fn relative_to_base_currency(
			&self,
			currency: &str,
			_options: &crate::ExchangeRateFnV2Options,
		) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>> {
			Ok(match currency {
				"EUR" | "USD" => 1.0,
				"GBP" => 0.9,
				"NZD" => 1.5,
				"HKD" => 8.0,
				"AUD" => 1.3,
				"PLN" => 0.2,
				"JPY" => 149.9,
				_ => panic!("unknown currency {currency}"),
			})
		}
	}
}
