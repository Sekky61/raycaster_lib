pub struct ValueRange {
    low: f32,
    high: f32,
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

    pub fn from_iter(iter: impl Iterator<Item = f32>) -> ValueRange {
        todo!()
    }

    pub fn extend(&mut self, val: f32) {
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
}
