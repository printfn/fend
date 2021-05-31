use super::{Value, ValueTrait};
use std::fmt;

#[derive(Clone)]
pub(crate) struct Func {
    name: &'static str,
    f: fn(bool) -> Value<'static>,
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ValueTrait for Func {
    fn type_name(&self) -> &'static str {
        "function"
    }

    fn box_clone(&self) -> Box<dyn ValueTrait> {
        Box::new(self.clone())
    }

    fn format(&self, _indent: usize, spans: &mut Vec<crate::Span>) {
        spans.push(crate::Span {
            string: self.name.to_string(),
            kind: crate::SpanKind::BuiltInFunction,
        });
    }

    fn apply(&self, arg: Value<'_>) -> Option<Result<Value<'static>, String>> {
        let dyn_val = match arg.expect_dyn() {
            Ok(b) => b,
            Err(msg) => return Some(Err(msg)),
        };
        let b = match dyn_val.as_bool() {
            Ok(b) => b,
            Err(msg) => return Some(Err(msg)),
        };
        let res = (self.f)(b);
        Some(Ok(res))
    }
}

fn not_fn(val: bool) -> Value<'static> {
    (!val).into()
}

pub(crate) const NOT: Func = Func {
    name: "not",
    f: not_fn,
};
