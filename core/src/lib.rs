#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::use_self)]
#![forbid(clippy::needless_borrow)]
#![forbid(unreachable_pub)]
#![forbid(elided_lifetimes_in_paths)]

mod ast;
mod date;
mod error;
mod eval;
mod format;
mod ident;
mod inline_substitutions;
mod interrupt;
mod json;
mod lexer;
mod num;
mod parser;
mod scope;
mod serialize;
mod units;
mod value;

use std::{collections::HashMap, io};

use error::FendError;
pub use interrupt::Interrupt;
use serialize::{deserialize_string, deserialize_usize, serialize_string, serialize_usize};

/// This contains the result of a computation.
#[derive(PartialEq, Eq, Debug)]
pub struct FendResult {
    plain_result: String,
    span_result: Vec<Span>,
    is_unit: bool, // is this the () type
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

    /// Returns whether or not the result is the `()` type. It can sometimes
    /// be useful to hide these values.
    #[must_use]
    pub fn is_unit_type(&self) -> bool {
        self.is_unit
    }

    /// This used to retrieve a list of other results of the computation,
    /// but now returns an empty iterator. This method is deprecated and
    /// may be removed in a future release.
    #[deprecated]
    #[allow(clippy::unused_self)]
    pub fn get_other_info(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }

    fn empty() -> Self {
        Self {
            plain_result: String::new(),
            span_result: vec![],
            is_unit: true,
        }
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

    /// Change the output mode to fixed-width terminal style. This enables ASCII
    /// graphs in the output.
    pub fn set_output_mode_terminal(&mut self) {
        self.output_mode = OutputMode::TerminalFixedWidth;
    }

    fn serialize_variables_internal(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        serialize_usize(self.variables.len(), write)?;
        for (k, v) in &self.variables {
            serialize_string(k.as_str(), write)?;
            v.serialize(write)?;
        }
        Ok(())
    }

    /// Serializes all variables defined in this context to a stream of bytes.
    /// Note that the specific format is NOT stable, and can change with any
    /// minor update. It is also not cross-platform compatible.
    ///
    /// # Errors
    /// This function returns an error if the input cannot be serialized.
    pub fn serialize_variables(&self, write: &mut impl io::Write) -> Result<(), String> {
        match self.serialize_variables_internal(write) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn deserialize_variables_internal(
        &mut self,
        read: &mut impl io::Read,
    ) -> Result<(), FendError> {
        let len = deserialize_usize(read)?;
        self.variables.clear();
        self.variables.reserve(len);
        for _ in 0..len {
            self.variables
                .insert(deserialize_string(read)?, value::Value::deserialize(read)?);
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

fn evaluate_with_interrupt_internal(
    input: &str,
    context: &mut Context,
    int: &impl Interrupt,
) -> Result<FendResult, String> {
    if input.is_empty() {
        // no or blank input: return no output
        return Ok(FendResult::empty());
    }
    let (result, is_unit) = match eval::evaluate_to_spans(input, None, context, int) {
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
        is_unit,
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
    context: &mut Context,
    int: &impl Interrupt,
) -> FendResult {
    let empty = FendResult::empty();
    // unfortunately making a complete copy of the context is necessary
    // because we want variables to still work in multi-statement inputs
    // like `a = 2; 5a`.
    let context_clone = context.clone();
    context.random_u32 = None;
    let result = evaluate_with_interrupt_internal(input, context, int);
    *context = context_clone;
    let result = match result {
        Ok(result) => result,
        Err(_) => return empty,
    };
    let s = result.get_main_result();
    if s.is_empty()
        || result.is_unit_type()
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

#[must_use]
pub fn get_completions_for_prefix(mut prefix: &str) -> (usize, Vec<Completion>) {
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
