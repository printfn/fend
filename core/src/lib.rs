#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::use_self)]
#![forbid(clippy::needless_borrow)]
//#![forbid(clippy::cognitive_complexity)]
#![forbid(unreachable_pub)]
#![forbid(elided_lifetimes_in_paths)]
#![doc(html_root_url = "https://docs.rs/fend-core/0.1.23")]

mod ast;
mod date;
mod error;
mod eval;
mod format;
mod ident;
mod interrupt;
mod lexer;
mod num;
mod parser;
mod scope;
mod units;
mod value;

use std::collections::HashMap;

pub use interrupt::Interrupt;

/// This contains the result of a computation.
#[derive(PartialEq, Eq, Debug)]
pub struct FendResult {
    plain_result: String,
    span_result: Vec<Span>,
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

    pub fn get_main_result_spans(&self) -> impl Iterator<Item = SpanRef<'_>> {
        self.span_result.iter().map(|span| SpanRef {
            string: &span.string,
            kind: span.kind,
        })
    }

    /// This used to retrieve a list of other results of the computation,
    /// but now returns an empty iterator. This method is deprecated and
    /// may be removed in a future release.
    #[deprecated]
    #[allow(clippy::unused_self)]
    pub fn get_other_info(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
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

/// This struct contains context used for `fend`. It should only be created once
/// at startup.
#[derive(Clone, Debug)]
pub struct Context {
    current_time: Option<CurrentTimeInfo>,
    variables: HashMap<String, value::Value>,
    fc_mode: FCMode,
    random_u32: Option<fn() -> u32>,
    output_mode: OutputMode,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new context instance. This can be fairly slow, and should
    /// only be done once if possible.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_time: None,
            variables: HashMap::new(),
            fc_mode: FCMode::CelsiusFahrenheit,
            random_u32: None,
            output_mode: OutputMode::SimpleText,
        }
    }

    /// This method currently has no effect!
    ///
    /// Set the current time. This API will likely change in the future!
    ///
    /// The first argument (`ms_since_1970`) must be the number of elapsed milliseconds
    /// since January 1, 1970 at midnight UTC, ignoring leap seconds in the same way
    /// as unix time.
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
    pub fn set_random_u32_fn(&mut self, random_u32: fn() -> u32) {
        self.random_u32 = Some(random_u32);
    }

    /// Clear the random number generator after setting it with via [`Self::set_random_u32_fn`]
    pub fn disable_rng(&mut self) {
        self.random_u32 = None;
    }

    /// Change the output mode fixed-width terminal style. This enables ASCII
    /// graphs in the output.
    pub fn set_output_mode_terminal(&mut self) {
        self.output_mode = OutputMode::TerminalFixedWidth;
    }
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
    evaluate_with_interrupt(input, context, &interrupt::Never::default())
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
    if input.is_empty() {
        // no or blank input: return no output
        return Ok(FendResult {
            plain_result: String::new(),
            span_result: vec![],
        });
    }
    let result = match eval::evaluate_to_spans(input, None, context, int) {
        Ok(value) => value,
        Err(e) => return Err(e.to_string()),
    };
    let mut plain_result = String::new();
    for s in &result {
        plain_result.push_str(&s.string);
    }
    Ok(FendResult {
        plain_result,
        span_result: result,
    })
}

const fn get_version_as_str() -> &'static str {
    "0.1.23"
}

/// Returns the current version of `fend-core`.
#[must_use]
pub fn get_version() -> String {
    get_version_as_str().to_string()
}

/// Deprecated: use `get_version()` instead.
#[must_use]
#[deprecated = "use `get_version()` instead"]
pub fn get_extended_version() -> String {
    get_version()
}
