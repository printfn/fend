use crate::ast::Expr2;
use crate::err::{IntErr, Interrupt, Never};
use crate::num::{Base, FormattedNumber, FormattingStyle, Number};
use crate::scope::Scope;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Num(Number),
    BuiltInFunction(BuiltInFunction),
    Format(FormattingStyle),
    Dp,
    Sf,
    Base(Base),
    // user-defined function with a named parameter
    Fn(&'a str, Box<Expr2<'a>>, Option<Arc<Scope<'a>>>),
    Version,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BuiltInFunction {
    Approximately,
    Abs,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Ln,
    Log2,
    Log10,
    Base,
}

impl BuiltInFunction {
    pub fn wrap_with_expr<'a>(
        self,
        lazy_fn: impl FnOnce(Box<Expr2>) -> Expr2,
        scope: Option<Arc<Scope<'a>>>,
    ) -> Value<'a> {
        Value::Fn(
            "x",
            Box::new(lazy_fn(Box::new(Expr2::ApplyFunctionCall(
                Box::new(Expr2::Ident(self.as_str())),
                Box::new(Expr2::Ident("x")),
            )))),
            scope,
        )
    }

    pub fn invert(self) -> Result<Value<'static>, String> {
        Ok(match self {
            Self::Sin => Value::BuiltInFunction(Self::Asin),
            Self::Cos => Value::BuiltInFunction(Self::Acos),
            Self::Tan => Value::BuiltInFunction(Self::Atan),
            Self::Asin => Value::BuiltInFunction(Self::Sin),
            Self::Acos => Value::BuiltInFunction(Self::Cos),
            Self::Atan => Value::BuiltInFunction(Self::Tan),
            Self::Sinh => Value::BuiltInFunction(Self::Asinh),
            Self::Cosh => Value::BuiltInFunction(Self::Acosh),
            Self::Tanh => Value::BuiltInFunction(Self::Atanh),
            Self::Asinh => Value::BuiltInFunction(Self::Sinh),
            Self::Acosh => Value::BuiltInFunction(Self::Cosh),
            Self::Atanh => Value::BuiltInFunction(Self::Tanh),
            _ => return Err(format!("Unable to invert function {}", self)),
        })
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Approximately => "approximately",
            Self::Abs => "abs",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Asinh => "asinh",
            Self::Acosh => "acosh",
            Self::Atanh => "atanh",
            Self::Ln => "ln",
            Self::Log2 => "log2",
            Self::Log10 => "log10",
            Self::Base => "base",
        }
    }
}

impl fmt::Display for BuiltInFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ApplyMulHandling {
    OnlyApply,
    Both,
}

impl<'a> Value<'a> {
    pub fn expect_num<I: Interrupt>(self) -> Result<Number, IntErr<String, I>> {
        match self {
            Self::Num(bigrat) => Ok(bigrat),
            _ => Err("Expected a number".to_string())?,
        }
    }

    pub fn handle_num<I: Interrupt>(
        self,
        eval_fn: impl FnOnce(Number) -> Result<Number, IntErr<String, I>>,
        lazy_fn: impl FnOnce(Box<Expr2>) -> Expr2,
        scope: Option<Arc<Scope<'a>>>,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(match self {
            Self::Num(n) => Self::Num(eval_fn(n)?),
            Self::Fn(param, expr, scope) => Self::Fn(param, Box::new(lazy_fn(expr)), scope),
            Self::BuiltInFunction(f) => f.wrap_with_expr(lazy_fn, scope),
            _ => return Err("Expected a number".to_string())?,
        })
    }

    pub fn handle_two_nums<
        I: Interrupt,
        F1: FnOnce(Box<Expr2>) -> Expr2,
        F2: FnOnce(Box<Expr2>) -> Expr2,
    >(
        self,
        rhs: Self,
        eval_fn: impl FnOnce(Number, Number) -> Result<Number, IntErr<String, I>>,
        lazy_fn_lhs: impl FnOnce(Number) -> F1,
        lazy_fn_rhs: impl FnOnce(Number) -> F2,
        scope: Option<Arc<Scope<'a>>>,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(match (self, rhs) {
            (Self::Num(a), Self::Num(b)) => Self::Num(eval_fn(a, b)?),
            (Self::BuiltInFunction(f), Self::Num(a)) => f.wrap_with_expr(lazy_fn_lhs(a), scope),
            (Self::Num(a), Self::BuiltInFunction(f)) => f.wrap_with_expr(lazy_fn_rhs(a), scope),
            (Self::Fn(param, expr, scope), Self::Num(a)) => {
                Self::Fn(param, Box::new(lazy_fn_lhs(a)(expr)), scope)
            }
            (Self::Num(a), Self::Fn(param, expr, scope)) => {
                Self::Fn(param, Box::new(lazy_fn_rhs(a)(expr)), scope)
            }
            _ => return Err("Expected a number".to_string())?,
        })
    }

    pub fn apply<I: Interrupt>(
        self,
        other: Expr2<'a>,
        apply_mul_handling: ApplyMulHandling,
        scope: Option<Arc<Scope<'a>>>,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(match self {
            Self::Num(n) => {
                let other = crate::ast::evaluate(other, scope.clone(), int)?;
                if let Self::Dp = other {
                    let num = Self::Num(n).expect_num()?.try_as_usize(int)?;
                    return Ok(Self::Format(FormattingStyle::DecimalPlaces(num)));
                }
                if let Self::Sf = other {
                    let num = Self::Num(n).expect_num()?.try_as_usize(int)?;
                    if num == 0 {
                        return Err(
                            "Cannot format a number with zero significant figures.".to_string()
                        )?;
                    }
                    return Ok(Self::Format(FormattingStyle::SignificantFigures(num)));
                }
                if apply_mul_handling == ApplyMulHandling::OnlyApply {
                    let self_ = Self::Num(n);
                    return Err(format!(
                        "{} is not a function",
                        self_.format(int)?.to_string()
                    ))?;
                } else {
                    let n2 = n.clone();
                    other.handle_num(
                        |x| n.mul(x, int).map_err(IntErr::into_string),
                        |x| Expr2::Mul(Box::new(Expr2::Num(n2)), x),
                        scope,
                    )?
                }
            }
            Self::BuiltInFunction(name) => {
                let other = crate::ast::evaluate(other, scope.clone(), int)?;
                Self::Num(match name {
                    BuiltInFunction::Approximately => other.expect_num()?.make_approximate(),
                    BuiltInFunction::Abs => other.expect_num()?.abs(int)?,
                    BuiltInFunction::Sin => other.expect_num()?.sin(scope, int)?,
                    BuiltInFunction::Cos => other.expect_num()?.cos(scope, int)?,
                    BuiltInFunction::Tan => other.expect_num()?.tan(scope, int)?,
                    BuiltInFunction::Asin => other.expect_num()?.asin(int)?,
                    BuiltInFunction::Acos => other.expect_num()?.acos(int)?,
                    BuiltInFunction::Atan => other.expect_num()?.atan(int)?,
                    BuiltInFunction::Sinh => other.expect_num()?.sinh(int)?,
                    BuiltInFunction::Cosh => other.expect_num()?.cosh(int)?,
                    BuiltInFunction::Tanh => other.expect_num()?.tanh(int)?,
                    BuiltInFunction::Asinh => other.expect_num()?.asinh(int)?,
                    BuiltInFunction::Acosh => other.expect_num()?.acosh(int)?,
                    BuiltInFunction::Atanh => other.expect_num()?.atanh(int)?,
                    BuiltInFunction::Ln => other.expect_num()?.ln(int)?,
                    BuiltInFunction::Log2 => other.expect_num()?.log2(int)?,
                    BuiltInFunction::Log10 => other.expect_num()?.log10(int)?,
                    BuiltInFunction::Base => {
                        use std::convert::TryInto;
                        let n: u8 = other
                            .expect_num()?
                            .try_as_usize(int)?
                            .try_into()
                            .map_err(|_| "Unable to convert number to a valid base".to_string())?;
                        return Ok(Self::Base(
                            Base::from_plain_base(n).map_err(|e| e.to_string())?,
                        ));
                    }
                })
            }
            Self::Fn(param, expr, custom_scope) => {
                let new_scope = Scope::with_variable(param, other, scope.clone(), custom_scope);
                return Ok(crate::ast::evaluate(*expr, Some(Arc::new(new_scope)), int)?);
            }
            _ => {
                return Err(format!(
                    "'{}' is not a function or a number",
                    self.format(int)?.to_string()
                ))?;
            }
        })
    }

    pub fn format<I: Interrupt>(&self, int: &I) -> Result<FormattedValue, IntErr<Never, I>> {
        Ok(match self {
            Self::Num(n) => FormattedValue::Number(Box::new(n.format(int)?)),
            Self::BuiltInFunction(name) => FormattedValue::Str(name.as_str()),
            Self::Format(fmt) => FormattedValue::String(fmt.to_string()),
            Self::Dp => FormattedValue::Str("dp"),
            Self::Sf => FormattedValue::Str("sf"),
            Self::Base(b) => FormattedValue::Base(b.base_as_u8()),
            Self::Fn(name, expr, _scope) => {
                let expr_str = (&**expr).format(int)?;
                let res = if name.contains('.') {
                    format!("{}:{}", name, expr_str)
                } else {
                    format!("\\{}.{}", name, expr_str)
                };
                FormattedValue::String(res)
            }
            Self::Version => FormattedValue::Str(crate::get_version_as_str()),
        })
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum FormattedValue {
    Str(&'static str),
    String(String),
    Base(u8),
    Number(Box<FormattedNumber>),
}

impl fmt::Display for FormattedValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{}", s),
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Base(n) => write!(f, "base {}", n),
        }
    }
}
