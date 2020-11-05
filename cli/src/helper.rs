use rustyline::{
    completion::{Candidate, Completer},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
    Helper,
};
use std::time::{Duration, Instant};

pub struct HintInterrupt {
    start: Instant,
    duration: Duration,
}

impl fend_core::Interrupt for HintInterrupt {
    fn should_interrupt(&self) -> bool {
        Instant::now().duration_since(self.start) >= self.duration
    }
}

impl Default for HintInterrupt {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            duration: Duration::from_millis(20),
        }
    }
}

pub struct FendHint(String);

impl rustyline::hint::Hint for FendHint {
    fn display(&self) -> &str {
        self.0.as_str()
    }

    fn completion(&self) -> Option<&str> {
        None
    }
}

pub struct FendHelper {
    ctx: fend_core::Context,
}

impl FendHelper {
    pub fn new(ctx: fend_core::Context) -> Self {
        Self { ctx }
    }
}

impl Hinter for FendHelper {
    type Hint = FendHint;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context) -> Option<FendHint> {
        let int = HintInterrupt::default();
        Some(
            match fend_core::evaluate_with_interrupt(line, &mut self.ctx.clone(), &int) {
                Ok(result) => {
                    let res = result.get_main_result();
                    if res.is_empty() || res.len() > 50 || res.trim() == line.trim() {
                        return None;
                    } else {
                        FendHint(format!("\n{}", res))
                    }
                }
                Err(_msg) => return None,
            },
        )
    }
}

impl Highlighter for FendHelper {}

impl Validator for FendHelper {}

pub struct FendCandidate {}
impl Candidate for FendCandidate {
    fn display(&self) -> &str {
        ""
    }
    fn replacement(&self) -> &str {
        ""
    }
}

impl Completer for FendHelper {
    type Candidate = FendCandidate;
}

impl Helper for FendHelper {}
