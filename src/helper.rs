use rustyline::{
    completion::{Candidate, Completer},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
    Helper,
};
use std::{env, time::{Duration, Instant}};

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

#[derive(Default)]
pub struct FendHelper {
    ctx: fend_core::Context,
}

impl Hinter for FendHelper {
    // TODO: Prevent the user from actually completing this hint.
    // Blocked on a rustyline update.
    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context) -> Option<String> {
        if !enable_live_output() {
            return None;
        }
        let int = HintInterrupt::default();
        Some(
            match fend_core::evaluate_with_interrupt(line, &mut self.ctx.clone(), &int) {
                Ok(result) => {
                    let res = result.get_main_result();
                    if res.is_empty() || res.len() > 50 || res.trim() == line.trim() {
                        return None;
                    } else {
                        format!("\n{}", res)
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

fn enable_live_output() -> bool {
    env::var_os("FEND_LIVE").is_some()
}
