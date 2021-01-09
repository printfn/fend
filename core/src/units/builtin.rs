#[allow(clippy::too_many_lines)]
#[rustfmt::skip::macros(define_units)]
pub fn query_unit<'a>(
    ident: &'a str,
    short_prefixes: bool,
) -> Option<(&'static str, &'static str, &'static str)> {
    macro_rules! define_units {
        (expr $name:literal $expr:literal) => {
            return Some(($name, $name, $expr))
        };
        (expr $s:literal $p:literal $expr:literal) => {
            return Some(($s, $p, $expr))
        };
        (
            $(($expr_name_s:literal $(/ $expr_name_p:literal)? $expr_def:literal))+
        ) => {
            match ident {
                $($expr_name_s $(| $expr_name_p)? => define_units!(expr $expr_name_s $($expr_name_p)? $expr_def),)+
                _ => ()
            }
        };
    }
    if short_prefixes {
        define_units!(
            ("Ki" "sp@kibi")
            ("Mi" "sp@mebi")
            ("Gi" "sp@gibi")
            ("Ti" "sp@tebi")
            ("Pi" "sp@pebi")
            ("Ei" "sp@exbi")
            ("Zi" "sp@zebi")
            ("Yi" "sp@yobi")

            ("Y"  "sp@yotta")
            ("Z"  "sp@zetta")
            ("E"  "sp@exa")
            ("P"  "sp@peta")
            ("T"  "sp@tera")
            ("G"  "sp@giga")
            ("M"  "sp@mega")
            ("k"  "sp@kilo")
            ("h"  "sp@hecto")
            ("da" "sp@deka")
            ("d"  "sp@deci")
            ("c"  "sp@centi")
            ("m"  "sp@milli")
            ("u"  "sp@micro") // alternative to µ
            ("\u{b5}"   "sp@micro") // U+00B5 (micro sign)
            ("\u{3bc}"  "sp@micro") // U+03BC (lowercase µ)
            ("n"  "sp@nano")
            ("p"  "sp@pico")
            ("f"  "sp@femto")
            ("a"  "sp@atto")
            ("z"  "sp@zepto")
            ("y"  "sp@yocto")
        );
    }
    define_units!(
("unitless"                       "=1")
    );
    None
}
