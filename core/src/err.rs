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
            impl<I: Interrupt> From<$a> for IntErr<$i, I> {
                fn from(v: $a) -> Self {
                    Self::Error(v.into())
                }
            }
        )*
        // eventually we should be able to remove this
        // (once all the string-based error handling is gone)
        impl<I: Interrupt> From<IntErr<$i, I>> for IntErr<String, I> {
            fn from(v: IntErr<$i, I>) -> Self {
                match v {
                    IntErr::<$i, I>::Interrupt(i) => Self::Interrupt(i),
                    IntErr::<$i, I>::Error(e) => Self::Error(e.to_string()),
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
        impl<I: Interrupt> From<$i> for IntErr<String, I> {
            fn from(v: $i) -> Self {
                Self::Error(v.to_string())
            }
        }
        impl<I: Interrupt> From<IntErr<$i, I>> for IntErr<String, I> {
            fn from(v: IntErr<$i, I>) -> Self {
                match v {
                    IntErr::<$i, I>::Interrupt(i) => Self::Interrupt(i),
                    IntErr::<$i, I>::Error(e) => Self::Error(e.to_string()),
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
            pub fn ierr<T, E: From<Self> + Error, I: Interrupt>() -> Result<T, IntErr<E, I>> {
                Err(IntErr::Error(E::from(Self::default())))
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum IntErr<E, I: Interrupt> {
    Interrupt(I::Int),
    Error(E),
}

impl<E, I: Interrupt> IntErr<E, I> {
    pub fn expect(self, msg: &'static str) -> IntErr<Never, I> {
        match self {
            Self::Interrupt(i) => IntErr::<Never, I>::Interrupt(i),
            Self::Error(_) => panic!(msg),
        }
    }

    pub fn unwrap(self) -> IntErr<Never, I> {
        match self {
            Self::Interrupt(i) => IntErr::<Never, I>::Interrupt(i),
            Self::Error(_) => panic!("Unwrap"),
        }
    }
}

impl<E> IntErr<E, NeverInterrupt> {
    pub fn get_error(self) -> E {
        match self {
            IntErr::Interrupt(i) => match i {},
            IntErr::Error(e) => e,
        }
    }
}

impl<E, I: Interrupt> From<E> for IntErr<E, I> {
    fn from(e: E) -> Self {
        Self::Error(e)
    }
}

impl<E: Error, I: Interrupt> From<IntErr<Never, I>> for IntErr<E, I> {
    fn from(e: IntErr<Never, I>) -> Self {
        match e {
            IntErr::Error(never) => match never {},
            IntErr::Interrupt(i) => Self::Interrupt(i),
        }
    }
}

impl<E: std::fmt::Debug, I: Interrupt> std::fmt::Debug for IntErr<E, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Interrupt(i) => write!(f, "{:?}", i)?,
            Self::Error(e) => write!(f, "{:?}", e)?,
        }
        Ok(())
    }
}

impl<E: std::fmt::Display, I: Interrupt> std::fmt::Display for IntErr<E, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Interrupt(i) => write!(f, "{:?}", i)?,
            Self::Error(e) => write!(f, "{}", e)?,
        }
        Ok(())
    }
}

impl Error for std::fmt::Error {}
impl Error for String {}

pub trait Interrupt {
    type Int: Debug;
    fn test(&self) -> Result<(), Self::Int>;
}

#[derive(Default)]
pub struct NeverInterrupt {}
impl Interrupt for NeverInterrupt {
    type Int = std::convert::Infallible;
    fn test(&self) -> Result<(), Self::Int> {
        Ok(())
    }
}

pub struct PossibleInterrupt {}
impl Interrupt for PossibleInterrupt {
    type Int = ();
    fn test(&self) -> Result<(), Self::Int> {
        Ok(())
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
