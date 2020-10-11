use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::complex::Complex;
use crate::num::{Base, DivideByZero, FormattingStyle};
use crate::scope::Scope;
use crate::value::Value;
use std::collections::HashMap;
use std::fmt;
use std::ops::Neg;

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct UnitValue {
    value: Complex,
    unit: Unit,
    exact: bool,
    base: Base,
    format: FormattingStyle,
}

#[cfg(feature = "gpl")]
const UNITS_DB: &str = include_str!("builtin-gnu.units");

#[cfg(not(feature = "gpl"))]
const UNITS_DB: &str = include_str!("builtin.units");

impl UnitValue {
    #[allow(clippy::too_many_lines)]
    pub fn create_initial_units<I: Interrupt>(int: &I) -> Result<Scope, IntErr<String, I>> {
        let mut scope = Scope::new_empty();
        Self::parse_units(UNITS_DB, &mut scope, int)?;
        Ok(scope)
    }

    pub fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, IntErr<String, I>> {
        if !self.is_unitless() {
            return Err("Cannot convert number with unit to integer".to_string())?;
        }
        if !self.exact {
            return Err("Cannot convert inexact number to integer".to_string())?;
        }
        Ok(self.value.try_as_usize(int)?)
    }

    /// Tries to read an identifier from the beginning of the string, and returns
    /// the remaining string.
    fn read_ident(input: &str) -> (&str, &str) {
        let input = input.trim_start();
        let mut count = 0;
        let mut prev_ch = None;
        for ch in input.chars() {
            if crate::lexer::is_valid_in_ident(ch, prev_ch) {
                count += ch.len_utf8();
            } else {
                break;
            }
            prev_ch = Some(ch);
        }
        let (ident, remaining) = input.split_at(count);
        assert!(!ident.is_empty());
        (ident, remaining)
    }

    fn parse_units<I: Interrupt>(
        unit_definitions: &str,
        scope: &mut Scope,
        int: &I,
    ) -> Result<(), IntErr<Never, I>> {
        let lines = unit_definitions.lines();
        let mut plurals = vec![];
        let mut current_plural = 0;
        let mut ignore_rules = vec![];
        let mut skip_next = false;
        'process_line: for line in lines {
            if skip_next {
                if !line.ends_with('\\') {
                    skip_next = false;
                }
                continue;
            }
            test_int(int)?;
            let line = line.split('#').next().unwrap_or(line).trim();
            if line.is_empty() {
                continue;
            }
            let plural_prefix = "!plural";
            if line.starts_with(plural_prefix) {
                let line = line.split_at(plural_prefix.len()).1.trim();
                let (singular, line) = Self::read_ident(line);
                let (plural, line) = Self::read_ident(line);
                assert!(line.is_empty());
                plurals.push((singular, plural));
                continue;
            }
            let ignore_prefix = "!ignore";
            if line.starts_with(ignore_prefix) {
                let ignore = line.split_at(ignore_prefix.len()).1.trim();
                ignore_rules.push(ignore);
                continue;
            }
            if line.ends_with('\\') {
                skip_next = true;
                continue;
            }
            for ignore in &ignore_rules {
                if line.contains(ignore) {
                    continue 'process_line;
                }
            }
            if line.starts_with('+') {
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
            if expr.starts_with('(') {
                // function definitions are not supported
                continue;
            }
            if let Some(expr) = expr.strip_prefix('-') {
                // unit prefixes like `kilo-`
                scope.insert_prefix(singular_name, expr.trim());
                continue;
            }
            let expr = expr.trim();
            //eprintln!("Adding unit '{}' '{}' '{}'", singular_name, plural_name, expr);
            if expr == "!" {
                let unit = Self::new_base_unit(singular_name.to_string(), plural_name.to_string());
                scope.insert(
                    singular_name.to_string(),
                    plural_name.to_string(),
                    Value::Num(unit),
                );
            } else {
                let expr = if expr == "!dimensionless" { "1" } else { expr };
                scope.insert_lazy_unit(
                    expr.to_string(),
                    singular_name.to_string(),
                    plural_name.to_string(),
                );
            }
            //crate::eval::evaluate_to_string(plural_name, scope, int).unwrap();
        }
        assert_eq!(current_plural, plurals.len());
        Ok(())
    }

    pub fn create_unit_value_from_value<I: Interrupt>(
        value: &Self,
        singular_name: String,
        plural_name: String,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        let (hashmap, scale, exact) = value.unit.to_hashmap_and_scale(int)?;
        let scale = scale.mul(&value.value, int)?;
        let resulting_unit = NamedUnit::new(singular_name, plural_name, hashmap, scale);
        let mut result = Self::new(1, vec![UnitExponent::new(resulting_unit, 1)]);
        result.exact = result.exact && value.exact && exact;
        Ok(result)
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
            value: self.value,
            unit: self.unit,
            exact: self.exact,
            base: self.base,
            format,
        }
    }

    pub fn with_base(self, base: Base) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: self.exact,
            format: self.format,
            base,
        }
    }

    pub fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        if !self.is_unitless() {
            return Err("Factorial is only supported for unitless numbers".to_string())?;
        }
        Ok(Self {
            value: self.value.factorial(int)?,
            unit: self.unit,
            exact: self.exact,
            base: self.base,
            format: self.format,
        })
    }

    fn new(value: impl Into<Complex>, unit_components: Vec<UnitExponent>) -> Self {
        Self {
            value: value.into(),
            unit: Unit {
                components: unit_components,
            },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }

    pub fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (scale_factor, exact_conv) = Unit::try_convert(&rhs.unit, &self.unit, int)?;
        Ok(Self {
            value: self.value.add(rhs.value.mul(&scale_factor, int)?, int)?,
            unit: self.unit,
            exact: require_both_exact(self.exact, rhs.exact) && exact_conv,
            base: self.base,
            format: self.format,
        })
    }

    pub fn convert_to<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        if rhs.value != 1.into() {
            return Err("Right-hand side of unit conversion has a numerical value".to_string())?;
        }
        let (scale_factor, exact_conv) = Unit::try_convert(&self.unit, &rhs.unit, int)?;
        let new_value = self.value.mul(&scale_factor, int)?;
        Ok(Self {
            value: new_value,
            unit: rhs.unit,
            exact: require_both_exact(self.exact, rhs.exact) && exact_conv,
            base: self.base,
            format: self.format,
        })
    }

    pub fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (scale_factor, exact_conv) = Unit::try_convert(&rhs.unit, &self.unit, int)?;
        Ok(Self {
            value: self.value.sub(rhs.value.mul(&scale_factor, int)?, int)?,
            unit: self.unit,
            exact: require_both_exact(self.exact, rhs.exact) && exact_conv,
            base: self.base,
            format: self.format,
        })
    }

    pub fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<DivideByZero, I>> {
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
            exact: require_both_exact(self.exact, rhs.exact),
            base: self.base,
            format: self.format,
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
        let (value, exact_res) = self.value.pow(rhs.value, int)?;
        Ok(Self {
            value,
            unit: new_unit,
            exact: self.exact && rhs.exact && exact_res,
            base: self.base,
            format: self.format,
        })
    }

    pub fn i() -> Self {
        Self {
            value: Complex::i(),
            unit: Unit { components: vec![] },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }

    pub fn pi<I: Interrupt>(int: &I) -> Result<Self, IntErr<Never, I>> {
        Ok(Self {
            value: Complex::pi(int)?,
            unit: Unit { components: vec![] },
            // TODO change this to true
            exact: false,
            base: Base::default(),
            format: FormattingStyle::default(),
        })
    }

    pub fn abs<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (value, res_exact) = self.value.abs(int)?;
        Ok(Self {
            value,
            unit: self.unit,
            exact: self.exact && res_exact,
            base: self.base,
            format: self.format,
        })
    }

    pub fn make_approximate(self) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: false,
            base: self.base,
            format: self.format,
        }
    }

    pub fn zero_with_base(base: Base) -> Self {
        Self {
            value: Complex::from(0),
            unit: Unit::unitless(),
            exact: true,
            base,
            format: FormattingStyle::default(),
        }
    }

    pub fn add_digit_in_base<I: Interrupt>(
        &mut self,
        digit: u64,
        base: Base,
        int: &I,
    ) -> Result<(), IntErr<String, I>> {
        if base != self.base {
            return Err(format!(
                "Base does not match: {} != {}",
                base.base_as_u8(),
                self.base.base_as_u8()
            ))?;
        }
        Ok(self
            .value
            .add_digit_in_base(digit, base.base_as_u8(), false, int)?)
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0.into()
    }

    fn apply_fn_exact<I: Interrupt>(
        self,
        f: impl FnOnce(Complex, &I) -> Result<(Complex, bool), IntErr<String, I>>,
        require_unitless: bool,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if require_unitless && !self.is_unitless() {
            return Err("Expected a unitless number".to_string())?;
        }
        let (value, exact) = f(self.value, int)?;
        Ok(Self {
            value,
            unit: self.unit,
            exact,
            base: self.base,
            format: self.format,
        })
    }

    fn apply_fn<I: Interrupt>(
        self,
        f: impl FnOnce(Complex, &I) -> Result<Complex, IntErr<String, I>>,
        require_unitless: bool,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if require_unitless && !self.is_unitless() {
            return Err("Expected a unitless number".to_string())?;
        }
        Ok(Self {
            value: f(self.value, int)?,
            unit: self.unit,
            exact: false,
            base: self.base,
            format: self.format,
        })
    }

    fn convert_angle_to_rad<I: Interrupt>(
        self,
        scope: &mut Scope,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(self.convert_to(scope.get("radians", int)?.expect_num()?, int)?)
    }

    fn unitless() -> Self {
        Self {
            value: 1.into(),
            unit: Unit::unitless(),
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }

    pub fn sin<I: Interrupt>(self, scope: &mut Scope, int: &I) -> Result<Self, IntErr<String, I>> {
        if let Ok(rad) = self.clone().convert_angle_to_rad(scope, int) {
            rad.apply_fn_exact(Complex::sin, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::sin, false, int)
        }
    }

    pub fn cos<I: Interrupt>(self, scope: &mut Scope, int: &I) -> Result<Self, IntErr<String, I>> {
        if let Ok(rad) = self.clone().convert_angle_to_rad(scope, int) {
            rad.apply_fn(Complex::cos, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn(Complex::cos, false, int)
        }
    }

    pub fn tan<I: Interrupt>(self, scope: &mut Scope, int: &I) -> Result<Self, IntErr<String, I>> {
        if let Ok(rad) = self.clone().convert_angle_to_rad(scope, int) {
            rad.apply_fn_exact(Complex::tan, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::tan, false, int)
        }
    }

    pub fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::asin, false, int)
    }

    pub fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::acos, false, int)
    }

    pub fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::atan, false, int)
    }

    pub fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::sinh, false, int)
    }

    pub fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::cosh, false, int)
    }

    pub fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::tanh, false, int)
    }

    pub fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::asinh, false, int)
    }

    pub fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::acosh, false, int)
    }

    pub fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::atanh, false, int)
    }

    pub fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::ln, true, int)
    }

    pub fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::log2, true, int)
    }

    pub fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::log10, true, int)
    }

    pub fn format<I: Interrupt>(
        &self,
        f: &mut fmt::Formatter,
        int: &I,
    ) -> Result<(), IntErr<fmt::Error, I>> {
        let use_parentheses = !self.unit.components.is_empty();
        self.value
            .format(f, self.exact, self.format, self.base, use_parentheses, int)?;
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
                unit_exponent.format(f, self.base, self.format, plural, invert, int)?;
            }
        }
        Ok(())
    }

    pub fn mul<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        let components = [self.unit.components, rhs.unit.components].concat();
        Ok(Self {
            value: self.value.mul(&rhs.value, int)?,
            unit: Unit { components },
            exact: require_both_exact(self.exact, rhs.exact),
            base: self.base,
            format: self.format,
        })
    }
}

impl Neg for UnitValue {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
            exact: self.exact,
            base: self.base,
            format: self.format,
        }
    }
}

impl From<u64> for UnitValue {
    fn from(i: u64) -> Self {
        Self {
            value: i.into(),
            unit: Unit::unitless(),
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }
}

#[derive(Clone, Debug)]
struct Unit {
    components: Vec<UnitExponent>,
}

type HashmapScale = (HashMap<BaseUnit, Complex>, Complex, bool);

impl Unit {
    fn to_hashmap_and_scale<I: Interrupt>(
        &self,
        int: &I,
    ) -> Result<HashmapScale, IntErr<String, I>> {
        let mut hashmap = HashMap::<BaseUnit, Complex>::new();
        let mut scale = Complex::from(1);
        let mut exact = true;
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
            let (pow_result, pow_result_exact) = named_unit_exp
                .unit
                .scale
                .clone()
                .pow(overall_exp.clone(), int)?;
            scale = scale.mul(&pow_result, int)?;
            exact = exact && pow_result_exact;
        }
        Ok((hashmap, scale, exact))
    }

    /// Returns the combined scale factor if successful
    fn try_convert<I: Interrupt>(
        from: &Self,
        into: &Self,
        int: &I,
    ) -> Result<(Complex, bool), IntErr<String, I>> {
        let (hash_a, scale_a, exact_a) = from.to_hashmap_and_scale(int)?;
        let (hash_b, scale_b, exact_b) = into.to_hashmap_and_scale(int)?;
        if hash_a == hash_b {
            Ok((
                scale_a.div(scale_b, int).map_err(IntErr::into_string)?,
                exact_a && exact_b,
            ))
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
    exponent: Complex,
}

impl UnitExponent {
    fn new(unit: NamedUnit, exponent: impl Into<Complex>) -> Self {
        Self {
            unit,
            exponent: exponent.into(),
        }
    }

    fn format<I: Interrupt>(
        &self,
        f: &mut fmt::Formatter,
        base: Base,
        format: FormattingStyle,
        plural: bool,
        invert_exp: bool,
        int: &I,
    ) -> Result<(), IntErr<fmt::Error, I>> {
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
            exp.format(f, true, format, base, true, int)?;
        }
        Ok(())
    }
}

/// A named unit, like kilogram, megabyte or percent.
#[derive(Clone, Debug)]
struct NamedUnit {
    singular_name: String,
    plural_name: String,
    base_units: HashMap<BaseUnit, Complex>,
    scale: Complex,
}

impl NamedUnit {
    fn new(
        singular_name: String,
        plural_name: String,
        base_units: HashMap<BaseUnit, Complex>,
        scale: impl Into<Complex>,
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
        // Alphabetic names like kg or m should have a space,
        // while non-alphabetic names like %, ° or ' shouldn't.
        // Empty names shouldn't really exist, but they might as well have a space.
        self.singular_name
            .chars()
            .next()
            .map_or(true, char::is_alphabetic)
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

const fn require_both_exact(a_exact: bool, b_exact: bool) -> bool {
    a_exact && b_exact
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interrupt::Never;

    fn to_string(n: &UnitValue) -> String {
        let int = &crate::interrupt::Never::default();
        crate::num::to_string(|f| n.format(f, int)).unwrap().0
    }

    #[test]
    fn test_basic_kg() {
        let base_kg = BaseUnit::new("kilogram".to_string());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("kg".to_string(), "kg".to_string(), hashmap, 1);
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let two_kg = UnitValue::new(2, vec![UnitExponent::new(kg, 1)]);
        let sum = one_kg.add(two_kg, &Never::default()).unwrap();
        assert_eq!(to_string(&sum), "3 kg");
    }

    #[test]
    fn test_basic_kg_and_g() {
        let int = &Never::default();
        let base_kg = BaseUnit::new("kilogram".to_string());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("kg".to_string(), "kg".to_string(), hashmap.clone(), 1);
        let g = NamedUnit::new(
            "g".to_string(),
            "g".to_string(),
            hashmap,
            Complex::from(1).div(1000.into(), int).unwrap(),
        );
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg, 1)]);
        let twelve_g = UnitValue::new(12, vec![UnitExponent::new(g, 1)]);
        assert_eq!(
            to_string(&one_kg.clone().add(twelve_g.clone(), int).unwrap()),
            "1.012 kg"
        );
        assert_eq!(to_string(&twelve_g.add(one_kg, int).unwrap()), "1012 g");
    }
}
