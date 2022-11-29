use crate::error::{FendError, Interrupt};
use crate::eval::evaluate_to_value;
use crate::ident::Ident;
use crate::interrupt::test_int;
use crate::num::{Base, FormattingStyle, Number};
use crate::scope::Scope;
use crate::serialize::{deserialize_u8, serialize_u8};
use crate::value::{built_in_function::BuiltInFunction, ApplyMulHandling, Value};
use crate::Attrs;
use std::sync::Arc;
use std::{fmt, io};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BitwiseBop {
    And,
    Or,
    Xor,
    LeftShift,
    RightShift,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Bop {
    Plus,
    ImplicitPlus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
    Bitwise(BitwiseBop),
    Combination,
    Permutation,
}

impl Bop {
    pub(crate) fn serialize(self, write: &mut impl io::Write) -> Result<(), FendError> {
        let n = match self {
            Self::Plus => 0,
            Self::ImplicitPlus => 1,
            Self::Minus => 2,
            Self::Mul => 3,
            Self::Div => 4,
            Self::Mod => 5,
            Self::Pow => 6,
            Self::Bitwise(BitwiseBop::And) => 7,
            Self::Bitwise(BitwiseBop::Or) => 8,
            Self::Bitwise(BitwiseBop::Xor) => 9,
            Self::Bitwise(BitwiseBop::LeftShift) => 10,
            Self::Bitwise(BitwiseBop::RightShift) => 11,
            Self::Combination => 12,
            Self::Permutation => 13,
        };
        serialize_u8(n, write)?;
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(match deserialize_u8(read)? {
            0 => Self::Plus,
            1 => Self::ImplicitPlus,
            2 => Self::Minus,
            3 => Self::Mul,
            4 => Self::Div,
            5 => Self::Mod,
            6 => Self::Pow,
            7 => Self::Bitwise(BitwiseBop::And),
            8 => Self::Bitwise(BitwiseBop::Or),
            9 => Self::Bitwise(BitwiseBop::Xor),
            10 => Self::Bitwise(BitwiseBop::LeftShift),
            11 => Self::Bitwise(BitwiseBop::RightShift),
            12 => Self::Combination,
            13 => Self::Permutation,
            _ => return Err(FendError::DeserializationError),
        })
    }
}

impl fmt::Display for Bop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Plus => "+",
            Self::ImplicitPlus => " ",
            Self::Minus => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => " mod ",
            Self::Pow => "^",
            Self::Bitwise(BitwiseBop::And) => "&",
            Self::Bitwise(BitwiseBop::Or) => "|",
            Self::Bitwise(BitwiseBop::Xor) => " xor ",
            Self::Bitwise(BitwiseBop::LeftShift) => "<<",
            Self::Bitwise(BitwiseBop::RightShift) => ">>",
            Self::Combination => "nCr",
            Self::Permutation => "nPr",
        };
        write!(f, "{s}")
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

impl Expr {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        match self {
            Self::Literal(x) => {
                serialize_u8(0, write)?;
                x.serialize(write)?;
            }
            Self::Ident(i) => {
                serialize_u8(1, write)?;
                i.serialize(write)?;
            }
            Self::Parens(e) => {
                serialize_u8(2, write)?;
                e.serialize(write)?;
            }
            Self::UnaryMinus(e) => {
                serialize_u8(3, write)?;
                e.serialize(write)?;
            }
            Self::UnaryPlus(e) => {
                serialize_u8(4, write)?;
                e.serialize(write)?;
            }
            Self::UnaryDiv(e) => {
                serialize_u8(5, write)?;
                e.serialize(write)?;
            }
            Self::Factorial(e) => {
                serialize_u8(6, write)?;
                e.serialize(write)?;
            }
            Self::Bop(op, a, b) => {
                serialize_u8(7, write)?;
                op.serialize(write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::Apply(a, b) => {
                serialize_u8(8, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::ApplyFunctionCall(a, b) => {
                serialize_u8(9, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::ApplyMul(a, b) => {
                serialize_u8(10, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::As(a, b) => {
                serialize_u8(11, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::Fn(a, b) => {
                serialize_u8(12, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::Of(a, b) => {
                serialize_u8(13, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::Assign(a, b) => {
                serialize_u8(14, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
            Self::Statements(a, b) => {
                serialize_u8(15, write)?;
                a.serialize(write)?;
                b.serialize(write)?;
            }
        }
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(match deserialize_u8(read)? {
            0 => Self::Literal(Value::deserialize(read)?),
            1 => Self::Ident(Ident::deserialize(read)?),
            2 => Self::Parens(Box::new(Self::deserialize(read)?)),
            3 => Self::UnaryMinus(Box::new(Self::deserialize(read)?)),
            4 => Self::UnaryPlus(Box::new(Self::deserialize(read)?)),
            5 => Self::UnaryDiv(Box::new(Self::deserialize(read)?)),
            6 => Self::Factorial(Box::new(Self::deserialize(read)?)),
            7 => Self::Bop(
                Bop::deserialize(read)?,
                Box::new(Self::deserialize(read)?),
                Box::new(Self::deserialize(read)?),
            ),
            8 => Self::Apply(
                Box::new(Self::deserialize(read)?),
                Box::new(Self::deserialize(read)?),
            ),
            9 => Self::ApplyFunctionCall(
                Box::new(Self::deserialize(read)?),
                Box::new(Self::deserialize(read)?),
            ),
            10 => Self::ApplyMul(
                Box::new(Self::deserialize(read)?),
                Box::new(Self::deserialize(read)?),
            ),
            11 => Self::As(
                Box::new(Self::deserialize(read)?),
                Box::new(Self::deserialize(read)?),
            ),
            12 => Self::Fn(
                Ident::deserialize(read)?,
                Box::new(Self::deserialize(read)?),
            ),
            13 => Self::Of(
                Ident::deserialize(read)?,
                Box::new(Self::deserialize(read)?),
            ),
            14 => Self::Assign(
                Ident::deserialize(read)?,
                Box::new(Self::deserialize(read)?),
            ),
            15 => Self::Statements(
                Box::new(Self::deserialize(read)?),
                Box::new(Self::deserialize(read)?),
            ),
            _ => return Err(FendError::DeserializationError),
        })
    }

    pub(crate) fn format<I: Interrupt>(
        &self,
        attrs: Attrs,
        ctx: &crate::Context,
        int: &I,
    ) -> Result<String, FendError> {
        Ok(match self {
            Self::Literal(Value::String(s)) => format!(r#""{}""#, s.as_ref()),
            Self::Literal(v) => v.format_to_plain_string(0, attrs, ctx, int)?,
            Self::Ident(ident) => ident.to_string(),
            Self::Parens(x) => format!("({})", x.format(attrs, ctx, int)?),
            Self::UnaryMinus(x) => format!("(-{})", x.format(attrs, ctx, int)?),
            Self::UnaryPlus(x) => format!("(+{})", x.format(attrs, ctx, int)?),
            Self::UnaryDiv(x) => format!("(/{})", x.format(attrs, ctx, int)?),
            Self::Factorial(x) => format!("{}!", x.format(attrs, ctx, int)?),
            Self::Bop(op, a, b) => {
                format!(
                    "({}{op}{})",
                    a.format(attrs, ctx, int)?,
                    b.format(attrs, ctx, int)?
                )
            }
            Self::Apply(a, b) => format!(
                "({} ({}))",
                a.format(attrs, ctx, int)?,
                b.format(attrs, ctx, int)?
            ),
            Self::ApplyFunctionCall(a, b) | Self::ApplyMul(a, b) => {
                format!(
                    "({} {})",
                    a.format(attrs, ctx, int)?,
                    b.format(attrs, ctx, int)?
                )
            }
            Self::As(a, b) => format!(
                "({} as {})",
                a.format(attrs, ctx, int)?,
                b.format(attrs, ctx, int)?
            ),
            Self::Fn(a, b) => {
                if a.as_str().contains('.') {
                    format!("({a}:{})", b.format(attrs, ctx, int)?)
                } else {
                    format!("\\{a}.{}", b.format(attrs, ctx, int)?)
                }
            }
            Self::Of(a, b) => format!("{a} of {}", b.format(attrs, ctx, int)?),
            Self::Assign(a, b) => format!("{a} = {}", b.format(attrs, ctx, int)?),
            Self::Statements(a, b) => format!(
                "{}; {}",
                a.format(attrs, ctx, int)?,
                b.format(attrs, ctx, int)?
            ),
        })
    }
}

/// returns true if rhs is '-1' or '(-1)'
fn should_compute_inverse<I: Interrupt>(rhs: &Expr, int: &I) -> Result<bool, FendError> {
    if let Expr::UnaryMinus(inner) = rhs {
        if let Expr::Literal(Value::Num(n)) = &**inner {
            if n.is_unitless_one(int)? {
                return Ok(true);
            }
        }
    } else if let Expr::Parens(inner) = rhs {
        if let Expr::UnaryMinus(inner2) = &**inner {
            if let Expr::Literal(Value::Num(n)) = &**inner2 {
                if n.is_unitless_one(int)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn evaluate<I: Interrupt>(
    expr: Expr,
    scope: Option<Arc<Scope>>,
    attrs: Attrs,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    macro_rules! eval {
        ($e:expr) => {
            evaluate($e, scope.clone(), attrs, context, int)
        };
    }
    test_int(int)?;
    Ok(match expr {
        Expr::Literal(v) => v,
        Expr::Ident(ident) => resolve_identifier(&ident, scope, attrs, context, int)?,
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
                Value::Date(a) => a.sub(eval!(*b)?)?,
                f @ (Value::BuiltInFunction(_) | Value::Fn(_, _, _)) => f.apply(
                    Expr::UnaryMinus(b),
                    ApplyMulHandling::OnlyApply,
                    scope,
                    attrs,
                    context,
                    int,
                )?,
                _ => return Err(FendError::InvalidOperandsForSubtraction),
            }
        }
        Expr::Bop(Bop::Pow, a, b) => {
            let lhs = eval!(*a)?;
            if should_compute_inverse(&b, int)? {
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
            |a, b| a.bop(bop, b, attrs, context, int),
            |a| |f| Expr::Bop(bop, f, Box::new(Expr::Literal(Value::Num(Box::new(a))))),
            |a| |f| Expr::Bop(bop, Box::new(Expr::Literal(Value::Num(Box::new(a)))), f),
            scope,
        )?,
        Expr::Apply(a, b) | Expr::ApplyMul(a, b) => {
            if let (Expr::Ident(a), Expr::Ident(b)) = (&*a, &*b) {
                let ident = format!("{a}_{b}");
                if let Ok(val) = crate::units::query_unit_static(&ident, attrs, context, int) {
                    return Ok(val);
                }
            }
            eval!(*a)?.apply(*b, ApplyMulHandling::Both, scope, attrs, context, int)?
        }
        Expr::ApplyFunctionCall(a, b) => {
            eval!(*a)?.apply(*b, ApplyMulHandling::OnlyApply, scope, attrs, context, int)?
        }
        Expr::As(a, b) => evaluate_as(*a, *b, scope, attrs, context, int)?,
        Expr::Fn(a, b) => Value::Fn(a, b, scope),
        Expr::Of(a, b) => eval!(*b)?.get_object_member(&a)?,
        Expr::Assign(a, b) => {
            let rhs = evaluate(*b, scope, attrs, context, int)?;
            context.variables.insert(a.to_string(), rhs.clone());
            rhs
        }
        Expr::Statements(a, b) => {
            let _lhs = evaluate(*a, scope.clone(), attrs, context, int)?;
            evaluate(*b, scope, attrs, context, int)?
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
        (Value::Date(d), b) => d.add(b)?,
        _ => return Err(FendError::ExpectedANumber),
    })
}

fn evaluate_as<I: Interrupt>(
    a: Expr,
    b: Expr,
    scope: Option<Arc<Scope>>,
    attrs: Attrs,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    if let Expr::Ident(ident) = &b {
        match ident.as_str() {
            "bool" | "boolean" => {
                let num = evaluate(a, scope, attrs, context, int)?.expect_num()?;
                return Ok(Value::Bool(!num.is_zero()));
            }
            "date" => {
                let a = evaluate(a, scope, attrs, context, int)?;
                return if let Value::String(s) = a {
                    Ok(Value::Date(crate::date::Date::parse(s.as_ref())?))
                } else {
                    Err(FendError::ExpectedAString)
                };
            }
            "string" => {
                return Ok(Value::String(
                    evaluate(a, scope, attrs, context, int)?
                        .format_to_plain_string(0, attrs, context, int)?
                        .into(),
                ));
            }
            "codepoint" => {
                let a = evaluate(a, scope, attrs, context, int)?;
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
    Ok(match evaluate(b, scope.clone(), attrs, context, int)? {
        Value::Num(b) => Value::Num(Box::new(
            evaluate(a, scope, attrs, context, int)?
                .expect_num()?
                .convert_to(*b, int)?,
        )),
        Value::Format(fmt) => Value::Num(Box::new(
            evaluate(a, scope, attrs, context, int)?
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
            evaluate(a, scope, attrs, context, int)?
                .expect_num()?
                .with_base(base),
        )),
        other => {
            return Err(FendError::CannotConvertValueTo(other.type_name()));
        }
    })
}

pub(crate) fn resolve_identifier<I: Interrupt>(
    ident: &Ident,
    scope: Option<Arc<Scope>>,
    attrs: Attrs,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    macro_rules! eval_box {
        ($input:expr) => {
            Box::new(evaluate_to_value(
                $input,
                scope.clone(),
                attrs,
                context,
                int,
            )?)
        };
    }
    if let Some(scope) = scope.clone() {
        if let Some(val) = scope.get(ident, attrs, context, int)? {
            return Ok(val);
        }
    }
    if let Some(val) = context.variables.get(ident.as_str()) {
        return Ok(val.clone());
    }
    Ok(match ident.as_str() {
        "pi" | "\u{3c0}" => Value::Num(Box::new(Number::pi())),
        "tau" | "\u{3c4}" => Value::Num(Box::new(Number::pi().mul(2.into(), int)?)),
        "e" => evaluate_to_value("approx. 2.718281828459045235", scope, attrs, context, int)?,
        "phi" => evaluate_to_value("(1 + sqrt(5))/2", scope, attrs, context, int)?,
        "i" => Value::Num(Box::new(Number::i())),
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "sample" | "roll" => Value::BuiltInFunction(BuiltInFunction::Sample),
        "sqrt" => evaluate_to_value("x: x^(1/2)", scope, attrs, context, int)?,
        "cbrt" => evaluate_to_value("x: x^(1/3)", scope, attrs, context, int)?,
        "conjugate" => Value::BuiltInFunction(BuiltInFunction::Conjugate),
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
        "cis" => evaluate_to_value(
            "theta => cos theta + i * sin theta",
            scope,
            attrs,
            context,
            int,
        )?,
        "ln" => Value::BuiltInFunction(BuiltInFunction::Ln),
        "log2" => Value::BuiltInFunction(BuiltInFunction::Log2),
        "log" | "log10" => Value::BuiltInFunction(BuiltInFunction::Log10),
        "not" => Value::BuiltInFunction(BuiltInFunction::Not),
        "exp" => evaluate_to_value("x: e^x", scope, attrs, context, int)?,
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
        "square" => evaluate_to_value("x: x^2", scope, attrs, context, int)?,
        "cubic" => evaluate_to_value("x: x^3", scope, attrs, context, int)?,
        "earth" => Value::Object(vec![
            ("axial_tilt".into(), eval_box!("23.4392811 degrees")),
            ("eccentricity".into(), eval_box!("0.0167086")),
            ("escape_velocity".into(), eval_box!("11.186 km/s")),
            ("gravity".into(), eval_box!("9.80665 m/s^2")),
            ("mass".into(), eval_box!("5.97237e24 kg")),
            ("volume".into(), eval_box!("1.08321e12 km^3")),
        ]),
        "today" => Value::Date(crate::date::Date::today(context)?),
        "tomorrow" => Value::Date(crate::date::Date::today(context)?.next()),
        "yesterday" => Value::Date(crate::date::Date::today(context)?.prev()),
        _ => return crate::units::query_unit(ident.as_str(), attrs, context, int),
    })
}
