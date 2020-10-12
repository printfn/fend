// helper struct for keeping track of which values are exact

use std::ops::Neg;

pub struct Exact<T: ?Sized> {
    pub exact: bool,
    pub value: T,
}

impl<T: Clone> Clone for Exact<T> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone(), exact: self.exact }
    }
}

impl<T: Copy> Copy for Exact<T> {
}

#[allow(clippy::use_self)]
impl<T> Exact<T> {
    pub fn new(value: impl Into<T>, exact: bool) -> Self {
        Self {
            value: value.into(),
            exact,
        }
    }

    pub fn new_ok<E>(value: impl Into<T>, exact: bool) -> Result<Self, E> {
        Ok(Self::new(value, exact))
    }

    pub fn apply<R, F: FnOnce(T) -> R>(self, f: F) -> Exact<R> {
        Exact::<R> {
            value: f(self.value),
            exact: self.exact,
        }
    }

    pub fn apply_x<R, E, F: FnOnce(T) -> Result<(R, bool), E>>(self, f: F) -> Result<Exact<R>, E> {
        let (value, exact) = f(self.value)?;
        Ok(Exact::<R> {
            value,
            exact: self.exact && exact,
        })
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
            exact: self.exact
        }
    }
}

impl<T: Neg + Neg<Output = T>> Neg for Exact<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            exact: self.exact,
        }
    }
}
