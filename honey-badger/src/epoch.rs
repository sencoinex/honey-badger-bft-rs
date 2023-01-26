use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Epoch(u64);

impl Epoch {
    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn as_u64(&self) -> &u64 {
        self.as_ref()
    }

    pub fn increment(&mut self) {
        self.0 += 1
    }
}

impl AsRef<u64> for Epoch {
    fn as_ref(&self) -> &u64 {
        &self.0
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

impl From<usize> for Epoch {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}
