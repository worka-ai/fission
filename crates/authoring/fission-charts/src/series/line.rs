use serde::{Deserialize, Serialize};
use fission_core::op::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineSeries {
    pub name: String,
    pub data: Vec<f32>,
    pub smooth: bool,
    pub color: Color,
}

impl LineSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            smooth: false,
            color: Color::BLUE,
        }
    }
    
    pub fn data(mut self, data: Vec<f32>) -> Self {
        self.data = data;
        self
    }
    
    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = smooth;
        self
    }
    
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Into<super::Series> for LineSeries {
    fn into(self) -> super::Series {
        super::Series::Line(self)
    }
}
