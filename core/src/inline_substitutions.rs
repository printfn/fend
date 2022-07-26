use crate::{Context, Interrupt};

pub enum InlineFendResultComponent {
    Unprocessed(String),
    FendOutput(String),
    FendError(String),
}

pub struct InlineFendResult {
    parts: Vec<InlineFendResultComponent>,
}

impl InlineFendResult {
    pub fn get_parts(&self) -> &[InlineFendResultComponent] {
        self.parts.as_slice()
    }
}

pub fn substitute_inline_fend_expressions(
    input: &str,
    context: &mut Context,
    int: &impl Interrupt,
) -> InlineFendResult {
    let mut result = InlineFendResult { parts: vec![] };
    let mut current_component = String::new();
    let mut inside_fend_expr = false;
    let mut inside_backticks = false;
    for ch in input.chars() {
        current_component.push(ch);
        if ch == '`' {
            inside_backticks = !inside_backticks;
        }
        if !inside_fend_expr && !inside_backticks && current_component.ends_with("[[") {
            current_component.truncate(current_component.len() - 2);
            result
                .parts
                .push(InlineFendResultComponent::Unprocessed(current_component));
            current_component = String::new();
            inside_fend_expr = true;
        } else if inside_fend_expr && !inside_backticks && current_component.ends_with("]]") {
            current_component.truncate(current_component.len() - 2);
            match crate::evaluate_with_interrupt(&current_component, context, int) {
                Ok(res) => result.parts.push(InlineFendResultComponent::FendOutput(
                    res.get_main_result().to_string(),
                )),
                Err(msg) => result.parts.push(InlineFendResultComponent::FendError(msg)),
            }
            current_component = String::new();
            inside_fend_expr = false;
        }
    }
    if inside_fend_expr {
        current_component.insert_str(0, "[[");
    }
    result
        .parts
        .push(InlineFendResultComponent::Unprocessed(current_component));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn simple_test(input: &str, expected: &str) {
        let mut ctx = crate::Context::new();
        let int = crate::interrupt::Never {};
        let mut result = String::new();
        for part in substitute_inline_fend_expressions(input, &mut ctx, &int).parts {
            match part {
                InlineFendResultComponent::Unprocessed(s)
                | InlineFendResultComponent::FendOutput(s)
                | InlineFendResultComponent::FendError(s) => result.push_str(&s),
            }
        }
        if expected == "=" {
            assert_eq!(result, input);
        } else {
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn trivial_tests() {
        simple_test("", "");
        simple_test("a", "a");
    }

    #[test]
    fn longer_unprocessed_test() {
        simple_test(
            "auidhwiaudb   \n\naiusdfba!!! `code`\n\n\n```rust\nfn foo() {}\n```",
            "=",
        );
    }

    #[test]
    fn simple_fend_expr() {
        simple_test("[[1+1]]", "2");
        simple_test("[[2+2]][[6*6]]", "436");
        simple_test("[[a = 5; 3a]]\n[[6a]]", "15\n30");
        simple_test("[[2+\n\r\n2\n\n\r\n]][[1]]", "41");
        simple_test(
            "The answer is [[\n  # let's work out 40 + 2:\n  40+2\n]].",
            "The answer is 42.",
        );
        simple_test("[[]]", "");
        simple_test("[[", "[[");
        simple_test("]]", "]]");
    }

    #[test]
    fn escaped_exprs() {
        simple_test("`[[1+1]]` = [[1+1]]", "`[[1+1]]` = 2");
        simple_test("`[[1+1]]` = [[1+1\n\n]]", "`[[1+1]]` = 2");
        simple_test("```\n[[2+2]]\n```", "=");
    }
}
