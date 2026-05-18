use fission_core::op::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkPoint {
    pub name: String,
    pub x: Option<f32>,
    pub y: f32,
    pub color: Color,
}

impl MarkPoint {
    pub fn y(name: impl Into<String>, y: f32) -> Self {
        Self {
            name: name.into(),
            x: None,
            y,
            color: Color::RED,
        }
    }

    pub fn xy(name: impl Into<String>, x: f32, y: f32) -> Self {
        Self {
            name: name.into(),
            x: Some(x),
            y,
            color: Color::RED,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkLine {
    pub name: String,
    pub y: f32,
    pub color: Color,
    pub width: f32,
}

impl MarkLine {
    pub fn y(name: impl Into<String>, y: f32) -> Self {
        Self {
            name: name.into(),
            y,
            color: Color::RED,
            width: 1.5,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkArea {
    pub name: String,
    pub y_min: f32,
    pub y_max: f32,
    pub color: Color,
}

impl MarkArea {
    pub fn y_range(name: impl Into<String>, y_min: f32, y_max: f32) -> Self {
        Self {
            name: name.into(),
            y_min,
            y_max,
            color: Color {
                r: 250,
                g: 204,
                b: 21,
                a: 50,
            },
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}
