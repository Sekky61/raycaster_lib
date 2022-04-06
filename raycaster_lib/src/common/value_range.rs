use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ValueRange {
    pub low: f32,
    pub high: f32,
}

impl ValueRange {
    pub fn new() -> ValueRange {
        ValueRange {
            low: f32::NAN,
            high: f32::NAN,
        }
    }

    pub fn seed(val: f32) -> ValueRange {
        ValueRange {
            low: val,
            high: val,
        }
    }

    pub fn from_range(range: std::ops::Range<f32>) -> ValueRange {
        ValueRange {
            low: range.start,
            high: range.end,
        }
    }

    pub fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> ValueRange
    where
        T: Deref<Target = f32>,
    {
        let mut range = ValueRange::new();
        for val in iter {
            range.extend(*val);
        }
        range
    }

    pub fn extend(&mut self, val: f32) {
        if self.low.is_nan() || self.high.is_nan() {
            self.low = val;
            self.high = val;
        }

        if val > self.high {
            self.high = val;
        }

        if val < self.low {
            self.low = val;
        }
    }

    pub fn contains(&self, val: f32) -> bool {
        self.low <= val && val <= self.high
    }

    /// Touching intervals intersect
    pub fn intersects(&self, other: &ValueRange) -> bool {
        // todo test
        (other.low <= self.high && other.low >= self.low)
            || (other.high >= self.low && other.high <= self.high)
            || (self.high >= other.low && self.high <= other.high)
            || (self.low <= other.high && self.low >= other.low)
    }

    pub fn limits(&self) -> (f32, f32) {
        (self.low, self.high)
    }
}

impl Default for ValueRange {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn value_range() {
        let values = [0.0, 5.0, 3.0, -2.5];

        let mut range = ValueRange::seed(1.0);

        assert!(range.contains(1.0));
        assert!(!range.contains(1.2));
        assert!(!range.contains(0.9));

        for val in values {
            range.extend(val);
        }

        assert_eq!(range.low, -2.5);
        assert_eq!(range.high, 5.0);

        assert!(range.contains(4.2));
        assert!(range.contains(-0.5));
        assert!(!range.contains(-12.5));
    }

    #[test]
    fn empty_value_range() {
        let mut range = ValueRange::new();

        assert!(!range.contains(2.0));
        assert!(!range.contains(0.0));

        range.extend(2.0);

        assert!(range.contains(2.0));
        assert_eq!(range.low, 2.0);
        assert_eq!(range.high, 2.0);
    }
}
