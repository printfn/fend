use crate::ast::eval;
use crate::err::{IntErr, Interrupt};
use crate::num::Number;
use crate::scope::{GetIdentError, Scope};
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
) -> Result<Value, IntErr<GetIdentError<'static>, I>> {
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
pub fn query_unit<'a, I: Interrupt>(
    ident: &'a str,
    int: &I,
) -> Result<Value, IntErr<GetIdentError<'a>, I>> {
    query_unit_internal(ident, int)
}

#[cfg(feature = "gpl")]
fn query_unit_internal<'a, I: Interrupt>(
    ident: &'a str,
    int: &I,
) -> Result<Value, IntErr<GetIdentError<'a>, I>> {
    builtin_gnu::query_unit(ident, int)
}

#[cfg(not(feature = "gpl"))]
fn query_unit_internal<'a, I: Interrupt>(
    ident: &'a str,
    int: &I,
) -> Result<Value, IntErr<GetIdentError<'a>, I>> {
    builtin::query_unit(ident, int)
}
