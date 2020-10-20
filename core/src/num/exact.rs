// helper struct for keeping track of which values are exact

use std::fmt;
use std::ops::Neg;

#[derive(Copy, Clone)]
pub struct Exact<T: fmt::Debug> {
    pub value: T,
    pub exact: bool,
}

impl<T: fmt::Debug> fmt::Debug for Exact<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.exact {
            write!(f, "exactly ")?;
        } else {
            write!(f, "approx. ")?;
        }
        write!(f, "{:?}", self.value)?;
        Ok(())
    }
}

#[allow(clippy::use_self)]
impl<T: fmt::Debug> Exact<T> {
    pub fn new(value: T, exact: bool) -> Self {
        Self { value, exact }
    }

    pub fn apply<R: fmt::Debug, F: FnOnce(T) -> R>(self, f: F) -> Exact<R> {
        Exact::<R> {
            value: f(self.value),
            exact: self.exact,
        }
    }

    pub fn combine(self, x: bool) -> Self {
        Self {
            value: self.value,
            exact: self.exact && x,
        }
    }

    pub fn re<'a>(&'a self) -> Exact<&'a T> {
        Exact::<&'a T> {
            value: &self.value,
            exact: self.exact,
        }
    }
}

#[allow(clippy::use_self)]
impl<A: fmt::Debug, B: fmt::Debug> Exact<(A, B)> {
    pub fn pair(self) -> (Exact<A>, Exact<B>) {
        (
            Exact {
                value: self.value.0,
                exact: self.exact,
            },
            Exact {
                value: self.value.1,
                exact: self.exact,
            },
        )
    }
}

impl<T: fmt::Debug + Neg + Neg<Output = T>> Neg for Exact<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            exact: self.exact,
        }
    }
}
