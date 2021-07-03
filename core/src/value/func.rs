use super::{Value, ValueTrait};
use crate::error::FendError;
use std::fmt;

#[derive(Clone)]
pub(crate) struct Func {
    name: &'static str,
    f: for<'a> fn(Value) -> Result<Value, FendError>,
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

    fn format(&self, _indent: usize, spans: &mut Vec<crate::Span>) {
        spans.push(crate::Span {
            string: self.name.to_string(),
            kind: crate::SpanKind::BuiltInFunction,
        });
    }

    fn apply<'a>(&self, arg: Value) -> Option<Result<Value, String>> {
        let res = match (self.f)(arg) {
            Ok(v) => v,
            Err(msg) => return Some(Err(msg.to_string())),
        };
        Some(Ok(res))
    }
}

pub(crate) const NOT: Func = Func {
    name: "not",
    f: |val| Ok((!val.expect_dyn()?.as_bool()?).into()),
};

pub(crate) const CONJUGATE: Func = Func {
    name: "conjugate",
    f: |val| Ok(Value::Num(Box::new(val.expect_num()?.conjugate()))),
};
