use crate::ast::Bop;
use crate::date::{Date, DayOfWeek, Month};
use crate::error::{FendError, Interrupt};
use crate::num::{Base, FormattingStyle, Number};
use crate::scope::Scope;
use crate::serialize::{
	deserialize_bool, deserialize_string, deserialize_u8, deserialize_usize, serialize_bool,
	serialize_string, serialize_u8, serialize_usize,
};
use crate::{ast::Expr, ident::Ident};
use crate::{date, Attrs, Span, SpanKind};
use std::borrow::Cow;
use std::io;
use std::{
	fmt::{self, Write},
	sync::Arc,
};

pub(crate) mod built_in_function;

use built_in_function::BuiltInFunction;

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
	Bool(bool),
	Unit, // unit value `()`
	Month(Month),
	DayOfWeek(DayOfWeek),
	Date(date::Date),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ApplyMulHandling {
	OnlyApply,
	Both,
}

impl Value {
	pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
		match self {
			Self::Num(n) => {
				serialize_u8(0, write)?;
				n.serialize(write)?;
			}
			Self::BuiltInFunction(f) => {
				serialize_u8(1, write)?;
				f.serialize(write)?;
			}
			Self::Format(f) => {
				serialize_u8(2, write)?;
				f.serialize(write)?;
			}
			Self::Dp => serialize_u8(3, write)?,
			Self::Sf => serialize_u8(4, write)?,
			Self::Base(b) => {
				serialize_u8(5, write)?;
				b.serialize(write)?;
			}
			Self::Fn(i, e, s) => {
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
			Self::Object(o) => {
				serialize_u8(7, write)?;
				serialize_usize(o.len(), write)?;
				for (k, v) in o {
					serialize_string(k.as_ref(), write)?;
					v.serialize(write)?;
				}
			}
			Self::String(s) => {
				serialize_u8(8, write)?;
				serialize_string(s, write)?;
			}
			Self::Unit => serialize_u8(9, write)?,
			Self::Bool(b) => {
				serialize_u8(10, write)?;
				serialize_bool(*b, write)?;
			}
			Self::Month(m) => {
				serialize_u8(11, write)?;
				m.serialize(write)?;
			}
			Self::DayOfWeek(d) => {
				serialize_u8(12, write)?;
				d.serialize(write)?;
			}
			Self::Date(d) => {
				serialize_u8(13, write)?;
				d.serialize(write)?;
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
					Some(Arc::new(Scope::deserialize(read)?))
				} else {
					None
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
			10 => Self::Bool(deserialize_bool(read)?),
			11 => Self::Month(Month::deserialize(read)?),
			12 => Self::DayOfWeek(DayOfWeek::deserialize(read)?),
			13 => Self::Date(Date::deserialize(read)?),
			_ => return Err(FendError::DeserializationError),
		})
	}

	pub(crate) fn type_name(&self) -> &'static str {
		match self {
			Self::Num(_) => "number",
			Self::BuiltInFunction(_) | Self::Fn(_, _, _) => "function",
			Self::Format(_) => "formatting style",
			Self::Dp => "decimal places",
			Self::Sf => "significant figures",
			Self::Base(_) => "base",
			Self::Object(_) => "object",
			Self::String(_) => "string",
			Self::Bool(_) => "bool",
			Self::Unit => "()",
			Self::Month(_) => "month",
			Self::DayOfWeek(_) => "day of week",
			Self::Date(_) => "date",
		}
	}

	fn as_bool(&self) -> Result<bool, FendError> {
		if let Self::Bool(b) = self {
			Ok(*b)
		} else {
			Err(FendError::ExpectedABool(self.type_name()))
		}
	}

	pub(crate) fn expect_num(self) -> Result<Number, FendError> {
		match self {
			Self::Num(bigrat) => Ok(*bigrat),
			_ => Err(FendError::ExpectedANumber),
		}
	}

	pub(crate) fn is_unit(&self) -> bool {
		matches!(self, Self::Unit)
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

	pub(crate) fn apply<I: Interrupt>(
		self,
		other: Expr,
		apply_mul_handling: ApplyMulHandling,
		scope: Option<Arc<Scope>>,
		attrs: Attrs,
		context: &mut crate::Context,
		int: &I,
	) -> Result<Self, FendError> {
		let stringified_self = self.format_to_plain_string(0, attrs, context, int)?;
		Ok(match self {
			Self::Num(n) => {
				let other = crate::ast::evaluate(other, scope.clone(), attrs, context, int)?;
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
						self_.format_to_plain_string(0, attrs, context, int)?,
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
				Self::apply_built_in_function(func, other, scope, attrs, context, int)?
			}
			Self::Fn(param, expr, custom_scope) => {
				let new_scope = Scope::with_variable(param, other, scope, custom_scope);
				return crate::ast::evaluate(*expr, Some(Arc::new(new_scope)), attrs, context, int);
			}
			_ => return Err(FendError::IsNotAFunctionOrNumber(stringified_self)),
		})
	}

	fn apply_built_in_function<I: Interrupt>(
		func: BuiltInFunction,
		arg: Expr,
		scope: Option<Arc<Scope>>,
		attrs: Attrs,
		context: &mut crate::Context,
		int: &I,
	) -> Result<Self, FendError> {
		let arg = crate::ast::evaluate(arg, scope.clone(), attrs, context, int)?;
		Ok(Self::Num(Box::new(match func {
			BuiltInFunction::Approximately => arg.expect_num()?.make_approximate(),
			BuiltInFunction::Abs => arg.expect_num()?.abs(int)?,
			BuiltInFunction::Sin => arg.expect_num()?.sin(scope, attrs, context, int)?,
			BuiltInFunction::Cos => arg.expect_num()?.cos(scope, attrs, context, int)?,
			BuiltInFunction::Tan => arg.expect_num()?.tan(scope, attrs, context, int)?,
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
			BuiltInFunction::Not => return Ok(Self::Bool(!arg.as_bool()?)),
			BuiltInFunction::Conjugate => arg.expect_num()?.conjugate()?,
		})))
	}

	pub(crate) fn format_to_plain_string<I: Interrupt>(
		&self,
		indent: usize,
		attrs: Attrs,
		ctx: &crate::Context,
		int: &I,
	) -> Result<String, FendError> {
		let mut spans = vec![];
		self.format(indent, &mut spans, attrs, ctx, int)?;
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
		attrs: Attrs,
		ctx: &crate::Context,
		int: &I,
	) -> Result<(), FendError> {
		match self {
			Self::Num(n) => {
				n.clone()
					.simplify(int)?
					.format(ctx, int)?
					.spans(spans, attrs);
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
				let expr_str = expr.format(attrs, ctx, int)?;
				let res = if name.as_str().contains('.') {
					format!("{name}:{expr_str}")
				} else {
					format!("\\{name}.{expr_str}")
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
					spans.push(Span::from_string(format!("{k}: ")));
					v.format(indent + 4, spans, attrs, ctx, int)?;
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
			Self::Bool(b) => spans.push(crate::Span {
				string: b.to_string(),
				kind: crate::SpanKind::Boolean,
			}),
			Self::Month(m) => spans.push(crate::Span {
				string: m.to_string(),
				kind: crate::SpanKind::Date,
			}),
			Self::DayOfWeek(d) => spans.push(crate::Span {
				string: d.to_string(),
				kind: crate::SpanKind::Date,
			}),
			Self::Date(d) => spans.push(crate::Span {
				string: d.to_string(),
				kind: crate::SpanKind::Date,
			}),
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
			Self::Date(d) => d.get_object_member(key),
			_ => Err(FendError::ExpectedAnObject),
		}
	}
}

impl fmt::Debug for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Num(n) => write!(f, "{n:?}"),
			Self::BuiltInFunction(name) => write!(f, "built-in function: {}", name.as_str()),
			Self::Format(fmt) => write!(f, "format: {fmt:?}"),
			Self::Dp => write!(f, "dp"),
			Self::Sf => write!(f, "sf"),
			Self::Base(b) => write!(f, "base: {b:?}"),
			Self::Fn(name, expr, scope) => {
				write!(f, "fn: {name} => {expr:?} (scope: {scope:?})")
			}
			Self::Object(kv) => {
				let mut s = "{".to_string();
				for (k, v) in kv {
					s.push_str(k);
					s.push(':');
					write!(s, "{:?}", *v)?;
					s.push(',');
				}
				s.push('}');
				write!(f, "{s}")
			}
			Self::String(s) => write!(f, r#""{}""#, s.as_ref()),
			Self::Unit => write!(f, "()"),
			Self::Bool(b) => write!(f, "{b}"),
			Self::Month(m) => write!(f, "{m}"),
			Self::DayOfWeek(d) => write!(f, "{d}"),
			Self::Date(d) => write!(f, "{d}"),
		}
	}
}
