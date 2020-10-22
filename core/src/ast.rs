use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::{Base, FormattingStyle, Number};
use crate::parser::ParseOptions;
use crate::scope::Scope;
use crate::value::{ApplyMulHandling, BuiltInFunction, Value};
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum Expr {
    Num(Number),
    Ident(Cow<'static, str>),
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
    Fn(Cow<'static, str>, Box<Expr>),
}

impl Expr {
    pub fn format<I: Interrupt>(&self, int: &I) -> Result<String, IntErr<Never, I>> {
        Ok(match self {
            Self::Num(n) => n.format(int)?.to_string(),
            Self::Ident(ident) => ident.to_string(),
            Self::Parens(x) => format!("({})", x.format(int)?),
            Self::UnaryMinus(x) => format!("(-{})", x.format(int)?),
            Self::UnaryPlus(x) => format!("(+{})", x.format(int)?),
            Self::UnaryDiv(x) => format!("(/{})", x.format(int)?),
            Self::Factorial(x) => format!("{}!", x.format(int)?),
            Self::Add(a, b) | Self::ImplicitAdd(a, b) => {
                format!("({}+{})", a.format(int)?, b.format(int)?)
            }
            Self::Sub(a, b) => format!("({}-{})", a.format(int)?, b.format(int)?),
            Self::Mul(a, b) => format!("({}*{})", a.format(int)?, b.format(int)?),
            Self::Div(a, b) => format!("({}/{})", a.format(int)?, b.format(int)?),
            Self::Pow(a, b) => format!("({}^{})", a.format(int)?, b.format(int)?),
            Self::Apply(a, b) => format!("({} ({}))", a.format(int)?, b.format(int)?),
            Self::ApplyFunctionCall(a, b) | Self::ApplyMul(a, b) => {
                format!("({} {})", a.format(int)?, b.format(int)?)
            }
            Self::As(a, b) => format!("({} as {})", a.format(int)?, b.format(int)?),
            Self::Fn(a, b) => {
                if a.contains('.') {
                    format!("({}:{})", a, b.format(int)?)
                } else {
                    format!("\\{}.{}", a, b.format(int)?)
                }
            }
        })
    }
}

/// returns true if rhs is '-1' or '(-1)'
fn should_compute_inverse(rhs: &Expr) -> bool {
    if let Expr::UnaryMinus(inner) = &*rhs {
        if let Expr::Num(n) = &**inner {
            if n.is_unitless_one() {
                return true;
            }
        }
    } else if let Expr::Parens(inner) = &*rhs {
        if let Expr::UnaryMinus(inner2) = &**inner {
            if let Expr::Num(n) = &**inner2 {
                if n.is_unitless_one() {
                    return true;
                }
            }
        }
    }
    false
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
        Expr::Ident(ident) => resolve_identifier(ident.as_ref(), scope, options, int)?,
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
        Expr::Sub(a, b) => {
            let a = eval!(*a)?;
            match a {
                Value::Num(a) => Value::Num(a.sub(eval!(*b)?.expect_num()?, int)?),
                f @ Value::BuiltInFunction(_) | f @ Value::Fn(_, _, _) => f.apply(
                    Expr::UnaryMinus(b),
                    ApplyMulHandling::OnlyApply,
                    scope,
                    options,
                    int,
                )?,
                _ => Err("Invalid operands for subtraction".to_string())?,
            }
        }
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
        Expr::Pow(a, b) => {
            let lhs = eval!(*a)?;
            if should_compute_inverse(&*b) {
                let result = match &lhs {
                    Value::BuiltInFunction(f) => Some(f.invert()?),
                    Value::Fn(_, _, _) => {
                        return Err(
                            "Inverses of lambda functions are not currently supported".to_string()
                        )?
                    }
                    _ => None,
                };
                if let Some(res) = result {
                    return Ok(res);
                }
            }
            lhs.handle_two_nums(
                eval!(*b)?,
                |a, b| a.pow(b, int),
                |a| |f| Expr::Pow(f, Box::new(Expr::Num(a))),
                |a| |f| Expr::Pow(Box::new(Expr::Num(a)), f),
                scope,
            )?
        }
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
        Expr::Fn(a, b) => Value::Fn(a, Box::new(*b), scope.clone()),
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
            _ => scope.get(ident.to_string(), int)?,
        });
    }
    Ok(match ident {
        "pi" | "π" => Value::Num(Number::pi()),
        "tau" | "τ" => Value::Num(Number::pi().mul(2.into(), int)?),
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
        "fraction" | "frac" => Value::Format(FormattingStyle::ExactFraction),
        "mixed_fraction" => Value::Format(FormattingStyle::MixedFraction),
        "float" => Value::Format(FormattingStyle::ExactFloat),
        "dp" => Value::Dp,
        "base" => Value::BuiltInFunction(BuiltInFunction::Base),
        "decimal" => Value::Base(Base::from_plain_base(10).map_err(|e| e.to_string())?),
        "hex" | "hexadecimal" => Value::Base(Base::from_plain_base(16).map_err(|e| e.to_string())?),
        "binary" => Value::Base(Base::from_plain_base(2).map_err(|e| e.to_string())?),
        "octal" => Value::Base(Base::from_plain_base(8).map_err(|e| e.to_string())?),
        "version" => Value::Version,
        _ => scope.get(ident.to_string(), int)?,
    })
}
