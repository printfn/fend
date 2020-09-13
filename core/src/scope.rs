use crate::err::{IntErr, Interrupt};
use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ScopeValue {
    Eager(Value),
    // expr, singular name, plural name, space
    LazyUnit(String, String, String, bool),
}

impl ScopeValue {
    fn eval<I: Interrupt>(
        &self,
        ident: &str,
        scope: &mut Scope,
        int: &I,
    ) -> Result<Value, IntErr<String, I>> {
        match self {
            ScopeValue::Eager(value) => Ok(value.clone()),
            ScopeValue::LazyUnit(expr, singular_name, plural_name, space) => {
                let value =
                    crate::eval::evaluate_to_value(expr.as_str(), scope, int)?.expect_num()?;
                let unit = crate::num::Number::create_unit_value_from_value(
                    &value,
                    singular_name.clone(),
                    plural_name.clone(),
                    *space,
                    int,
                )?;
                scope.hashmap.insert(
                    ident.to_string(),
                    ScopeValue::Eager(Value::Num(unit.clone())),
                );
                Ok(Value::Num(unit))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    hashmap: HashMap<String, ScopeValue>,
}

impl Scope {
    pub fn new_default<I: Interrupt>(int: &I) -> Result<Self, IntErr<String, I>> {
        crate::num::Number::create_initial_units(int)
    }

    pub fn new_empty() -> Self {
        Self {
            hashmap: HashMap::new(),
        }
    }

    pub fn insert(&mut self, ident: &str, value: Value) {
        self.hashmap
            .insert(ident.to_string(), ScopeValue::Eager(value));
    }

    pub fn insert_lazy_unit(
        &mut self,
        expr: String,
        singular_name: String,
        plural_name: String,
        space: bool,
    ) {
        let hashmap_val =
            ScopeValue::LazyUnit(expr, singular_name.clone(), plural_name.clone(), space);
        if singular_name != plural_name {
            self.hashmap.insert(plural_name, hashmap_val.clone());
        }
        self.hashmap.insert(singular_name, hashmap_val);
    }

    pub fn get<I: Interrupt>(&mut self, ident: &str, int: &I) -> Result<Value, IntErr<String, I>> {
        // TODO find a way to remove this 'cloned' call without upsetting the borrow checker
        let potential_value = self.hashmap.get(ident).cloned();
        if let Some(value) = potential_value {
            let value = value.eval(ident, self, int)?;
            Ok(value)
        } else {
            Err(format!("Unknown identifier '{}'", ident))?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_units() {
        let int = crate::interrupt::Never::default();
        let mut scope = Scope::new_default(&int).unwrap();
        let hashmap = scope.hashmap.clone();
        for key in hashmap.keys() {
            let _ = scope.get(key.as_str(), &int).unwrap();
        }
    }
}
