use crate::eval::evaluate_to_value;
use crate::err::{IntErr, Interrupt};
use crate::num::Number;
use crate::scope::{GetIdentError, Scope};
use crate::value::Value;

#[cfg(feature = "gpl")]
mod builtin_gnu;

#[cfg(not(feature = "gpl"))]
mod builtin;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PrefixRule {
    NoPrefixesAllowed,
    LongPrefixAllowed,
    LongPrefix,
}

pub struct UnitDef {
    singular: &'static str,
    plural: &'static str,
    prefix_rule: PrefixRule,
    value: Value,
}

fn expr_unit<I: Interrupt>(
    singular: &'static str,
    plural: &'static str,
    definition: &'static str,
    int: &I,
) -> Result<UnitDef, IntErr<GetIdentError<'static>, I>> {
    let mut definition = definition.trim();
    let mut rule = PrefixRule::NoPrefixesAllowed;
    if let Some(remaining) = definition.strip_prefix("l@") {
        definition = remaining;
        rule = PrefixRule::LongPrefixAllowed;
    }
    if let Some(remaining) = definition.strip_prefix("lp@") {
        definition = remaining;
        rule = PrefixRule::LongPrefix;
    }
    if definition == "!" {
        return Ok(UnitDef {
            value: Value::Num(Number::new_base_unit(singular, plural)),
            prefix_rule: rule,
            singular,
            plural,
        });
    }
    let (alias, definition) = definition
        .strip_prefix('=')
        .map_or((false, definition), |remaining| (true, remaining));
    let mut num = evaluate_to_value(definition, &mut Scope::new(), int)?.expect_num()?;
    if !alias && rule != PrefixRule::LongPrefix {
        num = Number::create_unit_value_from_value(&num, singular.into(), plural.into(), int)?;
    }
    Ok(UnitDef {
        value: Value::Num(num),
        prefix_rule: rule,
        singular,
        plural,
    })
}

fn construct_prefixed_unit<I: Interrupt>(
    a: UnitDef,
    b: UnitDef,
    int: &I,
) -> Result<Value, IntErr<String, I>> {
    let product = a.value.expect_num()?.mul(b.value.expect_num()?, int)?;
    let unit = Number::create_unit_value_from_value(
        &product,
        format!("{}{}", a.singular, b.singular).into(),
        format!("{}{}", a.plural, b.plural).into(),
        int,
    )?;
    Ok(Value::Num(unit))
}

pub fn query_unit<'a, I: Interrupt>(
    ident: &'a str,
    int: &I,
) -> Result<Value, IntErr<GetIdentError<'a>, I>> {
    match query_unit_internal(ident, int) {
        Err(IntErr::Error(GetIdentError::IdentifierNotFound(_))) => (),
        Err(e) => return Err(e),
        Ok(unit) => {
            if unit.prefix_rule == PrefixRule::NoPrefixesAllowed
                || unit.prefix_rule == PrefixRule::LongPrefixAllowed
                || unit.prefix_rule == PrefixRule::LongPrefix
            {
                // return value without prefix
                return Ok(unit.value);
            }
            return Err(GetIdentError::IdentifierNotFound(ident))?;
        }
    }
    let mut split_idx = ident.chars().next().unwrap().len_utf8();
    while split_idx < ident.len() {
        let (prefix, remaining_ident) = ident.split_at(split_idx);
        match (
            query_unit_internal(prefix, int),
            query_unit_internal(remaining_ident, int),
        ) {
            (Err(e @ IntErr::Interrupt(_)), _)
            | (_, Err(e @ IntErr::Interrupt(_)))
            | (Err(e @ IntErr::Error(GetIdentError::EvalError(_))), _)
            | (_, Err(e @ IntErr::Error(GetIdentError::EvalError(_)))) => return Err(e),
            (Ok(a), Ok(b)) => {
                if a.prefix_rule == PrefixRule::LongPrefix
                    && b.prefix_rule == PrefixRule::LongPrefixAllowed
                {
                    // now construct a new unit!
                    return Ok(construct_prefixed_unit(a, b, int)?);
                }
                return Err(GetIdentError::IdentifierNotFound(ident))?;
            }
            _ => (),
        };
        split_idx += remaining_ident.chars().next().unwrap().len_utf8();
    }
    Err(GetIdentError::IdentifierNotFound(ident))?
}

#[cfg(feature = "gpl")]
fn query_unit_internal<'a, I: Interrupt>(
    ident: &'a str,
    int: &I,
) -> Result<UnitDef, IntErr<GetIdentError<'a>, I>> {
    builtin_gnu::query_unit(ident, int)
}

#[cfg(not(feature = "gpl"))]
fn query_unit_internal<'a, I: Interrupt>(
    ident: &'a str,
    int: &I,
) -> Result<UnitDef, IntErr<GetIdentError<'a>, I>> {
    builtin::query_unit(ident, int)
}
