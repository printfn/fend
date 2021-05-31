use std::fmt;
use crate::value::ValueTrait;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl fmt::Debug for DayOfWeek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Sunday => "Sunday",
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ValueTrait for DayOfWeek {
    fn type_name(&self) -> &'static str {
        "month"
    }

    fn box_clone(&self) -> Box<dyn ValueTrait> {
        Box::new(*self)
    }

    fn format(&self, _indent: usize, spans: &mut Vec<crate::Span>) {
        spans.push(crate::Span {
            string: self.to_string(),
            kind: crate::SpanKind::Date,
        });
    }
}
