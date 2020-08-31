use crate::num::exact_base::ExactBase;
use crate::num::{Base, FormattingStyle};
use crate::value::Value;
use std::ops::{Mul, Neg};
use std::{
    collections::HashMap,
    fmt::{Display, Error, Formatter},
};

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct UnitValue {
    value: ExactBase,
    unit: Unit,
}

impl UnitValue {
    #[allow(clippy::too_many_lines)]
    pub fn create_initial_units() -> HashMap<String, Value> {
        Self::create_units(vec![
            ("percent", "percent", true, Some("0.01")),
            ("%", "%", false, Some("percent")),
            ("\u{2030}", "\u{2030}", false, Some("0.001")), // per mille (‰)
            ("s", "s", true, None),
            ("second", "seconds", true, Some("s")),
            ("m", "m", true, None),
            ("dm", "dm", true, Some("0.1m")),
            ("L", "L", true, Some("dm^3")),
            ("cm", "cm", true, Some("0.01m")),
            ("mm", "mm", true, Some("0.001m")),
            ("um", "um", true, Some("0.001mm")),
            ("\u{b5}m", "\u{b5}m", true, Some("0.001mm")), // micrometres (µm)
            ("nm", "nm", true, Some("1e-9m")),
            ("pm", "pm", true, Some("1e-12m")),
            ("fm", "fm", true, Some("1e-15m")),
            ("am", "am", true, Some("1e-18m")),
            ("angstrom", "angstrom", true, Some("0.1nm")),
            ("barn", "barn", true, Some("100fm^2")),
            ("inch", "inches", true, Some("2.54cm")),
            ("in", "in", true, Some("inch")),
            ("ft", "ft", true, Some("12 inches")),
            ("foot", "feet", true, Some("1ft")),
            ("\"", "\"", false, Some("inch")),
            ("\u{201d}", "\u{201d}", false, Some("inch")), // Unicode double quote (”)
            ("'", "'", false, Some("foot")),
            ("\u{2019}", "\u{2019}", false, Some("foot")), // Unicode single quote (’)
            ("yard", "yards", true, Some("3 feet")),
            ("mile", "miles", true, Some("1760 yards")),
            ("mi", "mi", true, Some("mile")),
            ("km", "km", true, Some("1000m")),
            ("AU", "AU", true, Some("149597870700m")),
            ("kg", "kg", true, None),
            ("lb", "lbs", true, Some("0.45359237kg")),
            ("pound", "pounds", true, Some("1lb")),
            ("ounce", "ounces", true, Some("1/16 lb")),
            ("oz", "oz", true, Some("1 ounce")),
            ("dram", "drams", true, Some("1/16 oz")),
            ("dr", "dr", true, Some("1 dram")),
            ("grain", "grains", true, Some("1/7000 lb")),
            ("gr", "gr", true, Some("1 grain")),
            ("quarter", "quarters", true, Some("25lb")),
            ("qr", "qr", true, Some("1 quarter")),
            ("hundredweight", "hundredweights", true, Some("100lb")),
            ("cwt", "cwt", true, Some("1 hundredweight")),
            ("short_ton", "short_tons", true, Some("2000lb")),
            ("A", "A", true, None),
            ("K", "K", true, None),
            ("kelvin", "kelvin", true, Some("K")),
            ("mol", "mol", true, None),
            ("cd", "cd", true, None),
            ("g", "g", true, Some("(1/1000)kg")),
            ("mg", "mg", true, Some("(1/1000)g")),
            ("N", "N", true, Some("1 kg m / s^2")),
            ("newton", "newtons", true, Some("1 N")),
            ("joule", "joules", true, Some("1 N m")),
            ("J", "J", true, Some("1 joule")),
            ("pascal", "pascals", true, Some("1 kg m^-1 s^-2")),
            ("Pa", "Pa", true, Some("1 pascal")),
            ("kPa", "kPa", true, Some("1000 Pa")),
            ("watt", "watts", true, Some("1 J/s")),
            ("W", "W", true, Some("1 watt")),
            ("coulomb", "coulombs", true, Some("1 A * 1 s")),
            ("C", "C", true, Some("1 coulomb")),
            ("volt", "volts", true, Some("1 J / C")),
            ("V", "V", true, Some("1 volt")),
            ("ohm", "ohms", true, Some("1 V / A")),
            ("\u{3a9}", "\u{3a9}", true, Some("1 ohm")), // Omega symbol (Ω)
            ("siemens", "siemens", true, Some("1 / ohm")),
            ("S", "S", true, Some("1 siemens")),
            ("farad", "farad", true, Some("1 s / ohm")),
            ("F", "F", true, Some("1 farad")),
            ("hertz", "hertz", true, Some("1/s")),
            ("Hz", "Hz", true, Some("1 hertz")),
            ("henry", "henry", true, Some("J / A^2")),
            ("H", "H", true, Some("1 henry")),
            ("weber", "weber", true, Some("V s")),
            ("Wb", "Wb", true, Some("1 weber")),
            ("tesla", "tesla", true, Some("weber / m^2")),
            ("T", "T", true, Some("1 tesla")),
            ("kgf", "kgf", true, Some("9.806650 N")),
            ("lbf", "lbf", true, Some("kgf / kg * lb")),
            ("psi", "psi", true, Some("lbf / inch^2")),
            ("min", "min", true, Some("60s")),
            ("hr", "hr", true, Some("60min")),
            ("hour", "hours", true, Some("hr")),
            ("minute", "minutes", true, Some("min")),
            ("day", "days", true, Some("24 hours")),
            ("year", "years", true, Some("365.25 days")),
            ("light", "light", true, Some("299_792_458m/s")),
            ("ly", "ly", true, Some("365.25 light days")),
            ("parsec", "parsecs", true, Some("648000AU/pi")),
            ("kph", "kph", true, Some("1 km / hr")),
            ("mph", "mph", true, Some("1 mile / hr")),
            ("bit", "bits", true, None),
            ("b", "b", true, Some("bit")),
            ("byte", "bytes", true, Some("8 bit")),
            ("B", "B", true, Some("byte")),
            ("KB", "KB", true, Some("1000 bytes")),
            ("MB", "MB", true, Some("1000 KB")),
            ("GB", "GB", true, Some("1000 MB")),
            ("TB", "TB", true, Some("1000 GB")),
            ("KiB", "KiB", true, Some("1024 bytes")),
            ("MiB", "MiB", true, Some("1024 KiB")),
            ("GiB", "GiB", true, Some("1024 MiB")),
            ("TiB", "TiB", true, Some("1024 GiB")),
            ("Kb", "Kb", true, Some("1000 bits")),
            ("Mb", "Mb", true, Some("1000 Kb")),
            ("Gb", "Gb", true, Some("1000 Mb")),
            ("Tb", "Tb", true, Some("1000 Gb")),
            ("Kib", "Kib", true, Some("1024 bits")),
            ("Mib", "Mib", true, Some("1024 Kib")),
            ("Gib", "Gib", true, Some("1024 Mib")),
            ("Tib", "Tib", true, Some("1024 Gib")),
            ("USD", "USD", true, None),
        ])
    }

    pub fn try_as_usize(self) -> Result<usize, String> {
        if !self.is_unitless() {
            return Err("Cannot convert number with unit to integer".to_string());
        }
        Ok(self.value.try_as_usize()?)
    }

    fn create_units(
        unit_descriptions: Vec<(impl ToString, impl ToString, bool, Option<impl ToString>)>,
    ) -> HashMap<String, Value> {
        let mut scope = HashMap::new();
        for (singular_name, plural_name, space, expr) in unit_descriptions {
            let unit = if let Some(expr) = expr {
                Self::new_unit(
                    singular_name.to_string(),
                    plural_name.to_string(),
                    space,
                    expr.to_string().as_str(),
                    &scope,
                )
            } else {
                Self::new_base_unit(singular_name.to_string(), plural_name.to_string(), space)
            };
            scope.insert(singular_name.to_string(), Value::Num(unit.clone()));
            if plural_name.to_string() != singular_name.to_string() {
                scope.insert(plural_name.to_string(), Value::Num(unit));
            }
        }
        scope
    }

    fn new_unit(
        singular_name: String,
        plural_name: String,
        space: bool,
        expression: &str,
        scope: &HashMap<String, Value>,
    ) -> Self {
        // todo remove unwraps
        let value = crate::evaluate_to_value(expression, scope)
            .unwrap()
            .expect_num()
            .unwrap();
        let (hashmap, scale) = value.unit.to_hashmap_and_scale();
        let scale = scale * value.value;
        let resulting_unit = NamedUnit::new(singular_name, plural_name, space, hashmap, scale);
        Self::new(1, vec![UnitExponent::new(resulting_unit, 1)])
    }

    fn new_base_unit(singular_name: String, plural_name: String, space: bool) -> Self {
        let base_kg = BaseUnit::new(singular_name.clone());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new(singular_name, plural_name, space, hashmap, 1);
        Self::new(1, vec![UnitExponent::new(kg, 1)])
    }

    pub fn with_format(self, format: FormattingStyle) -> Self {
        Self {
            value: self.value.with_format(format),
            unit: self.unit,
        }
    }

    fn new(value: impl Into<ExactBase>, unit_components: Vec<UnitExponent<NamedUnit>>) -> Self {
        Self {
            value: value.into(),
            unit: Unit {
                components: unit_components,
            },
        }
    }

    pub fn add(self, rhs: Self) -> Result<Self, String> {
        let scale_factor = Unit::try_convert(&rhs.unit, &self.unit)?;
        Ok(Self {
            value: self.value + rhs.value * scale_factor,
            unit: self.unit,
        })
    }

    pub fn convert_to(self, rhs: Self) -> Result<Self, String> {
        if rhs.value != 1.into() {
            return Err("Right-hand side of unit conversion has a numerical value".to_string());
        }
        let scale_factor = Unit::try_convert(&self.unit, &rhs.unit)?;
        let new_value = self.value * scale_factor;
        Ok(Self {
            value: new_value,
            unit: rhs.unit,
        })
    }

    pub fn sub(self, rhs: Self) -> Result<Self, String> {
        let scale_factor = Unit::try_convert(&rhs.unit, &self.unit)?;
        Ok(Self {
            value: self.value - rhs.value * scale_factor,
            unit: self.unit,
        })
    }

    pub fn div(self, rhs: Self) -> Result<Self, String> {
        let mut components = self.unit.components.clone();
        for rhs_component in rhs.unit.components {
            components.push(UnitExponent::<NamedUnit>::new(
                rhs_component.unit,
                -rhs_component.exponent,
            ));
        }
        Ok(Self {
            value: self.value.div(rhs.value)?,
            unit: Unit { components },
        })
    }

    fn is_unitless(&self) -> bool {
        // todo this is broken for unitless components
        self.unit.components.is_empty()
    }

    pub fn pow(self, rhs: Self) -> Result<Self, String> {
        if !rhs.is_unitless() {
            return Err("Only unitless exponents are currently supported".to_string());
        }
        let new_unit = Unit {
            components: self
                .unit
                .components
                .into_iter()
                .map(|unit_exp| UnitExponent {
                    unit: unit_exp.unit,
                    exponent: unit_exp.exponent * rhs.value.clone(),
                })
                .collect(),
        };
        Ok(Self {
            value: self.value.pow(rhs.value)?,
            unit: new_unit,
        })
    }

    pub fn root_n(self, rhs: &Self) -> Result<Self, String> {
        if !self.is_unitless() || !rhs.is_unitless() {
            return Err("Roots are currently only supported for unitless numbers.".to_string());
        }
        Ok(Self {
            value: self.value.root_n(&rhs.value)?,
            unit: self.unit,
        })
    }

    pub fn approx_pi() -> Self {
        Self {
            value: ExactBase::approx_pi(),
            unit: Unit { components: vec![] },
        }
    }

    pub fn approx_e() -> Self {
        Self {
            value: ExactBase::approx_e(),
            unit: Unit { components: vec![] },
        }
    }

    pub fn i() -> Self {
        Self {
            value: ExactBase::i(),
            unit: Unit { components: vec![] },
        }
    }

    pub fn abs(self) -> Result<Self, String> {
        Ok(Self {
            value: self.value.abs()?,
            unit: self.unit,
        })
    }

    pub fn make_approximate(self) -> Self {
        Self {
            value: self.value.make_approximate(),
            unit: self.unit,
        }
    }

    pub fn zero_with_base(base: Base) -> Self {
        Self {
            value: ExactBase::zero_with_base(base),
            unit: Unit::unitless(),
        }
    }

    pub fn add_digit_in_base(&mut self, digit: u64, base: Base) -> Result<(), String> {
        self.value.add_digit_in_base(digit, base)
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0.into()
    }

    fn apply_fn(
        self,
        f: impl FnOnce(ExactBase) -> Result<ExactBase, String>,
        require_unitless: bool,
    ) -> Result<Self, String> {
        if require_unitless && !self.is_unitless() {
            return Err("Expected a unitless number".to_string());
        }
        Ok(Self {
            value: f(self.value)?,
            unit: self.unit,
        })
    }

    pub fn sin(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::sin, false)
    }

    pub fn cos(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::cos, false)
    }

    pub fn tan(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::tan, false)
    }

    pub fn asin(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::asin, false)
    }

    pub fn acos(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::acos, false)
    }

    pub fn atan(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::atan, false)
    }

    pub fn sinh(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::sinh, false)
    }

    pub fn cosh(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::cosh, false)
    }

    pub fn tanh(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::tanh, false)
    }

    pub fn asinh(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::asinh, false)
    }

    pub fn acosh(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::acosh, false)
    }

    pub fn atanh(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::atanh, false)
    }

    pub fn ln(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::ln, true)
    }

    pub fn log2(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::log2, true)
    }

    pub fn log10(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::log10, true)
    }

    pub fn exp(self) -> Result<Self, String> {
        self.apply_fn(ExactBase::exp, true)
    }
}

impl Neg for UnitValue {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }
}

impl Mul for UnitValue {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let components = [self.unit.components, rhs.unit.components].concat();
        Self {
            value: self.value * rhs.value,
            unit: Unit { components },
        }
    }
}

impl From<u64> for UnitValue {
    fn from(i: u64) -> Self {
        Self {
            value: i.into(),
            unit: Unit::unitless(),
        }
    }
}

impl Display for UnitValue {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let use_parentheses = !self.unit.components.is_empty();
        self.value.format(f, use_parentheses)?;
        if !self.unit.components.is_empty() {
            let mut negative_components = vec![];
            let mut first = true;
            let mut positive_exponents = false;
            for unit_exponent in &self.unit.components {
                if unit_exponent.exponent < 0.into() {
                    negative_components.push(unit_exponent);
                } else {
                    if !first || unit_exponent.unit.spacing {
                        write!(f, " ")?;
                    }
                    first = false;
                    write!(f, "{}", unit_exponent.unit.singular_name)?;
                    if unit_exponent.exponent != 1.into() {
                        write!(f, "^")?;
                        unit_exponent.exponent.format(f, true)?;
                    }
                    positive_exponents = true;
                }
            }
            if !negative_components.is_empty() {
                if positive_exponents {
                    write!(f, " /")?;
                }
                for unit_exponent in negative_components {
                    write!(f, " {}", unit_exponent.unit.singular_name)?;
                    let exp = if positive_exponents {
                        -unit_exponent.exponent.clone()
                    } else {
                        unit_exponent.exponent.clone()
                    };
                    if exp != ExactBase::from(1) {
                        write!(f, "^")?;
                        exp.format(f, true)?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Unit {
    components: Vec<UnitExponent<NamedUnit>>,
}

impl Unit {
    fn to_hashmap_and_scale(&self) -> (HashMap<BaseUnit, ExactBase>, ExactBase) {
        let mut hashmap = HashMap::<BaseUnit, ExactBase>::new();
        let mut scale = ExactBase::from(1);
        for named_unit_exp in &self.components {
            let overall_exp = &named_unit_exp.exponent;
            for (base_unit, base_exp) in &named_unit_exp.unit.base_units {
                if let Some(exp) = hashmap.get_mut(base_unit) {
                    let new_exp = exp.clone() + overall_exp.clone() * base_exp.clone();
                    if new_exp == 0.into() {
                        hashmap.remove(base_unit);
                    } else {
                        *exp = new_exp;
                    }
                } else {
                    let new_exp = overall_exp.clone() * base_exp.clone();
                    if new_exp != 0.into() {
                        hashmap.insert(base_unit.clone(), overall_exp.clone() * base_exp.clone());
                    }
                }
            }
            // todo remove unwrap
            scale = scale
                * named_unit_exp
                    .unit
                    .scale
                    .clone()
                    .pow(overall_exp.clone())
                    .unwrap();
        }
        (hashmap, scale)
    }

    /// Returns the combined scale factor if successful
    fn try_convert(from: &Self, into: &Self) -> Result<ExactBase, String> {
        let (hash_a, scale_a) = from.to_hashmap_and_scale();
        let (hash_b, scale_b) = into.to_hashmap_and_scale();
        if hash_a == hash_b {
            // todo remove unwrap
            Ok(scale_a.div(scale_b).unwrap())
        } else {
            Err("Units are incompatible".to_string())
        }
    }

    const fn unitless() -> Self {
        Self { components: vec![] }
    }
}

#[derive(Clone, Debug)]
struct UnitExponent<T> {
    unit: T,
    exponent: ExactBase,
}

impl<T> UnitExponent<T> {
    fn new(unit: T, exponent: impl Into<ExactBase>) -> Self {
        Self {
            unit,
            exponent: exponent.into(),
        }
    }
}

/// A named unit, like kilogram, megabyte or percent.
#[derive(Clone, Debug)]
struct NamedUnit {
    singular_name: String,
    plural_name: String,
    spacing: bool, // true for most units, false for percentages and degrees (angles)
    base_units: HashMap<BaseUnit, ExactBase>,
    scale: ExactBase,
}

impl NamedUnit {
    fn new(
        singular_name: String,
        plural_name: String,
        spacing: bool,
        base_units: HashMap<BaseUnit, ExactBase>,
        scale: impl Into<ExactBase>,
    ) -> Self {
        Self {
            singular_name,
            plural_name,
            spacing,
            base_units,
            scale: scale.into(),
        }
    }
}

/// Represents a base unit, identified solely by its name. The name is not exposed to the user.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
struct BaseUnit {
    name: String,
}

impl BaseUnit {
    const fn new(name: String) -> Self {
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_kg() {
        let base_kg = BaseUnit::new("kilogram".to_string());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("kg".to_string(), "kg".to_string(), true, hashmap, 1);
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let two_kg = UnitValue::new(2, vec![UnitExponent::new(kg.clone(), 1)]);
        let sum = one_kg.add(two_kg).unwrap();
        assert_eq!(sum.to_string(), "3 kg");
    }

    #[test]
    fn test_basic_kg_and_g() {
        let base_kg = BaseUnit::new("kilogram".to_string());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg.clone(), 1.into());
        let kg = NamedUnit::new("kg".to_string(), "kg".to_string(), true, hashmap.clone(), 1);
        let g = NamedUnit::new(
            "g".to_string(),
            "g".to_string(),
            true,
            hashmap,
            ExactBase::from(1).div(1000.into()).unwrap(),
        );
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let twelve_g = UnitValue::new(12, vec![UnitExponent::new(g.clone(), 1)]);
        assert_eq!(
            one_kg.clone().add(twelve_g.clone()).unwrap().to_string(),
            "1.012 kg"
        );
        assert_eq!(twelve_g.add(one_kg).unwrap().to_string(), "1012 g");
    }
}
