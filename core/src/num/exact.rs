// helper struct for keeping track of which values are exact

use std::ops::Neg;

pub struct Exact<T> {
    pub value: T,
    pub exact: bool,
}

#[allow(clippy::use_self)]
impl<T> Exact<T> {
    pub fn new(value: T, exact: bool) -> Self {
        Self { value, exact }
    }

    pub fn new_ok<E>(value: T, exact: bool) -> Result<Self, E> {
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
