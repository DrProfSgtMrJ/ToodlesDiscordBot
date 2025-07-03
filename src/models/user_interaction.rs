
#[derive(Debug, Clone, Default)]
pub struct UserInteraction {
    pub num_positive: usize,
    pub num_negative: usize,
}

impl UserInteraction {
    pub fn increment_positive(&mut self) {
        self.num_positive += 1;
    }

    pub fn increment_negative(&mut self) {
        self.num_negative += 1;
    }

    pub fn reset(&mut self) {
        self.num_positive = 0;
        self.num_negative = 0;
    }
}