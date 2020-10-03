use crate::value::Value;
use crate::{
    ast::Expr,
    err::{IntErr, Interrupt},
    parser::ParseOptions,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ScopeValue {
    // value, singular name, plural name
    EagerUnit(Value, String, String),
    // expr, singular name, plural name
    LazyUnit(String, String, String),
    LazyExpr(String),
    //Variable(Value),
    LazyVariable(Expr, Scope, ParseOptions),
}

impl ScopeValue {
    fn eval<I: Interrupt>(
        &self,
        ident: &str,
        scope: &mut Scope,
        int: &I,
    ) -> Result<Value, IntErr<String, I>> {
        let options = crate::parser::ParseOptions::new_for_gnu_units();
        match self {
            ScopeValue::EagerUnit(value, _, _) => Ok(value.clone()),
            ScopeValue::LazyUnit(expr, singular_name, plural_name) => {
                let value = crate::eval::evaluate_to_value(expr.as_str(), options, scope, int)?
                    .expect_num()?;
                let unit = crate::num::Number::create_unit_value_from_value(
                    &value,
                    singular_name.clone(),
                    plural_name.clone(),
                    int,
                )?;
                scope.insert_scope_value(
                    ident.to_string(),
                    ScopeValue::EagerUnit(
                        Value::Num(unit.clone()),
                        singular_name.clone(),
                        plural_name.clone(),
                    ),
                );
                Ok(Value::Num(unit))
            }
            ScopeValue::LazyExpr(expr) => {
                let value = crate::eval::evaluate_to_value(expr.as_str(), options, scope, int)?;
                // todo add caching
                Ok(value)
            }
            //ScopeValue::Variable(val) => Ok(val.clone()),
            ScopeValue::LazyVariable(expr, scope, options) => {
                let value = crate::ast::evaluate(expr.clone(), &mut scope.clone(), *options, int)?;
                Ok(value)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    hashmap: HashMap<String, ScopeValue>,
    prefixes: Vec<(String, ScopeValue)>,
    inner: Option<Box<Scope>>,
}

impl Scope {
    pub fn new_default<I: Interrupt>(int: &I) -> Result<Self, IntErr<String, I>> {
        crate::num::Number::create_initial_units(int)
    }

    pub fn new_empty() -> Self {
        Self {
            hashmap: HashMap::new(),
            prefixes: vec![],
            inner: None,
        }
    }

    fn insert_scope_value(&mut self, ident: String, value: ScopeValue) {
        self.hashmap.insert(ident, value);
    }

    pub fn insert(&mut self, singular: String, plural: String, value: Value) {
        if singular != plural {
            self.insert_scope_value(
                plural.to_string(),
                ScopeValue::EagerUnit(value.clone(), singular.clone(), plural.clone()),
            );
        }
        self.insert_scope_value(
            singular.to_string(),
            ScopeValue::EagerUnit(value, singular, plural),
        );
    }

    pub fn insert_lazy_unit(&mut self, expr: String, singular_name: String, plural_name: String) {
        let hashmap_val = ScopeValue::LazyUnit(expr, singular_name.clone(), plural_name.clone());
        if singular_name != plural_name {
            self.insert_scope_value(plural_name, hashmap_val.clone());
        }
        self.insert_scope_value(singular_name, hashmap_val);
    }

    pub fn insert_variable(
        &mut self,
        name: String,
        expr: Expr,
        scope: Scope,
        options: ParseOptions,
    ) {
        self.insert_scope_value(name, ScopeValue::LazyVariable(expr, scope, options))
    }

    pub fn create_nested_scope(self) -> Self {
        let mut res = Self::new_empty();
        res.inner = Some(Box::from(self));
        res
    }

    pub fn insert_prefix(&mut self, ident: &str, expr: &str) {
        self.prefixes
            .push((ident.to_string(), ScopeValue::LazyExpr(expr.to_string())))
    }

    fn test_prefixes<I: Interrupt>(
        &mut self,
        ident: &str,
        int: &I,
    ) -> Result<Value, IntErr<String, I>> {
        for (prefix, scope_value) in &self.prefixes.clone() {
            if let Some(remaining) = ident.strip_prefix(prefix) {
                let prefix_value = scope_value.eval(prefix, self, int)?;
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
                            (s.clone(), p.clone())
                        }
                        _ => continue,
                    };
                    singular.insert_str(0, prefix);
                    plural.insert_str(0, prefix);
                    let value = remaining_value.eval(remaining, self, int)?;
                    let res = prefix_value.expect_num()?.mul(value.expect_num()?, int)?;
                    let unit = crate::num::Number::create_unit_value_from_value(
                        &res, singular, plural, int,
                    )?;
                    return Ok(Value::Num(unit));
                }
            }
        }
        Err(format!("Unknown identifier '{}'", ident))?
    }

    pub fn get<I: Interrupt>(&mut self, ident: &str, int: &I) -> Result<Value, IntErr<String, I>> {
        let potential_value = self.hashmap.get(ident).cloned();
        if let Some(value) = potential_value {
            let value = value.eval(ident, self, int)?;
            Ok(value)
        } else if let Ok(val) = self.test_prefixes(ident, int) {
            Ok(val)
        } else if let Some(inner) = &mut self.inner {
            inner.get(ident, int)
        } else {
            Err(format!("Unknown identifier '{}'", ident))?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::err::NeverInterrupt;

    #[test]
    fn test_alepint() -> Result<(), IntErr<String, NeverInterrupt>> {
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
        for key in hashmap.keys() {
            let mut scope = scope.clone();
            eprintln!("Testing {}", key);
            match scope.get(key.as_str(), &int) {
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
