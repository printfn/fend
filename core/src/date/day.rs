use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Day(u8);

impl Day {
    pub(crate) fn value(self) -> u8 {
        self.0
    }

    pub(crate) fn new(day: u8) -> Self {
        assert!(day != 0 && day < 32, "day value {} is out of range", day);
        Self(day)
    }
}

impl fmt::Debug for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn day_0() {
        Day::new(0);
    }

    #[test]
    #[should_panic]
    fn day_32() {
        Day::new(32);
    }

    #[test]
    fn day_to_string() {
        assert_eq!(Day::new(1).to_string(), "1");
    }
}
