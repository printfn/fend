use crate::error::FendError;
use crate::ident::Ident;
use crate::serialize::{deserialize_bool, serialize_bool};
use crate::value::Value;
use crate::{ast::Expr, error::Interrupt};
use std::io;
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
    ) -> Result<Value, FendError> {
        match self {
            Self::LazyVariable(expr, scope) => {
                let value = crate::ast::evaluate(expr.clone(), scope.clone(), context, int)?;
                Ok(value)
            }
        }
    }

    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        match self {
            Self::LazyVariable(e, s) => {
                e.serialize(write)?;
                match s {
                    None => serialize_bool(false, write)?,
                    Some(s) => {
                        serialize_bool(true, write)?;
                        s.serialize(write)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(Self::LazyVariable(Expr::deserialize(read)?, {
            if deserialize_bool(read)? {
                None
            } else {
                Some(Arc::new(Scope::deserialize(read)?))
            }
        }))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Scope {
    ident: Ident,
    value: ScopeValue,
    inner: Option<Arc<Scope>>,
}

impl Scope {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        self.ident.serialize(write)?;
        self.value.serialize(write)?;
        match &self.inner {
            None => serialize_bool(false, write)?,
            Some(s) => {
                serialize_bool(true, write)?;
                s.serialize(write)?;
            }
        }
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(Self {
            ident: Ident::deserialize(read)?,
            value: ScopeValue::deserialize(read)?,
            inner: {
                if deserialize_bool(read)? {
                    None
                } else {
                    Some(Arc::new(Self::deserialize(read)?))
                }
            },
        })
    }

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
    ) -> Result<Value, FendError> {
        if self.ident.as_str() == ident.as_str() {
            let value = self.value.eval(context, int)?;
            Ok(value)
        } else {
            self.inner.as_ref().map_or_else(
                || Err(FendError::IdentifierNotFound(ident.clone())),
                |inner| inner.get(ident, context, int),
            )
        }
    }
}
