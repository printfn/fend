use crate::num::exact_base::ExactBase;
use crate::num::Base;
use std::ops::{Mul, Neg};
use std::{
    collections::HashMap,
    fmt::{Display, Error, Formatter},
};

#[derive(Clone, Debug)]
pub struct UnitValue {
    value: ExactBase,
    unit: Unit,
}

impl UnitValue {
    pub fn kg() -> Self {
        Self::new_base_unit("kg", "kg", true)
    }

    pub fn g() -> Self {
        Self::new_unit("g", "g", true, "(1/1000) kg")
    }

    fn new_unit(
        singular_name: impl ToString,
        plural_name: impl ToString,
        space: bool,
        expression: impl ToString,
    ) -> Self {
        let expression_as_string = expression.to_string();
        // todo remove unwraps
        let value = crate::evaluate_to_value(expression_as_string.as_str())
            .unwrap()
            .expect_num()
            .unwrap();
        let (hashmap, scale) = value.unit.into_hashmap_and_scale();
        let scale = scale * value.value;
        let resulting_unit = NamedUnit::new(singular_name, plural_name, space, hashmap, scale);
        UnitValue::new(1, vec![UnitExponent::new(resulting_unit, 1)])
    }

    fn new_base_unit(
        singular_name: impl ToString,
        plural_name: impl ToString,
        space: bool,
    ) -> Self {
        let base_kg = BaseUnit::new(singular_name.to_string());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg.clone(), 1.into());
        let kg = NamedUnit::new(
            singular_name.to_string(),
            plural_name.to_string(),
            space,
            hashmap,
            1,
        );
        Self::new(1, vec![UnitExponent::new(kg.clone(), 1)])
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
        Ok(UnitValue {
            value: self.value + rhs.value * scale_factor,
            unit: self.unit,
        })
    }

    pub fn sub(self, rhs: Self) -> Result<Self, String> {
        let scale_factor = Unit::try_convert(&rhs.unit, &self.unit)?;
        Ok(UnitValue {
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
        if !self.is_unitless() || !rhs.is_unitless() {
            return Err("Exponents are currently only supported for unitless numbers.".to_string());
        }
        Ok(Self {
            value: self.value.pow(rhs.value)?,
            unit: self.unit,
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

    pub fn i() -> Self {
        Self {
            value: ExactBase::i(),
            unit: Unit { components: vec![] },
        }
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

    pub fn is_negative(&self) -> bool {
        self.value < 0.into()
    }
}

impl Neg for UnitValue {
    type Output = Self;
    fn neg(self) -> Self {
        UnitValue {
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

impl From<i32> for UnitValue {
    fn from(i: i32) -> Self {
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
            for unit_exponent in self.unit.components.iter() {
                if unit_exponent.exponent < 0.into() {
                    negative_components.push(unit_exponent);
                } else {
                    if !first || unit_exponent.unit.spacing == true {
                        write!(f, " ")?;
                    }
                    first = false;
                    write!(f, "{}", unit_exponent.unit.singular_name)?;
                    if unit_exponent.exponent != 1.into() {
                        write!(f, "^")?;
                        unit_exponent.exponent.format(f, true)?;
                    }
                }
            }
            if !negative_components.is_empty() {
                write!(f, " /")?;
                for unit_exponent in negative_components {
                    write!(f, " {}", unit_exponent.unit.singular_name)?;
                    if unit_exponent.exponent != (-1).into() {
                        write!(f, "^")?;
                        (-unit_exponent.exponent.clone()).format(f, true)?;
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
    // todo this breaks for exponents that are zero
    fn into_hashmap_and_scale(&self) -> (HashMap<BaseUnit, ExactBase>, ExactBase) {
        let mut hashmap = HashMap::<BaseUnit, ExactBase>::new();
        let mut scale = ExactBase::from(1);
        for named_unit_exp in self.components.iter() {
            let overall_exp = &named_unit_exp.exponent;
            for (base_unit, base_exp) in named_unit_exp.unit.base_units.iter() {
                match hashmap.get_mut(&base_unit) {
                    Some(exp) => *exp = exp.clone() + overall_exp.clone() * base_exp.clone(),
                    None => {
                        hashmap.insert(base_unit.clone(), overall_exp.clone());
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
    fn try_convert(from: &Unit, into: &Unit) -> Result<ExactBase, String> {
        let (hash_a, scale_a) = from.into_hashmap_and_scale();
        let (hash_b, scale_b) = into.into_hashmap_and_scale();
        if hash_a == hash_b {
            // todo remove unwrap
            Ok(scale_a.div(scale_b).unwrap())
        } else {
            Err(format!("Units are incompatible"))
        }
    }

    fn unitless() -> Self {
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
        singular_name: impl ToString,
        plural_name: impl ToString,
        spacing: bool,
        base_units: HashMap<BaseUnit, ExactBase>,
        scale: impl Into<ExactBase>,
    ) -> Self {
        Self {
            singular_name: singular_name.to_string(),
            plural_name: plural_name.to_string(),
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
    fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_kg() {
        let base_kg = BaseUnit::new("kilogram");
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("kg", "kg", true, hashmap, 1);
        let one_kg = UnitValue::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let two_kg = UnitValue::new(2, vec![UnitExponent::new(kg.clone(), 1)]);
        let sum = one_kg.add(two_kg).unwrap();
        assert_eq!(sum.to_string(), "3 kg");
    }

    #[test]
    fn test_basic_kg_and_g() {
        let base_kg = BaseUnit::new("kilogram");
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg.clone(), 1.into());
        let kg = NamedUnit::new("kg", "kg", true, hashmap.clone(), 1);
        let g = NamedUnit::new(
            "g",
            "g",
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
