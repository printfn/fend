use crate::num::bigrat::BigRat;

#[derive(Debug, Clone)]
pub enum Expr {
    Num(BigRat),
    Parens(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

pub fn evaluate(expr: Expr) -> Result<BigRat, String> {
    Ok(match expr {
        Expr::Num(n) => n,
        Expr::Parens(x) => evaluate(*x)?,
        Expr::Add(a, b) => evaluate(*a)? + evaluate(*b)?,
        Expr::Sub(a, b) => evaluate(*a)? - evaluate(*b)?,
        Expr::Mul(a, b) => evaluate(*a)? * evaluate(*b)?,
        Expr::Div(a, b) => evaluate(*a)?.div(evaluate(*b)?)?,
    })
}
