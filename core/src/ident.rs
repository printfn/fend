use std::fmt;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Ident<'a>(&'a str);

impl<'a> Ident<'a> {
    pub(crate) fn new(s: &'a str) -> Self {
        Self(s)
    }

    pub(crate) fn as_str(&self) -> &'a str {
        self.0
    }
}

impl fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.chars() {
            if ch == '_' {
                write!(f, " ")?;
            } else {
                write!(f, "{}", ch)?;
            }
        }
        Ok(())
    }
}
