use fission_core::op::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleAxisSeries {
    pub name: String,
    pub data: Vec<(f32, f32)>,
    pub color: Color,
}

impl SingleAxisSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            color: Color::BLUE,
        }
    }

    pub fn data(mut self, data: Vec<(f32, f32)>) -> Self {
        self.data = data;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Into<super::Series> for SingleAxisSeries {
    fn into(self) -> super::Series {
        super::Series::SingleAxis(self)
    }
}
