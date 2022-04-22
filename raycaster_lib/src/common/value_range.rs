use std::ops::{Deref, Range};

/// Represents a range of floating-point values.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ValueRange {
    /// Lower bound
    pub low: f32,
    /// Upper bound
    pub high: f32,
}

impl ValueRange {
    /// Constructs new, empty range.
    pub fn empty() -> ValueRange {
        ValueRange {
            low: f32::NAN,
            high: f32::NAN,
        }
    }

    /// Constructs new range with one element, `val`.
    pub fn seed(val: f32) -> ValueRange {
        ValueRange {
            low: val,
            high: val,
        }
    }

    /// Constructs minimal range, where all samples from an iterator
    /// are inside the range.
    pub fn from_samples<T, I>(iter: impl IntoIterator<Item = T>) -> ValueRange
    where
        T: Deref<Target = I>,
        I: Into<f32> + Copy,
    {
        let mut range = ValueRange::empty();
        for val in iter {
            range.extend((*val).into());
        }
        range
    }

    /// Extend the range with new value.
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

    /// Check if value is inside the range.
    pub fn contains(&self, val: f32) -> bool {
        self.low <= val && val <= self.high
    }

    /// Check if two `ValueRange`s have common items.
    /// Touching intervals intersect.
    pub fn intersects(&self, other: &ValueRange) -> bool {
        (other.low <= self.high && other.low >= self.low)
            || (other.high >= self.low && other.high <= self.high)
            || (self.high >= other.low && self.high <= other.high)
            || (self.low <= other.high && self.low >= other.low)
    }
}

impl Default for ValueRange {
    fn default() -> Self {
        Self::empty()
    }
}

/// Conversion from standard library type.
/// Unlocks simple syntax:
/// ```
/// # use raycaster_lib::common::ValueRange;
/// let range: ValueRange = (0.0..45.5).into();
/// ```
impl From<Range<f32>> for ValueRange {
    fn from(range: Range<f32>) -> Self {
        ValueRange {
            low: range.start,
            high: range.end,
        }
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
        let mut range = ValueRange::empty();

        assert!(!range.contains(2.0));
        assert!(!range.contains(0.0));

        range.extend(2.0);

        assert!(range.contains(2.0));
        assert_eq!(range.low, 2.0);
        assert_eq!(range.high, 2.0);
    }

    #[test]
    fn ranges_intersect() {
        let empty = ValueRange::empty();
        let r_low = ValueRange::from_samples(&[1u8, 6]);
        let r_mid = ValueRange::from_samples(&[3u8, 8]);
        let r_hi = ValueRange::from_samples(&[10u8, 30]);
        let inner = ValueRange::from_samples(&[10u8, 15]);
        let single = ValueRange::from_samples(&[6u8]);

        assert!(!empty.intersects(&r_low));
        assert!(!empty.intersects(&r_mid));
        assert!(!empty.intersects(&r_hi));
        assert!(!empty.intersects(&inner));
        assert!(!empty.intersects(&single));

        assert!(r_low.intersects(&r_mid));
        assert!(!r_low.intersects(&r_hi));
        assert!(!r_mid.intersects(&r_hi));

        assert!(r_low.intersects(&single));
        assert!(r_mid.intersects(&single));

        assert!(r_hi.intersects(&inner));
    }

    #[test]
    fn from_samples() {
        // Samples do not have to be floating point
        let samples = &[1u8, 2, 4, 10, 5, 0];

        let range = ValueRange::from_samples(samples.iter());

        assert_eq!(
            range,
            ValueRange {
                low: 0.0,
                high: 10.0
            }
        )
    }
}
