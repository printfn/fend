use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub trait Error: Display {}

pub type Never = std::convert::Infallible;

macro_rules! make_err {
    ($i:ident, $($a:ident, )*) => {
        #[derive(Debug)]
        pub enum $i { $($a($a), )* }
        impl Display for $i {
            fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                match self {
                    $(Self::$a(v) => write!(f, "{}", v),)*
                }
            }
        }
        $(
            impl From<$a> for $i {
                fn from(v: $a) -> Self {
                    Self::$a(v)
                }
            }
            impl From<$a> for IntErr<$i> {
                fn from(v: $a) -> Self {
                    Self::Error(v.into())
                }
            }
        )*
        // eventually we should be able to remove this
        // (once all the string-based error handling is gone)
        impl From<IntErr<$i>> for IntErr<String> {
            fn from(v: IntErr<$i>) -> Self {
                match v {
                    IntErr::<$i>::Interrupt(i) => Self::Interrupt(i),
                    IntErr::<$i>::Error(e) => Self::Error(e.to_string()),
                }
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
                    Self::$i => write!(f, "{}", $e),
                }
            }
        }
        impl Error for $i {}
        impl From<$i> for IntErr<String> {
            fn from(v: $i) -> Self {
                Self::Error(v.to_string())
            }
        }
        impl From<IntErr<$i>> for IntErr<String> {
            fn from(v: IntErr<$i>) -> Self {
                match v {
                    IntErr::<$i>::Interrupt(i) => Self::Interrupt(i),
                    IntErr::<$i>::Error(e) => Self::Error(e.to_string()),
                }
            }
        }
        impl Default for $i {
            fn default() -> Self {
                Self::$i
            }
        }
        impl $i {
            pub fn err<T, E: From<Self>>() -> Result<T, E> {
                Err(Self::default().into())
            }
            pub fn ierr<T, E: From<Self> + Error>() -> Result<T, IntErr<E>> {
                Err(IntErr::Error(E::from(Self::default())))
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum IntErr<E> {
    Interrupt(Interrupt),
    Error(E),
}

impl<E> IntErr<E> {
    pub fn expect(self, msg: &'static str) -> IntErr<Never> {
        match self {
            Self::Interrupt(i) => IntErr::<Never>::Interrupt(i),
            Self::Error(_) => panic!(msg),
        }
    }

    pub fn unwrap(self) -> IntErr<Never> {
        match self {
            Self::Interrupt(i) => IntErr::<Never>::Interrupt(i),
            Self::Error(_) => panic!("Unwrap"),
        }
    }
}

impl<E> From<Interrupt> for IntErr<E> {
    fn from(i: Interrupt) -> Self {
        Self::Interrupt(i)
    }
}

impl<E: Error> From<E> for IntErr<E> {
    fn from(e: E) -> Self {
        Self::Error(e)
    }
}

impl<E: Error> From<IntErr<Never>> for IntErr<E> {
    fn from(e: IntErr<Never>) -> Self {
        match e {
            IntErr::Error(never) => match never {},
            IntErr::Interrupt(i) => Self::Interrupt(i),
        }
    }
}

impl<E: std::fmt::Debug> std::fmt::Debug for IntErr<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Interrupt(i) => write!(f, "{:?}", i)?,
            Self::Error(e) => write!(f, "{:?}", e)?,
        }
        Ok(())
    }
}

impl Error for std::fmt::Error {}
impl Error for String {}

#[derive(Debug)]
pub enum Interrupt {
    Interrupt,
}
impl Default for Interrupt {
    fn default() -> Self {
        Self::Interrupt
    }
}

make_err!(ValueTooLarge, "Value too large");
make_err!(
    ZeroToThePowerOfZero,
    "Zero to the power of zero is undefined"
);
make_err!(ExponentTooLarge, "Exponent too large");
make_err!(IntegerPowerError, ExponentTooLarge, ZeroToThePowerOfZero,);
make_err!(DivideByZero, "Division by zero");
