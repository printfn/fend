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
// SI units
("second"/"seconds"               "l@!")
("meter"/"meters"                 "l@!")
("kilogram"/"kilograms"           "l@!")
("kelvin"                         "l@!")
("ampere"/"amperes"               "l@!")
("mole"/"moles"                   "l@!")
("candela"/"candelas"             "l@!")

("s"                              "s@second")
("metre"/"metres"                 "l@meter")
("m"                              "s@meter")
("gram"/"grams"                   "l@1/1000 kilogram")
("g"                              "s@gram")
("K"                              "s@kelvin")
("amp"/"amps"                     "l@ampere")
("A"                              "s@ampere")
("mol"                            "s@mole")
("cd"                             "s@candela")

("yotta"                          "lp@1e24")
("zetta"                          "lp@1e21")
("exa"                            "lp@1e18")
("peta"                           "lp@1e15")
("tera"                           "lp@1e12")
("giga"                           "lp@1e9")
("mega"                           "lp@1e6")
("myria"                          "lp@1e4")
("kilo"                           "lp@1e3")
("hecto"                          "lp@1e2")
("deca"                           "lp@1e1")
("deka"                           "lp@deca")
("deci"                           "lp@1e-1")
("centi"                          "lp@1e-2")
("milli"                          "lp@1e-3")
("micro"                          "lp@1e-6")
("nano"                           "lp@1e-9")
("pico"                           "lp@1e-12")
("femto"                          "lp@1e-15")
("atto"                           "lp@1e-18")
("zepto"                          "lp@1e-21")
("yocto"                          "lp@1e-24")

("quarter"                        "lp@1/4")
("semi"                           "lp@0.5")
("demi"                           "lp@0.5")
("hemi"                           "lp@0.5")
("half"                           "lp@0.5")
("double"                         "lp@2")
("triple"                         "lp@3")
("treble"                         "lp@3")

("kibi"                           "lp@2^10")
("mebi"                           "lp@2^20")
("gibi"                           "lp@2^30")
("tebi"                           "lp@2^40")
("pebi"                           "lp@2^50")
("exbi"                           "lp@2^60")
("zebi"                           "lp@2^70")
("yobi"                           "lp@2^80")

    );
    None
}
