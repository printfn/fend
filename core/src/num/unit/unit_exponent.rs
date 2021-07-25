use std::fmt;

use crate::{
    error::FendError,
    num::{
        complex::{self, Complex, UseParentheses},
        Base, Exact, FormattingStyle,
    },
    Interrupt,
};

use super::named_unit::NamedUnit;

#[derive(Clone)]
pub(crate) struct UnitExponent {
    pub(crate) unit: NamedUnit,
    pub(crate) exponent: Complex,
}

impl UnitExponent {
    pub(crate) fn new(unit: NamedUnit, exponent: impl Into<Complex>) -> Self {
        Self {
            unit,
            exponent: exponent.into(),
        }
    }

    pub(crate) fn format<I: Interrupt>(
        &self,
        base: Base,
        format: FormattingStyle,
        plural: bool,
        invert_exp: bool,
        int: &I,
    ) -> Result<Exact<FormattedExponent<'_>>, FendError> {
        let (prefix, name) = self.unit.prefix_and_name(plural);
        let exp = if invert_exp {
            -self.exponent.clone()
        } else {
            self.exponent.clone()
        };
        let (exact, exponent) = if exp == 1.into() {
            (true, None)
        } else {
            let formatted =
                exp.format(true, format, base, UseParentheses::IfComplexOrFraction, int)?;
            (formatted.exact, Some(formatted.value))
        };
        Ok(Exact::new(
            FormattedExponent {
                prefix,
                name,
                number: exponent,
            },
            exact,
        ))
    }
}

impl fmt::Debug for UnitExponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.unit)?;
        if !self.exponent.is_definitely_one() {
            write!(f, "^{:?}", self.exponent)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct FormattedExponent<'a> {
    prefix: &'a str,
    name: &'a str,
    number: Option<complex::Formatted>,
}

impl<'a> fmt::Display for FormattedExponent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.name.replace('_', " "))?;
        if let Some(number) = &self.number {
            write!(f, "^{}", number)?;
        }
        Ok(())
    }
}
