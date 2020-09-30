use std::fmt::{Display, Error, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[must_use]
pub enum FormattingStyle {
    /// Print value as an exact fraction
    ExactFraction,
    /// Print as an exact mixed fraction, e.g. 1 1/2
    MixedFraction,
    /// Print as an exact float, possibly indicating recurring digits
    /// with parentheses, e.g. 7/9 => 0.(81)
    ExactFloat,
    /// If possible, print as an exact float with no recurring digits,
    /// or fall back to an exact fraction
    ExactFloatWithFractionFallback,
    /// Print as an approximate float with up to some number of decimal places
    ApproxFloat(usize),
    /// If exact: ExactFloatWithFractionFallback, otherwise: ApproxFloat(10)
    Auto,
}

impl Default for FormattingStyle {
    fn default() -> Self {
        Self::Auto
    }
}

impl Display for FormattingStyle {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::ExactFraction => write!(f, "fraction"),
            Self::MixedFraction => write!(f, "mixed_fraction"),
            Self::ExactFloat => write!(f, "float"),
            Self::ExactFloatWithFractionFallback => write!(f, "exact"),
            Self::ApproxFloat(d) => write!(f, "{} dp", d),
            Self::Auto => write!(f, "auto"),
        }
    }
}
