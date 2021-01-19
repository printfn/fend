use crate::ast;
use crate::err::{IntErr, Interrupt, Never};
use crate::interrupt::test_int;
use crate::num::complex::{Complex, FormattedComplex, UseParentheses};
use crate::num::{Base, ConvertToUsizeError, FormattingStyle};
use crate::scope::Scope;
use std::collections::HashMap;
use std::fmt;
use std::ops::Neg;
use std::sync::Arc;

use super::Exact;

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub(crate) struct UnitValue<'a> {
    value: Complex,
    unit: Unit<'a>,
    exact: bool,
    base: Base,
    format: FormattingStyle,
}

impl<'a> UnitValue<'a> {
    pub(crate) fn try_as_usize<I: Interrupt>(
        self,
        int: &I,
    ) -> Result<usize, IntErr<ConvertToUsizeError, I>> {
        if !self.is_unitless() {
            return Err(ConvertToUsizeError::NumberWithUnit)?;
        }
        if !self.exact {
            return Err(ConvertToUsizeError::InexactNumber)?;
        }
        Ok(self.value.try_as_usize(int)?)
    }

    pub(crate) fn create_unit_value_from_value<I: Interrupt>(
        value: &Self,
        prefix: &'a str,
        singular_name: &'a str,
        plural_name: &'a str,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        let (hashmap, scale) = value.unit.to_hashmap_and_scale(int)?;
        let scale = scale.mul(&Exact::new(value.value.clone(), true), int)?;
        let resulting_unit =
            NamedUnit::new(prefix, singular_name, plural_name, hashmap, scale.value);
        let mut result = Self::new(1, vec![UnitExponent::new(resulting_unit, 1)]);
        result.exact = result.exact && value.exact && scale.exact;
        Ok(result)
    }

    pub(crate) fn new_base_unit(singular_name: &'static str, plural_name: &'static str) -> Self {
        let base_unit = BaseUnit::new(singular_name);
        let mut hashmap = HashMap::new();
        hashmap.insert(base_unit, 1.into());
        let unit = NamedUnit::new("", singular_name, plural_name, hashmap, 1);
        Self::new(1, vec![UnitExponent::new(unit, 1)])
    }

    pub(crate) fn with_format(self, format: FormattingStyle) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: self.exact,
            base: self.base,
            format,
        }
    }

    pub(crate) fn with_base(self, base: Base) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: self.exact,
            format: self.format,
            base,
        }
    }

    pub(crate) fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
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

    fn new(value: impl Into<Complex>, unit_components: Vec<UnitExponent<'a>>) -> Self {
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

    pub(crate) fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (scale_factor, _offset) = Unit::compute_scale_factor(&rhs.unit, &self.unit, int)?;
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

    pub(crate) fn convert_to<I: Interrupt>(
        self,
        rhs: Self,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if rhs.value != 1.into() {
            return Err("Right-hand side of unit conversion has a numerical value".to_string())?;
        }
        let (scale_factor, offset) = Unit::compute_scale_factor(&self.unit, &rhs.unit, int)?;
        let new_value = Exact::new(self.value, self.exact)
            .add(offset, int)?
            .mul(&scale_factor, int)?;
        Ok(Self {
            value: new_value.value,
            unit: rhs.unit,
            exact: self.exact && rhs.exact && new_value.exact,
            base: self.base,
            format: self.format,
        })
    }

    pub(crate) fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let (scale_factor, _offset) = Unit::compute_scale_factor(&rhs.unit, &self.unit, int)?;
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

    pub(crate) fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let mut components = self.unit.components.clone();
        for rhs_component in rhs.unit.components {
            components.push(UnitExponent::new(
                rhs_component.unit,
                -rhs_component.exponent,
            ));
        }
        let value = Exact::new(self.value, self.exact)
            .div(Exact::new(rhs.value, rhs.exact), int)
            .map_err(IntErr::into_string)?;
        Ok(Self {
            value: value.value,
            unit: Unit { components },
            exact: value.exact && self.exact && rhs.exact,
            base: self.base,
            format: self.format,
        }
        .simplify(int)?)
    }

    fn is_unitless(&self) -> bool {
        // todo this is broken for unitless components
        self.unit.components.is_empty()
    }

    pub(crate) fn is_unitless_one(&self) -> bool {
        self.is_unitless() && self.exact && self.value == Complex::from(1)
    }

    pub(crate) fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
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

    pub(crate) fn i() -> Self {
        Self {
            value: Complex::i(),
            unit: Unit { components: vec![] },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }

    pub(crate) fn pi() -> Self {
        Self {
            value: Complex::pi(),
            unit: Unit { components: vec![] },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
        }
    }

    pub(crate) fn abs<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        let value = self.value.abs(int)?;
        Ok(Self {
            value: value.value,
            unit: self.unit,
            exact: self.exact && value.exact,
            base: self.base,
            format: self.format,
        })
    }

    pub(crate) fn make_approximate(self) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: false,
            base: self.base,
            format: self.format,
        }
    }

    pub(crate) fn zero_with_base(base: Base) -> Self {
        Self {
            value: Complex::from(0),
            unit: Unit::unitless(),
            exact: true,
            base,
            format: FormattingStyle::default(),
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
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
        scope: Option<Arc<Scope<'a>>>,
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

    pub(crate) fn sin<I: Interrupt>(
        self,
        scope: Option<Arc<Scope<'a>>>,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if let Ok(rad) = self.clone().convert_angle_to_rad(scope, int) {
            rad.apply_fn_exact(Complex::sin, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::sin, false, int)
        }
    }

    pub(crate) fn cos<I: Interrupt>(
        self,
        scope: Option<Arc<Scope<'a>>>,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if let Ok(rad) = self.clone().convert_angle_to_rad(scope, int) {
            rad.apply_fn_exact(Complex::cos, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::cos, false, int)
        }
    }

    pub(crate) fn tan<I: Interrupt>(
        self,
        scope: Option<Arc<Scope<'a>>>,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        if let Ok(rad) = self.clone().convert_angle_to_rad(scope, int) {
            rad.apply_fn_exact(Complex::tan, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::tan, false, int)
        }
    }

    pub(crate) fn asin<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::asin, false, int)
    }

    pub(crate) fn acos<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::acos, false, int)
    }

    pub(crate) fn atan<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::atan, false, int)
    }

    pub(crate) fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::sinh, false, int)
    }

    pub(crate) fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::cosh, false, int)
    }

    pub(crate) fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::tanh, false, int)
    }

    pub(crate) fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::asinh, false, int)
    }

    pub(crate) fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::acosh, false, int)
    }

    pub(crate) fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::atanh, false, int)
    }

    pub(crate) fn ln<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::ln, true, int)
    }

    pub(crate) fn log2<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::log2, true, int)
    }

    pub(crate) fn log10<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        self.apply_fn(Complex::log10, true, int)
    }

    pub(crate) fn format<I: Interrupt>(
        &self,
        int: &I,
    ) -> Result<FormattedUnitValue, IntErr<Never, I>> {
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

    pub(crate) fn mul<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, IntErr<String, I>> {
        let components = [self.unit.components, rhs.unit.components].concat();
        let value =
            Exact::new(self.value, self.exact).mul(&Exact::new(rhs.value, rhs.exact), int)?;
        Ok(Self {
            value: value.value,
            unit: Unit { components },
            exact: self.exact && rhs.exact && value.exact,
            base: self.base,
            format: self.format,
        }
        .simplify(int)?)
    }

    fn simplify<I: Interrupt>(self, int: &I) -> Result<Self, IntErr<String, I>> {
        let mut res_components: Vec<UnitExponent> = vec![];
        let mut res_exact = self.exact;
        let mut res_value = self.value;

        // combine identical or compatible units by summing their exponents
        // and potentially adjusting the value
        'outer: for comp in self.unit.components {
            for res_comp in &mut res_components {
                if comp.unit.base_units.is_empty() && comp.unit != res_comp.unit {
                    continue;
                }
                let conversion = Unit::compute_scale_factor(
                    &Unit {
                        components: vec![UnitExponent {
                            unit: comp.unit.clone(),
                            exponent: 1.into(),
                        }],
                    },
                    &Unit {
                        components: vec![UnitExponent {
                            unit: res_comp.unit.clone(),
                            exponent: 1.into(),
                        }],
                    },
                    int,
                );
                match conversion {
                    Ok((scale, offset)) => {
                        if offset.value != 0.into() {
                            // don't merge units that have offsets
                            break;
                        }

                        let lhs = Exact {
                            value: res_comp.exponent.clone(),
                            exact: res_exact,
                        };
                        let rhs = Exact {
                            value: comp.exponent.clone(),
                            exact: res_exact,
                        };
                        let sum = lhs.add(rhs, int)?;
                        res_comp.exponent = sum.value;
                        res_exact = res_exact && sum.exact && scale.exact;

                        let scale = scale.value.pow(comp.exponent, int)?;
                        let adjusted_value = Exact {
                            value: res_value,
                            exact: res_exact,
                        }
                        .mul(&scale, int)?;
                        res_value = adjusted_value.value;
                        res_exact = res_exact && adjusted_value.exact;

                        continue 'outer;
                    }
                    Err(IntErr::Interrupt(i)) => return Err(IntErr::Interrupt(i)),
                    Err(IntErr::Error(_)) => (),
                };
            }
            res_components.push(comp.clone())
        }

        // remove units with exponent == 0
        res_components = res_components
            .into_iter()
            .filter(|unit_exponent| unit_exponent.exponent != 0.into())
            .collect();

        Ok(Self {
            value: res_value,
            unit: Unit {
                components: res_components,
            },
            exact: res_exact,
            base: self.base,
            format: self.format,
        })
    }
}

impl Neg for UnitValue<'_> {
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

impl From<u64> for UnitValue<'_> {
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

impl<'a> fmt::Debug for UnitValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        write!(
            f,
            "{:?} {:?} ({:?}, {:?})",
            self.value, self.unit, self.base, self.format
        )?;
        Ok(())
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub(crate) struct FormattedUnitValue {
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

#[derive(Clone)]
struct Unit<'a> {
    components: Vec<UnitExponent<'a>>,
}

type HashmapScale<'a> = (HashMap<BaseUnit<'a>, Complex>, Exact<Complex>);
type HashmapScaleOffset<'a> = (
    HashMap<BaseUnit<'a>, Complex>,
    Exact<Complex>,
    Exact<Complex>,
);

impl<'a> Unit<'a> {
    fn to_hashmap_and_scale<I: Interrupt>(
        &self,
        int: &I,
    ) -> Result<HashmapScale<'a>, IntErr<String, I>> {
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
            scale = scale.mul(&pow_result, int)?;
            exact = exact && pow_result.exact;
        }
        Ok((hashmap, Exact::new(scale.value, exact)))
    }

    fn reduce_hashmap<I: Interrupt>(
        hashmap: &HashMap<BaseUnit<'a>, Complex>,
        int: &I,
    ) -> Result<HashmapScaleOffset<'a>, IntErr<String, I>> {
        if hashmap.len() == 1 && hashmap.get(&BaseUnit::new("celsius")) == Some(&1.into()) {
            let mut result_hashmap = HashMap::new();
            result_hashmap.insert(BaseUnit::new("kelvin"), 1.into());
            return Ok((
                result_hashmap,
                Exact::new(1.into(), true),
                Exact::new(Complex::from(27315), true)
                    .div(Exact::new(Complex::from(100), true), int)
                    .map_err(IntErr::into_string)?,
            ));
        }
        if hashmap.len() == 1 && hashmap.get(&BaseUnit::new("fahrenheit")) == Some(&1.into()) {
            let mut result_hashmap = HashMap::new();
            result_hashmap.insert(BaseUnit::new("kelvin"), 1.into());
            return Ok((
                result_hashmap,
                Exact::new(Complex::from(5), true)
                    .div(Exact::new(Complex::from(9), true), int)
                    .map_err(IntErr::into_string)?,
                Exact::new(Complex::from(45967), true)
                    .div(Exact::new(Complex::from(180), true), int)
                    .map_err(IntErr::into_string)?,
            ));
        }
        let mut scale_adjustment = Exact::new(Complex::from(1), true);
        let mut result_hashmap = HashMap::new();
        for (mut base_unit, exponent) in hashmap {
            if base_unit.name == "celsius" {
                base_unit = &BaseUnit { name: "kelvin" };
            } else if base_unit.name == "fahrenheit" {
                base_unit = &BaseUnit { name: "kelvin" };
                scale_adjustment = scale_adjustment.mul(
                    &Exact::new(Complex::from(5), true)
                        .div(Exact::new(Complex::from(9), true), int)
                        .map_err(IntErr::into_string)?
                        .value
                        .pow(exponent.clone(), int)?,
                    int,
                )?;
            }
            result_hashmap.insert(base_unit.clone(), exponent.clone());
        }
        Ok((result_hashmap, scale_adjustment, Exact::new(0.into(), true)))
    }

    /// Returns the combined scale factor if successful
    fn compute_scale_factor<I: Interrupt>(
        from: &Self,
        into: &Self,
        int: &I,
    ) -> Result<(Exact<Complex>, Exact<Complex>), IntErr<String, I>> {
        let (hash_a, scale_a) = from.to_hashmap_and_scale(int)?;
        let (hash_b, scale_b) = into.to_hashmap_and_scale(int)?;
        let (hash_a, adj_a, offset_a) = Self::reduce_hashmap(&hash_a, int)?;
        let (hash_b, adj_b, offset_b) = Self::reduce_hashmap(&hash_b, int)?;
        if hash_a == hash_b {
            Ok((
                scale_a
                    .mul(&adj_a, int)?
                    .div(adj_b, int)
                    .map_err(IntErr::into_string)?
                    .div(scale_b, int)
                    .map_err(IntErr::into_string)?,
                offset_a.add(-offset_b, int)?,
            ))
        } else {
            Err("Units are incompatible".to_string())?
        }
    }

    const fn unitless() -> Self {
        Self { components: vec![] }
    }
}

impl<'a> fmt::Debug for Unit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.components.is_empty() {
            write!(f, "(unitless)")?;
        }
        let mut first = true;
        for component in &self.components {
            if !first {
                write!(f, " * ")?;
            }
            write!(f, "{:?}", component)?;
            first = false;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct UnitExponent<'a> {
    unit: NamedUnit<'a>,
    exponent: Complex,
}

impl<'a> UnitExponent<'a> {
    fn new(unit: NamedUnit<'a>, exponent: impl Into<Complex>) -> Self {
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
            self.unit.plural_name
        } else {
            self.unit.singular_name
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
                prefix: self.unit.prefix,
                name,
                number: exponent,
            },
            exact,
        ))
    }
}

impl<'a> fmt::Debug for UnitExponent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.unit)?;
        if !self.exponent.is_definitely_one() {
            write!(f, "^{:?}", self.exponent)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct FormattedExponent<'a> {
    prefix: &'a str,
    name: &'a str,
    number: Option<FormattedComplex>,
}

impl<'a> fmt::Display for FormattedExponent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.name)?;
        if let Some(number) = &self.number {
            write!(f, "^{}", number)?;
        }
        Ok(())
    }
}

/// A named unit, like kilogram, megabyte or percent.
#[derive(Clone, Eq, PartialEq)]
struct NamedUnit<'a> {
    prefix: &'a str,
    singular_name: &'a str,
    plural_name: &'a str,
    base_units: HashMap<BaseUnit<'a>, Complex>,
    scale: Complex,
}

impl<'a> NamedUnit<'a> {
    fn new(
        prefix: &'a str,
        singular_name: &'a str,
        plural_name: &'a str,
        base_units: HashMap<BaseUnit<'a>, Complex>,
        scale: impl Into<Complex>,
    ) -> Self {
        Self {
            prefix,
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

impl<'a> fmt::Debug for NamedUnit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.prefix.is_empty() {
            write!(f, "{}", self.singular_name)?;
        } else {
            write!(f, "{}-{}", self.prefix, self.singular_name)?;
        }
        write!(f, " (")?;
        if self.plural_name != self.singular_name {
            if self.prefix.is_empty() {
                write!(f, "{}, ", self.plural_name)?;
            } else {
                write!(f, "{}-{}, ", self.prefix, self.plural_name)?;
            }
        }
        write!(f, "= {:?}", self.scale)?;
        let mut it = self.base_units.iter().collect::<Vec<_>>();
        it.sort_by_key(|(k, _v)| k.name);
        for (base_unit, exponent) in &it {
            write!(f, " {:?}", base_unit)?;
            if !exponent.is_definitely_one() {
                write!(f, "^{:?}", exponent)?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// Represents a base unit, identified solely by its name. The name is not exposed to the user.
#[derive(Clone, PartialEq, Eq, Hash)]
struct BaseUnit<'a> {
    name: &'a str,
}

impl<'a> fmt::Debug for BaseUnit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> BaseUnit<'a> {
    fn new(name: &'a str) -> Self {
        Self { name }
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
        let base_kg = BaseUnit::new("kilogram");
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("k", "g", "g", hashmap, 1);
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let two_kg = UnitValue::new(2, vec![UnitExponent::new(kg, 1)]);
        let sum = one_kg.add(two_kg, &Never::default()).unwrap();
        assert_eq!(to_string(&sum), "3 kg");
    }

    #[test]
    fn test_basic_kg_and_g() {
        let int = &Never::default();
        let base_kg = BaseUnit::new("kilogram");
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("k", "g", "g", hashmap.clone(), 1);
        let g = NamedUnit::new(
            "",
            "g",
            "g",
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
