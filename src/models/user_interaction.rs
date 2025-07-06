
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

impl From<&str> for Sentiment {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "positive" => Sentiment::Positive,
            "negative" => Sentiment::Negative,
            "neutral" => Sentiment::Neutral,
            _ => panic!("Invalid sentiment value"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserInteraction {
    pub num_positive: usize,
    pub num_negative: usize,
    pub num_neutral: usize,
}

impl UserInteraction {
    pub fn increment_positive(&mut self) {
        self.num_positive += 1;
    }

    pub fn increment_negative(&mut self) {
        self.num_negative += 1;
    }

    pub fn increment_neutral(&mut self) {
        self.num_neutral += 1;
    }

    pub fn reset(&mut self) {
        self.num_positive = 0;
        self.num_negative = 0;
        self.num_neutral = 0;
    }
}