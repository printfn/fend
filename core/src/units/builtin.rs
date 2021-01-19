#[allow(clippy::too_many_lines)]
#[rustfmt::skip::macros(define_units)]
pub(crate) fn query_unit<'a>(
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
        match ident {
            "Ki" => return Some(("Ki", "Ki", "sp@kibi")),
            "Mi" => return Some(("Mi", "Mi", "sp@mebi")),
            "Gi" => return Some(("Gi", "Gi", "sp@gibi")),
            "Ti" => return Some(("Ti", "Ti", "sp@tebi")),
            "Pi" => return Some(("Pi", "Pi", "sp@pebi")),
            "Ei" => return Some(("Ei", "Ei", "sp@exbi")),
            "Zi" => return Some(("Zi", "Zi", "sp@zebi")),
            "Yi" => return Some(("Yi", "Yi", "sp@yobi")),

            "Y" => return Some(("Y", "Y", "sp@yotta")),
            "Z" => return Some(("Z", "Z", "sp@zetta")),
            "E" => return Some(("E", "E", "sp@exa")),
            "P" => return Some(("P", "P", "sp@peta")),
            "T" => return Some(("T", "T", "sp@tera")),
            "G" => return Some(("G", "G", "sp@giga")),
            "M" => return Some(("M", "M", "sp@mega")),
            "k" => return Some(("k", "k", "sp@kilo")),
            "h" => return Some(("h", "h", "sp@hecto")),
            "da" => return Some(("da", "da", "sp@deka")),
            "d" => return Some(("d", "d", "sp@deci")),
            "c" => return Some(("c", "c", "sp@centi")),
            "m" => return Some(("m", "m", "sp@milli")),
            "u" => return Some(("u", "u", "sp@micro")), // alternative to µ
            "\u{b5}" => return Some(("\u{b5}", "\u{b5}", "sp@micro")), // U+00B5 (micro sign)
            "\u{3bc}" => return Some(("\u{3bc}", "\u{3bc}", "sp@micro")), // U+03BC (lowercase µ)
            "n" => return Some(("n", "n", "sp@nano")),
            "p" => return Some(("p", "p", "sp@pico")),
            "f" => return Some(("f", "f", "sp@femto")),
            "a" => return Some(("a", "a", "sp@atto")),
            "z" => return Some(("z", "z", "sp@zepto")),
            "y" => return Some(("y", "y", "sp@yocto")),
            _ => (),
        }
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
("°K"                             "=K")
("amp"/"amps"                     "l@ampere")
("A"                              "s@ampere")
("mol"                            "s@mole")
("cd"                             "s@candela")

("celsius"                        "l@!")
("°C"                             "celsius")
("C"                              "=°C")
("rankine"                        "l@5/9 K")
("°R"                             "rankine")
("fahrenheit"                     "l@!")
("°F"                             "fahrenheit")
("F"                              "=°F")

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
