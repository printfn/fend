use crate::value::Value;
use crate::{
    ast::Expr,
    err::{IntErr, Interrupt},
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
enum ScopeValue {
    //Variable(Value),
    LazyVariable(Expr, Scope),
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

impl ScopeValue {
    fn eval<I: Interrupt>(&self, int: &I) -> Result<Value, IntErr<String, I>> {
        match self {
            Self::LazyVariable(expr, scope) => {
                let value = crate::ast::evaluate(expr.clone(), &mut scope.clone(), int)?;
                Ok(value)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    hashmap: HashMap<Cow<'static, str>, ScopeValue>,
    prefixes: Vec<(Cow<'static, str>, ScopeValue)>,
    inner: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
            prefixes: vec![],
            inner: None,
        }
    }

    fn insert_scope_value(&mut self, ident: Cow<'static, str>, value: ScopeValue) {
        self.hashmap.insert(ident, value);
    }

    pub fn insert_variable(&mut self, name: Cow<'static, str>, expr: Expr, scope: Self) {
        self.insert_scope_value(name, ScopeValue::LazyVariable(expr, scope))
    }

    pub fn create_nested_scope(self) -> Self {
        let mut res = Self::new();
        res.inner = Some(Box::from(self));
        res
    }

    pub fn get<'a, I: Interrupt>(
        &mut self,
        ident: &'a str,
        int: &I,
    ) -> Result<Value, IntErr<GetIdentError<'a>, I>> {
        let potential_value = self.hashmap.get(ident).cloned();
        if let Some(value) = potential_value {
            let value = value
                .eval(int)
                .map_err(|e| e.map(GetIdentError::EvalError))?;
            Ok(value)
        } else if let Some(inner) = &mut self.inner {
            inner.get(ident, int)
        } else {
            Err(GetIdentError::IdentifierNotFound(ident))?
        }
    }
}
