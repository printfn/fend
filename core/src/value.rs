use crate::err::{IntErr, Interrupt};
use crate::num::{Base, FormattingStyle, Number};
use std::fmt::{Error, Formatter};

#[derive(Debug, Clone)]
pub enum Value {
    Num(Number),
    Func(String),
    Format(FormattingStyle),
    Dp,
    Base(Base),
}

impl Value {
    pub fn expect_num<I: Interrupt>(&self) -> Result<Number, IntErr<String, I>> {
        match self {
            Self::Num(bigrat) => Ok(bigrat.clone()),
            _ => Err("Expected a number".to_string())?,
        }
    }

    pub fn apply<I: Interrupt>(
        &self,
        other: &Self,
        allow_multiplication: bool,
        force_multiplication: bool,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(Self::Num(match self {
            Self::Num(n) => {
                if let Self::Dp = other {
                    let num = self.expect_num()?;
                    return Ok(Self::Format(FormattingStyle::ApproxFloat(
                        num.try_as_usize(int)?,
                    )));
                }
                if allow_multiplication {
                    n.clone().mul(other.expect_num()?, int)?
                } else {
                    return Err(format!(
                        "{} is not a function",
                        crate::num::to_string(|f| self.format(f, int))?
                    ))?;
                }
            }
            Self::Func(name) => {
                if force_multiplication {
                    return Err(format!(
                        "Cannot apply function '{}' in this context",
                        crate::num::to_string(|f| self.format(f, int))?
                    ))?;
                }
                if name == "sqrt" {
                    other.expect_num()?.root_n(&2.into(), int)?
                } else if name == "cbrt" {
                    other.expect_num()?.root_n(&3.into(), int)?
                } else if name == "approximately" {
                    other.expect_num()?.make_approximate()
                } else if name == "abs" {
                    other.expect_num()?.abs(int)?
                } else if name == "sin" {
                    other.expect_num()?.sin(int)?
                } else if name == "cos" {
                    other.expect_num()?.cos(int)?
                } else if name == "tan" {
                    other.expect_num()?.tan(int)?
                } else if name == "asin" {
                    other.expect_num()?.asin(int)?
                } else if name == "acos" {
                    other.expect_num()?.acos(int)?
                } else if name == "atan" {
                    other.expect_num()?.atan(int)?
                } else if name == "sinh" {
                    other.expect_num()?.sinh(int)?
                } else if name == "cosh" {
                    other.expect_num()?.cosh(int)?
                } else if name == "tanh" {
                    other.expect_num()?.tanh(int)?
                } else if name == "asinh" {
                    other.expect_num()?.asinh(int)?
                } else if name == "acosh" {
                    other.expect_num()?.acosh(int)?
                } else if name == "atanh" {
                    other.expect_num()?.atanh(int)?
                } else if name == "ln" {
                    other.expect_num()?.ln(int)?
                } else if name == "log2" {
                    other.expect_num()?.log2(int)?
                } else if name == "log10" {
                    other.expect_num()?.log10(int)?
                } else if name == "exp" {
                    other.expect_num()?.exp(int)?
                } else if name == "base" {
                    use std::convert::TryInto;
                    let n: u8 = other
                        .expect_num()?
                        .try_as_usize(int)?
                        .try_into()
                        .map_err(|_| "Unable to convert number to a valid base".to_string())?;
                    return Ok(Value::Base(Base::from_plain_base(n)?));
                } else {
                    return Err(format!("Unknown function '{}'", name))?;
                }
            }
            _ => {
                return Err(format!(
                    "'{}' is not a function or a number",
                    crate::num::to_string(|f| self.format(f, int))?
                ))?;
            }
        }))
    }

    pub fn format<I: Interrupt>(&self, f: &mut Formatter, int: &I) -> Result<(), IntErr<Error, I>> {
        match self {
            Self::Num(n) => write!(f, "{}", crate::num::to_string(|f| n.format(f, int))?)?,
            Self::Func(name) => write!(f, "{}", name)?,
            Self::Format(fmt) => write!(f, "{}", fmt)?,
            Self::Dp => write!(f, "dp")?,
            Self::Base(b) => write!(f, "base {}", b.base_as_u8())?,
        }
        Ok(())
    }
}
