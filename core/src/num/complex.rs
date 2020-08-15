use crate::num::bigrat::{BigRat, FormattingStyle};
use crate::num::Base;
use std::cmp::Ordering;
use std::fmt::{Error, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Complex {
    real: BigRat,
    imag: BigRat,
}

impl PartialOrd for Complex {
    fn partial_cmp(&self, other: &Complex) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.real <= other.real && self.imag <= other.imag {
            Some(Ordering::Less)
        } else if self.real >= other.real && self.imag >= other.imag {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl Add for Complex {
    type Output = Complex;

    fn add(self, rhs: Complex) -> Complex {
        Complex {
            real: self.real + rhs.real,
            imag: self.imag + rhs.imag,
        }
    }
}

impl Neg for Complex {
    type Output = Complex;

    fn neg(self) -> Complex {
        Complex {
            real: -self.real,
            imag: -self.imag,
        }
    }
}

impl Neg for &Complex {
    type Output = Complex;

    fn neg(self) -> Complex {
        -self.clone()
    }
}

impl Sub for Complex {
    type Output = Complex;

    fn sub(self, rhs: Complex) -> Complex {
        self + -rhs
    }
}

impl Sub for &Complex {
    type Output = Complex;

    fn sub(self, rhs: &Complex) -> Complex {
        self.clone() + -rhs.clone()
    }
}

impl Mul for Complex {
    type Output = Complex;

    fn mul(self, rhs: Complex) -> Complex {
        // (a + bi) * (c + di)
        //     => ac + bci + adi - bd
        //     => (ac - bd) + (bc + ad)i
        Complex {
            real: self.real.clone() * rhs.real.clone() - self.imag.clone() * rhs.imag.clone(),
            imag: self.real * rhs.imag + self.imag * rhs.real,
        }
    }
}

impl From<u64> for Complex {
    fn from(i: u64) -> Self {
        Complex {
            real: i.into(),
            imag: 0.into(),
        }
    }
}

impl From<i32> for Complex {
    fn from(i: i32) -> Self {
        Complex {
            real: i.into(),
            imag: 0.into(),
        }
    }
}

impl Complex {
    pub fn conjugate(self) -> Self {
        Complex {
            real: self.real,
            imag: -self.imag,
        }
    }

    pub fn div(self, rhs: Complex) -> Result<Complex, String> {
        // (u + vi) / (x + yi) = (1/(x^2 + y^2)) * ((ux + vy) + (vx - uy)i)
        let u = self.real;
        let v = self.imag;
        let x = rhs.real;
        let y = rhs.imag;
        Ok(Complex {
            real: BigRat::from(1).div(x.clone() * x.clone() + y.clone() * y.clone())?,
            imag: 0.into(),
        } * Complex {
            real: u.clone() * x.clone() + v.clone() * y.clone(),
            imag: v.clone() * x.clone() - u.clone() * y.clone(),
        })
    }

    pub fn pow(self, rhs: Complex) -> Result<Complex, String> {
        if self.imag != 0.into() || rhs.imag != 0.into() {
            return Err("Exponentiation is currently unsupported for complex numbers".to_string());
        }
        Ok(Complex {
            real: self.real.pow(rhs.real)?,
            imag: 0.into(),
        })
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_digit_in_base(&mut self, digit: u64, base: u8) {
        self.real.add_digit_in_base(digit, base)
    }

    pub fn i() -> Complex {
        Complex {
            real: 0.into(),
            imag: 1.into(),
        }
    }
}

impl Complex {
    pub fn format(&self, f: &mut Formatter, exact: bool, base: Base) -> Result<(), Error> {
        let style = if exact {
            FormattingStyle::ExactFloatWithFractionFallback
        } else {
            FormattingStyle::ApproxFloat
        };
        if self.imag == 0.into() {
            self.real.format(f, base, style, false)?;
            return Ok(());
        }

        if self.real != 0.into() {
            self.real.format(f, base, style, false)?;
            if self.imag > 0.into() {
                write!(f, " + ")?;
                self.imag.format(f, base, style, true)?;
            } else {
                write!(f, " - ")?;
                (-self.imag.clone()).format(f, base, style, true)?;
            }
        } else {
            self.imag.format(f, base, style, true)?;
        }

        Ok(())
    }
}

impl Complex {
    pub fn root_n(self, n: &Complex) -> Result<(Complex, bool), String> {
        if self.imag != 0.into() || n.imag != 0.into() {
            return Err("Roots are currently unsupported for complex numbers".to_string());
        }
        let (real_root, real_root_exact) = self.real.root_n(&n.real)?;
        Ok((
            Complex {
                real: real_root,
                imag: 0.into(),
            },
            real_root_exact,
        ))
    }

    pub fn approx_pi() -> Complex {
        Complex {
            real: BigRat::approx_pi(),
            imag: 0.into(),
        }
    }
}
