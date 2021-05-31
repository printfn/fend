use crate::error::{IntErr, Interrupt};
use crate::num::{Base, FormattingStyle, Number};
use crate::scope::Scope;
use crate::{ast::Expr, ident::Ident};
use crate::{Span, SpanKind};
use std::{borrow, fmt, ops, sync::Arc};

pub(crate) trait ValueTrait: fmt::Debug {
    fn box_clone(&self) -> Box<dyn ValueTrait>;
    fn type_name(&self) -> &'static str;

    fn format(&self, spans: &mut Vec<Span>);
    fn get_object_member(&self, _key: &str) -> Option<Value<'static>> {
        None
    }
}

pub(crate) struct DynValue(Box<dyn ValueTrait>);

impl Clone for DynValue {
    fn clone(&self) -> Self {
        Self(self.0.box_clone())
    }
}

impl ops::Deref for DynValue {
    type Target = dyn ValueTrait;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T: ValueTrait + 'static> From<T> for Value<'static> {
    fn from(value: T) -> Self {
        Self::Dynamic(DynValue(Box::new(value)))
    }
}

#[derive(Clone)]
pub(crate) enum Value<'a> {
    Bool(bool),
    Num(Box<Number<'a>>),
    BuiltInFunction(BuiltInFunction),
    Format(FormattingStyle),
    Dp,
    Sf,
    Base(Base),
    // user-defined function with a named parameter
    Fn(Ident<'a>, Box<Expr<'a>>, Option<Arc<Scope<'a>>>),
    Object(Vec<(&'a str, Box<Value<'a>>)>),
    String(borrow::Cow<'a, str>),
    Dynamic(DynValue),
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
    Differentiate,
    Conjugate,
    Not,
}

impl BuiltInFunction {
    pub(crate) fn wrap_with_expr<'a>(
        self,
        lazy_fn: impl FnOnce(Box<Expr<'a>>) -> Expr<'a>,
        scope: Option<Arc<Scope<'a>>>,
    ) -> Value<'a> {
        Value::Fn(
            Ident::new("x"),
            Box::new(lazy_fn(Box::new(Expr::ApplyFunctionCall(
                Box::new(Expr::Ident(Ident::new(self.as_str()))),
                Box::new(Expr::Ident(Ident::new("x"))),
            )))),
            scope,
        )
    }

    pub(crate) fn invert(self) -> Result<Value<'static>, String> {
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
            _ => return Err(format!("unable to invert function {}", self)),
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
            Self::Differentiate => "differentiate",
            Self::Conjugate => "conjugate",
            Self::Not => "not",
        }
    }

    fn differentiate(self) -> Option<Value<'static>> {
        if self == Self::Sin {
            Some(Value::BuiltInFunction(Self::Cos))
        } else {
            None
        }
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

impl<'a> Value<'a> {
    pub(crate) fn expect_num<I: Interrupt>(self) -> Result<Number<'a>, IntErr<String, I>> {
        match self {
            Self::Num(bigrat) => Ok(*bigrat),
            _ => Err("expected a number".to_string().into()),
        }
    }

    pub(crate) fn expect_bool<I: Interrupt>(self) -> Result<bool, IntErr<String, I>> {
        match self {
            Self::Bool(b) => Ok(b),
            _ => Err("expected a boolean".to_string().into()),
        }
    }

    pub(crate) fn handle_num<I: Interrupt>(
        self,
        eval_fn: impl FnOnce(Number<'a>) -> Result<Number<'a>, IntErr<String, I>>,
        lazy_fn: impl FnOnce(Box<Expr<'a>>) -> Expr<'a>,
        scope: Option<Arc<Scope<'a>>>,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(match self {
            Self::Num(n) => Self::Num(Box::new(eval_fn(*n)?)),
            Self::Fn(param, expr, scope) => Self::Fn(param, Box::new(lazy_fn(expr)), scope),
            Self::BuiltInFunction(f) => f.wrap_with_expr(lazy_fn, scope),
            _ => return Err("expected a number".to_string().into()),
        })
    }

    pub(crate) fn handle_two_nums<
        I: Interrupt,
        F1: FnOnce(Box<Expr<'a>>) -> Expr<'a>,
        F2: FnOnce(Box<Expr<'a>>) -> Expr<'a>,
    >(
        self,
        rhs: Self,
        eval_fn: impl FnOnce(Number<'a>, Number<'a>) -> Result<Number<'a>, IntErr<String, I>>,
        lazy_fn_lhs: impl FnOnce(Number<'a>) -> F1,
        lazy_fn_rhs: impl FnOnce(Number<'a>) -> F2,
        scope: Option<Arc<Scope<'a>>>,
    ) -> Result<Self, IntErr<String, I>> {
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
            _ => return Err("expected a number".to_string().into()),
        })
    }

    #[allow(clippy::map_err_ignore)]
    pub(crate) fn apply<I: Interrupt>(
        self,
        other: Expr<'a>,
        apply_mul_handling: ApplyMulHandling,
        scope: Option<Arc<Scope<'a>>>,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(match self {
            Self::Num(n) => {
                let other = crate::ast::evaluate(other, scope.clone(), context, int)?;
                if let Self::Dp = other {
                    let num = Self::Num(n)
                        .expect_num()?
                        .try_as_usize(int)
                        .map_err(IntErr::into_string)?;
                    return Ok(Self::Format(FormattingStyle::DecimalPlaces(num)));
                }
                if let Self::Sf = other {
                    let num = Self::Num(n)
                        .expect_num()?
                        .try_as_usize(int)
                        .map_err(IntErr::into_string)?;
                    if num == 0 {
                        return Err("cannot format a number with zero significant figures."
                            .to_string()
                            .into());
                    }
                    return Ok(Self::Format(FormattingStyle::SignificantFigures(num)));
                }
                if apply_mul_handling == ApplyMulHandling::OnlyApply {
                    let self_ = Self::Num(n);
                    return Err(format!(
                        "{} is not a function",
                        self_.format_to_plain_string(0, int)?
                    )
                    .into());
                }
                let n2 = n.clone();
                other.handle_num(
                    |x| n.mul(x, int).map_err(IntErr::into_string),
                    |x| Expr::Mul(Box::new(Expr::Literal(Value::Num(n2))), x),
                    scope,
                )?
            }
            Self::BuiltInFunction(func) => {
                Self::apply_built_in_function(func, other, scope, context, int)?
            }
            Self::Fn(param, expr, custom_scope) => {
                let new_scope =
                    Scope::with_variable(param.as_str(), other, scope.clone(), custom_scope);
                return crate::ast::evaluate(*expr, Some(Arc::new(new_scope)), context, int);
            }
            _ => {
                return Err(format!(
                    "'{}' is not a function or a number",
                    self.format_to_plain_string(0, int)?
                )
                .into());
            }
        })
    }

    #[allow(clippy::cognitive_complexity)]
    fn apply_built_in_function<I: Interrupt>(
        func: BuiltInFunction,
        arg: Expr<'a>,
        scope: Option<Arc<Scope<'a>>>,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
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
                use std::convert::TryInto;
                let n: u8 = arg
                    .expect_num()?
                    .try_as_usize(int)
                    .map_err(IntErr::into_string)?
                    .try_into()
                    .map_err(|_| "unable to convert number to a valid base".to_string())?;
                return Ok(Self::Base(
                    Base::from_plain_base(n).map_err(|e| e.to_string())?,
                ));
            }
            BuiltInFunction::Differentiate => return arg.differentiate("x", int),
            BuiltInFunction::Conjugate => arg.expect_num()?.conjugate(),
            BuiltInFunction::Not => return Ok(Value::Bool(!arg.expect_bool()?)),
        })))
    }

    pub(crate) fn format_to_plain_string<I: Interrupt>(
        &self,
        indent: usize,
        int: &I,
    ) -> Result<String, IntErr<String, I>> {
        let mut spans = vec![];
        self.format(indent, &mut spans, int)?;
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
        int: &I,
    ) -> Result<(), IntErr<String, I>> {
        match self {
            Self::Bool(b) => spans.push(Span {
                string: format!("{}", b),
                kind: SpanKind::Boolean,
            }),
            Self::Num(n) => {
                n.clone().simplify(int)?.format(int)?.spans(spans);
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
                let expr_str = (&**expr).format(int)?;
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
                    v.format(indent + 4, spans, int)?;
                }
                spans.push(Span::from_string("\n}".to_string()));
            }
            Self::String(s) => {
                spans.push(Span {
                    string: s.to_string(),
                    kind: SpanKind::String,
                });
            }
            Self::Dynamic(d) => {
                d.format(spans);
            }
        }
        Ok(())
    }

    pub(crate) fn get_object_member(self, key: Ident<'_>) -> Result<Self, String> {
        match self {
            Self::Object(kv) => {
                for (k, v) in kv {
                    if k == key.as_str() {
                        return Ok(*v);
                    }
                }
                Err("could not find key in object".to_string())
            }
            Self::Dynamic(d) => match d.get_object_member(key.as_str()) {
                Some(v) => Ok(v),
                None => Err(format!("could not find key {}", key.as_str())),
            },
            _ => Err("expected an object".to_string()),
        }
    }

    pub(crate) fn differentiate<I: Interrupt>(
        self,
        _to: &str,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        match self {
            Self::Num(_) => Ok(Value::Num(Box::new(Number::from(0)))),
            Self::BuiltInFunction(f) => Ok(f
                .differentiate()
                .ok_or(format!("cannot differentiate built-in function {}", f))?),
            _ => Err(format!(
                "cannot differentiate {}",
                self.format_to_plain_string(0, int)?
            )
            .into()),
        }
    }
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
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
            Self::Dynamic(d) => write!(f, "{:?}", d.0),
        }
    }
}
