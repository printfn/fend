use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::{Base, FormattingStyle, Number};
use crate::parser::ParseOptions;
use crate::scope::Scope;
use crate::value::Value;
use std::fmt::{Debug, Error, Formatter};

#[derive(Clone, Debug)]
pub enum Expr {
    Num(Number),
    Ident(String),
    Parens(Box<Expr>),
    UnaryMinus(Box<Expr>),
    UnaryPlus(Box<Expr>),
    UnaryDiv(Box<Expr>),
    Factorial(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    ImplicitAdd(Box<Expr>, Box<Expr>),
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
    Fn(String, Box<Expr>),
}

impl Expr {
    pub fn format<I: Interrupt>(&self, f: &mut Formatter, int: &I) -> Result<(), IntErr<Error, I>> {
        let g = |x: &Expr| -> Result<String, IntErr<Error, I>> {
            Ok(crate::num::to_string(|f| (*x).format(f, int))?)
        };
        match self {
            Self::Num(n) => n.format(f, int)?,
            Self::Ident(ident) => write!(f, "{}", ident)?,
            Self::Parens(x) => write!(f, "({})", g(x)?)?,
            Self::UnaryMinus(x) => write!(f, "(-{})", g(x)?)?,
            Self::UnaryPlus(x) => write!(f, "(+{})", g(x)?)?,
            Self::UnaryDiv(x) => write!(f, "(/{})", g(x)?)?,
            Self::Factorial(x) => write!(f, "{}!", g(x)?)?,
            Self::Add(a, b) | Self::ImplicitAdd(a, b) => write!(f, "({}+{})", g(a)?, g(b)?)?,
            Self::Sub(a, b) => write!(f, "({}-{})", g(a)?, g(b)?)?,
            Self::Mul(a, b) => write!(f, "({}*{})", g(a)?, g(b)?)?,
            Self::Div(a, b) => write!(f, "({}/{})", g(a)?, g(b)?)?,
            Self::Pow(a, b) => write!(f, "({}^{})", g(a)?, g(b)?)?,
            Self::Apply(a, b) => write!(f, "({} ({}))", g(a)?, g(b)?)?,
            Self::ApplyFunctionCall(a, b) | Self::ApplyMul(a, b) => {
                write!(f, "({} {})", g(a)?, g(b)?)?
            }
            Self::As(a, b) => write!(f, "({} as {})", g(a)?, g(b)?)?,
            Self::Fn(a, b) => write!(f, "\\{}.{}", a, g(b)?)?,
        }
        Ok(())
    }
}

pub fn evaluate<I: Interrupt>(
    expr: Expr,
    scope: &mut Scope,
    options: ParseOptions,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    macro_rules! eval {
        ($e:expr) => {
            evaluate($e, scope, options, int)
        };
    }
    test_int(int)?;
    Ok(match expr {
        Expr::Num(n) => Value::Num(n),
        Expr::Ident(ident) => resolve_identifier(ident.as_str(), scope, options, int)?,
        Expr::Parens(x) => eval!(*x)?,
        Expr::UnaryMinus(x) => Value::Num(-eval!(*x)?.expect_num()?),
        Expr::UnaryPlus(x) => Value::Num(eval!(*x)?.expect_num()?),
        Expr::UnaryDiv(x) => Value::Num(
            Number::from(1)
                .div(eval!(*x)?.expect_num()?, int)
                .map_err(IntErr::into_string)?,
        ),
        Expr::Factorial(x) => Value::Num(eval!(*x)?.expect_num()?.factorial(int)?),
        Expr::Add(a, b) | Expr::ImplicitAdd(a, b) => Value::Num(
            eval!(*a)?
                .expect_num()?
                .add(eval!(*b)?.expect_num()?, int)?,
        ),
        Expr::Sub(a, b) => Value::Num(
            eval!(*a)?
                .expect_num()?
                .sub(eval!(*b)?.expect_num()?, int)?,
        ),
        Expr::Mul(a, b) => Value::Num(
            eval!(*a)?
                .expect_num()?
                .mul(eval!(*b)?.expect_num()?, int)?,
        ),
        Expr::ApplyMul(a, b) => eval!(*a)?.apply(*b, true, true, scope, options, int)?,
        Expr::Div(a, b) => Value::Num(
            eval!(*a)?
                .expect_num()?
                .div(eval!(*b)?.expect_num()?, int)
                .map_err(IntErr::into_string)?,
        ),
        Expr::Pow(a, b) => Value::Num(
            eval!(*a)?
                .expect_num()?
                .pow(eval!(*b)?.expect_num()?, int)?,
        ),
        Expr::Apply(a, b) => eval!(*a)?.apply(*b, true, false, scope, options, int)?,
        Expr::ApplyFunctionCall(a, b) => eval!(*a)?.apply(*b, false, false, scope, options, int)?,
        Expr::As(a, b) => match eval!(*b)? {
            Value::Num(b) => Value::Num(eval!(*a)?.expect_num()?.convert_to(b, int)?),
            Value::Format(fmt) => Value::Num(eval!(*a)?.expect_num()?.with_format(fmt)),
            Value::Dp => {
                return Err(
                    "You need to specify what number of decimal places to use, e.g. '10 dp'"
                        .to_string(),
                )?;
            }
            Value::Base(base) => Value::Num(eval!(*a)?.expect_num()?.with_base(base)),
            Value::Func(_) | Value::Fn(_, _, _) => {
                return Err("Unable to convert value to a function".to_string())?;
            }
        },
        Expr::Fn(a, b) => Value::Fn(a, *b, scope.clone()),
    })
}

fn eval<I: Interrupt>(
    input: &'static str,
    scope: &mut Scope,
    int: &I,
) -> Result<Value, IntErr<Never, I>> {
    let options = crate::parser::ParseOptions::default();
    crate::eval::evaluate_to_value(input, options, scope, int).map_err(crate::err::IntErr::unwrap)
}

fn resolve_identifier<I: Interrupt>(
    ident: &str,
    scope: &mut Scope,
    options: ParseOptions,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    if options.gnu_compatible {
        return Ok(match ident {
            "exp" => eval("x: e^x", scope, int)?,
            "sqrt" => eval("x: x^(1/2)", scope, int)?,
            "ln" => Value::Func("ln"),
            "log2" => Value::Func("log2"),
            "log10" => Value::Func("log10"),
            // sin and cos are only needed for tan
            "sin" => Value::Func("sin"),
            "cos" => Value::Func("cos"),
            "tan" => eval("x: (sin x)/(cos x)", scope, int)?,
            "asin" => Value::Func("asin"),
            "approx." | "approximately" => Value::Func("approximately"),
            _ => scope.get(ident, int)?,
        });
    }
    Ok(match ident {
        "pi" => eval("approx. 3.141592653589793238", scope, int)?,
        "e" => eval("approx. 2.718281828459045235", scope, int)?,
        "i" => Value::Num(Number::i()),
        // TODO: we want to forward any interrupt, but panic on any other error
        // or statically prove that no other error can occur
        //"c" => eval("299792458 m / s", scope, int)?,
        "sqrt" => eval("x: x^(1/2)", scope, int)?,
        "cbrt" => eval("x: x^(1/3)", scope, int)?,
        "abs" => Value::Func("abs"),
        "sin" => Value::Func("sin"),
        "cos" => Value::Func("cos"),
        "tan" => eval("x: (sin x)/(cos x)", scope, int)?,
        "asin" => Value::Func("asin"),
        "acos" => Value::Func("acos"),
        "atan" => Value::Func("atan"),
        "sinh" => Value::Func("sinh"),
        "cosh" => Value::Func("cosh"),
        "tanh" => Value::Func("tanh"),
        "asinh" => Value::Func("asinh"),
        "acosh" => Value::Func("acosh"),
        "atanh" => Value::Func("atanh"),
        "ln" => Value::Func("ln"),
        "log2" => Value::Func("log2"),
        "log10" => Value::Func("log10"),
        "exp" => eval("x: e^x", scope, int)?,
        "approx." | "approximately" => Value::Func("approximately"),
        "auto" => Value::Format(FormattingStyle::Auto),
        "exact" => Value::Format(FormattingStyle::ExactFloatWithFractionFallback),
        "fraction" => Value::Format(FormattingStyle::ExactFraction),
        "mixed_fraction" => Value::Format(FormattingStyle::MixedFraction),
        "float" => Value::Format(FormattingStyle::ExactFloat),
        "dp" => Value::Dp,
        "base" => Value::Func("base"),
        "decimal" => Value::Base(Base::from_plain_base(10).map_err(|e| e.to_string())?),
        "hex" | "hexadecimal" => Value::Base(Base::from_plain_base(16).map_err(|e| e.to_string())?),
        "binary" => Value::Base(Base::from_plain_base(2).map_err(|e| e.to_string())?),
        "octal" => Value::Base(Base::from_plain_base(8).map_err(|e| e.to_string())?),
        _ => scope.get(ident, int)?,
    })
}
