use std::{convert, error, fmt};

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum FendError {
    InvalidBasePrefix,
    BaseTooSmall,
    BaseTooLarge,
}

impl fmt::Display for FendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBasePrefix => write!(
                f,
                "unable to parse a valid base prefix, expected 0b, 0o, or 0x"
            ),
            Self::BaseTooSmall => write!(f, "base must be at least 2"),
            Self::BaseTooLarge => write!(f, "base cannot be larger than 36"),
        }
    }
}

impl error::Error for FendError {}

pub(crate) trait Error: fmt::Display {}

pub(crate) type Never = convert::Infallible;

pub(crate) enum IntErr<E, I: Interrupt> {
    Interrupt(I::Int),
    Error(E),
}

#[allow(clippy::use_self)]
impl<E, I: Interrupt> IntErr<E, I> {
    pub(crate) fn expect(self, msg: &'static str) -> IntErr<Never, I> {
        match self {
            Self::Interrupt(i) => IntErr::<Never, I>::Interrupt(i),
            Self::Error(_) => panic!("{}", msg),
        }
    }

    pub(crate) fn unwrap(self) -> IntErr<Never, I> {
        match self {
            Self::Interrupt(i) => IntErr::<Never, I>::Interrupt(i),
            Self::Error(_) => panic!("unwrap"),
        }
    }

    pub(crate) fn map<F>(self, f: impl FnOnce(E) -> F) -> IntErr<F, I> {
        match self {
            Self::Interrupt(i) => IntErr::Interrupt(i),
            Self::Error(e) => IntErr::Error(f(e)),
        }
    }
}

#[allow(clippy::use_self)]
impl<E: fmt::Display, I: Interrupt> IntErr<E, I> {
    pub(crate) fn into_string(self) -> IntErr<String, I> {
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

#[allow(clippy::use_self)]
impl<E: Error, I: Interrupt> From<IntErr<Never, I>> for IntErr<E, I> {
    fn from(e: IntErr<Never, I>) -> Self {
        match e {
            IntErr::Error(never) => match never {},
            IntErr::Interrupt(i) => Self::Interrupt(i),
        }
    }
}

impl<E: fmt::Debug, I: Interrupt> fmt::Debug for IntErr<E, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Interrupt(i) => write!(f, "{:?}", i)?,
            Self::Error(e) => write!(f, "{:?}", e)?,
        }
        Ok(())
    }
}

impl Error for String {}

pub(crate) trait Interrupt {
    type Int: fmt::Debug;
    fn test(&self) -> Result<(), Self::Int>;
}

#[derive(Default)]
pub(crate) struct NeverInterrupt {}
impl Interrupt for NeverInterrupt {
    type Int = convert::Infallible;
    fn test(&self) -> Result<(), Self::Int> {
        Ok(())
    }
}

pub(crate) struct PossibleInterrupt {}
impl Interrupt for PossibleInterrupt {
    type Int = ();
    fn test(&self) -> Result<(), Self::Int> {
        Ok(())
    }
}
