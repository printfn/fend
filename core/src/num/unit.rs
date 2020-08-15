use crate::num::complex::Complex;
use crate::num::Base;
use std::fmt::{Display, Error, Formatter};
use std::ops::Neg;

#[derive(Clone)]
pub struct Unit {
    value: Complex,
    scale: Complex,
    exponents: Vec<Complex>, // each value represents an exponent of a base unit
    names: Vec<(UnitName, Complex)>,
}

impl Neg for Unit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Unit {
            value: -self.value,
            scale: self.scale,
            exponents: self.exponents,
            names: self.names,
        }
    }
}

impl Unit {
    #[allow(dead_code)]
    pub fn add(self: Unit, b: Unit) -> Result<Unit, String> {
        for (x, y) in self.exponents.iter().zip(b.exponents.iter()) {
            if x != y {
                return Err(format!("Units {} and {} are incompatible", self, b));
            }
        }
        let sum = self.value
            + if self.scale == b.scale {
                b.value
            } else {
                (b.value * b.scale.clone()).div(self.scale.clone())?
            };
        Ok(Unit {
            value: sum,
            scale: self.scale,
            exponents: self.exponents,
            names: self.names,
        })
    }

    #[allow(dead_code)]
    pub fn sub(self: Unit, b: Unit) -> Result<Unit, String> {
        self.add(-b)
    }
}

impl From<Complex> for Unit {
    fn from(value: Complex) -> Self {
        Unit {
            value,
            scale: 1.into(),
            exponents: vec![],
            names: vec![],
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.value.format(f, true, Base::Decimal)?;
        for (i, (name, exp)) in self.names.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", name.singular_name)?;
            if exp != &1.into() {
                write!(f, "^")?;
                exp.format(f, true, Base::Decimal)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct UnitName {
    singular_name: String,
    plural_name: String,
}

impl UnitName {
    #[allow(dead_code)]
    pub fn new(singular_name: impl ToString, plural_name: impl ToString) -> UnitName {
        UnitName {
            singular_name: singular_name.to_string(),
            plural_name: plural_name.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn new_single(name: impl ToString) -> UnitName {
        UnitName {
            singular_name: name.to_string(),
            plural_name: name.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct UnitSystem {
    base_units: Vec<UnitName>,
    unit_mappings: Vec<(UnitName, Unit)>,
}

impl UnitSystem {
    #[allow(dead_code)]
    pub fn si_unit_system() -> UnitSystem {
        let second = UnitName::new("second", "seconds");
        let meter = UnitName::new("meter", "meters");
        let kilogram = UnitName::new("kilogram", "kilograms");
        let ampere = UnitName::new("ampere", "amperes");
        let kelvin = UnitName::new("kelvin", "kelvins");
        let mole = UnitName::new("mole", "moles");
        let candela = UnitName::new("candela", "candelas");
        UnitSystem {
            base_units: vec![second, meter, kilogram, ampere, kelvin, mole, candela],
            unit_mappings: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Unit, UnitName};
    use crate::num::complex::Complex;

    #[test]
    fn test_percentage_addition() {
        let percent_name = UnitName::new_single("%");
        let five_percent = Unit {
            value: 5.into(),
            scale: Complex::from(1).div(Complex::from(100)).unwrap(),
            exponents: vec![],
            names: vec![(percent_name, 1.into())],
        };
        let half = Unit::from(Complex::from(1).div(Complex::from(2)).unwrap());
        assert_eq!(
            "55%",
            five_percent.clone().add(half.clone()).unwrap().to_string()
        );
        assert_eq!("0.55", half.add(five_percent).unwrap().to_string());
    }
}
