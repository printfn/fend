use crate::ident::Ident;
use crate::value::Value;
use crate::{
    ast::Expr,
    error::{IntErr, Interrupt},
};
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
enum ScopeValue {
    //Variable(Value),
    LazyVariable(Expr, Option<Arc<Scope>>),
}

#[derive(Debug)]
pub(crate) enum GetIdentError {
    EvalError(String),
    IdentifierNotFound(Ident),
}

impl fmt::Display for GetIdentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvalError(s) => write!(f, "{}", s),
            Self::IdentifierNotFound(s) => write!(f, "unknown identifier '{}'", s),
        }
    }
}

#[allow(clippy::use_self)]
impl<'a, I: Interrupt> From<IntErr<String, I>> for IntErr<GetIdentError, I> {
    fn from(e: IntErr<String, I>) -> Self {
        match e {
            IntErr::Interrupt(i) => IntErr::Interrupt(i),
            IntErr::Error(s) => IntErr::Error(GetIdentError::EvalError(s)),
        }
    }
}

#[allow(clippy::use_self)]
impl<'a, I: Interrupt> From<String> for IntErr<GetIdentError, I> {
    fn from(e: String) -> Self {
        IntErr::Error(GetIdentError::EvalError(e))
    }
}

impl ScopeValue {
    fn eval<I: Interrupt>(
        &self,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Value, IntErr<String, I>> {
        match self {
            Self::LazyVariable(expr, scope) => {
                let value = crate::ast::evaluate(expr.clone(), scope.clone(), context, int)?;
                Ok(value)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Scope {
    ident: Ident,
    value: ScopeValue,
    inner: Option<Arc<Scope>>,
}

impl Scope {
    const fn with_scope_value(ident: Ident, value: ScopeValue, inner: Option<Arc<Self>>) -> Self {
        Self {
            ident,
            value,
            inner,
        }
    }

    pub(crate) fn with_variable(
        name: Ident,
        expr: Expr,
        scope: Option<Arc<Self>>,
        inner: Option<Arc<Self>>,
    ) -> Self {
        Self::with_scope_value(name, ScopeValue::LazyVariable(expr, scope), inner)
    }

    pub(crate) fn get<I: Interrupt>(
        &self,
        ident: &Ident,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Value, IntErr<GetIdentError, I>> {
        if self.ident.as_str() == ident.as_str() {
            let value = self
                .value
                .eval(context, int)
                .map_err(|e| e.map(GetIdentError::EvalError))?;
            Ok(value)
        } else {
            self.inner.as_ref().map_or_else(
                || Err(GetIdentError::IdentifierNotFound(ident.clone()).into()),
                |inner| inner.get(ident, context, int),
            )
        }
    }
}
