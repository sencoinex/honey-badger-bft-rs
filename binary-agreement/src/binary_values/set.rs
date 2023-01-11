use super::BinaryValues;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BinaryValueSet(Option<BinaryValues>);

impl BinaryValueSet {
    pub fn new(values: BinaryValues) -> Self {
        Self(Some(values))
    }

    pub fn insert(&mut self, value: bool) -> bool {
        if let Some(values) = self.0 {
            self.0 = Some(values + BinaryValues::from(value));
            self.0.unwrap() != values
        } else {
            self.0 = Some(value.into());
            true
        }
    }

    pub fn includes(&self, values: BinaryValues) -> bool {
        if let Some(mine) = self.0 {
            mine.includes(values)
        } else {
            false
        }
    }

    pub fn is_set(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_not_set(&self) -> bool {
        self.0.is_none()
    }

    pub fn values(&self) -> &BinaryValues {
        self.0.as_ref().unwrap()
    }
}
