use std::fmt;
use std::fmt::{Display, Formatter};

pub trait Error : Display + Into<String> {
}

pub enum Never {
}

impl Error for Never {}
impl Display for Never {
    fn fmt(&self, _: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {}
    }
}
impl From<Never> for String {
    fn from(n: Never) -> Self {
        match n {}
    }
}

pub enum Combined<T: Error, U: Error> {
    A(T),
    B(U),
}
impl<T: Error, U: Error> Display for Combined<T, U> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::A(a) => a.fmt(f),
            Self::B(b) => b.fmt(f),
        }
    }
}
impl<T: Error, U: Error> From<Combined<T, U>> for String {
    fn from(c: Combined<T, U>) -> Self {
        c.to_string()
    }
}
impl<T: Error, U: Error> Error for Combined<T, U> {}

macro_rules! make_err {
    ($i:ident, $e:expr) => {
        pub enum $i { $i }
        impl Display for $i {
            fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                match self {
                    Self::$i => write!(f, $e),
                }
            }
        }
        impl Error for $i {}
        impl From<$i> for String {
            fn from(e: $i) -> Self {
                e.to_string()
            }
        }
        impl Default for $i {
            fn default() -> Self {
                Self::$i
            }
        }
    }
}

pub fn err<T, E>() -> Result<T, E> where E: Error + Default {
    Err(E::default())
}

make_err!(ValueTooLarge, "Value too large");
