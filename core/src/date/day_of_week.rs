use std::fmt;

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
