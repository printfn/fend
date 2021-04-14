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

const UNIT_DEFS: &[UnitDef] = &[
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
    // number words
    s("one", "=1"),
    s("two", "=2"),
    s("couple", "=2"),
    s("three", "=3"),
    s("four", "=4"),
    s("quadruple", "=4"),
    s("five", "=5"),
    s("quintuple", "=5"),
    s("six", "=6"),
    s("seven", "=7"),
    s("eight", "=8"),
    s("nine", "=9"),
    s("ten", "=10"),
    s("eleven", "=11"),
    s("twelve", "=12"),
    s("thirteen", "=13"),
    s("fourteen", "=14"),
    s("fifteen", "=15"),
    s("sixteen", "=16"),
    s("seventeen", "=17"),
    s("eighteen", "=18"),
    s("nineteen", "=19"),
    s("twenty", "=20"),
    s("thirty", "=30"),
    s("forty", "=40"),
    s("fifty", "=50"),
    s("sixty", "=60"),
    s("seventy", "=70"),
    s("eighty", "=80"),
    s("ninety", "=90"),
    s("hundred", "=100"),
    s("thousand", "=1000"),
    s("million", "=1e6"),
    s("billion", "=1e9"),
    s("trillion", "=1e12"),
    s("quadrillion", "=1e15"),
    s("quintillion", "=1e18"),
    s("sextillion", "=1e21"),
    s("septillion", "=1e24"),
    s("octillion", "=1e27"),
    s("nonillion", "=1e30"),
    s("decillion", "=1e33"),
    s("undecillion", "=1e36"),
    s("duodecillion", "=1e39"),
    s("tredecillion", "=1e42"),
    s("quattuordecillion", "=1e45"),
    s("quindecillion", "=1e48"),
    s("sexdecillion", "=1e51"),
    s("septendecillion", "=1e54"),
    s("octodecillion", "=1e57"),
    s("novemdecillion", "=1e60"),
    s("vigintillion", "=1e63"),
    s("unvigintillion", "=1e66"),
    s("duovigintillion", "=1e69"),
    s("trevigintillion", "=1e72"),
    s("quattuorvigintillion", "=1e75"),
    s("quinvigintillion", "=1e78"),
    s("sexvigintillion", "=1e81"),
    s("septenvigintillion", "=1e84"),
    s("octovigintillion", "=1e87"),
    s("novemvigintillion", "=1e90"),
    s("trigintillion", "=1e93"),
    s("untrigintillion", "=1e96"),
    s("duotrigintillion", "=1e99"),
    s("googol", "=1e100"),
    s("tretrigintillion", "=1e102"),
    s("quattuortrigintillion", "=1e105"),
    s("quintrigintillion", "=1e108"),
    s("sextrigintillion", "=1e111"),
    s("septentrigintillion", "=1e114"),
    s("octotrigintillion", "=1e117"),
    s("novemtrigintillion", "=1e120"),
    s("centillion", "=1e303"),
    // constants
    s("c", "=299792458 m/s"),            // speed of light in vacuum (exact)
    s("h", "s@=6.62607015e-34 J s"),     // Planck constant (exact)
    s("boltzmann", "=1.380649e-23 J/K"), // Boltzmann constant (exact)
    s("electroncharge", "=1.602176634e-19 coulomb"), // electron charge (exact)
    s("avogadro", "=6.02214076e23 / mol"), // Size of a mole (exact)
    s("N_A", "=avogadro"),
    // angles
    p("radian", "radians", "l@1"),
    p("steradian", "steradians", "l@1"),
    s("sr", "s@steradian"),
    // common SI derived units
    p("newton", "newtons", "l@kg m / s^2"), // force
    s("N", "s@newton"),
    p("pascal", "pascals", "l@N/m^2"), // pressure or stress
    s("Pa", "s@pascal"),
    p("joule", "joules", "l@N m"), // energy
    s("J", "s@joule"),
    p("watt", "watts", "l@J/s"), // power
    s("W", "s@watt"),
    s("coulomb", "l@A s"), // charge
    s("C", "s@coulomb"),
    p("volt", "volts", "l@W/A"), // potential difference
    s("V", "s@volt"),
    p("ohm", "ohms", "l@V/A"), // electrical resistance
    s("siemens", "l@A/V"),     // electrical conductance
    s("S", "s@siemens"),
    s("farad", "l@C/V"), // capacitance
    s("F", "s@farad"),
    s("weber", "l@V s"), // magnetic flux
    s("Wb", "s@weber"),
    s("henry", "l@V s / A"), // inductance
    s("H", "s@henry"),
    s("tesla", "l@Wb/m^2"), // magnetic flux density
    s("T", "s@tesla"),
    s("hertz", "l@/s"), // frequency
    s("Hz", "s@hertz"),
    s("\u{2030}", "0.001"), // per mille
];

#[allow(clippy::too_many_lines)]
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
    for def in UNIT_DEFS {
        if def.singular == ident || def.plural == ident {
            return Some((def.singular, def.plural, def.definition));
        }
    }
    None
}
