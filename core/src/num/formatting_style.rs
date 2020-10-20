use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[must_use]
pub enum FormattingStyle {
    /// Print value as an improper fraction
    ExactFraction,
    /// Print as a mixed fraction, e.g. 1 1/2
    MixedFraction,
    /// Print as a float, possibly indicating recurring digits
    /// with parentheses, e.g. 7/9 => 0.(81)
    ExactFloat,
    /// If possible, print as an exact float with no recurring digits,
    /// or fall back to an exact fraction
    ExactFloatWithFractionFallback,
    /// Print with the given number of decimal places
    DecimalPlaces(usize),
    /// If exact: ExactFloatWithFractionFallback, otherwise: DecimalPlaces(10)
    Auto,
}

impl Default for FormattingStyle {
    fn default() -> Self {
        Self::Auto
    }
}

impl fmt::Display for FormattingStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::ExactFraction => write!(f, "fraction"),
            Self::MixedFraction => write!(f, "mixed_fraction"),
            Self::ExactFloat => write!(f, "float"),
            Self::ExactFloatWithFractionFallback => write!(f, "exact"),
            Self::DecimalPlaces(d) => write!(f, "{} dp", d),
            Self::Auto => write!(f, "auto"),
        }
    }
}
