use crate::{config, context::Context};

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
        let result = self.ctx.eval_hint(line);
        let s = result.get_main_result();
        Some(if s.is_empty() {
            return None;
        } else if self.config.enable_colors {
            Hint(format!(
                "\n{}",
                crate::print_spans(result.get_main_result_spans().collect(), self.config)
            ))
        } else {
            Hint(format!("\n{s}"))
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
        let (pos, completions) = fend_core::get_completions_for_prefix(&line[..pos]);
        let v: Vec<_> = completions
            .into_iter()
            .map(|c| FendCandidate { completion: c })
            .collect();
        Ok((pos, v))
    }
}

impl rustyline::Helper for Helper<'_> {}
