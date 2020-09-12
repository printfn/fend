use crate::value::Value;
use crate::err::{Interrupt, IntErr, Never};
use std::collections::HashMap;

fn _eval<I: Interrupt>(input: &'static str, scope: &Scope, int: &I) -> Result<Value, IntErr<Never, I>> {
    crate::eval::evaluate_to_value(input, scope, int)
        .map_err(crate::err::IntErr::unwrap)
}

#[derive(Debug, Clone)]
pub struct Scope {
    hashmap: HashMap<String, Value>
}

impl Scope {
    pub fn new_default<I: Interrupt>(int: &I) -> Result<Self, IntErr<String, I>> {
        crate::num::Number::create_initial_units(int)
    }

    pub fn new_empty() -> Self {
        Self { hashmap: HashMap::new() }
    }

    pub fn insert(&mut self, ident: &str, value: Value) {
        self.hashmap.insert(ident.to_string(), value);
    }

    pub fn get<I: Interrupt>(&self, ident: &str, _int: &I) -> Result<Value, IntErr<String, I>> {
        if let Some(value) = self.hashmap.get(&ident.to_string()) {
            Ok(value.clone())
        } else {
            Err(format!("Unknown identifier '{}'", ident))?
        }
    }
}
