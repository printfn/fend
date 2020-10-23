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
    if definition.trim() == "!" {
        return Ok(Value::Num(Number::new_base_unit(singular, plural)));
    }
    let num = eval(definition, &mut Scope::new(), int)?.expect_num()?;
    let unit = Number::create_unit_value_from_value(&num, singular.into(), plural.into(), int)?;
    Ok(Value::Num(unit))
}

#[cfg(feature = "gpl")]
pub fn query_unit<I: Interrupt>(ident: &str, int: &I) -> Result<Value, IntErr<String, I>> {
    builtin_gnu::query_unit(ident, int)
}

#[cfg(not(feature = "gpl"))]
pub fn query_unit<I: Interrupt>(ident: &str, int: &I) -> Result<Value, IntErr<String, I>> {
    builtin::query_unit(ident, int)
}
