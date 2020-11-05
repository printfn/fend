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
        ("s"                              "!")
        ("m"                              "!")
        ("kg"                             "!")
        ("A"                              "!")
        ("K"                              "!")
        ("mol"                            "!")
        ("cd"                             "!")
        ("bit"/"bits"                     "!")
        ("USD"                            "!")
        ("percent"                        "0.01")
        ("%"                              "percent")
        ("‰"                              "0.001")
        ("kilometer"/"kilometers"         "1000 m")
        ("dm"                             "0.1 m")
        ("L"                              "dm^3")
        ("cm"                             "0.01 m")
        ("mm"                             "0.001 m")
        ("um"                             "0.001 mm")
        ("µm"                             "um")
        ("nm"                             "1e-9 m")
        ("pm"                             "1e-12 m")
        ("fm"                             "1e-15 m")
        ("am"                             "1e-18 m")
        ("angstrom"                       "0.1 nm")
        ("barn"                           "100 fm^2")

        ("inch"/"inches"                  "2.54 cm")
        ("in"                             "inch")
        ("\""                             "inch")
        ("”"                              "inch") // unicode double quote
        ("foot"/"feet"                    "12 inches")
        ("ft"                             "foot")
        ("'"                              "foot")
        ("’"                              "foot") // unicode single quote
        ("yard"/"yards"                   "3 feet")
        ("mile"/"miles"                   "1760 yards")
        ("mi"                             "mile")
        ("NM"                             "1852 m")
        ("km"                             "1000 m")
        ("AU"                             "149,597,870,700 m")

        ("g"                              "1|1000 kg")
        ("mg"                             "1|1000 g")
        ("pound"/"pounds"                 "0.45359237 kg")
        ("lb"/"lbs"                       "pound")
        ("ounce"/"ounces"                 "1|16 lb")
        ("oz"                             "ounce")
        ("dram"/"drams"                   "1|16 oz")
        ("dr"                             "dram")
        ("grain"/"grains"                 "1|7000 lb")
        ("gr"                             "grain")
        ("quarter"/"quarters"             "25 lb")
        ("qr"                             "quarter")
        ("hundredweight"/"hundredweights" "100 lb")
        ("cwt"                            "hundredweight")
        ("short_ton"/"short_tons"         "2000 lb")

        ("kelvin"                         "K")

        ("N"                              "1 kg m / s^2")
        ("newton"/"newtons"               "1 N")
        ("joule"/"joules"                 "1 N m")
        ("J"                              "1 joule")
        ("pascal"/"pascals"               "1 kg m^-1 s^-2")
        ("Pa"                             "1 pascal")
        ("kPa"                            "1000 Pa")
        ("watt"/"watts"                   "1 J/s")
        ("W"                              "1 watt")
        ("coulomb"/"coulombs"             "1 A * 1 s")
        ("C"                              "1 coulomb")
        ("volt"/"volts"                   "1 J / C")
        ("V"                              "1 volt")
        ("ohm"/"ohms"                     "1 V / A")
        ("Ω"                              "ohm")
        ("siemens"                        "1 / ohm")
        ("S"                              "1 siemens")
        ("farad"                          "1 s / ohm")
        ("F"                              "1 farad")
        ("hertz"                          "1/s")
        ("Hz"                             "1 hertz")
        ("henry"                          "J / A^2")
        ("H"                              "1 henry")
        ("weber"                          "V s")
        ("Wb"                             "1 weber")
        ("tesla"                          "weber / m^2")
        ("T"                              "1 tesla")

        ("kgf"                            "9.806650 N")
        ("lbf"                            "kgf / kg * lb")
        ("psi"                            "lbf / inch^2")

        ("second"/"seconds"               "s")
        ("minute"/"minutes"               "60 s")
        ("min"/"mins"                     "minute")
        ("hour"/"hours"                   "60 minutes")
        ("hr"                             "hour")
        ("day"/"days"                     "24 hours")
        ("year"/"years"                   "365.25 days")

        // speed of light, needed for 'light years' etc.
        ("light"                          "299,792,458 m/s")
        ("ly"                             "light year")
        ("parsec"/"parsecs"               "648000 AU/pi")

        ("kph"                            "km / hr")
        ("mph"                            "mi / hr")

        ("b"                              "bit")
        ("byte"/"bytes"                   "8 bits")
        ("B"                              "byte")
        ("KB"                             "1000 byte")
        ("MB"                             "1000 KB")
        ("GB"                             "1000 MB")
        ("TB"                             "1000 GB")
        ("KiB"                            "1024 bytes")
        ("MiB"                            "1024 KiB")
        ("GiB"                            "1024 MiB")
        ("TiB"                            "1024 GiB")
        ("Kb"                             "1000 bits")
        ("Mb"                             "1000 Kb")
        ("Gb"                             "1000 Mb")
        ("Tb"                             "1000 Gb")
        ("Kib"                            "1024 bits")
        ("Mib"                            "1024 Kib")
        ("Gib"                            "1024 Mib")
        ("Tib"                            "1024 Gib")

        ("radian"/"radians"               "1")
        ("circle"/"circles"               "2 pi radians")
        ("degree"/"degrees"               "1|360 circle")
        ("°"                              "degree")
    )
}
