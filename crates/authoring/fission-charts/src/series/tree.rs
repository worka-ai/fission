use super::treemap::TreemapNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeSeries {
    pub name: String,
    pub data: Vec<TreemapNode>,
    pub radial: bool,
}

impl TreeSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            radial: false,
        }
    }

    pub fn data(mut self, data: Vec<TreemapNode>) -> Self {
        self.data = data;
        self
    }

    pub fn radial(mut self, radial: bool) -> Self {
        self.radial = radial;
        self
    }
}

impl Into<super::Series> for TreeSeries {
    fn into(self) -> super::Series {
        super::Series::Tree(self)
    }
}
