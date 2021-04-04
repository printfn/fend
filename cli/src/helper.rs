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

pub struct Hint(String);

impl rustyline::hint::Hint for Hint {
    fn display(&self) -> &str {
        self.0.as_str()
    }

    fn completion(&self) -> Option<&str> {
        None
    }
}

pub struct Helper {
    ctx: fend_core::Context,
    enable_color: bool,
}

impl Helper {
    pub fn new(ctx: fend_core::Context, enable_color: bool) -> Self {
        Self { ctx, enable_color }
    }
}

impl rustyline::hint::Hinter for Helper {
    type Hint = Hint;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context) -> Option<Hint> {
        let int = HintInterrupt::default();
        Some(
            match fend_core::evaluate_with_interrupt(line, &mut self.ctx.clone(), &int) {
                Ok(result) => {
                    let res = result.get_main_result();
                    if res.is_empty()
                        || res.len() > 50
                        || res.trim() == line.trim()
                        || res.contains(|c| c < ' ')
                    {
                        return None;
                    }
                    if self.enable_color {
                        Hint(format!(
                            "\n{}",
                            crate::print_spans(result.get_main_result_spans().collect())
                        ))
                    } else {
                        Hint(format!("\n{}", result.get_main_result()))
                    }
                }
                Err(_msg) => return None,
            },
        )
    }
}

impl rustyline::highlight::Highlighter for Helper {}

impl rustyline::validate::Validator for Helper {}

pub struct FendCandidate {}
impl rustyline::completion::Candidate for FendCandidate {
    fn display(&self) -> &str {
        ""
    }
    fn replacement(&self) -> &str {
        ""
    }
}

impl rustyline::completion::Completer for Helper {
    type Candidate = FendCandidate;
}

impl rustyline::Helper for Helper {}
