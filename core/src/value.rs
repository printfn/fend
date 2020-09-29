use crate::ast::Expr;
use crate::err::{IntErr, Interrupt};
use crate::num::{Base, FormattingStyle, Number};
use crate::{parser::ParseOptions, scope::Scope};
use std::fmt::{Error, Formatter};

#[derive(Debug, Clone)]
pub enum Value {
    Num(Number),
    // built-in function
    Func(&'static str),
    Format(FormattingStyle),
    Dp,
    Base(Base),
    // user-defined function with a named parameter
    Fn(String, Expr, Scope),
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
        other: Expr,
        allow_multiplication: bool,
        force_multiplication: bool,
        scope: &mut Scope,
        options: ParseOptions,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(Self::Num(match self {
            Self::Num(n) => {
                let other = crate::ast::evaluate(other, scope, options, int)?;
                if let Self::Dp = other {
                    let num = self.expect_num()?.try_as_usize(int)?;
                    return Ok(Self::Format(FormattingStyle::ApproxFloat(num)));
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
                let other = crate::ast::evaluate(other, scope, options, int)?;
                let name = *name;
                if force_multiplication {
                    return Err(format!(
                        "Cannot apply function '{}' in this context",
                        crate::num::to_string(|f| self.format(f, int))?
                    ))?;
                } else if name == "approximately" {
                    other.expect_num()?.make_approximate()
                } else if name == "abs" {
                    other.expect_num()?.abs(int)?
                } else if name == "sin" {
                    other.expect_num()?.sin(scope, int)?
                } else if name == "cos" {
                    other.expect_num()?.cos(scope, int)?
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
                } else if name == "base" {
                    use std::convert::TryInto;
                    let n: u8 = other
                        .expect_num()?
                        .try_as_usize(int)?
                        .try_into()
                        .map_err(|_| "Unable to convert number to a valid base".to_string())?;
                    return Ok(Value::Base(
                        Base::from_plain_base(n).map_err(|e| e.to_string())?,
                    ));
                } else {
                    return Err(format!("Unknown function '{}'", name))?;
                }
            }
            Self::Fn(param, expr, custom_scope) => {
                let mut new_scope = custom_scope.clone().create_nested_scope();
                new_scope.insert_variable(param.clone(), other, scope.clone(), options);
                return Ok(crate::ast::evaluate(
                    expr.clone(),
                    &mut new_scope,
                    options,
                    int,
                )?);
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
            Self::Fn(name, expr, _scope) => {
                let expr_str = crate::num::to_string(|f| expr.format(f, int))?;
                write!(f, "\\{}.{}", name, expr_str)?
            }
        }
        Ok(())
    }
}
