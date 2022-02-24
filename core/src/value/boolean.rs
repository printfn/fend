use super::ValueTrait;
use crate::error::FendError;

impl ValueTrait for bool {
    fn type_name(&self) -> &'static str {
        "bool"
    }

    fn format(&self, _indent: usize, spans: &mut Vec<crate::Span>) {
        spans.push(crate::Span {
            string: self.to_string(),
            kind: crate::SpanKind::Boolean,
        });
    }

    fn as_bool(&self) -> Result<bool, FendError> {
        Ok(*self)
    }
}
