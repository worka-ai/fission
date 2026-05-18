use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartTimeline {
    pub labels: Vec<String>,
    pub current_index: usize,
    pub play_interval_ms: Option<u64>,
}

impl ChartTimeline {
    pub fn new(labels: Vec<impl Into<String>>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            current_index: 0,
            play_interval_ms: None,
        }
    }

    pub fn current_index(mut self, index: usize) -> Self {
        self.current_index = index;
        self
    }

    pub fn play_interval_ms(mut self, interval: u64) -> Self {
        self.play_interval_ms = Some(interval);
        self
    }
}
