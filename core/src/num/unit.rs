use crate::num::complex::Complex;

#[derive(Clone, PartialEq, Eq, Debug)]
struct UnitNumber {
    value: Complex,
    unit_components: Vec<UnitExponent<NamedUnit>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct UnitExponent<T> {
    unit: T,
    exponent: Complex,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct NamedUnit {
    singular_name: String,
    plural_name: String,
    base_units: Vec<UnitExponent<BaseUnit>>,
    scale: Complex,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct BaseUnit {
    name: String,
}
