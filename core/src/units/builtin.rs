#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
struct UnitDef {
    singular: &'static str,
    plural: &'static str,
    definition: &'static str,
}

/// construct a unit with only a singular name
const fn s(name: &'static str, definition: &'static str) -> UnitDef {
    UnitDef {
        singular: name,
        plural: name,
        definition,
    }
}

/// construct a unit with only singular and plural names
const fn p(singular: &'static str, plural: &'static str, definition: &'static str) -> UnitDef {
    UnitDef {
        singular,
        plural,
        definition,
    }
}

const UNIT_DEFS: [UnitDef; 71] = [
    s("unitless", "=1"),
    // SI base units
    p("second", "seconds", "l@!"),
    p("meter", "meters", "l@!"),
    p("kilogram", "kilograms", "l@!"),
    s("kelvin", "l@!"),
    p("ampere", "amperes", "l@!"),
    p("mole", "moles", "l@!"),
    p("candela", "candelas", "l@!"),
    // SI base unit abbreviations
    s("s", "s@second"),
    p("metre", "metres", "l@meter"),
    s("m", "s@meter"),
    p("gram", "grams", "l@1/1000 kilogram"),
    s("g", "s@gram"),
    s("K", "s@kelvin"),
    s("\u{b0}K", "=K"),
    p("amp", "amps", "l@ampere"),
    s("A", "s@ampere"),
    s("mol", "s@mole"),
    s("cd", "s@candela"),
    // temperature scales (these have special support for conversions)
    s("celsius", "l@!"),
    s("\u{b0}C", "celsius"), // degree symbol
    s("C", "=\u{b0}C"),
    s("rankine", "l@5/9 K"),
    s("\u{b0}R", "rankine"),
    s("fahrenheit", "l@!"),
    s("\u{b0}F", "fahrenheit"),
    s("F", "=\u{b0}F"),
    // bits and bytes
    p("bit", "bits", "l@!"),
    s("bps", "s@bits/second"),
    p("byte", "bytes", "l@8 bits"),
    s("b", "s@bit"),
    s("B", "s@byte"),
    p("octet", "octets", "l@8 bits"),
    // standard prefixes
    s("yotta", "lp@1e24"),
    s("zetta", "lp@1e21"),
    s("exa", "lp@1e18"),
    s("peta", "lp@1e15"),
    s("tera", "lp@1e12"),
    s("giga", "lp@1e9"),
    s("mega", "lp@1e6"),
    s("myria", "lp@1e4"),
    s("kilo", "lp@1e3"),
    s("hecto", "lp@1e2"),
    s("deca", "lp@1e1"),
    s("deka", "lp@deca"),
    s("deci", "lp@1e-1"),
    s("centi", "lp@1e-2"),
    s("milli", "lp@1e-3"),
    s("micro", "lp@1e-6"),
    s("nano", "lp@1e-9"),
    s("pico", "lp@1e-12"),
    s("femto", "lp@1e-15"),
    s("atto", "lp@1e-18"),
    s("zepto", "lp@1e-21"),
    s("yocto", "lp@1e-24"),
    // non-standard prefixes
    s("quarter", "lp@1/4"),
    s("semi", "lp@0.5"),
    s("demi", "lp@0.5"),
    s("hemi", "lp@0.5"),
    s("half", "lp@0.5"),
    s("double", "lp@2"),
    s("triple", "lp@3"),
    s("treble", "lp@3"),
    // binary prefixes
    s("kibi", "lp@2^10"),
    s("mebi", "lp@2^20"),
    s("gibi", "lp@2^30"),
    s("tebi", "lp@2^40"),
    s("pebi", "lp@2^50"),
    s("exbi", "lp@2^60"),
    s("zebi", "lp@2^70"),
    s("yobi", "lp@2^80"),
];

#[allow(clippy::too_many_lines)]
#[rustfmt::skip::macros(define_units)]
pub(crate) fn query_unit<'a>(
    ident: &'a str,
    short_prefixes: bool,
) -> Option<(&'static str, &'static str, &'static str)> {
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
    for def in &UNIT_DEFS {
        if def.singular == ident || def.plural == ident {
            return Some((def.singular, def.plural, def.definition));
        }
    }
    None
}
