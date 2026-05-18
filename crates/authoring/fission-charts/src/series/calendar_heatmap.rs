use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarHeatmapSeries {
    pub name: String,
    pub data: Vec<(String, f32)>,
    pub start: Option<String>,
    pub end: Option<String>,
}

impl CalendarHeatmapSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            start: None,
            end: None,
        }
    }

    pub fn data(mut self, data: Vec<(&str, f32)>) -> Self {
        self.data = data
            .into_iter()
            .map(|(date, value)| (date.into(), value))
            .collect();
        self
    }

    pub fn range(mut self, start: &str, end: &str) -> Self {
        self.start = Some(start.into());
        self.end = Some(end.into());
        self
    }
}

impl Into<super::Series> for CalendarHeatmapSeries {
    fn into(self) -> super::Series {
        super::Series::CalendarHeatmap(self)
    }
}
