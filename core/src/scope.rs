use crate::error::FendError;
use crate::ident::Ident;
use crate::value::Value;
use crate::{
    ast::Expr,
    error::{IntErr, Interrupt},
};
use std::sync::Arc;

#[derive(Debug, Clone)]
enum ScopeValue {
    //Variable(Value),
    LazyVariable(Expr, Option<Arc<Scope>>),
}

impl ScopeValue {
    fn eval<I: Interrupt>(
        &self,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Value, IntErr<FendError, I>> {
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
    ) -> Result<Value, IntErr<FendError, I>> {
        if self.ident.as_str() == ident.as_str() {
            let value = self.value.eval(context, int)?;
            Ok(value)
        } else {
            self.inner.as_ref().map_or_else(
                || Err(FendError::IdentifierNotFound(ident.clone()).into()),
                |inner| inner.get(ident, context, int),
            )
        }
    }
}
