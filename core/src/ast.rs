use crate::error::{IntErr, Interrupt, Never};
use crate::eval::evaluate_to_value;
use crate::interrupt::test_int;
use crate::num::{Base, FormattingStyle, Number};
use crate::scope::{GetIdentError, Scope};
use crate::value::{ApplyMulHandling, BuiltInFunction, Value};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) enum Expr<'a> {
    Num(Number<'a>),
    String(Cow<'a, str>),
    Ident(&'a str),
    Parens(Box<Expr<'a>>),
    UnaryMinus(Box<Expr<'a>>),
    UnaryPlus(Box<Expr<'a>>),
    UnaryDiv(Box<Expr<'a>>),
    Factorial(Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    ImplicitAdd(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),
    Pow(Box<Expr<'a>>, Box<Expr<'a>>),
    // Call a function or multiply the expressions
    Apply(Box<Expr<'a>>, Box<Expr<'a>>),
    // Call a function, or throw an error if lhs is not a function
    ApplyFunctionCall(Box<Expr<'a>>, Box<Expr<'a>>),
    // Multiply the expressions
    ApplyMul(Box<Expr<'a>>, Box<Expr<'a>>),

    As(Box<Expr<'a>>, Box<Expr<'a>>),
    Fn(&'a str, Box<Expr<'a>>),

    Of(Vec<&'a str>),
}

impl<'a> Expr<'a> {
    pub(crate) fn format<I: Interrupt>(&self, int: &I) -> Result<String, IntErr<Never, I>> {
        Ok(match self {
            Self::Num(n) => n.format(int)?.to_string(),
            Self::String(s) => format!(r#""{}""#, s.as_ref()),
            Self::Ident(ident) => (*ident).to_string(),
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
            Self::Of(parts) => parts.join(" of "),
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

pub(crate) fn evaluate<'a, I: Interrupt>(
    expr: Expr<'a>,
    scope: Option<Arc<Scope<'a>>>,
    int: &I,
) -> Result<Value<'a>, IntErr<String, I>> {
    macro_rules! eval {
        ($e:expr) => {
            evaluate($e, scope.clone(), int)
        };
    }
    test_int(int)?;
    Ok(match expr {
        Expr::<'a>::Num(n) => Value::Num(n),
        Expr::<'a>::String(s) => Value::String(s),
        Expr::<'a>::Ident(ident) => resolve_identifier(ident, scope, int)?,
        Expr::<'a>::Parens(x) => eval!(*x)?,
        Expr::<'a>::UnaryMinus(x) => eval!(*x)?.handle_num(|x| Ok(-x), Expr::UnaryMinus, scope)?,
        Expr::<'a>::UnaryPlus(x) => eval!(*x)?.handle_num(Ok, Expr::UnaryPlus, scope)?,
        Expr::<'a>::UnaryDiv(x) => eval!(*x)?.handle_num(
            |x| Number::from(1).div(x, int).map_err(IntErr::into_string),
            Expr::UnaryDiv,
            scope,
        )?,
        Expr::<'a>::Factorial(x) => {
            eval!(*x)?.handle_num(|x| x.factorial(int), Expr::Factorial, scope)?
        }
        Expr::<'a>::Add(a, b) | Expr::<'a>::ImplicitAdd(a, b) => {
            evaluate_add(eval!(*a)?, eval!(*b)?, scope, int)?
        }
        Expr::<'a>::Sub(a, b) => {
            let a = eval!(*a)?;
            match a {
                Value::Num(a) => Value::Num(a.sub(eval!(*b)?.expect_num()?, int)?),
                f @ Value::BuiltInFunction(_) | f @ Value::Fn(_, _, _) => f.apply(
                    Expr::<'a>::UnaryMinus(b),
                    ApplyMulHandling::OnlyApply,
                    scope,
                    int,
                )?,
                _ => return Err("Invalid operands for subtraction".to_string().into()),
            }
        }
        Expr::<'a>::Mul(a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.mul(b, int).map_err(IntErr::into_string),
            |a| |f| Expr::Mul(f, Box::new(Expr::Num(a))),
            |a| |f| Expr::Mul(Box::new(Expr::Num(a)), f),
            scope,
        )?,
        Expr::<'a>::Apply(a, b) | Expr::<'a>::ApplyMul(a, b) => {
            eval!(*a)?.apply(*b, ApplyMulHandling::Both, scope, int)?
        }
        Expr::<'a>::Div(a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.div(b, int).map_err(IntErr::into_string),
            |a| |f| Expr::Div(f, Box::new(Expr::Num(a))),
            |a| |f| Expr::Div(Box::new(Expr::Num(a)), f),
            scope,
        )?,
        Expr::<'a>::Pow(a, b) => {
            let lhs = eval!(*a)?;
            if should_compute_inverse(&*b.clone()) {
                let result = match &lhs {
                    Value::BuiltInFunction(f) => Some(f.invert()?),
                    Value::Fn(_, _, _) => {
                        return Err("Inverses of lambda functions are not currently supported"
                            .to_string()
                            .into())
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
        Expr::<'a>::ApplyFunctionCall(a, b) => {
            eval!(*a)?.apply(*b, ApplyMulHandling::OnlyApply, scope, int)?
        }
        Expr::<'a>::As(a, b) => evaluate_as(*a, *b, scope, int)?,
        Expr::<'a>::Fn(a, b) => Value::Fn(a, b, scope),
        Expr::<'a>::Of(parts) => {
            let mut value = resolve_identifier(parts[0], scope, int)?;
            for part in &parts[1..] {
                value = match value.get_object_member(part) {
                    Ok(value) => value,
                    Err(msg) => return Err(msg.to_string().into()),
                };
            }
            value
        }
    })
}

fn evaluate_add<'a, I: Interrupt>(
    a: Value<'a>,
    b: Value<'a>,
    scope: Option<Arc<Scope<'a>>>,
    int: &I,
) -> Result<Value<'a>, IntErr<String, I>> {
    Ok(match (a, b) {
        (Value::Num(a), Value::Num(b)) => Value::Num(a.add(b, int)?),
        (Value::String(a), Value::String(b)) => {
            Value::String(format!("{}{}", a.as_ref(), b.as_ref()).into())
        }
        (Value::BuiltInFunction(f), Value::Num(a)) => {
            f.wrap_with_expr(|f| Expr::Add(f, Box::new(Expr::Num(a))), scope)
        }
        (Value::Num(a), Value::BuiltInFunction(f)) => {
            f.wrap_with_expr(|f| Expr::Add(Box::new(Expr::Num(a)), f), scope)
        }
        (Value::Fn(param, expr, scope), Value::Num(a)) => Value::Fn(
            param,
            Box::new(Expr::Add(expr, Box::new(Expr::Num(a)))),
            scope,
        ),
        (Value::Num(a), Value::Fn(param, expr, scope)) => Value::Fn(
            param,
            Box::new(Expr::Add(Box::new(Expr::Num(a)), expr)),
            scope,
        ),
        _ => return Err("Expected a number".to_string().into()),
    })
}

fn evaluate_as<'a, I: Interrupt>(
    a: Expr<'a>,
    b: Expr<'a>,
    scope: Option<Arc<Scope<'a>>>,
    int: &I,
) -> Result<Value<'a>, IntErr<String, I>> {
    Ok(match evaluate(b, scope.clone(), int)? {
        Value::Num(b) => Value::Num(evaluate(a, scope, int)?.expect_num()?.convert_to(b, int)?),
        Value::Format(fmt) => Value::Num(evaluate(a, scope, int)?.expect_num()?.with_format(fmt)),
        Value::Dp => {
            return Err(
                "You need to specify what number of decimal places to use, e.g. '10 dp'"
                    .to_string()
                    .into(),
            );
        }
        Value::Sf => {
            return Err(
                "You need to specify what number of significant figures to use, e.g. '10 sf'"
                    .to_string()
                    .into(),
            );
        }
        Value::Base(base) => Value::Num(evaluate(a, scope, int)?.expect_num()?.with_base(base)),
        Value::BuiltInFunction(_) | Value::Fn(_, _, _) | Value::Version => {
            return Err("Unable to convert value to a function".to_string().into());
        }
        Value::Object(_) => {
            return Err("Cannot convert value to object".to_string().into());
        }
        Value::String(_) => {
            return Err("Cannot convert value to string".to_string().into());
        }
    })
}

pub(crate) fn resolve_identifier<'a, I: Interrupt>(
    ident: &'a str,
    scope: Option<Arc<Scope<'a>>>,
    int: &I,
) -> Result<Value<'a>, IntErr<String, I>> {
    let eval_box = |input| -> Result<Box<Value<'_>>, IntErr<String, I>> {
        Ok(Box::new(evaluate_to_value(input, scope.clone(), int)?))
    };
    if let Some(scope) = scope.clone() {
        match scope.get(ident, int) {
            Ok(val) => return Ok(val),
            Err(IntErr::Interrupt(int)) => return Err(IntErr::Interrupt(int)),
            Err(IntErr::Error(GetIdentError::IdentifierNotFound(_))) => (),
            Err(IntErr::Error(err @ GetIdentError::EvalError(_))) => {
                return Err(IntErr::Error(err.to_string()))
            }
        }
    }
    Ok(match ident {
        "pi" | "\u{3c0}" => Value::Num(Number::pi()),
        "tau" | "\u{3c4}" => Value::Num(Number::pi().mul(2.into(), int)?),
        "e" => evaluate_to_value("approx. 2.718281828459045235", scope, int)?,
        "i" => Value::Num(Number::i()),
        "sqrt" => evaluate_to_value("x: x^(1/2)", scope, int)?,
        "cbrt" => evaluate_to_value("x: x^(1/3)", scope, int)?,
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
        "cis" => evaluate_to_value("theta => cos theta + i * sin theta", scope, int)?,
        "ln" => Value::BuiltInFunction(BuiltInFunction::Ln),
        "log2" => Value::BuiltInFunction(BuiltInFunction::Log2),
        "log" | "log10" => Value::BuiltInFunction(BuiltInFunction::Log10),
        "exp" => evaluate_to_value("x: e^x", scope, int)?,
        "approx." | "approximately" => Value::BuiltInFunction(BuiltInFunction::Approximately),
        "auto" => Value::Format(FormattingStyle::Auto),
        "exact" => Value::Format(FormattingStyle::Exact),
        "frac" | "fraction" => Value::Format(FormattingStyle::ImproperFraction),
        "mixed_fraction" => Value::Format(FormattingStyle::MixedFraction),
        "float" => Value::Format(FormattingStyle::ExactFloat),
        "dp" => Value::Dp,
        "sf" => Value::Sf,
        "base" => Value::BuiltInFunction(BuiltInFunction::Base),
        "dec" | "decimal" => Value::Base(Base::from_plain_base(10).map_err(|e| e.to_string())?),
        "hex" | "hexadecimal" => Value::Base(Base::from_plain_base(16).map_err(|e| e.to_string())?),
        "binary" => Value::Base(Base::from_plain_base(2).map_err(|e| e.to_string())?),
        "oct" | "octal" => Value::Base(Base::from_plain_base(8).map_err(|e| e.to_string())?),
        "version" => Value::Version,
        "square" => evaluate_to_value("x: x^2", scope, int)?,
        "cubic" => evaluate_to_value("x: x^3", scope, int)?,
        "earth" => Value::Object(vec![
            ("axial_tilt", eval_box("23.4392811 degrees")?),
            ("eccentricity", eval_box("0.0167086")?),
            ("escape_velocity", eval_box("11.186 km/s")?),
            ("gravity", eval_box("9.80665 m/s^2")?),
            ("mass", eval_box("5.97237e24 kg")?),
            ("volume", eval_box("1.08321e12 km^3")?),
        ]),
        "differentiate" => Value::BuiltInFunction(BuiltInFunction::Differentiate),
        _ => return crate::units::query_unit(ident, int).map_err(IntErr::into_string),
    })
}
