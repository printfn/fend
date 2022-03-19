use crate::error::{FendError, Interrupt};
use crate::eval::evaluate_to_value;
use crate::ident::Ident;
use crate::interrupt::test_int;
use crate::num::{Base, FormattingStyle, Number};
use crate::scope::Scope;
use crate::value::{ApplyMulHandling, BuiltInFunction, Value};
use std::fmt;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Bop {
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
}

impl fmt::Display for Bop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, " mod "),
            Self::Pow => write!(f, "^"),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Literal(Value),
    Ident(Ident),
    Parens(Box<Expr>),
    UnaryMinus(Box<Expr>),
    UnaryPlus(Box<Expr>),
    UnaryDiv(Box<Expr>),
    Factorial(Box<Expr>),
    Bop(Bop, Box<Expr>, Box<Expr>),
    // Call a function or multiply the expressions
    Apply(Box<Expr>, Box<Expr>),
    // Call a function, or throw an error if lhs is not a function
    ApplyFunctionCall(Box<Expr>, Box<Expr>),
    // Multiply the expressions
    ApplyMul(Box<Expr>, Box<Expr>),

    As(Box<Expr>, Box<Expr>),
    Fn(Ident, Box<Expr>),

    Of(Ident, Box<Expr>),

    Assign(Ident, Box<Expr>),
    Statements(Box<Expr>, Box<Expr>),
}

impl<'a> Expr {
    pub(crate) fn format<I: Interrupt>(
        &self,
        ctx: &crate::Context,
        int: &I,
    ) -> Result<String, FendError> {
        Ok(match self {
            Self::Literal(Value::String(s)) => format!(r#""{}""#, s.as_ref()),
            Self::Literal(v) => v.format_to_plain_string(0, ctx, int)?,
            Self::Ident(ident) => ident.to_string(),
            Self::Parens(x) => format!("({})", x.format(ctx, int)?),
            Self::UnaryMinus(x) => format!("(-{})", x.format(ctx, int)?),
            Self::UnaryPlus(x) => format!("(+{})", x.format(ctx, int)?),
            Self::UnaryDiv(x) => format!("(/{})", x.format(ctx, int)?),
            Self::Factorial(x) => format!("{}!", x.format(ctx, int)?),
            Self::Bop(op, a, b) => {
                format!("({}{}{})", a.format(ctx, int)?, op, b.format(ctx, int)?)
            }
            Self::Apply(a, b) => format!("({} ({}))", a.format(ctx, int)?, b.format(ctx, int)?),
            Self::ApplyFunctionCall(a, b) | Self::ApplyMul(a, b) => {
                format!("({} {})", a.format(ctx, int)?, b.format(ctx, int)?)
            }
            Self::As(a, b) => format!("({} as {})", a.format(ctx, int)?, b.format(ctx, int)?),
            Self::Fn(a, b) => {
                if a.as_str().contains('.') {
                    format!("({}:{})", a, b.format(ctx, int)?)
                } else {
                    format!("\\{}.{}", a, b.format(ctx, int)?)
                }
            }
            Self::Of(a, b) => format!("{} of {}", a, b.format(ctx, int)?),
            Self::Assign(a, b) => format!("{} = {}", a, b.format(ctx, int)?),
            Self::Statements(a, b) => format!("{}; {}", a.format(ctx, int)?, b.format(ctx, int)?),
        })
    }
}

/// returns true if rhs is '-1' or '(-1)'
fn should_compute_inverse(rhs: &Expr) -> bool {
    if let Expr::UnaryMinus(inner) = &*rhs {
        if let Expr::Literal(Value::Num(n)) = &**inner {
            if n.is_unitless_one() {
                return true;
            }
        }
    } else if let Expr::Parens(inner) = &*rhs {
        if let Expr::UnaryMinus(inner2) = &**inner {
            if let Expr::Literal(Value::Num(n)) = &**inner2 {
                if n.is_unitless_one() {
                    return true;
                }
            }
        }
    }
    false
}

#[allow(clippy::too_many_lines)]
pub(crate) fn evaluate<I: Interrupt>(
    expr: Expr,
    scope: Option<Arc<Scope>>,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    macro_rules! eval {
        ($e:expr) => {
            evaluate($e, scope.clone(), context, int)
        };
    }
    test_int(int)?;
    Ok(match expr {
        Expr::Literal(v) => v,
        Expr::Ident(ident) => resolve_identifier(&ident, scope, context, int)?,
        Expr::Parens(x) => eval!(*x)?,
        Expr::UnaryMinus(x) => eval!(*x)?.handle_num(|x| Ok(-x), Expr::UnaryMinus, scope)?,
        Expr::UnaryPlus(x) => eval!(*x)?.handle_num(Ok, Expr::UnaryPlus, scope)?,
        Expr::UnaryDiv(x) => {
            eval!(*x)?.handle_num(|x| Number::from(1).div(x, int), Expr::UnaryDiv, scope)?
        }
        Expr::Factorial(x) => {
            eval!(*x)?.handle_num(|x| x.factorial(int), Expr::Factorial, scope)?
        }
        Expr::Bop(Bop::Plus, a, b) => evaluate_add(eval!(*a)?, eval!(*b)?, scope, int)?,
        Expr::Bop(Bop::Minus, a, b) => {
            let a = eval!(*a)?;
            match a {
                Value::Num(a) => Value::Num(Box::new(a.sub(eval!(*b)?.expect_num()?, int)?)),
                f @ (Value::BuiltInFunction(_) | Value::Fn(_, _, _)) => f.apply(
                    Expr::UnaryMinus(b),
                    ApplyMulHandling::OnlyApply,
                    scope,
                    context,
                    int,
                )?,
                _ => return Err(FendError::InvalidOperandsForSubtraction),
            }
        }
        Expr::Bop(Bop::Pow, a, b) => {
            let lhs = eval!(*a)?;
            if should_compute_inverse(&*b) {
                let result = match &lhs {
                    Value::BuiltInFunction(f) => Some(f.invert()?),
                    Value::Fn(_, _, _) => return Err(FendError::InversesOfLambdasUnsupported),
                    _ => None,
                };
                if let Some(res) = result {
                    return Ok(res);
                }
            }
            lhs.handle_two_nums(
                eval!(*b)?,
                |a, b| a.pow(b, int),
                |a| {
                    |f| {
                        Expr::Bop(
                            Bop::Pow,
                            f,
                            Box::new(Expr::Literal(Value::Num(Box::new(a)))),
                        )
                    }
                },
                |a| {
                    |f| {
                        Expr::Bop(
                            Bop::Pow,
                            Box::new(Expr::Literal(Value::Num(Box::new(a)))),
                            f,
                        )
                    }
                },
                scope,
            )?
        }
        Expr::Bop(bop, a, b) => eval!(*a)?.handle_two_nums(
            eval!(*b)?,
            |a, b| a.bop(bop, b, int),
            |a| |f| Expr::Bop(bop, f, Box::new(Expr::Literal(Value::Num(Box::new(a))))),
            |a| |f| Expr::Bop(bop, Box::new(Expr::Literal(Value::Num(Box::new(a)))), f),
            scope,
        )?,
        Expr::Apply(a, b) | Expr::ApplyMul(a, b) => {
            if let (Expr::Ident(a), Expr::Ident(b)) = (&*a, &*b) {
                let ident = format!("{}_{}", a, b);
                if let Ok(val) = crate::units::query_unit_static(&ident, context, int) {
                    return Ok(val);
                }
            }
            eval!(*a)?.apply(*b, ApplyMulHandling::Both, scope, context, int)?
        }
        Expr::ApplyFunctionCall(a, b) => {
            eval!(*a)?.apply(*b, ApplyMulHandling::OnlyApply, scope, context, int)?
        }
        Expr::As(a, b) => evaluate_as(*a, *b, scope, context, int)?,
        Expr::Fn(a, b) => Value::Fn(a, b, scope),
        Expr::Of(a, b) => eval!(*b)?.get_object_member(&a)?,
        Expr::Assign(a, b) => {
            let rhs = evaluate(*b, scope, context, int)?;
            context.variables.insert(a.to_string(), rhs.clone());
            rhs
        }
        Expr::Statements(a, b) => {
            let _lhs = evaluate(*a, scope.clone(), context, int)?;
            evaluate(*b, scope, context, int)?
        }
    })
}

fn evaluate_add<I: Interrupt>(
    a: Value,
    b: Value,
    scope: Option<Arc<Scope>>,
    int: &I,
) -> Result<Value, FendError> {
    Ok(match (a, b) {
        (Value::Num(a), Value::Num(b)) => Value::Num(Box::new(a.add(*b, int)?)),
        (Value::String(a), Value::String(b)) => {
            Value::String(format!("{}{}", a.as_ref(), b.as_ref()).into())
        }
        (Value::BuiltInFunction(f), Value::Num(a)) => f.wrap_with_expr(
            |f| Expr::Bop(Bop::Plus, f, Box::new(Expr::Literal(Value::Num(a)))),
            scope,
        ),
        (Value::Num(a), Value::BuiltInFunction(f)) => f.wrap_with_expr(
            |f| Expr::Bop(Bop::Plus, Box::new(Expr::Literal(Value::Num(a))), f),
            scope,
        ),
        (Value::Fn(param, expr, scope), Value::Num(a)) => Value::Fn(
            param,
            Box::new(Expr::Bop(
                Bop::Plus,
                expr,
                Box::new(Expr::Literal(Value::Num(a))),
            )),
            scope,
        ),
        (Value::Num(a), Value::Fn(param, expr, scope)) => Value::Fn(
            param,
            Box::new(Expr::Bop(
                Bop::Plus,
                Box::new(Expr::Literal(Value::Num(a))),
                expr,
            )),
            scope,
        ),
        (a, b) => return a.add_dyn(b),
    })
}

fn evaluate_as<I: Interrupt>(
    a: Expr,
    b: Expr,
    scope: Option<Arc<Scope>>,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    if let Expr::Ident(ident) = &b {
        match ident.as_str() {
            "bool" | "boolean" => {
                let num = evaluate(a, scope, context, int)?.expect_num()?;
                return Ok((!num.is_zero()).into());
            }
            "date" => {
                let a = evaluate(a, scope, context, int)?;
                return if let Value::String(s) = a {
                    Ok(crate::date::Date::parse(s.as_ref())?.into())
                } else {
                    Err(FendError::ExpectedAString)
                };
            }
            "string" => {
                return Ok(Value::String(
                    evaluate(a, scope, context, int)?
                        .format_to_plain_string(0, context, int)?
                        .into(),
                ));
            }
            "codepoint" => {
                let a = evaluate(a, scope, context, int)?;
                if let Value::String(s) = a {
                    let ch = s
                        .as_ref()
                        .chars()
                        .next()
                        .ok_or(FendError::StringCannotBeEmpty)?;
                    if s.len() > ch.len_utf8() {
                        return Err(FendError::StringCannotBeLonger);
                    }
                    let value = Value::Num(Box::new(
                        Number::from(u64::from(ch as u32)).with_base(Base::HEX),
                    ));
                    return Ok(value);
                }
                return Err(FendError::ExpectedAString);
            }
            _ => (),
        }
    }
    Ok(match evaluate(b, scope.clone(), context, int)? {
        Value::Num(b) => Value::Num(Box::new(
            evaluate(a, scope, context, int)?
                .expect_num()?
                .convert_to(*b, int)?,
        )),
        Value::Format(fmt) => Value::Num(Box::new(
            evaluate(a, scope, context, int)?
                .expect_num()?
                .with_format(fmt),
        )),
        Value::Dp => {
            return Err(FendError::SpecifyNumDp);
        }
        Value::Sf => {
            return Err(FendError::SpecifyNumSf);
        }
        Value::Base(base) => Value::Num(Box::new(
            evaluate(a, scope, context, int)?
                .expect_num()?
                .with_base(base),
        )),
        Value::BuiltInFunction(_) | Value::Fn(_, _, _) => {
            return Err(FendError::CannotConvertValueTo("function"));
        }
        Value::Object(_) => {
            return Err(FendError::CannotConvertValueTo("object"));
        }
        Value::String(_) => {
            return Err(FendError::CannotConvertValueTo("string"));
        }
        Value::Dynamic(d) => {
            return Err(FendError::CannotConvertValueTo(d.type_name()));
        }
    })
}

pub(crate) fn resolve_identifier<I: Interrupt>(
    ident: &Ident,
    scope: Option<Arc<Scope>>,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    macro_rules! eval_box {
        ($input:expr) => {
            Box::new(evaluate_to_value($input, scope.clone(), context, int)?)
        };
    }
    if let Some(scope) = scope.clone() {
        match scope.get(ident, context, int) {
            Ok(val) => return Ok(val),
            Err(FendError::IdentifierNotFound(_)) => (),
            Err(err) => return Err(err),
        }
    }
    if let Some(val) = context.variables.get(ident.as_str()) {
        return Ok(val.clone());
    }
    Ok(match ident.as_str() {
        "pi" | "\u{3c0}" => Value::Num(Box::new(Number::pi())),
        "tau" | "\u{3c4}" => Value::Num(Box::new(Number::pi().mul(2.into(), int)?)),
        "e" => evaluate_to_value("approx. 2.718281828459045235", scope, context, int)?,
        "phi" => evaluate_to_value("(1 + sqrt(5))/2", scope, context, int)?,
        "i" => Value::Num(Box::new(Number::i())),
        "true" => Value::from(true),
        "false" => Value::from(false),
        "sample" | "roll" => Value::BuiltInFunction(BuiltInFunction::Sample),
        "sqrt" => evaluate_to_value("x: x^(1/2)", scope, context, int)?,
        "cbrt" => evaluate_to_value("x: x^(1/3)", scope, context, int)?,
        "conjugate" => crate::value::func::CONJUGATE.into(),
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
        "cis" => evaluate_to_value("theta => cos theta + i * sin theta", scope, context, int)?,
        "ln" => Value::BuiltInFunction(BuiltInFunction::Ln),
        "log2" => Value::BuiltInFunction(BuiltInFunction::Log2),
        "log" | "log10" => Value::BuiltInFunction(BuiltInFunction::Log10),
        "not" => crate::value::func::NOT.into(),
        "exp" => evaluate_to_value("x: e^x", scope, context, int)?,
        "approx." | "approximately" => Value::BuiltInFunction(BuiltInFunction::Approximately),
        "auto" => Value::Format(FormattingStyle::Auto),
        "exact" => Value::Format(FormattingStyle::Exact),
        "frac" | "fraction" => Value::Format(FormattingStyle::ImproperFraction),
        "mixed_frac" | "mixed_fraction" => Value::Format(FormattingStyle::MixedFraction),
        "float" => Value::Format(FormattingStyle::ExactFloat),
        "dp" => Value::Dp,
        "sf" => Value::Sf,
        "base" => Value::BuiltInFunction(BuiltInFunction::Base),
        "dec" | "decimal" => Value::Base(Base::from_plain_base(10)?),
        "hex" | "hexadecimal" => Value::Base(Base::from_plain_base(16)?),
        "binary" => Value::Base(Base::from_plain_base(2)?),
        "ternary" => Value::Base(Base::from_plain_base(3)?),
        "senary" | "seximal" => Value::Base(Base::from_plain_base(6)?),
        "oct" | "octal" => Value::Base(Base::from_plain_base(8)?),
        "version" => Value::String(crate::get_version_as_str().into()),
        "square" => evaluate_to_value("x: x^2", scope, context, int)?,
        "cubic" => evaluate_to_value("x: x^3", scope, context, int)?,
        "earth" => Value::Object(vec![
            ("axial_tilt".into(), eval_box!("23.4392811 degrees")),
            ("eccentricity".into(), eval_box!("0.0167086")),
            ("escape_velocity".into(), eval_box!("11.186 km/s")),
            ("gravity".into(), eval_box!("9.80665 m/s^2")),
            ("mass".into(), eval_box!("5.97237e24 kg")),
            ("volume".into(), eval_box!("1.08321e12 km^3")),
        ]),
        "today" => crate::date::Date::today(context)?.into(),
        "tomorrow" => crate::date::Date::today(context)?.next().into(),
        "yesterday" => crate::date::Date::today(context)?.prev().into(),
        _ => return crate::units::query_unit(ident.as_str(), context, int),
    })
}
