use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::exact_base::ExactBase;
use crate::num::{Base, FormattingStyle};
use crate::scope::Scope;
use crate::value::Value;
use std::ops::Neg;
use std::{
    collections::HashMap,
    fmt::{Error, Formatter},
};

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct UnitValue {
    value: ExactBase,
    unit: Unit,
}

impl UnitValue {
    #[allow(clippy::too_many_lines)]
    pub fn create_initial_units<I: Interrupt>(int: &I) -> Result<Scope, IntErr<String, I>> {
        let mut scope = Scope::new_empty();
        Self::parse_units(
            include_str!("builtin.units"),
            &mut scope,
            &[("bit", "bits")],
            int,
        )?;
        Self::create_units(
            vec![
                ("percent", "percent", Some("0.01")),
                ("%", "%", Some("percent")),
                ("\u{2030}", "\u{2030}", Some("0.001")), // per mille (‰)
                ("second", "seconds", Some("s")),
                ("dm", "dm", Some("0.1m")),
                ("L", "L", Some("dm^3")),
                ("cm", "cm", Some("0.01m")),
                ("mm", "mm", Some("0.001m")),
                ("um", "um", Some("0.001mm")),
                ("\u{b5}m", "\u{b5}m", Some("0.001mm")), // micrometres (µm)
                ("nm", "nm", Some("1e-9m")),
                ("pm", "pm", Some("1e-12m")),
                ("fm", "fm", Some("1e-15m")),
                ("am", "am", Some("1e-18m")),
                ("angstrom", "angstrom", Some("0.1nm")),
                ("barn", "barn", Some("100fm^2")),
                ("inch", "inches", Some("2.54cm")),
                ("in", "in", Some("inch")),
                ("ft", "ft", Some("12 inches")),
                ("foot", "feet", Some("1ft")),
                ("\"", "\"", Some("inch")),
                ("\u{201d}", "\u{201d}", Some("inch")), // Unicode double quote (”)
                ("'", "'", Some("foot")),
                ("\u{2019}", "\u{2019}", Some("foot")), // Unicode single quote (’)
                ("yard", "yards", Some("3 feet")),
                ("mile", "miles", Some("1760 yards")),
                ("mi", "mi", Some("mile")),
                ("NM", "NM", Some("1852m")),
                ("km", "km", Some("1000m")),
                ("AU", "AU", Some("149597870700m")),
                ("lb", "lbs", Some("0.45359237kg")),
                ("pound", "pounds", Some("1lb")),
                ("ounce", "ounces", Some("1/16 lb")),
                ("oz", "oz", Some("1 ounce")),
                ("dram", "drams", Some("1/16 oz")),
                ("dr", "dr", Some("1 dram")),
                ("grain", "grains", Some("1/7000 lb")),
                ("gr", "gr", Some("1 grain")),
                ("quarter", "quarters", Some("25lb")),
                ("qr", "qr", Some("1 quarter")),
                ("hundredweight", "hundredweights", Some("100lb")),
                ("cwt", "cwt", Some("1 hundredweight")),
                ("short_ton", "short_tons", Some("2000lb")),
                ("kelvin", "kelvin", Some("K")),
                ("g", "g", Some("(1/1000)kg")),
                ("mg", "mg", Some("(1/1000)g")),
                ("N", "N", Some("1 kg m / s^2")),
                ("newton", "newtons", Some("1 N")),
                ("joule", "joules", Some("1 N m")),
                ("J", "J", Some("1 joule")),
                ("pascal", "pascals", Some("1 kg m^-1 s^-2")),
                ("Pa", "Pa", Some("1 pascal")),
                ("kPa", "kPa", Some("1000 Pa")),
                ("watt", "watts", Some("1 J/s")),
                ("W", "W", Some("1 watt")),
                ("coulomb", "coulombs", Some("1 A * 1 s")),
                ("C", "C", Some("1 coulomb")),
                ("volt", "volts", Some("1 J / C")),
                ("V", "V", Some("1 volt")),
                ("ohm", "ohms", Some("1 V / A")),
                ("\u{3a9}", "\u{3a9}", Some("1 ohm")), // Omega symbol (Ω)
                ("siemens", "siemens", Some("1 / ohm")),
                ("S", "S", Some("1 siemens")),
                ("farad", "farad", Some("1 s / ohm")),
                ("F", "F", Some("1 farad")),
                ("hertz", "hertz", Some("1/s")),
                ("Hz", "Hz", Some("1 hertz")),
                ("henry", "henry", Some("J / A^2")),
                ("H", "H", Some("1 henry")),
                ("weber", "weber", Some("V s")),
                ("Wb", "Wb", Some("1 weber")),
                ("tesla", "tesla", Some("weber / m^2")),
                ("T", "T", Some("1 tesla")),
                ("kgf", "kgf", Some("9.806650 N")),
                ("lbf", "lbf", Some("kgf / kg * lb")),
                ("psi", "psi", Some("lbf / inch^2")),
                ("min", "min", Some("60s")),
                ("hr", "hr", Some("60min")),
                ("hour", "hours", Some("hr")),
                ("minute", "minutes", Some("min")),
                ("day", "days", Some("24 hours")),
                ("year", "years", Some("365.25 days")),
                ("light", "light", Some("299_792_458m/s")),
                ("ly", "ly", Some("365.25 light days")),
                ("parsec", "parsecs", Some("648000AU/pi")),
                ("kph", "kph", Some("1 km / hr")),
                ("mph", "mph", Some("1 mile / hr")),
                ("b", "b", Some("bit")),
                ("byte", "bytes", Some("8 bits")),
                ("B", "B", Some("byte")),
                ("KB", "KB", Some("1000 bytes")),
                ("MB", "MB", Some("1000 KB")),
                ("GB", "GB", Some("1000 MB")),
                ("TB", "TB", Some("1000 GB")),
                ("KiB", "KiB", Some("1024 bytes")),
                ("MiB", "MiB", Some("1024 KiB")),
                ("GiB", "GiB", Some("1024 MiB")),
                ("TiB", "TiB", Some("1024 GiB")),
                ("Kb", "Kb", Some("1000 bits")),
                ("Mb", "Mb", Some("1000 Kb")),
                ("Gb", "Gb", Some("1000 Mb")),
                ("Tb", "Tb", Some("1000 Gb")),
                ("Kib", "Kib", Some("1024 bits")),
                ("Mib", "Mib", Some("1024 Kib")),
                ("Gib", "Gib", Some("1024 Mib")),
                ("Tib", "Tib", Some("1024 Gib")),
            ],
            &mut scope,
            int,
        )?;
        Ok(scope)
    }

    pub fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, IntErr<String, I>> {
        if !self.is_unitless() {
            return Err("Cannot convert number with unit to integer".to_string())?;
        }
        Ok(self.value.try_as_usize(int)?)
    }

    /// Tries to read an identifier from the beginning of the string, and returns
    /// the remaining string.
    fn read_ident(input: &str) -> (&str, &str) {
        let mut count = 0;
        for ch in input.chars() {
            if ch.is_alphabetic() || "_".contains(ch) {
                count += ch.len_utf8();
            } else {
                break;
            }
        }
        let (ident, remaining) = input.split_at(count);
        (ident, remaining.trim())
    }

    fn parse_units<I: Interrupt>(
        unit_definitions: &str,
        scope: &mut Scope,
        plurals: &[(&str, &str)],
        int: &I,
    ) -> Result<(), IntErr<Never, I>> {
        let lines = unit_definitions.lines();
        let mut current_plural = 0;
        for line in lines {
            test_int(int)?;
            let line = line.split('#').next().unwrap_or(line).trim();
            if line.is_empty() {
                continue;
            }
            let (singular_name, expr) = Self::read_ident(line);
            let plural_name = if Some(singular_name) == plurals.get(current_plural).map(|t| t.0) {
                let plural_name = plurals[current_plural].1;
                current_plural += 1;
                assert_ne!(singular_name, plural_name);
                plural_name
            } else {
                singular_name
            };
            if expr == "!" {
                let unit = Self::new_base_unit(singular_name.to_string(), plural_name.to_string());
                if plural_name != singular_name {
                    scope.insert(plural_name, Value::Num(unit.clone()));
                }
                scope.insert(singular_name, Value::Num(unit));
            } else {
                unimplemented!("Derived units are not currently supported");
            }
        }
        Ok(())
    }

    fn create_units<I: Interrupt>(
        unit_descriptions: Vec<(impl ToString, impl ToString, Option<impl ToString>)>,
        scope: &mut Scope,
        int: &I,
    ) -> Result<(), IntErr<String, I>> {
        for (singular_name, plural_name, expr) in unit_descriptions {
            test_int(int)?;
            if let Some(expr) = expr {
                scope.insert_lazy_unit(
                    expr.to_string(),
                    singular_name.to_string(),
                    plural_name.to_string(),
                );
            } else {
                let unit = Self::new_base_unit(singular_name.to_string(), plural_name.to_string());
                scope.insert(singular_name.to_string().as_str(), Value::Num(unit.clone()));
                if plural_name.to_string() != singular_name.to_string() {
                    scope.insert(plural_name.to_string().as_str(), Value::Num(unit));
                }
            };
        }
        Ok(())
    }

    pub fn create_unit_value_from_value<I: Interrupt>(
        value: &Self,
        singular_name: String,
        plural_name: String,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        let (hashmap, scale) = value.unit.to_hashmap_and_scale(int)?;
        let scale = scale.mul(&value.value, int)?;
        let resulting_unit = NamedUnit::new(singular_name, plural_name, hashmap, scale);
        Ok(Self::new(1, vec![UnitExponent::new(resulting_unit, 1)]))
    }

    fn new_base_unit(singular_name: String, plural_name: String) -> Self {
        let base_kg = BaseUnit::new(singular_name.clone());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new(singular_name, plural_name, hashmap, 1);
        Self::new(1, vec![UnitExponent::new(kg, 1)])
    }

    pub fn with_format(self, format: FormattingStyle) -> Self {
        Self {
            value: self.value.with_format(format),
            unit: self.unit,
        }
    }

    pub fn with_base(self, base: Base) -> Self {
        Self {
            value: self.value.with_base(base),
            unit: self.unit,
        }
    }

    pub fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        if !self.is_unitless() {
            return Err("Factorial is only supported for unitless numbers".to_string())?;
        }
        Ok(Self {
            value: self.value.factorial(int)?,
            unit: self.unit,
        })
    }

    fn new(value: impl Into<ExactBase>, unit_components: Vec<UnitExponent>) -> Self {
        Self {
            value: value.into(),
            unit: Unit {
                components: unit_components,
            },
        }
    }

    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let scale_factor = Unit::try_convert(&rhs.unit, &self.unit, int)?;
        Ok(Self {
            value: self.value.add(rhs.value.mul(&scale_factor, int)?, int)?,
            unit: self.unit,
        })
    }

    pub fn convert_to<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        if rhs.value != 1.into() {
            return Err("Right-hand side of unit conversion has a numerical value".to_string())?;
        }
        let scale_factor = Unit::try_convert(&self.unit, &rhs.unit, int)?;
        let new_value = self.value.mul(&scale_factor, int)?;
        Ok(Self {
            value: new_value,
            unit: rhs.unit,
        })
    }

    pub fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let scale_factor = Unit::try_convert(&rhs.unit, &self.unit, int)?;
        Ok(Self {
            value: self.value.sub(rhs.value.mul(&scale_factor, int)?, int)?,
            unit: self.unit,
        })
    }

    pub fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let mut components = self.unit.components.clone();
        for rhs_component in rhs.unit.components {
            components.push(UnitExponent::new(
                rhs_component.unit,
                -rhs_component.exponent,
            ));
        }
        Ok(Self {
            value: self.value.div(rhs.value, int)?,
            unit: Unit { components },
        })
    }

    fn is_unitless(&self) -> bool {
        // todo this is broken for unitless components
        self.unit.components.is_empty()
    }

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        if !rhs.is_unitless() {
            return Err("Only unitless exponents are currently supported".to_string())?;
        }
        let mut new_components = vec![];
        for unit_exp in self.unit.components {
            new_components.push(UnitExponent {
                unit: unit_exp.unit,
                exponent: unit_exp.exponent.mul(&rhs.value, int)?,
            });
        }
        let new_unit = Unit {
            components: new_components,
        };
        Ok(Self {
            value: self.value.pow(rhs.value, int)?,
            unit: new_unit,
        })
    }

    pub fn root_n<I: Interrupt>(self, rhs: &Self, int: &I) -> Result<Self, IntErr<String, I>> {
        if !self.is_unitless() || !rhs.is_unitless() {
            return Err("Roots are currently only supported for unitless numbers.".to_string())?;
        }
        Ok(Self {
            value: self.value.root_n(&rhs.value, int)?,
            unit: self.unit,
        })
    }

    pub fn i() -> Self {
        Self {
            value: ExactBase::i(),
            unit: Unit { components: vec![] },
        }
    }

    pub fn abs<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        Ok(Self {
            value: self.value.abs(int)?,
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

    pub fn add_digit_in_base<I: Interrupt>(
        &mut self,
        digit: u64,
        base: Base,
        int: &I,
    ) -> Result<(), IntErr<String, I>> {
        self.value.add_digit_in_base(digit, base, int)
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0.into()
    }

    fn apply_fn<I: Interrupt>(
        self,
        f: impl FnOnce(ExactBase, &I) -> Result<ExactBase, IntErr<String, I>>,
        require_unitless: bool,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if require_unitless && !self.is_unitless() {
            return Err("Expected a unitless number".to_string())?;
        }
        Ok(Self {
            value: f(self.value, int)?,
            unit: self.unit,
        })
    }

    pub fn sin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::sin, false, int)
    }

    pub fn cos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::cos, false, int)
    }

    pub fn tan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::tan, false, int)
    }

    pub fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::asin, false, int)
    }

    pub fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::acos, false, int)
    }

    pub fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::atan, false, int)
    }

    pub fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::sinh, false, int)
    }

    pub fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::cosh, false, int)
    }

    pub fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::tanh, false, int)
    }

    pub fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::asinh, false, int)
    }

    pub fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::acosh, false, int)
    }

    pub fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::atanh, false, int)
    }

    pub fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::ln, true, int)
    }

    pub fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::log2, true, int)
    }

    pub fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::log10, true, int)
    }

    pub fn exp<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(ExactBase::exp, true, int)
    }

    pub fn format<I: Interrupt>(&self, f: &mut Formatter, int: &I) -> Result<(), IntErr<Error, I>> {
        let use_parentheses = !self.unit.components.is_empty();
        self.value.format(f, use_parentheses, int)?;
        if !self.unit.components.is_empty() {
            // Pluralisation:
            // All units should be singular, except for the last unit
            // that has a positive exponent, iff the number is not equal to 1
            let mut positive_components = vec![];
            let mut negative_components = vec![];
            let mut first = true;
            for unit_exponent in &self.unit.components {
                if unit_exponent.exponent < 0.into() {
                    negative_components.push(unit_exponent);
                } else {
                    positive_components.push(unit_exponent);
                }
            }
            let invert_negative_component =
                !positive_components.is_empty() && negative_components.len() == 1;
            let mut merged_components = vec![];
            let pluralised_idx = if positive_components.is_empty() {
                usize::MAX
            } else {
                positive_components.len() - 1
            };
            for pos_comp in positive_components {
                merged_components.push((pos_comp, false));
            }
            for neg_comp in negative_components {
                merged_components.push((neg_comp, invert_negative_component));
            }
            let last_component_plural = self.value != 1.into();
            for (i, (unit_exponent, invert)) in merged_components.into_iter().enumerate() {
                if !first || unit_exponent.unit.print_with_space() {
                    write!(f, " ")?;
                }
                first = false;
                if invert {
                    write!(f, "/ ")?;
                }
                let plural = last_component_plural && i == pluralised_idx;
                unit_exponent.format(f, plural, invert, int)?;
            }
        }
        Ok(())
    }

    pub fn mul<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        let components = [self.unit.components, rhs.unit.components].concat();
        Ok(Self {
            value: self.value.mul(&rhs.value, int)?,
            unit: Unit { components },
        })
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

impl From<u64> for UnitValue {
    fn from(i: u64) -> Self {
        Self {
            value: i.into(),
            unit: Unit::unitless(),
        }
    }
}

#[derive(Clone, Debug)]
struct Unit {
    components: Vec<UnitExponent>,
}

impl Unit {
    fn to_hashmap_and_scale<I: Interrupt>(
        &self,
        int: &I,
    ) -> Result<(HashMap<BaseUnit, ExactBase>, ExactBase), IntErr<String, I>> {
        let mut hashmap = HashMap::<BaseUnit, ExactBase>::new();
        let mut scale = ExactBase::from(1);
        for named_unit_exp in &self.components {
            test_int(int)?;
            let overall_exp = &named_unit_exp.exponent;
            for (base_unit, base_exp) in &named_unit_exp.unit.base_units {
                test_int(int)?;
                if let Some(exp) = hashmap.get_mut(base_unit) {
                    let new_exp = exp
                        .clone()
                        .add(overall_exp.clone().mul(&base_exp, int)?, int)?;
                    if new_exp == 0.into() {
                        hashmap.remove(base_unit);
                    } else {
                        *exp = new_exp;
                    }
                } else {
                    let new_exp = overall_exp.clone().mul(&base_exp, int)?;
                    if new_exp != 0.into() {
                        hashmap.insert(base_unit.clone(), overall_exp.clone().mul(&base_exp, int)?);
                    }
                }
            }
            scale = scale.mul(
                &named_unit_exp
                    .unit
                    .scale
                    .clone()
                    .pow(overall_exp.clone(), int)?,
                int,
            )?;
        }
        Ok((hashmap, scale))
    }

    /// Returns the combined scale factor if successful
    fn try_convert<I: Interrupt>(
        from: &Self,
        into: &Self,
        int: &I,
    ) -> Result<ExactBase, IntErr<String, I>> {
        let (hash_a, scale_a) = from.to_hashmap_and_scale(int)?;
        let (hash_b, scale_b) = into.to_hashmap_and_scale(int)?;
        if hash_a == hash_b {
            Ok(scale_a.div(scale_b, int)?)
        } else {
            Err("Units are incompatible".to_string())?
        }
    }

    const fn unitless() -> Self {
        Self { components: vec![] }
    }
}

#[derive(Clone, Debug)]
struct UnitExponent {
    unit: NamedUnit,
    exponent: ExactBase,
}

impl UnitExponent {
    fn new(unit: NamedUnit, exponent: impl Into<ExactBase>) -> Self {
        Self {
            unit,
            exponent: exponent.into(),
        }
    }

    fn format<I: Interrupt>(
        &self,
        f: &mut Formatter,
        plural: bool,
        invert_exp: bool,
        int: &I,
    ) -> Result<(), IntErr<Error, I>> {
        let name = if plural {
            self.unit.plural_name.as_str()
        } else {
            self.unit.singular_name.as_str()
        };
        write!(f, "{}", name)?;
        let exp = if invert_exp {
            -self.exponent.clone()
        } else {
            self.exponent.clone()
        };
        if exp != 1.into() {
            write!(f, "^")?;
            exp.format(f, true, int)?;
        }
        Ok(())
    }
}

/// A named unit, like kilogram, megabyte or percent.
#[derive(Clone, Debug)]
struct NamedUnit {
    singular_name: String,
    plural_name: String,
    base_units: HashMap<BaseUnit, ExactBase>,
    scale: ExactBase,
}

impl NamedUnit {
    fn new(
        singular_name: String,
        plural_name: String,
        base_units: HashMap<BaseUnit, ExactBase>,
        scale: impl Into<ExactBase>,
    ) -> Self {
        Self {
            singular_name,
            plural_name,
            base_units,
            scale: scale.into(),
        }
    }

    /// Returns whether or not this unit should be printed with a
    /// space (between the number and the unit). This should be true for most
    /// units like kg or m, but not for % or °
    fn print_with_space(&self) -> bool {
        if let Some(ch) = self.singular_name.chars().next() {
            // alphabetic names like kg or m should have a space,
            // while non-alphabetic names like %, ° or ' shouldn't
            ch.is_alphabetic()
        } else {
            // empty name?!
            true
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
    use crate::interrupt::Never;

    fn to_string(n: UnitValue) -> String {
        let int = &crate::interrupt::Never::default();
        crate::num::to_string(|f| n.format(f, int)).unwrap()
    }

    #[test]
    fn test_basic_kg() {
        let base_kg = BaseUnit::new("kilogram".to_string());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("kg".to_string(), "kg".to_string(), hashmap, 1);
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let two_kg = UnitValue::new(2, vec![UnitExponent::new(kg.clone(), 1)]);
        let sum = one_kg.add(two_kg, &Never::default()).unwrap();
        assert_eq!(to_string(sum), "3 kg");
    }

    #[test]
    fn test_basic_kg_and_g() {
        let int = &Never::default();
        let base_kg = BaseUnit::new("kilogram".to_string());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg.clone(), 1.into());
        let kg = NamedUnit::new("kg".to_string(), "kg".to_string(), hashmap.clone(), 1);
        let g = NamedUnit::new(
            "g".to_string(),
            "g".to_string(),
            hashmap,
            ExactBase::from(1).div(1000.into(), int).unwrap(),
        );
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let twelve_g = UnitValue::new(12, vec![UnitExponent::new(g.clone(), 1)]);
        assert_eq!(
            to_string(one_kg.clone().add(twelve_g.clone(), int).unwrap()),
            "1.012 kg"
        );
        assert_eq!(to_string(twelve_g.add(one_kg, int).unwrap()), "1012 g");
    }
}
