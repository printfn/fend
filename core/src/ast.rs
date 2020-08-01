use crate::num::bigrat::BigRat;
use std::fmt::{Debug, Formatter, Error};

#[derive(Clone)]
pub enum Expr {
    Num(BigRat),
    Ident(String),
    Parens(Box<Expr>),
    Negate(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Apply(Box<Expr>, Box<Expr>),
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Expr::Num(n) => write!(f, "{}", n)?,
            Expr::Ident(ident) => write!(f, "{}", ident)?,
            Expr::Parens(x) => write!(f, "({:?})", *x)?,
            Expr::Negate(x) => write!(f, "(-{:?})", *x)?,
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

pub fn evaluate(expr: Expr) -> Result<BigRat, String> {
    Ok(match expr {
        Expr::Num(n) => n,
        Expr::Ident(_ident) => 0.into(),
        Expr::Parens(x) => evaluate(*x)?,
        Expr::Negate(x) => -evaluate(*x)?,
        Expr::Add(a, b) => evaluate(*a)? + evaluate(*b)?,
        Expr::Sub(a, b) => evaluate(*a)? - evaluate(*b)?,
        Expr::Mul(a, b) => evaluate(*a)? * evaluate(*b)?,
        Expr::Div(a, b) => evaluate(*a)?.div(evaluate(*b)?)?,
        Expr::Pow(a, b) => evaluate(*a)?.pow(evaluate(*b)?)?,
        Expr::Apply(a, b) => evaluate(*a)? * evaluate(*b)?,
    })
}
