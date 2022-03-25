use std::{borrow::Cow, fmt, io};

use crate::{
    error::FendError,
    serialize::{deserialize_string, serialize_string},
};

/// Represents a base unit, identified solely by its name. The name is not exposed to the user.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct BaseUnit {
    name: Cow<'static, str>,
}

impl fmt::Debug for BaseUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl BaseUnit {
    pub(crate) const fn new(name: Cow<'static, str>) -> Self {
        Self { name }
    }

    pub(crate) const fn new_static(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
        serialize_string(self.name.as_ref(), write)?;
        Ok(())
    }

    pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
        Ok(Self {
            name: Cow::Owned(deserialize_string(read)?),
        })
    }
}
