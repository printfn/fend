use std::fmt;
use std::fmt::{Display, Formatter};

pub trait Error: Display + Into<String> {}

#[allow(clippy::empty_enum)]
pub enum Never {}

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
    ($i:ident, $($a:ident, )*) => {
        pub enum $i { $($a($a), )* }
        impl Display for $i {
            fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                match self {
                    $(Self::$a(v) => v.fmt(f),)*
                }
            }
        }
        $(
            impl From<$a> for $i {
                fn from(v: $a) -> Self {
                    Self::$a(v)
                }
            }
        )*
        // eventually we should be able to remove this
        // (once all the string-based error handling is gone)
        impl From<$i> for String {
            fn from(c: $i) -> Self {
                c.to_string()
            }
        }
        impl Error for $i {}
    };

    ($i:ident, $e:expr) => {
        #[derive(Debug)]
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
        impl $i {
            pub fn err<T, E: From<Self>>() -> Result<Result<T, E>, Interrupt> {
                Ok(Err(Self::default().into()))
            }
        }
    }
}

pub fn err<T, E>() -> Result<T, E>
where
    E: Error + Default,
{
    Err(E::default())
}

pub fn ret<T, E>(value: T) -> Result<Result<T, E>, Interrupt> {
    Ok(Ok(value))
}

make_err!(Interrupt, "Interrupted");

make_err!(ValueTooLarge, "Value too large");
make_err!(
    ZeroToThePowerOfZero,
    "Zero to the power of zero is undefined"
);
make_err!(ExponentTooLarge, "Exponent too large");
make_err!(
    IntegerPowerError,
    ExponentTooLarge,
    ZeroToThePowerOfZero,
);
make_err!(DivideByZero, "Division by zero");
