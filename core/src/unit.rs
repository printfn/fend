use crate::num::bigrat::BigRat;
use std::fmt::{Display, Error, Formatter};
use std::ops::Neg;

#[derive(Clone)]
pub struct Unit {
    value: BigRat,
    scale: BigRat,
    exponents: Vec<BigRat>, // each value represents an exponent of a base unit
    name: String,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}{}", self.value, self.name)?;
        Ok(())
    }
}

impl Neg for Unit {
    type Output = Unit;

    fn neg(self) -> Self::Output {
        Unit {
            value: -self.value,
            scale: self.scale,
            exponents: self.exponents,
            name: self.name,
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
        let sum = self.value + if self.scale == b.scale {
            b.value
        } else {
            (b.value * b.scale.clone()).div(self.scale.clone())?
        };
        Ok(Unit {
            value: sum,
            scale: self.scale,
            exponents: self.exponents,
            name: self.name
        })
    }

    #[allow(dead_code)]
    pub fn sub(self: Unit, b: Unit) -> Result<Unit, String> {
        self.add(-b)
    }
}

impl From<BigRat> for Unit {
    fn from(value: BigRat) -> Self {
        Unit {
            value,
            scale: 1.into(),
            exponents: vec![],
            name: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Unit;
    use crate::num::bigrat::BigRat;

    #[test]
    fn test_percentage_addition() {
        let five_percent = Unit {
            value: 5.into(),
            scale: BigRat::from(1).div(BigRat::from(100)).unwrap(),
            exponents: vec![],
            name: "%".to_string()
        };
        let half = Unit::from(BigRat::from(1).div(BigRat::from(2)).unwrap());
        assert_eq!("55%", five_percent.clone().add(half.clone()).unwrap().to_string());
        assert_eq!("0.55", half.add(five_percent).unwrap().to_string());
    }
}
