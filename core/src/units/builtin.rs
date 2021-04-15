#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
struct UnitDef {
    singular: &'static str,
    plural: &'static str,
    definition: &'static str,
}

// singular, plural (or empty), definition, description
type UnitTuple = (&'static str, &'static str, &'static str, &'static str);

const BASE_UNITS: &[UnitTuple] = &[
    ("unitless", "", "=1", ""),
    ("second", "seconds", "l@!", ""),
    ("meter", "meters", "l@!", ""),
    ("kilogram", "kilograms", "l@!", ""),
    ("kelvin", "", "l@!", ""),
    ("ampere", "amperes", "l@!", ""),
    ("mole", "moles", "l@!", ""),
    ("candela", "candelas", "l@!", ""),
];

const BASE_UNIT_ABBREVIATIONS: &[UnitTuple] = &[
    ("s", "", "s@second", ""),
    ("metre", "metres", "l@meter", ""),
    ("m", "", "s@meter", ""),
    ("gram", "grams", "l@1/1000 kilogram", ""),
    ("g", "", "s@gram", ""),
    ("K", "", "s@kelvin", ""),
    ("\u{b0}K", "", "=K", ""),
    ("amp", "amps", "l@ampere", ""),
    ("A", "", "s@ampere", ""),
    ("mol", "", "s@mole", ""),
    ("cd", "", "s@candela", ""),
];

// some temperature scales have special support for conversions
const TEMPERATURE_SCALES: &[UnitTuple] = &[
    ("celsius", "", "l@!", ""),
    ("\u{b0}C", "", "celsius", ""), // degree symbol
    ("C", "", "=\u{b0}C", ""),
    ("rankine", "", "l@5/9 K", ""),
    ("\u{b0}R", "", "rankine", ""),
    ("fahrenheit", "", "l@!", ""),
    ("\u{b0}F", "", "fahrenheit", ""),
    ("F", "", "=\u{b0}F", ""),
];

const BITS_AND_BYTES: &[UnitTuple] = &[
    ("bit", "bits", "l@!", ""),
    ("bps", "", "s@bits/second", ""),
    ("byte", "bytes", "l@8 bits", ""),
    ("b", "", "s@bit", ""),
    ("B", "", "s@byte", ""),
    ("octet", "octets", "l@8 bits", ""),
];

const STANDARD_PREFIXES: &[UnitTuple] = &[
    ("yotta", "", "lp@1e24", ""),
    ("zetta", "", "lp@1e21", ""),
    ("exa", "", "lp@1e18", ""),
    ("peta", "", "lp@1e15", ""),
    ("tera", "", "lp@1e12", ""),
    ("giga", "", "lp@1e9", ""),
    ("mega", "", "lp@1e6", ""),
    ("myria", "", "lp@1e4", ""),
    ("kilo", "", "lp@1e3", ""),
    ("hecto", "", "lp@1e2", ""),
    ("deca", "", "lp@1e1", ""),
    ("deka", "", "lp@deca", ""),
    ("deci", "", "lp@1e-1", ""),
    ("centi", "", "lp@1e-2", ""),
    ("milli", "", "lp@1e-3", ""),
    ("micro", "", "lp@1e-6", ""),
    ("nano", "", "lp@1e-9", ""),
    ("pico", "", "lp@1e-12", ""),
    ("femto", "", "lp@1e-15", ""),
    ("atto", "", "lp@1e-18", ""),
    ("zepto", "", "lp@1e-21", ""),
    ("yocto", "", "lp@1e-24", ""),
    ("k", "", "=1000", ""),
];

const NON_STANDARD_PREFIXES: &[UnitTuple] = &[
    ("quarter", "", "lp@1/4", ""),
    ("semi", "", "lp@0.5", ""),
    ("demi", "", "lp@0.5", ""),
    ("hemi", "", "lp@0.5", ""),
    ("half", "", "lp@0.5", ""),
    ("double", "", "lp@2", ""),
    ("triple", "", "lp@3", ""),
    ("treble", "", "lp@3", ""),
];

const BINARY_PREFIXES: &[UnitTuple] = &[
    ("kibi", "", "lp@2^10", ""),
    ("mebi", "", "lp@2^20", ""),
    ("gibi", "", "lp@2^30", ""),
    ("tebi", "", "lp@2^40", ""),
    ("pebi", "", "lp@2^50", ""),
    ("exbi", "", "lp@2^60", ""),
    ("zebi", "", "lp@2^70", ""),
    ("yobi", "", "lp@2^80", ""),
];

const NUMBER_WORDS: &[UnitTuple] = &[
    ("tithe", "", "=1/10", ""),
    ("one", "", "=1", ""),
    ("two", "", "=2", ""),
    ("couple", "", "=2", ""),
    ("three", "", "=3", ""),
    ("four", "", "=4", ""),
    ("quadruple", "", "=4", ""),
    ("five", "", "=5", ""),
    ("quintuple", "", "=5", ""),
    ("six", "", "=6", ""),
    ("seven", "", "=7", ""),
    ("eight", "", "=8", ""),
    ("nine", "", "=9", ""),
    ("ten", "", "=10", ""),
    ("eleven", "", "=11", ""),
    ("twelve", "", "=12", ""),
    ("dozen", "", "=12", ""),
    ("thirteen", "", "=13", ""),
    ("bakersdozen", "", "=13", ""),
    ("fourteen", "", "=14", ""),
    ("fifteen", "", "=15", ""),
    ("sixteen", "", "=16", ""),
    ("seventeen", "", "=17", ""),
    ("eighteen", "", "=18", ""),
    ("nineteen", "", "=19", ""),
    ("twenty", "", "=20", ""),
    ("score", "", "=20", ""),
    ("thirty", "", "=30", ""),
    ("forty", "", "=40", ""),
    ("fifty", "", "=50", ""),
    ("sixty", "", "=60", ""),
    ("seventy", "", "=70", ""),
    ("eighty", "", "=80", ""),
    ("ninety", "", "=90", ""),
    ("hundred", "", "=100", ""),
    ("gross", "", "=144", ""),
    ("greatgross", "", "=12 gross", ""),
    ("thousand", "", "=1000", ""),
    ("million", "", "=1e6", ""),
    ("billion", "", "=1e9", ""),
    ("trillion", "", "=1e12", ""),
    ("quadrillion", "", "=1e15", ""),
    ("quintillion", "", "=1e18", ""),
    ("sextillion", "", "=1e21", ""),
    ("septillion", "", "=1e24", ""),
    ("octillion", "", "=1e27", ""),
    ("nonillion", "", "=1e30", ""),
    ("decillion", "", "=1e33", ""),
    ("undecillion", "", "=1e36", ""),
    ("duodecillion", "", "=1e39", ""),
    ("tredecillion", "", "=1e42", ""),
    ("quattuordecillion", "", "=1e45", ""),
    ("quindecillion", "", "=1e48", ""),
    ("sexdecillion", "", "=1e51", ""),
    ("septendecillion", "", "=1e54", ""),
    ("octodecillion", "", "=1e57", ""),
    ("novemdecillion", "", "=1e60", ""),
    ("vigintillion", "", "=1e63", ""),
    ("unvigintillion", "", "=1e66", ""),
    ("duovigintillion", "", "=1e69", ""),
    ("trevigintillion", "", "=1e72", ""),
    ("quattuorvigintillion", "", "=1e75", ""),
    ("quinvigintillion", "", "=1e78", ""),
    ("sexvigintillion", "", "=1e81", ""),
    ("septenvigintillion", "", "=1e84", ""),
    ("octovigintillion", "", "=1e87", ""),
    ("novemvigintillion", "", "=1e90", ""),
    ("trigintillion", "", "=1e93", ""),
    ("untrigintillion", "", "=1e96", ""),
    ("duotrigintillion", "", "=1e99", ""),
    ("googol", "", "=1e100", ""),
    ("tretrigintillion", "", "=1e102", ""),
    ("quattuortrigintillion", "", "=1e105", ""),
    ("quintrigintillion", "", "=1e108", ""),
    ("sextrigintillion", "", "=1e111", ""),
    ("septentrigintillion", "", "=1e114", ""),
    ("octotrigintillion", "", "=1e117", ""),
    ("novemtrigintillion", "", "=1e120", ""),
    ("centillion", "", "=1e303", ""),
];

const CONSTANTS: &[UnitTuple] = &[
    (
        "c",
        "",
        "=299792458 m/s",
        "speed of light in vacuum (exact)",
    ),
    ("h", "", "s@=6.62607015e-34 J s", "Planck constant (exact)"),
    (
        "boltzmann",
        "",
        "=1.380649e-23 J/K",
        "Boltzmann constant (exact)",
    ),
    (
        "electron_charge",
        "",
        "=1.602176634e-19 coulomb",
        "electron charge (exact)",
    ),
    (
        "avogadro",
        "",
        "=6.02214076e23 / mol",
        "size of a mole (exact)",
    ),
    ("N_A", "", "=avogadro", ""),
    (
        "G",
        "",
        "=6.67430e-11 N m^2 / kg^2",
        "gravitational constant",
    ),
    ("gravity", "", "=9.80665 m/s^2", ""),
    ("force", "", "gravity", ""), // used to convert some units
];

const ANGLES: &[UnitTuple] = &[
    ("radian", "radians", "l@1", ""),
    ("circle", "circles", "l@2 pi radian", ""),
    ("degree", "degrees", "l@1/360 circle", ""),
    ("deg", "degs", "l@degree", ""),
    ("\u{b0}", "", "degree", ""), // degree symbol
    ("arcdeg", "arcdegs", "degree", ""),
    ("arcmin", "arcmins", "l@1/60 degree", ""),
    ("arcminute", "arcminutes", "l@arcmin", ""),
    ("arcsec", "arcsecs", "l@1/60 arcmin", ""),
    ("arcsecond", "arcseconds", "l@arcsec", ""),
    ("rightangle", "rightangles", "l@90 degrees", ""),
    ("quadrant", "quadrants", "l@1/4 circle", ""),
    ("quintant", "quintants", "l@1/5 circle", ""),
    ("sextant", "sextants", "l@1/6 circle", ""),
    (
        "zodiac_sign",
        "zodiac_signs",
        "l@1/12 circle",
        "Angular extent of one sign of the zodiac",
    ),
    ("turn", "turns", "l@circle", ""),
    ("revolution", "revolutions", "l@circle", ""),
    ("rev", "revs", "l@circle", ""),
    ("gradian", "gradians", "l@1/100 rightangle", ""),
    ("gon", "gons", "l@gradian", ""),
    ("grad", "", "l@gradian", ""),
    ("mas", "", "milliarcsec", ""),
];

const SOLID_ANGLES: &[UnitTuple] = &[
    ("steradian", "steradians", "l@1", ""),
    ("sr", "sr", "s@steradian", ""),
    ("sphere", "spheres", "4 pi steradians", ""),
    (
        "squaredegree",
        "squaredegrees",
        "(1/180)^2 pi^2 steradians",
        "",
    ),
    ("squareminute", "squareminutes", "(1/60)^2 squaredegree", ""),
    ("squaresecond", "squareseconds", "(1/60)^2 squareminute", ""),
    ("squarearcmin", "squarearcmins", "squareminute", ""),
    ("squarearcsec", "squarearcsecs", "squaresecond", ""),
    (
        "sphericalrightangle",
        "sphericalrightangles",
        "0.5 pi steradians",
        "",
    ),
    ("octant", "octants", "0.5 pi steradians", ""),
];

const COMMON_SI_DERIVED_UNITS: &[UnitTuple] = &[
    ("newton", "newtons", "l@kg m / s^2", "force"),
    ("N", "", "s@newton", ""),
    ("pascal", "pascals", "l@N/m^2", "pressure or stress"),
    ("Pa", "", "s@pascal", ""),
    ("joule", "joules", "l@N m", "energy"),
    ("J", "", "s@joule", ""),
    ("watt", "watts", "l@J/s", "power"),
    ("W", "", "s@watt", ""),
    ("coulomb", "", "l@A s", "charge"),
    ("C", "", "s@coulomb", ""),
    ("volt", "volts", "l@W/A", "potential difference"),
    ("V", "", "s@volt", ""),
    ("ohm", "ohms", "l@V/A", "electrical resistance"),
    ("siemens", "", "l@A/V", "electrical conductance"),
    ("S", "", "s@siemens", ""),
    ("farad", "", "l@C/V", "capacitance"),
    ("F", "", "s@farad", ""),
    ("weber", "", "l@V s", "magnetic flux"),
    ("Wb", "", "s@weber", ""),
    ("henry", "", "l@V s / A", "inductance"),
    ("H", "", "s@henry", ""),
    ("tesla", "", "l@Wb/m^2", "magnetic flux density"),
    ("T", "", "s@tesla", ""),
    ("hertz", "", "l@/s", "frequency"),
    ("Hz", "", "s@hertz", ""),
];

const TIME_UNITS: &[UnitTuple] = &[
    ("sec", "secs", "s@second", ""),
    ("minute", "minutes", "l@60 seconds", ""),
    ("min", "mins", "s@minute", ""),
    ("hour", "hours", "l@60 minutes", ""),
    ("hr", "hrs", "s@hour", ""),
    ("day", "days", "l@24 hours", ""),
    ("d", "", "s@day", ""),
    ("da", "", "s@day", ""),
    ("week", "weeks", "l@7 days", ""),
    ("wk", "", "s@week", ""),
    ("fortnight", "fortnights", "l@14 day", ""),
    (
        "sidereal_year",
        "sidereal_years",
        "365.256363004 days",
        concat!(
            "the time taken for the Earth to complete one revolution of its orbit, ",
            "as measured against a fixed frame of reference (such as the fixed stars, ",
            "Latin sidera, singular sidus)"
        ),
    ),
    (
        "tropical_year",
        "tropical_years",
        "365.242198781 days",
        "the period of time for the mean ecliptic longitude of the Sun to increase by 360 degrees",
    ),
    (
        "anomalistic_year",
        "anomalistic_years",
        "365.259636 days",
        "the time taken for the Earth to complete one revolution with respect to its apsides",
    ),
    ("year", "years", "l@tropical_year", ""),
    ("yr", "", "year", ""),
    ("month", "months", "l@1/12 year", ""),
    ("mo", "", "month", ""),
    ("decade", "decades", "10 years", ""),
    ("century", "centuries", "100 years", ""),
    ("millennium", "millennia", "1000 years", ""),
    ("solar_year", "solar_years", "year", ""),
    ("calendar_year", "calendar_years", "365 days", ""),
    ("common_year", "common_years", "365 days", ""),
    ("leap_year", "leap_years", "366 days", ""),
    ("julian_year", "julian_years", "365.25 days", ""),
    ("gregorian_year", "gregorian_years", "365.2425 days", ""),
    // french revolutionary time
    ("decimal_hour", "decimal_hours", "l@1/10 day", ""),
    (
        "decimal_minute",
        "decimal_minutes",
        "l@1/100 decimalhour",
        "",
    ),
    (
        "decimal_second",
        "decimal_seconds",
        "l@1/100 decimalminute",
        "",
    ),
    ("beat", "beats", "l@decimalminute", "Swatch Internet Time"),
];

const RATIOS: &[UnitTuple] = &[
    ("\u{2030}", "", "0.001", ""), // per mille
    ("percent", "", "0.01", ""),
    ("%", "", "percent", ""),
    ("mill", "mills", "0.001", ""),
    ("ppm", "", "1e-6", ""),
    ("parts_per_million", "", "ppm", ""),
    ("ppb", "", "1e-9", ""),
    ("parts_per_billion", "", "ppb", ""),
    ("ppt", "", "1e-12", ""),
    ("parts_per_trillion", "", "ppt", ""),
    ("karat", "", "1/24", "measure of gold purity"),
    ("basispoint", "", "0.01 %", ""),
];

const COMMON_PHYSICAL_UNITS: &[UnitTuple] = &[
    ("electron_volt", "electron_volts", "l@electron_charge V", ""),
    ("eV", "", "s@electron_volt", ""),
    ("light_year", "light_years", "c julian_year", ""),
    ("lightyear", "lightyears", "light_year", ""), // TODO remove this compatibility unit
    ("ly", "", "lightyear", ""),
    ("light_second", "light_seconds", "c second", ""),
    ("light_minute", "light_minutes", "c minute", ""),
    ("light_hour", "light_hours", "c hour", ""),
    ("light_day", "light_days", "c day", ""),
    ("parsec", "parsecs", "au / tan(arcsec)", ""),
    ("pc", "", "parsec", ""),
    (
        "astronomical_unit",
        "astronomical_units",
        "149597870700 m",
        "",
    ),
    ("au", "", "astronomical_unit", ""),
    ("AU", "", "astronomical_unit", ""),
    ("barn", "", "l@1e-28 m^2", ""),
    ("shed", "", "l@1e-24 barn", ""),
    ("cc", "", "cm^3", ""),
    ("are", "ares", "l@100 meter^2", ""),
    ("liter", "liters", "l@1000 cc", ""),
    ("l", "", "s@liter", ""),
    ("L", "", "s@liter", ""),
    ("micron", "microns", "l@micrometer", ""),
    ("bicron", "bicrons", "l@picometer", ""),
    ("gsm", "", "grams / meter^2", ""),
    ("hectare", "hectares", "hectoare", ""),
    ("calorie", "calories", "l@4.184 J", ""),
    ("cal", "", "s@calorie", ""),
    (
        "british_thermal_unit",
        "british_thermal_units",
        "1055.05585 J",
        "",
    ),
    ("btu", "", "british_thermal_unit", ""),
    ("Wh", "", "s@W hour", ""),
    ("bar", "", "l@1e5 Pa", "about 1 atmosphere"),
    ("diopter", "", "l@/m", "reciprocal of focal length"),
];

const IMPERIAL_UNITS: &[UnitTuple] = &[
    ("inch", "inches", "2.54 cm", ""),
    ("in", "", "inch", ""),
    ("\u{2019}", "", "foot", ""), // unicode single quote
    ("\u{201d}", "", "inch", ""), // unicode double quote
    ("'", "", "foot", ""),
    ("\"", "", "inch", ""),
    ("foot", "feet", "l@12 inch", ""),
    ("ft", "", "foot", ""),
    ("yard", "yards", "l@3 ft", ""),
    ("yd", "", "yard", ""),
    ("mile", "miles", "l@5280 ft", ""),
    ("line", "lines", "1/12 inch", ""),
    ("rod", "", "5.5 yard", ""),
    ("perch", "", "rod", ""),
    ("furlong", "", "40 rod", ""),
    ("statute_mile", "statute_miles", "mile", ""),
    ("league", "", "3 mile", ""),
    ("chain", "chains", "66 feet", ""),
    ("link", "links", "1/100 chain", ""),
    ("ch", "", "chain", ""),
    ("acre", "acres", "10 chain^2", ""),
    ("section", "sections", "mile^2", ""),
    ("township", "townships", "36 sections", ""),
    ("homestead", "homesteads", "160 acres", ""),
];

const AVOIRDUPOIS_WEIGHT: &[UnitTuple] = &[
    ("pound", "pounds", "0.45359237 kg", ""),
    ("lb", "lbs", "pound", ""),
    ("grain", "grains", "1/7000 pound", ""),
    ("ounce", "ounces", "1/16 pound", ""),
    ("oz", "", "ounce", ""),
    ("dram", "drams", "1/16 ounce", ""),
    ("dr", "", "dram", ""),
    ("hundredweight", "hundredweights", "100 pounds", ""),
    ("cwt", "", "hundredweight", ""),
    ("short_ton", "short_tons", "2000 pounds", ""),
    ("quarterweight", "quarterweights", "1/4 short_ton", ""),
];

const TROY_WEIGHT: &[UnitTuple] = &[
    ("troy_pound", "troy_pounds", "5760 grains", ""),
    ("troy_ounce", "troy_ounces", "1/12 troy_pound", ""),
    ("ozt", "", "troyounce", ""),
    ("pennyweight", "pennyweights", "1/20 troy_ounce", ""),
    ("dwt", "", "pennyweight", ""),
];

const OTHER_WEIGHTS: &[UnitTuple] = &[
    ("metric_grain", "metric_grains", "50 mg", ""),
    ("carat", "carats", "0.2 grams", ""),
    ("ct", "", "carat", ""),
    ("jewellers_point", "jewellers_points", "1/100 carat", ""),
    ("tonne", "tonnes", "l@1000 kg", ""),
    ("t", "", "tonne", ""),
];

const IMPERIAL_ABBREVIATIONS: &[UnitTuple] = &[
    ("mph", "", "mile/hr", ""),
    ("mpg", "", "mile/gal", ""),
    ("kph", "", "km/hr", ""),
    ("fL", "", "footlambert", ""),
    ("fpm", "", "ft/min", ""),
    ("fps", "", "ft/s", ""),
    ("rpm", "", "rev/min", ""),
    ("rps", "", "rev/sec", ""),
    ("mi", "", "mile", ""),
    ("smi", "", "mile", ""),
    ("nmi", "", "nauticalmile", ""),
    ("mbh", "", "1e3 btu/hour", ""),
    ("mcm", "", "1e3 circularmil", ""),
    ("ipy", "", "inch/year", ""),
    ("ccf", "", "100 ft^3", ""),
    ("Mcf", "", "1000 ft^3", ""),
    ("hph", "", "hp hour", ""),
    ("plf", "", "lb / foot", "pounds per linear foot"),
    ("lbf", "", "lb force", ""),
    ("psi", "", "pound force / inch^2", ""),
];

const NAUTICAL_UNITS: &[UnitTuple] = &[
    ("fathom", "fathoms", "6 ft", ""),
    ("nautical_mile", "nautical_miles", "1852 m", ""),
    ("cable", "cables", "1/10 nautical_mile", ""),
    ("marine_league", "marine_leagues", "3 nautical_mile", ""),
    ("knot", "knots", "nautical_mile / hr", ""),
    ("click", "clicks", "km", ""),
    ("NM", "", "nautical_mile", ""),
];

const ALL_UNIT_DEFS: &[&[UnitTuple]] = &[
    BASE_UNITS,
    BASE_UNIT_ABBREVIATIONS,
    TEMPERATURE_SCALES,
    BITS_AND_BYTES,
    STANDARD_PREFIXES,
    NON_STANDARD_PREFIXES,
    BINARY_PREFIXES,
    NUMBER_WORDS,
    CONSTANTS,
    ANGLES,
    SOLID_ANGLES,
    COMMON_SI_DERIVED_UNITS,
    TIME_UNITS,
    RATIOS,
    COMMON_PHYSICAL_UNITS,
    IMPERIAL_UNITS,
    AVOIRDUPOIS_WEIGHT,
    TROY_WEIGHT,
    OTHER_WEIGHTS,
    IMPERIAL_ABBREVIATIONS,
    NAUTICAL_UNITS,
];

const SHORT_PREFIXES: &[(&str, &str)] = &[
    ("Ki", "sp@kibi"),
    ("Mi", "sp@mebi"),
    ("Gi", "sp@gibi"),
    ("Ti", "sp@tebi"),
    ("Pi", "sp@pebi"),
    ("Ei", "sp@exbi"),
    ("Zi", "sp@zebi"),
    ("Yi", "sp@yobi"),
    ("Y", "sp@yotta"),
    ("Z", "sp@zetta"),
    ("E", "sp@exa"),
    ("P", "sp@peta"),
    ("T", "sp@tera"),
    ("G", "sp@giga"),
    ("M", "sp@mega"),
    ("k", "sp@kilo"),
    ("h", "sp@hecto"),
    ("da", "sp@deka"),
    ("d", "sp@deci"),
    ("c", "sp@centi"),
    ("m", "sp@milli"),
    ("u", "sp@micro"),       // alternative to µ
    ("\u{b5}", "sp@micro"),  // U+00B5 (micro sign)
    ("\u{3bc}", "sp@micro"), // U+03BC (lowercase µ)
    ("n", "sp@nano"),
    ("p", "sp@pico"),
    ("f", "sp@femto"),
    ("a", "sp@atto"),
    ("z", "sp@zepto"),
    ("y", "sp@yocto"),
];

#[allow(clippy::too_many_lines)]
pub(crate) fn query_unit<'a>(
    ident: &'a str,
    short_prefixes: bool,
    case_sensitive: bool,
) -> Option<(&'static str, &'static str, &'static str)> {
    if short_prefixes {
        for (name, def) in SHORT_PREFIXES {
            if *name == ident {
                return Some((name, name, def));
            }
        }
    }
    let mut candidates = vec![];
    for group in ALL_UNIT_DEFS {
        for def in *group {
            let def = UnitDef {
                singular: def.0,
                plural: if def.1.is_empty() { def.0 } else { def.1 },
                definition: def.2,
            };
            if def.singular == ident || def.plural == ident {
                return Some((def.singular, def.plural, def.definition));
            }
            if !case_sensitive
                && (def.singular.eq_ignore_ascii_case(ident)
                    || def.plural.eq_ignore_ascii_case(ident))
            {
                candidates.push(Some((def.singular, def.plural, def.definition)));
            }
        }
    }
    if candidates.len() == 1 {
        return candidates[0];
    }
    None
}
