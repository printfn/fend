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
            Self::Num(n) => write!(f, "{}", n),
            Self::Ident(ident) => write!(f, "{}", ident),
            Self::Parens(x) => write!(f, "({:?})", *x),
            Self::UnaryMinus(x) => write!(f, "(-{:?})", *x),
            Self::UnaryPlus(x) => write!(f, "(+{:?})", *x),
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

pub fn evaluate(expr: Expr, scope: &HashMap<String, Value>) -> Result<Value, String> {
    Ok(match expr {
        Expr::Num(n) => Value::Num(n),
        Expr::Ident(ident) => resolve_identifier(ident.as_str(), scope)?,
        Expr::Parens(x) => evaluate(*x, scope)?,
        Expr::UnaryMinus(x) => Value::Num(-evaluate(*x, scope)?.expect_num()?),
        Expr::UnaryPlus(x) => Value::Num(evaluate(*x, scope)?.expect_num()?),
        Expr::Add(a, b) => Value::Num(
            evaluate(*a, scope)?
                .expect_num()?
                .add(evaluate(*b, scope)?.expect_num()?)?,
        ),
        Expr::Sub(a, b) => Value::Num(
            evaluate(*a, scope)?
                .expect_num()?
                .sub(evaluate(*b, scope)?.expect_num()?)?,
        ),
        Expr::Mul(a, b) => {
            Value::Num(evaluate(*a, scope)?.expect_num()? * evaluate(*b, scope)?.expect_num()?)
        }
        Expr::ApplyMul(a, b) => evaluate(*a, scope)?.apply(&evaluate(*b, scope)?, true, true)?,
        Expr::Div(a, b) => Value::Num(
            evaluate(*a, scope)?
                .expect_num()?
                .div(evaluate(*b, scope)?.expect_num()?)?,
        ),
        Expr::Pow(a, b) => Value::Num(
            evaluate(*a, scope)?
                .expect_num()?
                .pow(evaluate(*b, scope)?.expect_num()?)?,
        ),
        Expr::Apply(a, b) => evaluate(*a, scope)?.apply(&evaluate(*b, scope)?, true, false)?,
        Expr::ApplyFunctionCall(a, b) => {
            evaluate(*a, scope)?.apply(&evaluate(*b, scope)?, false, false)?
        }
        Expr::As(a, b) => match evaluate(*b, scope)? {
            Value::Num(b) => Value::Num(evaluate(*a, scope)?.expect_num()?.convert_to(b)?),
            Value::Format(fmt) => Value::Num(evaluate(*a, scope)?.expect_num()?.with_format(fmt)),
            Value::Dp => Value::Num(
                evaluate(*a, scope)?
                    .expect_num()?
                    .with_format(FormattingStyle::ApproxFloat(10)),
            ),
            Value::Func(_) => {
                return Err("Unable to convert value to a function".to_string());
            }
        },
    })
}

fn resolve_identifier(ident: &str, scope: &HashMap<String, Value>) -> Result<Value, String> {
    Ok(match ident {
        "pi" => Value::Num(Number::approx_pi()),
        "e" => Value::Num(Number::approx_e()),
        "i" => Value::Num(Number::i()),
        "c" => crate::evaluate_to_value("299792458 m / s", scope).unwrap(),
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
                return Err(format!("Unknown identifier '{}'", ident));
            }
        }
    })
}
