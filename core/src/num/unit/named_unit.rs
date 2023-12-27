use std::{borrow::Cow, collections::HashMap, fmt, io};

use super::base_unit::BaseUnit;
use crate::{
	error::FendError,
	num::complex::Complex,
	serialize::{
		deserialize_bool, deserialize_string, deserialize_usize, serialize_bool, serialize_string,
		serialize_usize,
	},
};

/// A named unit, like kilogram, megabyte or percent.
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct NamedUnit {
	prefix: Cow<'static, str>,
	pub(super) singular_name: Cow<'static, str>,
	plural_name: Cow<'static, str>,
	alias: bool,
	pub(super) base_units: HashMap<BaseUnit, Complex>,
	pub(super) scale: Complex,
}

impl NamedUnit {
	pub(crate) fn new(
		prefix: Cow<'static, str>,
		singular_name: Cow<'static, str>,
		plural_name: Cow<'static, str>,
		alias: bool,
		base_units: HashMap<BaseUnit, Complex>,
		scale: impl Into<Complex>,
	) -> Self {
		Self {
			prefix,
			singular_name,
			plural_name,
			alias,
			base_units,
			scale: scale.into(),
		}
	}

	pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
		serialize_string(self.prefix.as_ref(), write)?;
		serialize_string(self.singular_name.as_ref(), write)?;
		serialize_string(self.plural_name.as_ref(), write)?;
		serialize_bool(self.alias, write)?;

		serialize_usize(self.base_units.len(), write)?;
		for (a, b) in &self.base_units {
			a.serialize(write)?;
			b.serialize(write)?;
		}

		self.scale.serialize(write)?;
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
		let prefix = deserialize_string(read)?;
		let singular_name = deserialize_string(read)?;
		let plural_name = deserialize_string(read)?;
		let alias = deserialize_bool(read)?;

		let len = deserialize_usize(read)?;
		let mut hashmap = HashMap::with_capacity(len);
		for _ in 0..len {
			let k = BaseUnit::deserialize(read)?;
			let v = Complex::deserialize(read)?;
			hashmap.insert(k, v);
		}
		Ok(Self {
			prefix: Cow::Owned(prefix),
			singular_name: Cow::Owned(singular_name),
			plural_name: Cow::Owned(plural_name),
			alias,
			base_units: hashmap,
			scale: Complex::deserialize(read)?,
		})
	}

	pub(crate) fn new_from_base(base_unit: BaseUnit) -> Self {
		Self {
			prefix: "".into(),
			singular_name: base_unit.name().to_string().into(),
			plural_name: base_unit.name().to_string().into(),
			alias: false,
			base_units: {
				let mut base_units = HashMap::new();
				base_units.insert(base_unit, 1.into());
				base_units
			},
			scale: 1.into(),
		}
	}

	pub(crate) fn prefix_and_name(&self, plural: bool) -> (&str, &str) {
		(
			self.prefix.as_ref(),
			if plural {
				self.plural_name.as_ref()
			} else {
				self.singular_name.as_ref()
			},
		)
	}

	pub(crate) fn has_no_base_units(&self) -> bool {
		self.base_units.is_empty()
	}

	pub(crate) fn is_alias(&self) -> bool {
		self.alias && self.has_no_base_units()
	}

	/// Returns whether or not this unit should be printed with a
	/// space (between the number and the unit). This should be true for most
	/// units like kg or m, but not for % or °
	pub(crate) fn print_with_space(&self) -> bool {
		// Alphabetic names like kg or m should have a space,
		// while non-alphabetic names like % or ' shouldn't.
		// Empty names shouldn't really exist, but they might as well have a space.

		// degree symbol
		if self.singular_name == "\u{b0}" {
			return false;
		}

		// if it starts with a quote and is more than one character long, print it with a space
		if (self.singular_name.starts_with('\'') || self.singular_name.starts_with('\"'))
			&& self.singular_name.len() > 1
		{
			return true;
		}

		self.singular_name
			.chars()
			.next()
			.map_or(true, |first_char| {
				char::is_alphabetic(first_char) || first_char == '\u{b0}'
			})
	}
}

impl fmt::Debug for NamedUnit {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
		it.sort_by_key(|(k, _v)| k.name());
		for (base_unit, exponent) in &it {
			write!(f, " {base_unit:?}")?;
			if !exponent.is_definitely_one() {
				write!(f, "^{exponent:?}")?;
			}
		}
		write!(f, ")")?;
		Ok(())
	}
}
