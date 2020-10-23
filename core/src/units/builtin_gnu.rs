use super::expr_unit;
use crate::err::{IntErr, Interrupt};
use crate::scope::GetIdentError;
use crate::value::Value;

#[allow(clippy::too_many_lines)]
pub fn query_unit<I: Interrupt>(ident: &str, int: &I) -> Result<Value, IntErr<String, I>> {
    macro_rules! define_units {
        (expr $name:literal $expr:literal) => {
            expr_unit($name, $name, $expr, int)
        };
        (expr $s:literal $p:literal $expr:literal) => {
            expr_unit($s, $p, $expr, int)
        };
        (
            $(($expr_name_s:literal $(/ $expr_name_p:literal)? $expr_def:literal))+
        ) => {
            Ok(match ident {
                $($expr_name_s $(| $expr_name_p)? => define_units!(expr $expr_name_s $($expr_name_p)? $expr_def)?,)+
                _ => return Err(GetIdentError::IdentifierNotFound(ident).to_string())?
            })
        };
    }
    define_units!(
        ("s"  "!")
    )
}
