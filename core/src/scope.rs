use crate::value::Value;
use crate::{
    ast::Expr,
    err::{IntErr, Interrupt},
    parser::ParseOptions,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
enum ScopeValue {
    // value, singular name, plural name
    EagerUnit(Value, Cow<'static, str>, Cow<'static, str>),
    // expr, singular name, plural name
    LazyUnit(String, Cow<'static, str>, Cow<'static, str>),
    LazyExpr(String),
    //Variable(Value),
    LazyVariable(Expr, Scope, ParseOptions),
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

impl ScopeValue {
    fn eval<I: Interrupt>(
        &self,
        ident: Cow<'static, str>,
        scope: &mut Scope,
        int: &I,
    ) -> Result<Value, IntErr<String, I>> {
        let options = crate::parser::ParseOptions::new_for_gnu_units();
        match self {
            Self::EagerUnit(value, _, _) => Ok(value.clone()),
            Self::LazyUnit(expr, singular_name, plural_name) => {
                let value = crate::eval::evaluate_to_value(expr.as_str(), options, scope, int)?
                    .expect_num()?;
                let unit = crate::num::Number::create_unit_value_from_value(
                    &value,
                    singular_name.clone(),
                    plural_name.clone(),
                    int,
                )?;
                scope.insert_scope_value(
                    ident,
                    Self::EagerUnit(
                        Value::Num(unit.clone()),
                        singular_name.clone(),
                        plural_name.clone(),
                    ),
                );
                Ok(Value::Num(unit))
            }
            Self::LazyExpr(expr) => {
                let value = crate::eval::evaluate_to_value(expr.as_str(), options, scope, int)?;
                // todo add caching
                Ok(value)
            }
            //Self::Variable(val) => Ok(val.clone()),
            Self::LazyVariable(expr, scope, options) => {
                let value =
                    crate::ast::evaluate(expr.clone().into(), &mut scope.clone(), *options, int)?;
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
    pub fn new_default<I: Interrupt>(int: &I) -> Result<Self, IntErr<String, I>> {
        crate::num::Number::create_initial_units(int)
    }

    pub fn new_empty_with_capacity(capacity: usize) -> Self {
        Self {
            hashmap: HashMap::with_capacity(capacity),
            prefixes: vec![],
            inner: None,
        }
    }

    fn insert_scope_value(&mut self, ident: Cow<'static, str>, value: ScopeValue) {
        self.hashmap.insert(ident, value);
    }

    pub fn insert(
        &mut self,
        singular: impl Into<Cow<'static, str>>,
        plural: impl Into<Cow<'static, str>>,
        value: Value,
    ) {
        let singular = singular.into();
        let plural = plural.into();
        if singular != plural {
            self.insert_scope_value(
                plural.clone(),
                ScopeValue::EagerUnit(value.clone(), singular.clone(), plural.clone()),
            );
        }
        self.insert_scope_value(
            singular.clone(),
            ScopeValue::EagerUnit(value, singular, plural),
        );
    }

    pub fn insert_lazy_unit(
        &mut self,
        expr: String,
        singular_name: Cow<'static, str>,
        plural_name: Cow<'static, str>,
    ) {
        let hashmap_val = ScopeValue::LazyUnit(expr, singular_name.clone(), plural_name.clone());
        if singular_name != plural_name {
            self.insert_scope_value(plural_name, hashmap_val.clone());
        }
        self.insert_scope_value(singular_name, hashmap_val);
    }

    pub fn insert_variable(
        &mut self,
        name: Cow<'static, str>,
        expr: Expr,
        scope: Self,
        options: ParseOptions,
    ) {
        self.insert_scope_value(name, ScopeValue::LazyVariable(expr, scope, options))
    }

    pub fn create_nested_scope(self) -> Self {
        let mut res = Self::new_empty_with_capacity(10);
        res.inner = Some(Box::from(self));
        res
    }

    pub fn insert_prefix(&mut self, ident: Cow<'static, str>, expr: &str) {
        self.prefixes
            .push((ident, ScopeValue::LazyExpr(expr.to_string())))
    }

    fn test_prefixes<I: Interrupt>(
        &mut self,
        ident: &str,
        int: &I,
    ) -> Result<Value, IntErr<String, I>> {
        for (prefix, scope_value) in &self.prefixes.clone() {
            if let Some(remaining) = ident.strip_prefix(prefix.as_ref()) {
                let prefix_value = scope_value.eval(prefix.clone(), self, int)?;
                if remaining.is_empty() {
                    let unit = crate::num::Number::create_unit_value_from_value(
                        &prefix_value.expect_num()?,
                        prefix.clone(),
                        prefix.clone(),
                        int,
                    )?;
                    return Ok(Value::Num(unit));
                }
                if let Some(remaining_value) = self.hashmap.get(remaining).cloned() {
                    let (mut singular, mut plural) = match &remaining_value {
                        ScopeValue::EagerUnit(_, s, p) | ScopeValue::LazyUnit(_, s, p) => {
                            (s.clone().into_owned(), p.clone().into_owned())
                        }
                        _ => continue,
                    };
                    singular.insert_str(0, prefix);
                    plural.insert_str(0, prefix);
                    let value = remaining_value.eval(remaining.to_string().into(), self, int)?;
                    let res = prefix_value.expect_num()?.mul(value.expect_num()?, int)?;
                    let unit = crate::num::Number::create_unit_value_from_value(
                        &res,
                        singular.into(),
                        plural.into(),
                        int,
                    )?;
                    return Ok(Value::Num(unit));
                }
            }
        }
        Err(format!("Unknown identifier '{}'", ident))?
    }

    pub fn get<'a, I: Interrupt>(
        &mut self,
        ident: &'a str,
        int: &I,
    ) -> Result<Value, IntErr<GetIdentError<'a>, I>> {
        let potential_value = self.hashmap.get(ident).cloned();
        if let Some(value) = potential_value {
            let value = value
                .eval(ident.to_string().into(), self, int)
                .map_err(|e| e.map(GetIdentError::EvalError))?;
            Ok(value)
        } else if let Ok(val) = self.test_prefixes(ident.as_ref(), int) {
            Ok(val)
        } else if let Some(inner) = &mut self.inner {
            inner.get(ident, int)
        } else {
            Err(GetIdentError::IdentifierNotFound(ident))?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::err::NeverInterrupt;

    #[test]
    fn test_alepint() -> Result<(), IntErr<GetIdentError<'static>, NeverInterrupt>> {
        let int = NeverInterrupt::default();
        let mut scope = Scope::new_default(&int).unwrap();
        scope.get("beergallon", &int)?;
        scope.get("alepint", &int)?;
        Ok(())
    }

    #[test]
    fn test_lazy_units() {
        let int = NeverInterrupt::default();
        let mut scope = Scope::new_default(&int).unwrap();
        let hashmap = scope.hashmap.clone();
        let mut success = 0;
        let mut failures = 0;
        // this is needed to prevent rare stack overflows
        scope.get("beergallon", &int).unwrap();
        scope.get("parsec", &int).unwrap();
        scope.get("Mpc", &int).unwrap();
        scope.get("hubble", &int).unwrap();
        scope.get("atomicmass", &int).unwrap();
        scope.get("rydberg", &int).unwrap();
        scope.get("atomicvelocity", &int).unwrap();
        scope.get("troyounce", &int).unwrap();
        scope.get("apscruple", &int).unwrap();
        scope.get("brfloz", &int).unwrap();
        scope.get("brscruple", &int).unwrap();
        scope.get("tesla", &int).unwrap();
        scope.get("B_FIELD", &int).unwrap();
        scope.get("USD", &int).unwrap();
        scope.get("ustsp", &int).unwrap();
        for key in hashmap.keys() {
            //let mut scope = scope.clone();
            //eprintln!("Testing {}", key);
            match scope.get(key.as_ref(), &int) {
                Ok(_) => success += 1,
                Err(msg) => {
                    let error_message = match msg {
                        IntErr::Error(e) => e,
                        IntErr::Interrupt(n) => match n {},
                    };
                    eprintln!("{}: {}", key, error_message);
                    failures += 1;
                }
            }
        }
        if failures != 0 {
            eprintln!("{}/{} succeeded", success, success + failures);
        }
        assert_eq!(failures, 0);
    }
}
