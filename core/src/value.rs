use crate::ast::Bop;
use crate::error::{FendError, Interrupt};
use crate::num::{Base, FormattingStyle, Number};
use crate::scope::Scope;
use crate::serialize::{
    deserialize_bool, deserialize_string, deserialize_u8, deserialize_usize, serialize_bool,
    serialize_string, serialize_u8, serialize_usize,
};
use crate::{ast::Expr, ident::Ident};
use crate::{Span, SpanKind};
use std::borrow::Cow;
use std::io;
use std::{fmt, sync::Arc};

mod boolean;
pub(crate) mod func;

pub(crate) trait BoxClone {
    fn box_clone(&self) -> Box<dyn ValueTrait>;
}

impl<T: Clone + ValueTrait> BoxClone for T {
    fn box_clone(&self) -> Box<dyn ValueTrait> {
        Box::new(self.clone())
    }
}

pub(crate) trait ValueTrait: fmt::Debug + BoxClone + 'static {
    fn type_name(&self) -> &'static str;

    fn format(&self, indent: usize, spans: &mut Vec<Span>);
    fn get_object_member(&self, _key: &str) -> Option<Value> {
        None
    }

    fn as_bool(&self) -> Result<bool, FendError> {
        Err(FendError::ExpectedABool(self.type_name()))
    }

    fn apply(&self, _arg: Value) -> Option<Result<Value, FendError>> {
        None
    }

    fn add(&self, _rhs: Value) -> Result<Value, FendError> {
        Err(FendError::ExpectedANumber)
    }
}

impl Clone for Box<dyn ValueTrait> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl<T: ValueTrait> From<T> for Value {
    fn from(value: T) -> Self {
        Self::Dynamic(Box::new(value))
    }
}

#[derive(Clone)]
pub(crate) enum Value {
    Num(Box<Number>),
    BuiltInFunction(BuiltInFunction),
    Format(FormattingStyle),
    Dp,
    Sf,
    Base(Base),
    // user-defined function with a named parameter
    Fn(Ident, Box<Expr>, Option<Arc<Scope>>),
    Object(Vec<(Cow<'static, str>, Box<Value>)>),
    String(Cow<'static, str>),
    Dynamic(Box<dyn ValueTrait>),
    Unit, // unit value `()`
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum BuiltInFunction {
    Approximately,
    Abs,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Ln,
    Log2,
    Log10,
    Base,
    Sample,
}

impl BuiltInFunction {
    pub(crate) fn wrap_with_expr(
        self,
        lazy_fn: impl FnOnce(Box<Expr>) -> Expr,
        scope: Option<Arc<Scope>>,
    ) -> Value {
        Value::Fn(
            Ident::new_str("x"),
            Box::new(lazy_fn(Box::new(Expr::ApplyFunctionCall(
                Box::new(Expr::Ident(Ident::new_str(self.as_str()))),
                Box::new(Expr::Ident(Ident::new_str("x"))),
            )))),
            scope,
        )
    }

    pub(crate) fn invert(self) -> Result<Value, FendError> {
        Ok(match self {
            Self::Sin => Value::BuiltInFunction(Self::Asin),
            Self::Cos => Value::BuiltInFunction(Self::Acos),
            Self::Tan => Value::BuiltInFunction(Self::Atan),
            Self::Asin => Value::BuiltInFunction(Self::Sin),
            Self::Acos => Value::BuiltInFunction(Self::Cos),
            Self::Atan => Value::BuiltInFunction(Self::Tan),
            Self::Sinh => Value::BuiltInFunction(Self::Asinh),
            Self::Cosh => Value::BuiltInFunction(Self::Acosh),
            Self::Tanh => Value::BuiltInFunction(Self::Atanh),
            Self::Asinh => Value::BuiltInFunction(Self::Sinh),
            Self::Acosh => Value::BuiltInFunction(Self::Cosh),
            Self::Atanh => Value::BuiltInFunction(Self::Tanh),
            _ => return Err(FendError::UnableToInvertFunction(self.as_str())),
        })
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Approximately => "approximately",
            Self::Abs => "abs",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Asinh => "asinh",
            Self::Acosh => "acosh",
            Self::Atanh => "atanh",
            Self::Ln => "ln",
            Self::Log2 => "log2",
            Self::Log10 => "log10",
            Self::Base => "base",
            Self::Sample => "sample",
        }
    }

    fn try_from_str(s: &str) -> Result<Self, FendError> {
        Ok(match s {
            "approximately" => Self::Approximately,
            "abs" => Self::Abs,
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "asin" => Self::Asin,
            "acos" => Self::Acos,
            "atan" => Self::Atan,
            "sinh" => Self::Sinh,
            "cosh" => Self::Cosh,
            "tanh" => Self::Tanh,
            "asinh" => Self::Asinh,
            "acosh" => Self::Acosh,
            "atanh" => Self::Atanh,
            "ln" => Self::Ln,
            "log2" => Self::Log2,
            "log10" => Self::Log10,
            "base" => Self::Base,
            "sample" => Self::Sample,
            _ => return Err(FendError::DeserializationError),
        })
    }

    pub(crate) fn serialize(self, write: &mut impl io::Write) -> Result<(), FendError> {
        serialize_string(self.as_str(), write)?;
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Self::try_from_str(deserialize_string(read)?.as_str())
    }
}

impl fmt::Display for BuiltInFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ApplyMulHandling {
    OnlyApply,
    Both,
}

impl Value {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        match self {
            Value::Num(n) => {
                serialize_u8(0, write)?;
                n.serialize(write)?;
            }
            Value::BuiltInFunction(f) => {
                serialize_u8(1, write)?;
                f.serialize(write)?;
            }
            Value::Format(f) => {
                serialize_u8(2, write)?;
                f.serialize(write)?;
            }
            Value::Dp => serialize_u8(3, write)?,
            Value::Sf => serialize_u8(4, write)?,
            Value::Base(b) => {
                serialize_u8(5, write)?;
                b.serialize(write)?;
            }
            Value::Fn(i, e, s) => {
                serialize_u8(6, write)?;
                i.serialize(write)?;
                e.serialize(write)?;
                match s {
                    None => serialize_bool(false, write)?,
                    Some(s) => {
                        serialize_bool(true, write)?;
                        s.serialize(write)?;
                    }
                }
            }
            Value::Object(o) => {
                serialize_u8(7, write)?;
                serialize_usize(o.len(), write)?;
                for (k, v) in o {
                    serialize_string(k.as_ref(), write)?;
                    v.serialize(write)?;
                }
            }
            Value::String(s) => {
                serialize_u8(8, write)?;
                serialize_string(s, write)?;
            }
            Value::Unit => serialize_u8(9, write)?,
            Value::Dynamic(_) => {
                // TODO add support for dynamic variables
                return Err(FendError::SerializationError);
            }
        }
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(match deserialize_u8(read)? {
            0 => Self::Num(Box::new(Number::deserialize(read)?)),
            1 => Self::BuiltInFunction(BuiltInFunction::deserialize(read)?),
            2 => Self::Format(FormattingStyle::deserialize(read)?),
            3 => Self::Dp,
            4 => Self::Sf,
            5 => Self::Base(Base::deserialize(read)?),
            6 => Self::Fn(
                Ident::deserialize(read)?,
                Box::new(Expr::deserialize(read)?),
                if deserialize_bool(read)? {
                    None
                } else {
                    Some(Arc::new(Scope::deserialize(read)?))
                },
            ),
            7 => Self::Object({
                let len = deserialize_usize(read)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push((
                        Cow::Owned(deserialize_string(read)?),
                        Box::new(Self::deserialize(read)?),
                    ));
                }
                v
            }),
            8 => Self::String(Cow::Owned(deserialize_string(read)?)),
            9 => Self::Unit,
            // TODO add support for dynamic objects
            _ => return Err(FendError::DeserializationError),
        })
    }

    pub(crate) fn expect_num(self) -> Result<Number, FendError> {
        match self {
            Self::Num(bigrat) => Ok(*bigrat),
            _ => Err(FendError::ExpectedANumber),
        }
    }

    pub(crate) fn expect_dyn(self) -> Result<Box<dyn ValueTrait>, FendError> {
        match self {
            Self::Dynamic(d) => Ok(d),
            _ => Err(FendError::InvalidType),
        }
    }

    pub(crate) fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    pub(crate) fn add_dyn(self, rhs: Self) -> Result<Self, FendError> {
        match self {
            Self::Dynamic(d) => d.add(rhs),
            _ => Err(FendError::ExpectedANumber),
        }
    }

    pub(crate) fn handle_num(
        self,
        eval_fn: impl FnOnce(Number) -> Result<Number, FendError>,
        lazy_fn: impl FnOnce(Box<Expr>) -> Expr,
        scope: Option<Arc<Scope>>,
    ) -> Result<Self, FendError> {
        Ok(match self {
            Self::Num(n) => Self::Num(Box::new(eval_fn(*n)?)),
            Self::Fn(param, expr, scope) => Self::Fn(param, Box::new(lazy_fn(expr)), scope),
            Self::BuiltInFunction(f) => f.wrap_with_expr(lazy_fn, scope),
            _ => return Err(FendError::ExpectedANumber),
        })
    }

    pub(crate) fn handle_two_nums<F1: FnOnce(Box<Expr>) -> Expr, F2: FnOnce(Box<Expr>) -> Expr>(
        self,
        rhs: Self,
        eval_fn: impl FnOnce(Number, Number) -> Result<Number, FendError>,
        lazy_fn_lhs: impl FnOnce(Number) -> F1,
        lazy_fn_rhs: impl FnOnce(Number) -> F2,
        scope: Option<Arc<Scope>>,
    ) -> Result<Self, FendError> {
        Ok(match (self, rhs) {
            (Self::Num(a), Self::Num(b)) => Self::Num(Box::new(eval_fn(*a, *b)?)),
            (Self::BuiltInFunction(f), Self::Num(a)) => f.wrap_with_expr(lazy_fn_lhs(*a), scope),
            (Self::Num(a), Self::BuiltInFunction(f)) => f.wrap_with_expr(lazy_fn_rhs(*a), scope),
            (Self::Fn(param, expr, scope), Self::Num(a)) => {
                Self::Fn(param, Box::new(lazy_fn_lhs(*a)(expr)), scope)
            }
            (Self::Num(a), Self::Fn(param, expr, scope)) => {
                Self::Fn(param, Box::new(lazy_fn_rhs(*a)(expr)), scope)
            }
            _ => return Err(FendError::ExpectedANumber),
        })
    }

    #[allow(clippy::map_err_ignore)]
    pub(crate) fn apply<I: Interrupt>(
        self,
        other: Expr,
        apply_mul_handling: ApplyMulHandling,
        scope: Option<Arc<Scope>>,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        let stringified_self = self.format_to_plain_string(0, context, int)?;
        Ok(match self {
            Self::Num(n) => {
                let other = crate::ast::evaluate(other, scope.clone(), context, int)?;
                if let Self::Dp = other {
                    let num = Self::Num(n).expect_num()?.try_as_usize(int)?;
                    return Ok(Self::Format(FormattingStyle::DecimalPlaces(num)));
                }
                if let Self::Sf = other {
                    let num = Self::Num(n).expect_num()?.try_as_usize(int)?;
                    if num == 0 {
                        return Err(FendError::CannotFormatWithZeroSf);
                    }
                    return Ok(Self::Format(FormattingStyle::SignificantFigures(num)));
                }
                if apply_mul_handling == ApplyMulHandling::OnlyApply {
                    let self_ = Self::Num(n);
                    return Err(FendError::IsNotAFunction(
                        self_.format_to_plain_string(0, context, int)?,
                    ));
                }
                let n2 = n.clone();
                other.handle_num(
                    |x| n.mul(x, int),
                    |x| Expr::Bop(Bop::Mul, Box::new(Expr::Literal(Self::Num(n2))), x),
                    scope,
                )?
            }
            Self::BuiltInFunction(func) => {
                Self::apply_built_in_function(func, other, scope, context, int)?
            }
            Self::Fn(param, expr, custom_scope) => {
                let new_scope = Scope::with_variable(param, other, scope, custom_scope);
                return crate::ast::evaluate(*expr, Some(Arc::new(new_scope)), context, int);
            }
            Self::Dynamic(d) => {
                let other = crate::ast::evaluate(other, scope, context, int)?;
                match d.apply(other) {
                    None => return Err(FendError::IsNotAFunctionOrNumber(stringified_self)),
                    Some(Err(msg)) => return Err(msg),
                    Some(Ok(val)) => val,
                }
            }
            _ => return Err(FendError::IsNotAFunctionOrNumber(stringified_self)),
        })
    }

    fn apply_built_in_function<I: Interrupt>(
        func: BuiltInFunction,
        arg: Expr,
        scope: Option<Arc<Scope>>,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        let arg = crate::ast::evaluate(arg, scope.clone(), context, int)?;
        Ok(Self::Num(Box::new(match func {
            BuiltInFunction::Approximately => arg.expect_num()?.make_approximate(),
            BuiltInFunction::Abs => arg.expect_num()?.abs(int)?,
            BuiltInFunction::Sin => arg.expect_num()?.sin(scope, context, int)?,
            BuiltInFunction::Cos => arg.expect_num()?.cos(scope, context, int)?,
            BuiltInFunction::Tan => arg.expect_num()?.tan(scope, context, int)?,
            BuiltInFunction::Asin => arg.expect_num()?.asin(int)?,
            BuiltInFunction::Acos => arg.expect_num()?.acos(int)?,
            BuiltInFunction::Atan => arg.expect_num()?.atan(int)?,
            BuiltInFunction::Sinh => arg.expect_num()?.sinh(int)?,
            BuiltInFunction::Cosh => arg.expect_num()?.cosh(int)?,
            BuiltInFunction::Tanh => arg.expect_num()?.tanh(int)?,
            BuiltInFunction::Asinh => arg.expect_num()?.asinh(int)?,
            BuiltInFunction::Acosh => arg.expect_num()?.acosh(int)?,
            BuiltInFunction::Atanh => arg.expect_num()?.atanh(int)?,
            BuiltInFunction::Ln => arg.expect_num()?.ln(int)?,
            BuiltInFunction::Log2 => arg.expect_num()?.log2(int)?,
            BuiltInFunction::Log10 => arg.expect_num()?.log10(int)?,
            BuiltInFunction::Base => {
                let n: u8 = arg
                    .expect_num()?
                    .try_as_usize(int)?
                    .try_into()
                    .map_err(|_| FendError::UnableToConvertToBase)?;
                return Ok(Self::Base(Base::from_plain_base(n)?));
            }
            BuiltInFunction::Sample => arg.expect_num()?.sample(context, int)?,
        })))
    }

    pub(crate) fn format_to_plain_string<I: Interrupt>(
        &self,
        indent: usize,
        ctx: &crate::Context,
        int: &I,
    ) -> Result<String, FendError> {
        let mut spans = vec![];
        self.format(indent, &mut spans, ctx, int)?;
        let mut res = String::new();
        for span in spans {
            res.push_str(&span.string);
        }
        Ok(res)
    }

    pub(crate) fn format<I: Interrupt>(
        &self,
        indent: usize,
        spans: &mut Vec<Span>,
        ctx: &crate::Context,
        int: &I,
    ) -> Result<(), FendError> {
        match self {
            Self::Num(n) => {
                n.clone().simplify(int)?.format(ctx, int)?.spans(spans);
            }
            Self::BuiltInFunction(name) => {
                spans.push(Span {
                    string: name.to_string(),
                    kind: SpanKind::BuiltInFunction,
                });
            }
            Self::Format(fmt) => {
                spans.push(Span {
                    string: fmt.to_string(),
                    kind: SpanKind::Keyword,
                });
            }
            Self::Dp => {
                spans.push(Span {
                    string: "dp".to_string(),
                    kind: SpanKind::Keyword,
                });
            }
            Self::Sf => {
                spans.push(Span {
                    string: "sf".to_string(),
                    kind: SpanKind::Keyword,
                });
            }
            Self::Base(b) => {
                spans.push(Span {
                    string: "base ".to_string(),
                    kind: SpanKind::Keyword,
                });
                spans.push(Span {
                    string: b.base_as_u8().to_string(),
                    kind: SpanKind::Number,
                });
            }
            Self::Fn(name, expr, _scope) => {
                let expr_str = (&**expr).format(ctx, int)?;
                let res = if name.as_str().contains('.') {
                    format!("{}:{}", name, expr_str)
                } else {
                    format!("\\{}.{}", name, expr_str)
                };
                spans.push(Span {
                    string: res,
                    kind: SpanKind::Other,
                });
            }
            Self::Object(kv) => {
                spans.push(Span::from_string("{".to_string()));
                for (i, (k, v)) in kv.iter().enumerate() {
                    if i != 0 {
                        spans.push(Span::from_string(",".to_string()));
                    }
                    spans.push(Span::from_string("\n".to_string()));
                    for _ in 0..(indent + 4) {
                        spans.push(Span::from_string(" ".to_string()));
                    }
                    spans.push(Span::from_string(format!("{}: ", k)));
                    v.format(indent + 4, spans, ctx, int)?;
                }
                spans.push(Span::from_string("\n}".to_string()));
            }
            Self::String(s) => {
                spans.push(Span {
                    string: s.to_string(),
                    kind: SpanKind::String,
                });
            }
            Self::Unit => {
                spans.push(crate::Span {
                    string: "()".to_string(),
                    kind: crate::SpanKind::Ident,
                });
            }
            Self::Dynamic(d) => {
                d.format(indent, spans);
            }
        }
        Ok(())
    }

    pub(crate) fn get_object_member(self, key: &Ident) -> Result<Self, FendError> {
        match self {
            Self::Object(kv) => {
                for (k, v) in kv {
                    if k == key.as_str() {
                        return Ok(*v);
                    }
                }
                Err(FendError::CouldNotFindKeyInObject)
            }
            Self::Dynamic(d) => match d.get_object_member(key.as_str()) {
                Some(v) => Ok(v),
                None => Err(FendError::CouldNotFindKey(key.to_string())),
            },
            _ => Err(FendError::ExpectedAnObject),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{:?}", n),
            Self::BuiltInFunction(name) => write!(f, "built-in function: {}", name.as_str()),
            Self::Format(fmt) => write!(f, "format: {:?}", fmt),
            Self::Dp => write!(f, "dp"),
            Self::Sf => write!(f, "sf"),
            Self::Base(b) => write!(f, "base: {:?}", b),
            Self::Fn(name, expr, scope) => {
                write!(f, "fn: {} => {:?} (scope: {:?})", name, expr, scope)
            }
            Self::Object(kv) => {
                let mut s = "{".to_string();
                for (k, v) in kv {
                    s.push_str(k);
                    s.push(':');
                    s.push_str(&format!("{:?}", *v));
                    s.push(',');
                }
                s.push('}');
                write!(f, "{}", s)
            }
            Self::String(s) => write!(f, r#""{}""#, s.as_ref()),
            Self::Unit => write!(f, "()"),
            Self::Dynamic(d) => write!(f, "{:?}", d),
        }
    }
}
