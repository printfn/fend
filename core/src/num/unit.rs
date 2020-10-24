use crate::ast;
use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::complex::{Complex, FormattedComplex, UseParentheses};
use crate::num::{Base, DivideByZero, FormattingStyle};
use crate::scope::Scope;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::ops::Neg;

use super::Exact;

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct UnitValue {
    value: Complex,
    unit: Unit,
    exact: bool,
    base: Base,
    format: FormattingStyle,
}

// #[cfg(feature = "gpl")]
// const UNITS_DB: &str = include_str!("builtin-gnu.units");

// #[cfg(not(feature = "gpl"))]
//const UNITS_DB: &str = include_str!("builtin.units");

impl UnitValue {
    pub fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, IntErr<String, I>> {
        if !self.is_unitless() {
            return Err("Cannot convert number with unit to integer".to_string())?;
        }
        if !self.exact {
            return Err("Cannot convert inexact number to integer".to_string())?;
        }
        Ok(self.value.try_as_usize(int)?)
    }

    pub fn create_unit_value_from_value<I: Interrupt>(
        value: &Self,
        singular_name: Cow<'static, str>,
        plural_name: Cow<'static, str>,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        let (hashmap, scale, exact) = value.unit.to_hashmap_and_scale(int)?;
        let scale = Exact::new(scale, true).mul(&Exact::new(value.value.clone(), true), int)?;
        let resulting_unit = NamedUnit::new(singular_name, plural_name, hashmap, scale.value);
        let mut result = Self::new(1, vec![UnitExponent::new(resulting_unit, 1)]);
        result.exact = result.exact && value.exact && exact && scale.exact;
        Ok(result)
    }

    pub fn new_base_unit(singular_name: &'static str, plural_name: &'static str) -> Self {
        let base_unit = BaseUnit::new(singular_name);
        let mut hashmap = HashMap::new();
        hashmap.insert(base_unit, 1.into());
        let unit = NamedUnit::new(singular_name, plural_name, hashmap, 1);
        Self::new(1, vec![UnitExponent::new(unit, 1)])
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
        let scale_factor = Unit::try_convert(&rhs.unit, &self.unit, int)?;
        let scaled = Exact::new(rhs.value, rhs.exact).mul(&scale_factor, int)?;
        let value = Exact::new(self.value, self.exact).add(scaled, int)?;
        Ok(Self {
            value: value.value,
            unit: self.unit,
            exact: self.exact && rhs.exact && value.exact,
            base: self.base,
            format: self.format,
        })
    }

    pub fn convert_to<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        if rhs.value != 1.into() {
            return Err("Right-hand side of unit conversion has a numerical value".to_string())?;
        }
        let scale_factor = Unit::try_convert(&self.unit, &rhs.unit, int)?;
        let new_value = Exact::new(self.value, self.exact).mul(&scale_factor, int)?;
        Ok(Self {
            value: new_value.value,
            unit: rhs.unit,
            exact: self.exact && rhs.exact && new_value.exact,
            base: self.base,
            format: self.format,
        })
    }

    pub fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let scale_factor = Unit::try_convert(&rhs.unit, &self.unit, int)?;
        let scaled = Exact::new(rhs.value, rhs.exact).mul(&scale_factor, int)?;
        let value = Exact::new(self.value, self.exact).add(-scaled, int)?;
        Ok(Self {
            value: value.value,
            unit: self.unit,
            exact: self.exact && rhs.exact && value.exact,
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
        let value =
            Exact::new(self.value, self.exact).div(Exact::new(rhs.value, rhs.exact), int)?;
        Ok(Self {
            value: value.value,
            unit: Unit { components },
            exact: value.exact && self.exact && rhs.exact,
            base: self.base,
            format: self.format,
        })
    }

    fn is_unitless(&self) -> bool {
        // todo this is broken for unitless components
        self.unit.components.is_empty()
    }

    pub fn is_unitless_one(&self) -> bool {
        self.is_unitless() && self.exact && self.value == Complex::from(1)
    }

    pub fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        if !rhs.is_unitless() {
            return Err("Only unitless exponents are currently supported".to_string())?;
        }
        let mut new_components = vec![];
        let mut exact_res = true;
        for unit_exp in self.unit.components {
            let exponent = Exact::new(unit_exp.exponent, self.exact)
                .mul(&Exact::new(rhs.value.clone(), rhs.exact), int)?;
            exact_res = exact_res && exponent.exact;
            new_components.push(UnitExponent {
                unit: unit_exp.unit,
                exponent: exponent.value,
            });
        }
        let new_unit = Unit {
            components: new_components,
        };
        let value = self.value.pow(rhs.value, int)?;
        Ok(Self {
            value: value.value,
            unit: new_unit,
            exact: self.exact && rhs.exact && exact_res && value.exact,
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

    pub fn pi() -> Self {
        Self {
            value: Complex::pi(),
            unit: Unit { components: vec![] },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }

    pub fn abs<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        let value = self.value.abs(int)?;
        Ok(Self {
            value: value.value,
            unit: self.unit,
            exact: self.exact && value.exact,
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

    pub fn is_zero(&self) -> bool {
        self.value == 0.into()
    }

    fn apply_fn_exact<I: Interrupt>(
        self,
        f: impl FnOnce(Complex, &I) -> Result<Exact<Complex>, IntErr<String, I>>,
        require_unitless: bool,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if require_unitless && !self.is_unitless() {
            return Err("Expected a unitless number".to_string())?;
        }
        let exact = f(self.value, int)?;
        Ok(Self {
            value: exact.value,
            unit: self.unit,
            exact: self.exact && exact.exact,
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
        let radians = ast::resolve_identifier("radians", scope, int)?.expect_num()?;
        Ok(self.convert_to(radians, int)?)
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
            rad.apply_fn_exact(Complex::cos, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::cos, false, int)
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

    pub fn format<I: Interrupt>(&self, int: &I) -> Result<FormattedUnitValue, IntErr<Never, I>> {
        let use_parentheses = if self.unit.components.is_empty() {
            UseParentheses::No
        } else {
            UseParentheses::IfComplex
        };
        let formatted_value =
            self.value
                .format(self.exact, self.format, self.base, use_parentheses, int)?;
        let mut exact = formatted_value.exact;
        let mut unit_string = String::new();
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
                    unit_string.push(' ');
                }
                first = false;
                if invert {
                    unit_string.push('/');
                    unit_string.push(' ');
                }
                let plural = last_component_plural && i == pluralised_idx;
                let exp_format = if self.format == FormattingStyle::Auto {
                    FormattingStyle::Exact
                } else {
                    self.format
                };
                let formatted_exp =
                    unit_exponent.format(self.base, exp_format, plural, invert, int)?;
                unit_string.push_str(formatted_exp.value.to_string().as_str());
                exact = exact && formatted_exp.exact;
            }
        }
        Ok(FormattedUnitValue {
            number: formatted_value.value,
            exact,
            unit_str: unit_string,
        })
    }

    pub fn mul<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<Never, I>> {
        let components = [self.unit.components, rhs.unit.components].concat();
        let value =
            Exact::new(self.value, self.exact).mul(&Exact::new(rhs.value, rhs.exact), int)?;
        Ok(Self {
            value: value.value,
            unit: Unit { components },
            exact: self.exact && rhs.exact && value.exact,
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

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct FormattedUnitValue {
    exact: bool,
    number: FormattedComplex,
    unit_str: String,
}

impl fmt::Display for FormattedUnitValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        write!(f, "{}{}", self.number, self.unit_str)?;
        Ok(())
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
        let mut scale = Exact::new(Complex::from(1), true);
        let mut exact = true;
        for named_unit_exp in &self.components {
            test_int(int)?;
            let overall_exp = &Exact::new(named_unit_exp.exponent.clone(), true);
            for (base_unit, base_exp) in &named_unit_exp.unit.base_units {
                test_int(int)?;
                let base_exp = Exact::new(base_exp.clone(), true);
                if let Some(exp) = hashmap.get_mut(base_unit) {
                    let product = overall_exp.clone().mul(&base_exp, int)?;
                    let new_exp = Exact::new(exp.clone(), true).add(product, int)?;
                    exact = exact && new_exp.exact;
                    if new_exp.value == 0.into() {
                        hashmap.remove(base_unit);
                    } else {
                        *exp = new_exp.value;
                    }
                } else {
                    let new_exp = overall_exp.clone().mul(&base_exp, int)?;
                    exact = exact && new_exp.exact;
                    if new_exp.value != 0.into() {
                        let adj_exp = overall_exp.clone().mul(&base_exp, int)?;
                        hashmap.insert(base_unit.clone(), adj_exp.value);
                        exact = exact && adj_exp.exact;
                    }
                }
            }
            let pow_result = named_unit_exp
                .unit
                .scale
                .clone()
                .pow(overall_exp.value.clone(), int)?;
            let new_scale = scale.mul(&pow_result, int)?;
            scale = new_scale;
            exact = exact && pow_result.exact;
        }
        Ok((hashmap, scale.value, exact))
    }

    /// Returns the combined scale factor if successful
    fn try_convert<I: Interrupt>(
        from: &Self,
        into: &Self,
        int: &I,
    ) -> Result<Exact<Complex>, IntErr<String, I>> {
        let (hash_a, scale_a, exact_a) = from.to_hashmap_and_scale(int)?;
        let (hash_b, scale_b, exact_b) = into.to_hashmap_and_scale(int)?;
        if hash_a == hash_b {
            Ok(Exact::new(scale_a, exact_a)
                .div(Exact::new(scale_b, exact_b), int)
                .map_err(IntErr::into_string)?)
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
        base: Base,
        format: FormattingStyle,
        plural: bool,
        invert_exp: bool,
        int: &I,
    ) -> Result<Exact<FormattedExponent>, IntErr<Never, I>> {
        let name = if plural {
            self.unit.plural_name.as_ref()
        } else {
            self.unit.singular_name.as_ref()
        };
        let exp = if invert_exp {
            -self.exponent.clone()
        } else {
            self.exponent.clone()
        };
        let (exact, exponent) = if exp == 1.into() {
            (true, None)
        } else {
            let formatted =
                exp.format(true, format, base, UseParentheses::IfComplexOrFraction, int)?;
            (formatted.exact, Some(formatted.value))
        };
        Ok(Exact::new(
            FormattedExponent {
                name,
                number: exponent,
            },
            exact,
        ))
    }
}

#[derive(Debug)]
struct FormattedExponent<'a> {
    name: &'a str,
    number: Option<FormattedComplex>,
}

impl<'a> fmt::Display for FormattedExponent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(number) = &self.number {
            write!(f, "^{}", number)?;
        }
        Ok(())
    }
}

/// A named unit, like kilogram, megabyte or percent.
#[derive(Clone, Debug)]
struct NamedUnit {
    singular_name: Cow<'static, str>,
    plural_name: Cow<'static, str>,
    base_units: HashMap<BaseUnit, Complex>,
    scale: Complex,
}

impl NamedUnit {
    fn new(
        singular_name: impl Into<Cow<'static, str>>,
        plural_name: impl Into<Cow<'static, str>>,
        base_units: HashMap<BaseUnit, Complex>,
        scale: impl Into<Complex>,
    ) -> Self {
        Self {
            singular_name: singular_name.into(),
            plural_name: plural_name.into(),
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
    name: Cow<'static, str>,
}

impl BaseUnit {
    fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { name: name.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interrupt::Never;

    fn to_string(n: &UnitValue) -> String {
        let int = &crate::interrupt::Never::default();
        // TODO: this unwrap call should be unnecessary
        n.format(int).unwrap().to_string()
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
            Exact::new(Complex::from(1), true)
                .div(Exact::new(1000.into(), true), int)
                .unwrap()
                .value,
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
