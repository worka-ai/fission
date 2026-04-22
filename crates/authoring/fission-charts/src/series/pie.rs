use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieSeries {
    pub name: String,
    pub data: Vec<(String, f32)>, // Label, value
}

impl PieSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
        }
    }
    
    pub fn data(mut self, data: Vec<(&str, f32)>) -> Self {
        self.data = data.into_iter().map(|(l, v)| (l.into(), v)).collect();
        self
    }
}

impl Into<super::Series> for PieSeries {
    fn into(self) -> super::Series {
        super::Series::Pie(self)
    }
}
