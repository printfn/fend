use rustyline::hint::Hinter;
use rustyline_derive::{Completer, Helper, Highlighter, Validator};
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
            duration: Duration::from_millis(100),
        }
    }
}

#[derive(Default, Completer, Helper, Highlighter, Validator)]
pub struct FendHelper {
    ctx: fend_core::Context,
}

impl Hinter for FendHelper {
    // TODO: Prevent the user from actually completing this hint.
    // Blocked on a rustyline update.
    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context) -> Option<String> {
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
