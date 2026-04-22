use serde::{Deserialize, Serialize};
use fission_core::op::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarSeries {
    pub name: String,
    pub data: Vec<f32>,
    pub color: Color,
}

impl BarSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            color: Color::BLUE,
        }
    }
    
    pub fn data(mut self, data: Vec<f32>) -> Self {
        self.data = data;
        self
    }
    
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Into<super::Series> for BarSeries {
    fn into(self) -> super::Series {
        super::Series::Bar(self)
    }
}
