use crate::value::Value;
use crate::{
    ast::Expr,
    error::{IntErr, Interrupt},
};
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
enum ScopeValue<'a> {
    //Variable(Value),
    LazyVariable(Expr<'a>, Option<Arc<Scope<'a>>>),
}

#[derive(Debug)]
pub enum GetIdentError<'a> {
    EvalError(String),
    IdentifierNotFound(&'a str),
}

impl<'a> fmt::Display for GetIdentError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EvalError(s) => write!(f, "{}", s),
            Self::IdentifierNotFound(s) => write!(f, "Unknown identifier '{}'", s),
        }
    }
}

#[allow(clippy::use_self)]
impl<'a, I: Interrupt> From<IntErr<String, I>> for IntErr<GetIdentError<'a>, I> {
    fn from(e: IntErr<String, I>) -> Self {
        match e {
            IntErr::Interrupt(i) => IntErr::Interrupt(i),
            IntErr::Error(s) => IntErr::Error(GetIdentError::EvalError(s)),
        }
    }
}

impl<'a> ScopeValue<'a> {
    fn eval<I: Interrupt>(&self, int: &I) -> Result<Value<'a>, IntErr<String, I>> {
        match self {
            Self::LazyVariable(expr, scope) => {
                let value = crate::ast::evaluate(expr.clone(), scope.clone(), int)?;
                Ok(value)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Scope<'a> {
    ident: &'a str,
    value: ScopeValue<'a>,
    inner: Option<Arc<Scope<'a>>>,
}

impl<'a> Scope<'a> {
    fn with_scope_value(ident: &'a str, value: ScopeValue<'a>, inner: Option<Arc<Self>>) -> Self {
        Self {
            ident,
            value,
            inner,
        }
    }

    pub(crate) fn with_variable(
        name: &'a str,
        expr: Expr<'a>,
        scope: Option<Arc<Self>>,
        inner: Option<Arc<Self>>,
    ) -> Self {
        Self::with_scope_value(name, ScopeValue::LazyVariable(expr, scope), inner)
    }

    pub(crate) fn get<I: Interrupt>(
        &self,
        ident: &'a str,
        int: &I,
    ) -> Result<Value<'a>, IntErr<GetIdentError<'a>, I>> {
        if self.ident == ident {
            let value = self
                .value
                .eval(int)
                .map_err(|e| e.map(GetIdentError::EvalError))?;
            Ok(value)
        } else {
            self.inner.as_ref().map_or_else(
                || Err(GetIdentError::IdentifierNotFound(ident).into()),
                |inner| inner.get(ident, int),
            )
        }
    }
}
