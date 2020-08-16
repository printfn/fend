use crate::num::Number;
use crate::value::Value;
use std::fmt::{Debug, Error, Formatter};

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
    Apply(Box<Expr>, Box<Expr>),
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Expr::Num(n) => write!(f, "{}", n)?,
            Expr::Ident(ident) => write!(f, "{}", ident)?,
            Expr::Parens(x) => write!(f, "({:?})", *x)?,
            Expr::UnaryMinus(x) => write!(f, "(-{:?})", *x)?,
            Expr::UnaryPlus(x) => write!(f, "(+{:?})", *x)?,
            Expr::Add(a, b) => write!(f, "({:?}+{:?})", *a, *b)?,
            Expr::Sub(a, b) => write!(f, "({:?}-{:?})", *a, *b)?,
            Expr::Mul(a, b) => write!(f, "({:?}*{:?})", *a, *b)?,
            Expr::Div(a, b) => write!(f, "({:?}/{:?})", *a, *b)?,
            Expr::Pow(a, b) => write!(f, "({:?}^{:?})", *a, *b)?,
            Expr::Apply(a, b) => write!(f, "({:?} {:?})", *a, *b)?,
        };
        Ok(())
    }
}

pub fn evaluate(expr: Expr) -> Result<Value, String> {
    Ok(match expr {
        Expr::Num(n) => Value::Num(n),
        Expr::Ident(ident) => resolve_identifier(ident.as_str())?,
        Expr::Parens(x) => evaluate(*x)?,
        Expr::UnaryMinus(x) => Value::Num(-evaluate(*x)?.expect_num()?),
        Expr::UnaryPlus(x) => Value::Num(evaluate(*x)?.expect_num()?),
        Expr::Add(a, b) => Value::Num(evaluate(*a)?.expect_num()? + evaluate(*b)?.expect_num()?),
        Expr::Sub(a, b) => Value::Num(evaluate(*a)?.expect_num()? - evaluate(*b)?.expect_num()?),
        Expr::Mul(a, b) => Value::Num(evaluate(*a)?.expect_num()? * evaluate(*b)?.expect_num()?),
        Expr::Div(a, b) => Value::Num(
            evaluate(*a)?
                .expect_num()?
                .div(evaluate(*b)?.expect_num()?)?,
        ),
        Expr::Pow(a, b) => Value::Num(
            evaluate(*a)?
                .expect_num()?
                .pow(evaluate(*b)?.expect_num()?)?,
        ),
        Expr::Apply(a, b) => evaluate(*a)?.apply(evaluate(*b)?)?,
    })
}

fn resolve_identifier(ident: &str) -> Result<Value, String> {
    Ok(match ident {
        "pi" => Value::Num(Number::approx_pi()),
        "i" => Value::Num(Number::i()),
        "sqrt" => Value::Func("sqrt".to_string()),
        "cbrt" => Value::Func("cbrt".to_string()),
        "abs" => Value::Func("abs".to_string()),
        "approx." => Value::Func("approximately".to_string()),
        "approximately" => Value::Func("approximately".to_string()),
        _ => return Err(format!("Unknown identifier '{}'", ident)),
    })
}
