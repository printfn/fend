use super::ValueTrait;

impl ValueTrait for () {
    fn type_name(&self) -> &'static str {
        "()"
    }

    fn format(&self, _indent: usize, spans: &mut Vec<crate::Span>) {
        spans.push(crate::Span {
            string: "()".to_string(),
            kind: crate::SpanKind::Ident,
        });
    }
}
