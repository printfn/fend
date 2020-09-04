use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub trait Error: Display {}

#[allow(clippy::empty_enum)]
pub enum Never {}

impl Display for Never {
    fn fmt(&self, _: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {}
    }
}
impl Debug for Never {
    fn fmt(&self, _: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {}
    }
}
impl From<Never> for String {
    fn from(n: Never) -> Self {
        match n {}
    }
}

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
                    Self::$i => write!(f, "{}", $e),
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

impl<E: Error> From<IntErr<E>> for String {
    fn from(e: IntErr<E>) -> String {
        match e {
            IntErr::Interrupt(i) => i.to_string(),
            IntErr::Error(e) => e.to_string(),
        }
    }
}

impl From<std::fmt::Error> for IntErr<std::fmt::Error> {
    fn from(e: std::fmt::Error) -> Self {
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
            Self::Interrupt(i) => write!(f, "{}", i)?,
            Self::Error(e) => write!(f, "{:?}", e)?,
        }
        Ok(())
    }
}

impl From<IntErr<Never>> for String {
    fn from(e: IntErr<Never>) -> Self {
        match e {
            IntErr::Error(never) => match never {},
            IntErr::Interrupt(i) => i.to_string(),
        }
    }
}

impl Error for std::fmt::Error {}

#[derive(Debug)]
pub enum Interrupt {
    Interrupt,
}
impl Display for Interrupt {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Interrupt => write!(f, "Interrupted"),
        }
    }
}
impl From<Interrupt> for String {
    fn from(e: Interrupt) -> Self {
        e.to_string()
    }
}
impl Default for Interrupt {
    fn default() -> Self {
        Self::Interrupt
    }
}
impl Interrupt {
    pub fn err<T, E: From<Self>>() -> Result<Result<T, E>, Interrupt> {
        Ok(Err(Self::default().into()))
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
