use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Epoch(u64);

impl Epoch {
    pub fn to_u64(&self) -> u64 {
        self.0
    }

    pub fn as_value(&self) -> &u64 {
        &self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1
    }
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Epoch {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
