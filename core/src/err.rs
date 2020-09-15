use std::fmt::{Debug, Display};

pub trait Error: Display {}

pub type Never = std::convert::Infallible;

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

impl<E: Display, I: Interrupt> IntErr<E, I> {
    pub fn into_string(self) -> IntErr<String, I> {
        match self {
            Self::Interrupt(i) => IntErr::<String, I>::Interrupt(i),
            Self::Error(e) => IntErr::<String, I>::Error(e.to_string()),
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
