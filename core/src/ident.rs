use std::{borrow::Cow, fmt};

#[derive(Clone, Debug)]
pub(crate) struct Ident(Cow<'static, str>);

impl Ident {
    pub(crate) fn new_str(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub(crate) fn new_string(s: String) -> Self {
        Self(Cow::Owned(s))
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub(crate) fn is_prefix_unit(&self) -> bool {
        // when changing this also make sure to change number output formatting
        // lexer identifier splitting
        self.0 == "$" || self.0 == "\u{a3}"
    }
}

impl From<String> for Ident {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl From<&'static str> for Ident {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let use_spaces = !self.0.starts_with('_');
        for ch in self.0.chars() {
            if use_spaces && ch == '_' {
                write!(f, " ")?;
            } else {
                write!(f, "{}", ch)?;
            }
        }
        Ok(())
    }
}
