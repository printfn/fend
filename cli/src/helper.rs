use crate::{config, context::Context};
use std::time;

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

pub struct Hint(String);

impl rustyline::hint::Hint for Hint {
    fn display(&self) -> &str {
        self.0.as_str()
    }

    fn completion(&self) -> Option<&str> {
        None
    }
}

pub struct Helper<'a> {
    ctx: Context<'a>,
    config: &'a config::Config,
}

impl<'a> Helper<'a> {
    pub fn new(ctx: Context<'a>, config: &'a config::Config) -> Self {
        Self { ctx, config }
    }
}

impl rustyline::hint::Hinter for Helper<'_> {
    type Hint = Hint;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Hint> {
        let int = HintInterrupt::default();
        Some(match self.ctx.eval(line, false, &int) {
            Ok(result) => {
                let res = result.get_main_result();
                if res.is_empty()
                    || result.is_unit_type()
                    || res.len() > 50
                    || res.trim() == line.trim()
                    || res.contains(|c| c < ' ')
                {
                    return None;
                }
                if self.config.enable_colors {
                    Hint(format!(
                        "\n{}",
                        crate::print_spans(result.get_main_result_spans().collect(), self.config)
                    ))
                } else {
                    Hint(format!("\n{}", res))
                }
            }
            Err(_msg) => return None,
        })
    }
}

impl rustyline::highlight::Highlighter for Helper<'_> {}

impl rustyline::validate::Validator for Helper<'_> {}

#[derive(Debug)]
pub struct FendCandidate {
    completion: fend_core::Completion,
}
impl rustyline::completion::Candidate for FendCandidate {
    fn display(&self) -> &str {
        self.completion.display()
    }
    fn replacement(&self) -> &str {
        self.completion.insert()
    }
}

impl rustyline::completion::Completer for Helper<'_> {
    type Candidate = FendCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let completions = fend_core::get_completions_for_prefix(&line[..pos]);
        let v: Vec<_> = completions
            .into_iter()
            .map(|c| FendCandidate { completion: c })
            .collect();
        Ok((1, v))
    }
}

impl rustyline::Helper for Helper<'_> {}
