use crate::err::{IntErr, Interrupt};
use crate::interrupt::test_int;
use crate::num::{FormattingStyle, Number};
use crate::value::Value;
use std::{
    collections::HashMap,
    fmt::{Debug, Error, Formatter},
};

#[derive(Clone)]
pub enum Expr {
    Num(Number),
    Ident(String),
    Parens(Box<Expr>),
    UnaryMinus(Box<Expr>),
    UnaryPlus(Box<Expr>),
    Factorial(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    // Call a function or multiply the expressions
    Apply(Box<Expr>, Box<Expr>),
    // Call a function, or throw an error if lhs is not a function
    ApplyFunctionCall(Box<Expr>, Box<Expr>),
    // Multiply the expressions
    ApplyMul(Box<Expr>, Box<Expr>),

    As(Box<Expr>, Box<Expr>),
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Num(n) => write!(f, "{:?}", n),
            Self::Ident(ident) => write!(f, "{}", ident),
            Self::Parens(x) => write!(f, "({:?})", *x),
            Self::UnaryMinus(x) => write!(f, "(-{:?})", *x),
            Self::UnaryPlus(x) => write!(f, "(+{:?})", *x),
            Self::Factorial(x) => write!(f, "{:?}!", *x),
            Self::Add(a, b) => write!(f, "({:?}+{:?})", *a, *b),
            Self::Sub(a, b) => write!(f, "({:?}-{:?})", *a, *b),
            Self::Mul(a, b) => write!(f, "({:?}*{:?})", *a, *b),
            Self::Div(a, b) => write!(f, "({:?}/{:?})", *a, *b),
            Self::Pow(a, b) => write!(f, "({:?}^{:?})", *a, *b),
            Self::Apply(a, b) => write!(f, "({:?} ({:?}))", *a, *b),
            Self::ApplyFunctionCall(a, b) | Self::ApplyMul(a, b) => {
                write!(f, "({:?} {:?})", *a, *b)
            }
            Self::As(a, b) => write!(f, "({:?} as {:?})", *a, *b),
        }
    }
}

pub fn evaluate<I: Interrupt>(
    expr: Expr,
    scope: &HashMap<String, Value>,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    test_int(int)?;
    Ok(match expr {
        Expr::Num(n) => Value::Num(n),
        Expr::Ident(ident) => resolve_identifier(ident.as_str(), scope, int)?,
        Expr::Parens(x) => evaluate(*x, scope, int)?,
        Expr::UnaryMinus(x) => Value::Num(-evaluate(*x, scope, int)?.expect_num()?),
        Expr::UnaryPlus(x) => Value::Num(evaluate(*x, scope, int)?.expect_num()?),
        Expr::Factorial(x) => Value::Num(evaluate(*x, scope, int)?.expect_num()?.factorial(int)?),
        Expr::Add(a, b) => Value::Num(
            evaluate(*a, scope, int)?
                .expect_num()?
                .add(evaluate(*b, scope, int)?.expect_num()?, int)?,
        ),
        Expr::Sub(a, b) => Value::Num(
            evaluate(*a, scope, int)?
                .expect_num()?
                .sub(evaluate(*b, scope, int)?.expect_num()?, int)?,
        ),
        Expr::Mul(a, b) => Value::Num(
            evaluate(*a, scope, int)?
                .expect_num()?
                .mul(evaluate(*b, scope, int)?.expect_num()?, int)?,
        ),
        Expr::ApplyMul(a, b) => {
            evaluate(*a, scope, int)?.apply(&evaluate(*b, scope, int)?, true, true, int)?
        }
        Expr::Div(a, b) => Value::Num(
            evaluate(*a, scope, int)?
                .expect_num()?
                .div(evaluate(*b, scope, int)?.expect_num()?, int)?,
        ),
        Expr::Pow(a, b) => Value::Num(
            evaluate(*a, scope, int)?
                .expect_num()?
                .pow(evaluate(*b, scope, int)?.expect_num()?, int)?,
        ),
        Expr::Apply(a, b) => {
            evaluate(*a, scope, int)?.apply(&evaluate(*b, scope, int)?, true, false, int)?
        }
        Expr::ApplyFunctionCall(a, b) => {
            evaluate(*a, scope, int)?.apply(&evaluate(*b, scope, int)?, false, false, int)?
        }
        Expr::As(a, b) => match evaluate(*b, scope, int)? {
            Value::Num(b) => {
                Value::Num(evaluate(*a, scope, int)?.expect_num()?.convert_to(b, int)?)
            }
            Value::Format(fmt) => {
                Value::Num(evaluate(*a, scope, int)?.expect_num()?.with_format(fmt))
            }
            Value::Dp => Value::Num(
                evaluate(*a, scope, int)?
                    .expect_num()?
                    .with_format(FormattingStyle::ApproxFloat(10)),
            ),
            Value::Func(_) => {
                return Err("Unable to convert value to a function".to_string())?;
            }
        },
    })
}

fn resolve_identifier<I: Interrupt>(
    ident: &str,
    scope: &HashMap<String, Value>,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    Ok(match ident {
        "pi" => Value::Num(Number::approx_pi()),
        "e" => Value::Num(Number::approx_e()),
        "i" => Value::Num(Number::i()),
        // TODO: we want to forward any interrupt, but panic on any other error
        // or statically prove that no other error can occur
        "c" => crate::eval::evaluate_to_value("299792458 m / s", scope, int)
            .map_err(crate::err::IntErr::unwrap)?,
        "sqrt" => Value::Func("sqrt".to_string()),
        "cbrt" => Value::Func("cbrt".to_string()),
        "abs" => Value::Func("abs".to_string()),
        "sin" => Value::Func("sin".to_string()),
        "cos" => Value::Func("cos".to_string()),
        "tan" => Value::Func("tan".to_string()),
        "asin" => Value::Func("asin".to_string()),
        "acos" => Value::Func("acos".to_string()),
        "atan" => Value::Func("atan".to_string()),
        "sinh" => Value::Func("sinh".to_string()),
        "cosh" => Value::Func("cosh".to_string()),
        "tanh" => Value::Func("tanh".to_string()),
        "asinh" => Value::Func("asinh".to_string()),
        "acosh" => Value::Func("acosh".to_string()),
        "atanh" => Value::Func("atanh".to_string()),
        "ln" => Value::Func("ln".to_string()),
        "log2" => Value::Func("log2".to_string()),
        "log10" => Value::Func("log10".to_string()),
        "exp" => Value::Func("exp".to_string()),
        "approx." | "approximately" => Value::Func("approximately".to_string()),
        "auto" => Value::Format(FormattingStyle::Auto),
        "exact" => Value::Format(FormattingStyle::ExactFloatWithFractionFallback),
        "fraction" => Value::Format(FormattingStyle::ExactFraction),
        "float" => Value::Format(FormattingStyle::ExactFloat),
        "dp" => Value::Dp,
        _ => {
            if let Some(value) = scope.get(&ident.to_string()) {
                value.clone()
            } else {
                return Err(format!("Unknown identifier '{}'", ident))?;
            }
        }
    })
}
