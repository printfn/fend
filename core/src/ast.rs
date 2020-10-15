use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::{Base, FormattingStyle, Number};
use crate::parser::ParseOptions;
use crate::scope::Scope;
use crate::value::{ApplyMulHandling, BuiltInFunction, Value};
use std::fmt;

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
    pub fn format<I: Interrupt>(
        &self,
        f: &mut fmt::Formatter,
        int: &I,
    ) -> Result<(), IntErr<fmt::Error, I>> {
        let g = |x: &Self| -> Result<String, IntErr<fmt::Error, I>> {
            Ok(crate::num::to_string(|f| (*x).format(f, int))?.0)
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
            Self::Fn(a, b) => {
                if a.contains('.') {
                    write!(f, "({}:{})", a, g(b)?)?
                } else {
                    write!(f, "\\{}.{}", a, g(b)?)?
                }
            }
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
        Expr::UnaryMinus(x) => eval!(*x)?.handle_num(|x| Ok(-x), Expr::UnaryMinus, scope)?,
        Expr::UnaryPlus(x) => eval!(*x)?.handle_num(Ok, Expr::UnaryPlus, scope)?,
        Expr::UnaryDiv(x) => eval!(*x)?.handle_num(
            |x| Number::from(1).div(x, int).map_err(IntErr::into_string),
            Expr::UnaryDiv,
            scope,
        )?,
        Expr::Factorial(x) => {
            eval!(*x)?.handle_num(|x| x.factorial(int), Expr::Factorial, scope)?
        }
        Expr::Add(a, b) | Expr::ImplicitAdd(a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.add(b, int),
            |a| |f| Expr::Add(f, Box::new(Expr::Num(a))),
            |a| |f| Expr::Add(Box::new(Expr::Num(a)), f),
            scope,
        )?,
        Expr::Sub(a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.sub(b, int),
            |a| |f| Expr::Sub(f, Box::new(Expr::Num(a))),
            |a| |f| Expr::Sub(Box::new(Expr::Num(a)), f),
            scope,
        )?,
        Expr::Mul(a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.mul(b, int).map_err(IntErr::into_string),
            |a| |f| Expr::Mul(f, Box::new(Expr::Num(a))),
            |a| |f| Expr::Mul(Box::new(Expr::Num(a)), f),
            scope,
        )?,
        Expr::ApplyMul(a, b) => {
            eval!(*a)?.apply(*b, ApplyMulHandling::Both, scope, options, int)?
        }
        Expr::Div(a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.div(b, int).map_err(IntErr::into_string),
            |a| |f| Expr::Div(f, Box::new(Expr::Num(a))),
            |a| |f| Expr::Div(Box::new(Expr::Num(a)), f),
            scope,
        )?,
        Expr::Pow(a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.pow(b, int),
            |a| |f| Expr::Pow(f, Box::new(Expr::Num(a))),
            |a| |f| Expr::Pow(Box::new(Expr::Num(a)), f),
            scope,
        )?,
        Expr::Apply(a, b) => eval!(*a)?.apply(*b, ApplyMulHandling::Both, scope, options, int)?,
        Expr::ApplyFunctionCall(a, b) => {
            eval!(*a)?.apply(*b, ApplyMulHandling::OnlyApply, scope, options, int)?
        }
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
            Value::BuiltInFunction(_) | Value::Fn(_, _, _) | Value::Version => {
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
            "pi" => Value::Num(Number::pi()),
            "exp" => eval("x: approx. 2.718281828459045235^x", scope, int)?,
            "sqrt" => eval("x: x^(1/2)", scope, int)?,
            "ln" => Value::BuiltInFunction(BuiltInFunction::Ln),
            "log2" => Value::BuiltInFunction(BuiltInFunction::Log2),
            "log10" => Value::BuiltInFunction(BuiltInFunction::Log10),
            // sin and cos are only needed for tan
            "sin" => Value::BuiltInFunction(BuiltInFunction::Sin),
            "cos" => Value::BuiltInFunction(BuiltInFunction::Cos),
            "tan" => Value::BuiltInFunction(BuiltInFunction::Tan),
            "asin" => Value::BuiltInFunction(BuiltInFunction::Asin),
            "approx." | "approximately" => Value::BuiltInFunction(BuiltInFunction::Approximately),
            _ => scope.get(ident, int)?,
        });
    }
    Ok(match ident {
        "pi" => Value::Num(Number::pi()),
        "e" => eval("approx. 2.718281828459045235", scope, int)?,
        "i" => Value::Num(Number::i()),
        // TODO: we want to forward any interrupt, but panic on any other error
        // or statically prove that no other error can occur
        //"c" => eval("299792458 m / s", scope, int)?,
        "sqrt" => eval("x: x^(1/2)", scope, int)?,
        "cbrt" => eval("x: x^(1/3)", scope, int)?,
        "abs" => Value::BuiltInFunction(BuiltInFunction::Abs),
        "sin" => Value::BuiltInFunction(BuiltInFunction::Sin),
        "cos" => Value::BuiltInFunction(BuiltInFunction::Cos),
        "tan" => Value::BuiltInFunction(BuiltInFunction::Tan),
        "asin" => Value::BuiltInFunction(BuiltInFunction::Asin),
        "acos" => Value::BuiltInFunction(BuiltInFunction::Acos),
        "atan" => Value::BuiltInFunction(BuiltInFunction::Atan),
        "sinh" => Value::BuiltInFunction(BuiltInFunction::Sinh),
        "cosh" => Value::BuiltInFunction(BuiltInFunction::Cosh),
        "tanh" => Value::BuiltInFunction(BuiltInFunction::Tanh),
        "asinh" => Value::BuiltInFunction(BuiltInFunction::Asinh),
        "acosh" => Value::BuiltInFunction(BuiltInFunction::Acosh),
        "atanh" => Value::BuiltInFunction(BuiltInFunction::Atanh),
        "ln" => Value::BuiltInFunction(BuiltInFunction::Ln),
        "log2" => Value::BuiltInFunction(BuiltInFunction::Log2),
        "log10" => Value::BuiltInFunction(BuiltInFunction::Log10),
        "exp" => eval("x: e^x", scope, int)?,
        "approx." | "approximately" => Value::BuiltInFunction(BuiltInFunction::Approximately),
        "auto" => Value::Format(FormattingStyle::Auto),
        "exact" => Value::Format(FormattingStyle::ExactFloatWithFractionFallback),
        "fraction" => Value::Format(FormattingStyle::ExactFraction),
        "mixed_fraction" => Value::Format(FormattingStyle::MixedFraction),
        "float" => Value::Format(FormattingStyle::ExactFloat),
        "dp" => Value::Dp,
        "base" => Value::BuiltInFunction(BuiltInFunction::Base),
        "decimal" => Value::Base(Base::from_plain_base(10).map_err(|e| e.to_string())?),
        "hex" | "hexadecimal" => Value::Base(Base::from_plain_base(16).map_err(|e| e.to_string())?),
        "binary" => Value::Base(Base::from_plain_base(2).map_err(|e| e.to_string())?),
        "octal" => Value::Base(Base::from_plain_base(8).map_err(|e| e.to_string())?),
        "version" => Value::Version,
        _ => scope.get(ident, int)?,
    })
}
