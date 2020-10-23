use crate::ast::eval;
use crate::err::{IntErr, Interrupt};
use crate::num::Number;
use crate::scope::Scope;
use crate::value::Value;

#[cfg(feature = "gpl")]
mod builtin_gnu;

#[cfg(not(feature = "gpl"))]
mod builtin;

fn expr_unit<I: Interrupt>(
    singular: &'static str,
    plural: &'static str,
    definition: &'static str,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    let definition = definition.trim();
    if definition == "!" {
        return Ok(Value::Num(Number::new_base_unit(singular, plural)));
    }
    let (alias, definition) = if let Some(remaining) = definition.strip_prefix('=') {
        (true, remaining)
    } else {
        (false, definition)
    };
    let mut num = eval(definition, &mut Scope::new(), int)?.expect_num()?;
    if !alias {
        num = Number::create_unit_value_from_value(&num, singular.into(), plural.into(), int)?;
    }
    Ok(Value::Num(num))
}

#[cfg(feature = "gpl")]
pub fn query_unit<I: Interrupt>(ident: &str, int: &I) -> Result<Value, IntErr<String, I>> {
    builtin_gnu::query_unit(ident, int)
}

#[cfg(not(feature = "gpl"))]
pub fn query_unit<I: Interrupt>(ident: &str, int: &I) -> Result<Value, IntErr<String, I>> {
    builtin::query_unit(ident, int)
}
