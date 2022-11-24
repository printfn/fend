use crate::ast::{BitwiseBop, Bop};
use crate::error::{FendError, Interrupt};
use crate::num::complex::{Complex, UseParentheses};
use crate::num::dist::Dist;
use crate::num::{Base, FormattingStyle};
use crate::scope::Scope;
use crate::serialize::{deserialize_bool, deserialize_usize, serialize_bool, serialize_usize};
use crate::{ast, ident::Ident};
use crate::{Attrs, Span, SpanKind};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Neg;
use std::sync::Arc;
use std::{fmt, io};

pub(crate) mod base_unit;
pub(crate) mod named_unit;
pub(crate) mod unit_exponent;

use base_unit::BaseUnit;
use named_unit::NamedUnit;
use unit_exponent::UnitExponent;

use super::Exact;

#[derive(Clone)]
pub(crate) struct Value {
    value: Dist,
    unit: Unit,
    exact: bool,
    base: Base,
    format: FormattingStyle,
    simplifiable: bool,
}

impl Value {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        self.value.serialize(write)?;
        self.unit.serialize(write)?;
        serialize_bool(self.exact, write)?;
        self.base.serialize(write)?;
        self.format.serialize(write)?;
        serialize_bool(self.simplifiable, write)?;
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(Self {
            value: Dist::deserialize(read)?,
            unit: Unit::deserialize(read)?,
            exact: deserialize_bool(read)?,
            base: Base::deserialize(read)?,
            format: FormattingStyle::deserialize(read)?,
            simplifiable: deserialize_bool(read)?,
        })
    }

    pub(crate) fn try_as_usize<I: Interrupt>(self, int: &I) -> Result<usize, FendError> {
        if !self.is_unitless(int)? {
            return Err(FendError::NumberWithUnitToInt);
        }
        self.try_as_usize_unit(int)
    }

    pub(crate) fn try_as_usize_unit<I: Interrupt>(self, int: &I) -> Result<usize, FendError> {
        if !self.exact {
            return Err(FendError::InexactNumberToInt);
        }
        self.value.one_point()?.try_as_usize(int)
    }

    pub(crate) fn create_unit_value_from_value<I: Interrupt>(
        value: &Self,
        prefix: Cow<'static, str>,
        singular_name: Cow<'static, str>,
        plural_name: Cow<'static, str>,
        int: &I,
    ) -> Result<Self, FendError> {
        let (hashmap, scale) = value.unit.to_hashmap_and_scale(int)?;
        let scale = scale.mul(&Exact::new(value.value.one_point_ref()?.clone(), true), int)?;
        let resulting_unit =
            NamedUnit::new(prefix, singular_name, plural_name, hashmap, scale.value);
        let mut result = Self::new(1, vec![UnitExponent::new(resulting_unit, 1)]);
        result.exact = result.exact && value.exact && scale.exact;
        Ok(result)
    }

    pub(crate) fn new_base_unit(
        singular_name: Cow<'static, str>,
        plural_name: Cow<'static, str>,
    ) -> Self {
        let base_unit = BaseUnit::new(singular_name.clone());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_unit, 1.into());
        let unit = NamedUnit::new(Cow::Borrowed(""), singular_name, plural_name, hashmap, 1);
        Self::new(1, vec![UnitExponent::new(unit, 1)])
    }

    pub(crate) fn with_format(self, format: FormattingStyle) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: self.exact,
            base: self.base,
            simplifiable: self.simplifiable,
            format,
        }
    }

    pub(crate) fn with_base(self, base: Base) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: self.exact,
            format: self.format,
            simplifiable: self.simplifiable,
            base,
        }
    }

    pub(crate) fn factorial<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        if !self.is_unitless(int)? {
            return Err(FendError::FactorialUnitless);
        }
        Ok(Self {
            value: Dist::from(self.value.one_point()?.factorial(int)?),
            unit: self.unit,
            exact: self.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    fn new(value: impl Into<Dist>, unit_components: Vec<UnitExponent>) -> Self {
        Self {
            value: value.into(),
            unit: Unit {
                components: unit_components,
            },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
            simplifiable: true,
        }
    }

    pub(crate) fn add<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        let scale_factor = Unit::compute_scale_factor(&rhs.unit, &self.unit, int)?;
        let scaled = Exact::new(rhs.value, rhs.exact)
            .mul(&scale_factor.scale_1.apply(Dist::from), int)?
            .div(&scale_factor.scale_2.apply(Dist::from), int)?;
        let value =
            Exact::new(self.value, self.exact).add(&Exact::new(scaled.value, scaled.exact), int)?;
        Ok(Self {
            value: value.value,
            unit: self.unit,
            exact: self.exact && rhs.exact && value.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    /// Called for implicit addition to modify the second operand.
    /// For example, when evaluating `5'0`, this function can change the second
    /// operand's unit from `unitless` to `"`.
    fn fudge_implicit_rhs_unit<I: Interrupt>(
        &self,
        rhs: Self,
        attrs: Attrs,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        for (lhs_unit, rhs_unit) in crate::units::IMPLICIT_UNIT_MAP {
            if self.unit.equal_to(lhs_unit) && rhs.is_unitless(int)? {
                let inches =
                    ast::resolve_identifier(&Ident::new_str(rhs_unit), None, attrs, context, int)?
                        .expect_num()?;
                return rhs.mul(inches, int);
            }
        }
        Ok(rhs)
    }

    pub(crate) fn convert_to<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        if rhs.value.one_point()? != 1.into() {
            return Err(FendError::ConversionRhsNumerical);
        }
        let scale_factor = Unit::compute_scale_factor(&self.unit, &rhs.unit, int)?;
        let new_value = Exact::new(self.value, self.exact)
            .mul(&scale_factor.scale_1.apply(Dist::from), int)?
            .add(&scale_factor.offset.apply(Dist::from), int)?
            .div(&scale_factor.scale_2.apply(Dist::from), int)?;
        Ok(Self {
            value: new_value.value,
            unit: rhs.unit,
            exact: self.exact && rhs.exact && new_value.exact,
            base: self.base,
            format: self.format,
            simplifiable: false,
        })
    }

    pub(crate) fn sub<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        let scale_factor = Unit::compute_scale_factor(&rhs.unit, &self.unit, int)?;
        let scaled = Exact::new(rhs.value, rhs.exact)
            .mul(&scale_factor.scale_1.apply(Dist::from), int)?
            .div(&scale_factor.scale_2.apply(Dist::from), int)?;
        let value = Exact::new(self.value, self.exact).add(&-scaled, int)?;
        Ok(Self {
            value: value.value,
            unit: self.unit,
            exact: self.exact && rhs.exact && value.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn div<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        let mut components = self.unit.components.clone();
        for rhs_component in rhs.unit.components {
            components.push(UnitExponent::new(
                rhs_component.unit,
                -rhs_component.exponent,
            ));
        }
        let value =
            Exact::new(self.value, self.exact).div(&Exact::new(rhs.value, rhs.exact), int)?;
        Ok(Self {
            value: value.value,
            unit: Unit { components },
            exact: value.exact && self.exact && rhs.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    fn modulo<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        if !self.is_unitless(int)? || !rhs.is_unitless(int)? {
            return Err(FendError::ModuloUnitless);
        }
        Ok(Self {
            value: Dist::from(
                self.value
                    .one_point()?
                    .modulo(rhs.value.one_point()?, int)?,
            ),
            unit: self.unit,
            exact: self.exact && rhs.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    fn bitwise<I: Interrupt>(self, rhs: Self, op: BitwiseBop, int: &I) -> Result<Self, FendError> {
        if !self.is_unitless(int)? || !rhs.is_unitless(int)? {
            return Err(FendError::ExpectedAUnitlessNumber);
        }
        Ok(Self {
            value: Dist::from(
                self.value
                    .one_point()?
                    .bitwise(rhs.value.one_point()?, op, int)?,
            ),
            unit: self.unit,
            exact: self.exact && rhs.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn combination<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        if !self.is_unitless(int)? || !rhs.is_unitless(int)? {
            return Err(FendError::ExpectedAUnitlessNumber);
        }
        Ok(Self {
            value: Dist::from(
                self.value
                    .one_point()?
                    .combination(rhs.value.one_point()?, int)?,
            ),
            unit: self.unit,
            exact: self.exact && rhs.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn permutation<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        if !self.is_unitless(int)? || !rhs.is_unitless(int)? {
            return Err(FendError::ExpectedAUnitlessNumber);
        }
        Ok(Self {
            value: Dist::from(
                self.value
                    .one_point()?
                    .permutation(rhs.value.one_point()?, int)?,
            ),
            unit: self.unit,
            exact: self.exact && rhs.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn bop<I: Interrupt>(
        self,
        op: Bop,
        rhs: Self,
        attrs: Attrs,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        match op {
            Bop::Plus => self.add(rhs, int),
            Bop::ImplicitPlus => {
                let rhs = self.fudge_implicit_rhs_unit(rhs, attrs, context, int)?;
                self.add(rhs, int)
            }
            Bop::Minus => self.sub(rhs, int),
            Bop::Mul => self.mul(rhs, int),
            Bop::Div => self.div(rhs, int),
            Bop::Mod => self.modulo(rhs, int),
            Bop::Pow => self.pow(rhs, int),
            Bop::Bitwise(bitwise_bop) => self.bitwise(rhs, bitwise_bop, int),
            Bop::Combination => self.combination(rhs, int),
            Bop::Permutation => self.permutation(rhs, int),
        }
    }

    fn is_unitless<I: Interrupt>(&self, int: &I) -> Result<bool, FendError> {
        // todo this is broken for unitless components
        if self.unit.components.is_empty() {
            return Ok(true);
        }
        let (hashmap, _scale) = self.unit.to_hashmap_and_scale(int)?;
        if hashmap.is_empty() {
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) fn is_unitless_one<I: Interrupt>(&self, int: &I) -> Result<bool, FendError> {
        Ok(self.exact && self.value.equals_int(1) && self.is_unitless(int)?)
    }

    pub(crate) fn pow<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        if !rhs.is_unitless(int)? {
            return Err(FendError::ExpUnitless);
        }
        let mut new_components = vec![];
        let mut exact_res = true;
        for unit_exp in self.unit.components {
            let exponent = Exact::new(unit_exp.exponent, self.exact)
                .mul(&Exact::new(rhs.value.clone().one_point()?, rhs.exact), int)?;
            exact_res = exact_res && exponent.exact;
            new_components.push(UnitExponent {
                unit: unit_exp.unit,
                exponent: exponent.value,
            });
        }
        let new_unit = Unit {
            components: new_components,
        };
        let value = self.value.one_point()?.pow(rhs.value.one_point()?, int)?;
        Ok(Self {
            value: value.value.into(),
            unit: new_unit,
            exact: self.exact && rhs.exact && exact_res && value.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn i() -> Self {
        Self {
            value: Complex::i().into(),
            unit: Unit { components: vec![] },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
            simplifiable: true,
        }
    }

    pub(crate) fn pi() -> Self {
        Self {
            value: Complex::pi().into(),
            unit: Unit { components: vec![] },
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
            simplifiable: true,
        }
    }

    pub(crate) fn abs<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        let value = self.value.one_point()?.abs(int)?;
        Ok(Self {
            value: value.value.into(),
            unit: self.unit,
            exact: self.exact && value.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn make_approximate(self) -> Self {
        Self {
            value: self.value,
            unit: self.unit,
            exact: false,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        }
    }

    pub(crate) fn zero_with_base(base: Base) -> Self {
        Self {
            value: Dist::from(0),
            unit: Unit::unitless(),
            exact: true,
            base,
            format: FormattingStyle::default(),
            simplifiable: true,
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.value.equals_int(0)
    }

    pub(crate) fn new_die<I: Interrupt>(
        count: u32,
        faces: u32,
        int: &I,
    ) -> Result<Self, FendError> {
        Ok(Self::new(Dist::new_die(count, faces, int)?, vec![]))
    }

    fn apply_fn_exact<I: Interrupt>(
        self,
        f: impl FnOnce(Complex, &I) -> Result<Exact<Complex>, FendError>,
        require_unitless: bool,
        int: &I,
    ) -> Result<Self, FendError> {
        if require_unitless && !self.is_unitless(int)? {
            return Err(FendError::ExpectedAUnitlessNumber);
        }
        let exact = f(self.value.one_point()?, int)?;
        Ok(Self {
            value: exact.value.into(),
            unit: self.unit,
            exact: self.exact && exact.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    fn apply_fn<I: Interrupt>(
        self,
        f: impl FnOnce(Complex, &I) -> Result<Complex, FendError>,
        require_unitless: bool,
        int: &I,
    ) -> Result<Self, FendError> {
        if require_unitless && !self.is_unitless(int)? {
            return Err(FendError::ExpectedAUnitlessNumber);
        }
        Ok(Self {
            value: f(self.value.one_point()?, int)?.into(),
            unit: self.unit,
            exact: false,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn sample<I: Interrupt>(
        self,
        ctx: &crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        Ok(Self {
            value: self.value.sample(ctx, int)?,
            ..self
        })
    }

    fn convert_angle_to_rad<I: Interrupt>(
        self,
        scope: Option<Arc<Scope>>,
        attrs: Attrs,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        let radians =
            ast::resolve_identifier(&Ident::new_str("radians"), scope, attrs, context, int)?
                .expect_num()?;
        self.convert_to(radians, int)
    }

    fn unitless() -> Self {
        Self {
            value: 1.into(),
            unit: Unit::unitless(),
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
            simplifiable: true,
        }
    }

    pub(crate) fn conjugate(self) -> Result<Self, FendError> {
        Ok(Self {
            value: self.value.one_point()?.conjugate().into(),
            ..self
        })
    }

    pub(crate) fn sin<I: Interrupt>(
        self,
        scope: Option<Arc<Scope>>,
        attrs: Attrs,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        if let Ok(rad) = self
            .clone()
            .convert_angle_to_rad(scope, attrs, context, int)
        {
            Ok(rad
                .apply_fn_exact(Complex::sin, false, int)?
                .convert_to(Self::unitless(), int)?)
        } else {
            self.apply_fn_exact(Complex::sin, false, int)
        }
    }

    pub(crate) fn cos<I: Interrupt>(
        self,
        scope: Option<Arc<Scope>>,
        attrs: Attrs,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        if let Ok(rad) = self
            .clone()
            .convert_angle_to_rad(scope, attrs, context, int)
        {
            rad.apply_fn_exact(Complex::cos, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::cos, false, int)
        }
    }

    pub(crate) fn tan<I: Interrupt>(
        self,
        scope: Option<Arc<Scope>>,
        attrs: Attrs,
        context: &mut crate::Context,
        int: &I,
    ) -> Result<Self, FendError> {
        if let Ok(rad) = self
            .clone()
            .convert_angle_to_rad(scope, attrs, context, int)
        {
            rad.apply_fn_exact(Complex::tan, false, int)?
                .convert_to(Self::unitless(), int)
        } else {
            self.apply_fn_exact(Complex::tan, false, int)
        }
    }

    pub(crate) fn asin<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::asin, false, int)
    }

    pub(crate) fn acos<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::acos, false, int)
    }

    pub(crate) fn atan<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::atan, false, int)
    }

    pub(crate) fn sinh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::sinh, false, int)
    }

    pub(crate) fn cosh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::cosh, false, int)
    }

    pub(crate) fn tanh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::tanh, false, int)
    }

    pub(crate) fn asinh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::asinh, false, int)
    }

    pub(crate) fn acosh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::acosh, false, int)
    }

    pub(crate) fn atanh<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::atanh, false, int)
    }

    pub(crate) fn ln<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::ln, true, int)
    }

    pub(crate) fn log2<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::log2, true, int)
    }

    pub(crate) fn log10<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        self.apply_fn(Complex::log10, true, int)
    }

    pub(crate) fn format<I: Interrupt>(
        &self,
        ctx: &crate::Context,
        int: &I,
    ) -> Result<FormattedValue, FendError> {
        let use_parentheses = if self.unit.components.is_empty() {
            UseParentheses::No
        } else {
            UseParentheses::IfComplex
        };
        let mut formatted_value = String::new();
        let mut exact = self
            .value
            .format(
                self.exact,
                self.format,
                self.base,
                use_parentheses,
                &mut formatted_value,
                ctx,
                int,
            )?
            .exact;
        let unit_string = self.unit.format(
            "",
            self.value.equals_int(1),
            self.base,
            self.format,
            true,
            int,
        )?;
        exact = exact && unit_string.exact;
        Ok(FormattedValue {
            number: formatted_value,
            exact,
            unit_str: unit_string.value,
        })
    }

    pub(crate) fn mul<I: Interrupt>(self, rhs: Self, int: &I) -> Result<Self, FendError> {
        let components = [self.unit.components, rhs.unit.components].concat();
        let value =
            Exact::new(self.value, self.exact).mul(&Exact::new(rhs.value, rhs.exact), int)?;
        Ok(Self {
            value: value.value,
            unit: Unit { components },
            exact: self.exact && rhs.exact && value.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn simplify<I: Interrupt>(self, int: &I) -> Result<Self, FendError> {
        if !self.simplifiable {
            return Ok(self);
        }

        let mut res_components: Vec<UnitExponent> = vec![];
        let mut res_exact = self.exact;
        let mut res_value = self.value;

        /*
         * In fend, percentages are units.
         * Without any special handling, multiplication would
         * unconditionally combine them.
         *
         * This would (and did) lead to unexpected results,
         * that fell into two broad categories:
         * 1. mixing percentages with units: `80 kg * 5% = 400 kg %`.
         * 2. percenteges squared: "5% * 5% = 25%^2"
         *
         * In practice, percentages are usually used as scalar multipliers,
         * not as "units" in the traditional sense.
         *
         * To avoid this, we have the following rules
         * for simplifying percentages.
         *
         * 1. If there are any other units (kg, lbs, etc..),
         *    then all of the percentages are removed (fixing first problem)
         * 2. No more than one "percentage unit" is permitted.
         *
         * This (mostly) fixes issue #164
         *
         * There is still some ambiguity here even after
         * going through these rules (although this fixes most of it).
         * See discussion on the "5_percent_times_100" test for more details.
         */
        let mut have_percentage_unit = false;
        let has_nonpercentage_components =
            self.unit.components.iter().any(|u| !u.is_percentage_unit());
        // combine identical or compatible units by summing their exponents
        // and potentially adjusting the value
        'outer: for comp in self.unit.components {
            if comp.is_percentage_unit() {
                if have_percentage_unit || has_nonpercentage_components {
                    let adjusted_res = Exact {
                        value: res_value,
                        exact: res_exact,
                    }
                    .mul(
                        &Exact {
                            value: comp.unit.scale.clone().into(),
                            exact: true,
                        },
                        int,
                    )?;
                    res_value = adjusted_res.value;
                    res_exact = adjusted_res.exact;
                    continue 'outer;
                }
                // already encountered one (if we see another one, strip it)
                have_percentage_unit = true;
            }
            for res_comp in &mut res_components {
                if comp.unit.has_no_base_units() && comp.unit != res_comp.unit {
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
                    Ok(scale_factor) => {
                        if scale_factor.offset.value != 0.into() {
                            // don't merge units that have offsets
                            break;
                        }
                        let scale = scale_factor.scale_1.div(scale_factor.scale_2, int)?;

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
                            value: res_value.one_point()?,
                            exact: res_exact,
                        }
                        .mul(&scale, int)?;
                        res_value = Dist::from(adjusted_value.value);
                        res_exact = res_exact && adjusted_value.exact;

                        continue 'outer;
                    }
                    Err(FendError::Interrupted) => return Err(FendError::Interrupted),
                    Err(_) => (),
                };
            }
            res_components.push(comp.clone());
        }

        // remove units with exponent == 0
        res_components.retain(|unit_exponent| unit_exponent.exponent != 0.into());

        Ok(Self {
            value: res_value,
            unit: Unit {
                components: res_components,
            },
            exact: res_exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        })
    }

    pub(crate) fn unit_equal_to(&self, rhs: &str) -> bool {
        self.unit.equal_to(rhs)
    }
}

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
            exact: self.exact,
            base: self.base,
            format: self.format,
            simplifiable: self.simplifiable,
        }
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self {
        Self {
            value: i.into(),
            unit: Unit::unitless(),
            exact: true,
            base: Base::default(),
            format: FormattingStyle::default(),
            simplifiable: true,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        let simplifiable = if self.simplifiable { "" } else { "not " };
        write!(
            f,
            "{:?} {:?} ({:?}, {:?}, {simplifiable}simplifiable)",
            self.value, self.unit, self.base, self.format
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct FormattedValue {
    exact: bool,
    number: String,
    unit_str: String,
}

impl FormattedValue {
    pub(crate) fn spans(self, spans: &mut Vec<Span>, attrs: Attrs) {
        if !self.exact && attrs.show_approx && !attrs.plain_number {
            spans.push(Span {
                string: "approx. ".to_string(),
                kind: SpanKind::Ident,
            });
        }
        if self.unit_str == "$" || self.unit_str == "\u{a3}" && !attrs.plain_number {
            spans.push(Span {
                string: self.unit_str,
                kind: SpanKind::Ident,
            });
            spans.push(Span {
                string: self.number,
                kind: SpanKind::Number,
            });
            return;
        }
        spans.push(Span {
            string: self.number.to_string(),
            kind: SpanKind::Number,
        });
        if !attrs.plain_number {
            spans.push(Span {
                string: self.unit_str,
                kind: SpanKind::Ident,
            });
        }
    }
}

impl fmt::Display for FormattedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        write!(f, "{}{}", self.number, self.unit_str)?;
        Ok(())
    }
}

#[derive(Clone)]
struct Unit {
    components: Vec<UnitExponent>,
}

type HashmapScale = (HashMap<BaseUnit, Complex>, Exact<Complex>);
type HashmapScaleOffset = (HashMap<BaseUnit, Complex>, Exact<Complex>, Exact<Complex>);

struct ScaleFactor {
    scale_1: Exact<Complex>,
    offset: Exact<Complex>,
    scale_2: Exact<Complex>,
}

impl Unit {
    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        serialize_usize(self.components.len(), write)?;
        for c in &self.components {
            c.serialize(write)?;
        }
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        let len = deserialize_usize(read)?;
        let mut cs = Vec::with_capacity(len);
        for _ in 0..len {
            cs.push(UnitExponent::deserialize(read)?);
        }
        Ok(Self { components: cs })
    }

    pub(crate) fn equal_to(&self, rhs: &str) -> bool {
        if self.components.len() != 1 {
            return false;
        }
        let unit = &self.components[0];
        if unit.exponent != 1.into() {
            return false;
        }
        let (prefix, name) = unit.unit.prefix_and_name(false);
        prefix.is_empty() && name == rhs
    }

    /// guarantees that base units with an cancelled exponents do not appear in the hashmap
    fn to_hashmap_and_scale<I: Interrupt>(&self, int: &I) -> Result<HashmapScale, FendError> {
        let mut hashmap = HashMap::<BaseUnit, Complex>::new();
        let mut scale = Complex::from(1);
        let mut exact = true;
        for named_unit_exp in &self.components {
            named_unit_exp.add_to_hashmap(&mut hashmap, &mut scale, &mut exact, int)?;
        }
        Ok((hashmap, Exact::new(scale, exact)))
    }

    fn reduce_hashmap<I: Interrupt>(
        hashmap: HashMap<BaseUnit, Complex>,
        int: &I,
    ) -> Result<HashmapScaleOffset, FendError> {
        if hashmap.len() == 1
            && hashmap.get(&BaseUnit::new(Cow::Borrowed("celsius"))) == Some(&1.into())
        {
            let mut result_hashmap = HashMap::new();
            result_hashmap.insert(BaseUnit::new(Cow::Borrowed("kelvin")), 1.into());
            return Ok((
                result_hashmap,
                Exact::new(1.into(), true),
                Exact::new(Complex::from(27315), true)
                    .div(Exact::new(Complex::from(100), true), int)?,
            ));
        }
        if hashmap.len() == 1
            && hashmap.get(&BaseUnit::new(Cow::Borrowed("fahrenheit"))) == Some(&1.into())
        {
            let mut result_hashmap = HashMap::new();
            result_hashmap.insert(BaseUnit::new(Cow::Borrowed("kelvin")), 1.into());
            return Ok((
                result_hashmap,
                Exact::new(Complex::from(5), true).div(Exact::new(Complex::from(9), true), int)?,
                Exact::new(Complex::from(45967), true)
                    .div(Exact::new(Complex::from(180), true), int)?,
            ));
        }
        let mut scale_adjustment = Exact::new(Complex::from(1), true);
        let mut result_hashmap = HashMap::new();
        for (mut base_unit, exponent) in hashmap {
            if base_unit.name() == "celsius" {
                base_unit = BaseUnit::new_static("kelvin");
            } else if base_unit.name() == "fahrenheit" {
                base_unit = BaseUnit::new_static("kelvin");
                scale_adjustment = scale_adjustment.mul(
                    &Exact::new(Complex::from(5), true)
                        .div(Exact::new(Complex::from(9), true), int)?
                        .value
                        .pow(exponent.clone(), int)?,
                    int,
                )?;
            }
            result_hashmap.insert(base_unit.clone(), exponent.clone());
        }
        Ok((result_hashmap, scale_adjustment, Exact::new(0.into(), true)))
    }

    fn print_base_units<I: Interrupt>(
        hash: HashMap<BaseUnit, Complex>,
        int: &I,
    ) -> Result<String, FendError> {
        let from_base_units: Vec<_> = hash
            .into_iter()
            .map(|(base_unit, exponent)| {
                UnitExponent::new(NamedUnit::new_from_base(base_unit), exponent)
            })
            .collect();
        Ok(Self {
            components: from_base_units,
        }
        .format(
            "unitless",
            false,
            Base::default(),
            FormattingStyle::Auto,
            false,
            int,
        )?
        .value)
    }

    /// Returns the combined scale factor if successful
    fn compute_scale_factor<I: Interrupt>(
        from: &Self,
        into: &Self,
        int: &I,
    ) -> Result<ScaleFactor, FendError> {
        let (hash_a, scale_a) = from.to_hashmap_and_scale(int)?;
        let (hash_b, scale_b) = into.to_hashmap_and_scale(int)?;
        let (hash_a, adj_a, offset_a) = Self::reduce_hashmap(hash_a, int)?;
        let (hash_b, adj_b, offset_b) = Self::reduce_hashmap(hash_b, int)?;
        if hash_a == hash_b {
            Ok(ScaleFactor {
                scale_1: scale_a.mul(&adj_a, int)?,
                offset: offset_a.add(-offset_b, int)?,
                scale_2: scale_b.mul(&adj_b, int)?,
            })
        } else {
            let from_formatted = from
                .format(
                    "unitless",
                    false,
                    Base::default(),
                    FormattingStyle::Auto,
                    false,
                    int,
                )?
                .value;
            let into_formatted = into
                .format(
                    "unitless",
                    false,
                    Base::default(),
                    FormattingStyle::Auto,
                    false,
                    int,
                )?
                .value;
            Err(FendError::IncompatibleConversion {
                from: from_formatted,
                to: into_formatted,
                from_base: Self::print_base_units(hash_a, int)?,
                to_base: Self::print_base_units(hash_b, int)?,
            })
        }
    }

    const fn unitless() -> Self {
        Self { components: vec![] }
    }

    fn format<I: Interrupt>(
        &self,
        unitless: &str,
        value_is_one: bool,
        base: Base,
        format: FormattingStyle,
        consider_printing_space: bool,
        int: &I,
    ) -> Result<Exact<String>, FendError> {
        let mut unit_string = String::new();
        if self.components.is_empty() {
            unit_string.push_str(unitless);
            return Ok(Exact::new(unit_string, true));
        }
        // Pluralisation:
        // All units should be singular, except for the last unit
        // that has a positive exponent, iff the number is not equal to 1
        let mut exact = true;
        let mut positive_components = vec![];
        let mut negative_components = vec![];
        let mut first = true;
        for unit_exponent in &self.components {
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
        let last_component_plural = !value_is_one;
        for (i, (unit_exponent, invert)) in merged_components.into_iter().enumerate() {
            if !first || (consider_printing_space && unit_exponent.unit.print_with_space()) {
                unit_string.push(' ');
            }
            first = false;
            if invert {
                unit_string.push('/');
                unit_string.push(' ');
            }
            let plural = last_component_plural && i == pluralised_idx;
            let exp_format = if format == FormattingStyle::Auto {
                FormattingStyle::Exact
            } else {
                format
            };
            let formatted_exp = unit_exponent.format(base, exp_format, plural, invert, int)?;
            unit_string.push_str(formatted_exp.value.to_string().as_str());
            exact = exact && formatted_exp.exact;
        }
        Ok(Exact::new(unit_string, true))
    }
}

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interrupt::Never;

    fn to_string(n: &Value) -> String {
        let int = &crate::interrupt::Never::default();
        n.format(&crate::Context::new(), int).unwrap().to_string()
    }

    #[test]
    fn test_basic_kg() {
        let base_kg = BaseUnit::new("kilogram".into());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("k".into(), "g".into(), "g".into(), hashmap, 1);
        let one_kg = Value::new(1, vec![UnitExponent::new(kg.clone(), 1)]);
        let two_kg = Value::new(2, vec![UnitExponent::new(kg, 1)]);
        let sum = one_kg.add(two_kg, &Never::default()).unwrap();
        assert_eq!(to_string(&sum), "3 kg");
    }

    #[test]
    fn test_basic_kg_and_g() {
        let int = &Never::default();
        let base_kg = BaseUnit::new("kilogram".into());
        let mut hashmap = HashMap::new();
        hashmap.insert(base_kg, 1.into());
        let kg = NamedUnit::new("k".into(), "g".into(), "g".into(), hashmap.clone(), 1);
        let g = NamedUnit::new(
            Cow::Borrowed(""),
            Cow::Borrowed("g"),
            Cow::Borrowed("g"),
            hashmap,
            Exact::new(Complex::from(1), true)
                .div(Exact::new(1000.into(), true), int)
                .unwrap()
                .value,
        );
        let one_kg = Value::new(1, vec![UnitExponent::new(kg, 1)]);
        let twelve_g = Value::new(12, vec![UnitExponent::new(g, 1)]);
        assert_eq!(
            to_string(&one_kg.clone().add(twelve_g.clone(), int).unwrap()),
            "1.012 kg"
        );
        assert_eq!(to_string(&twelve_g.add(one_kg, int).unwrap()), "1012 g");
    }
}
