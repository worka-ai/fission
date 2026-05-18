use fission_core::op::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartGraphicKind {
    Rect,
    Circle,
    Text,
    Line,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartGraphic {
    pub kind: ChartGraphicKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: Option<String>,
    pub color: Color,
    pub stroke: Option<Color>,
}

impl ChartGraphic {
    pub fn rect(x: f32, y: f32, width: f32, height: f32, color: Color) -> Self {
        Self {
            kind: ChartGraphicKind::Rect,
            x,
            y,
            width,
            height,
            text: None,
            color,
            stroke: None,
        }
    }

    pub fn circle(x: f32, y: f32, radius: f32, color: Color) -> Self {
        Self {
            kind: ChartGraphicKind::Circle,
            x,
            y,
            width: radius * 2.0,
            height: radius * 2.0,
            text: None,
            color,
            stroke: None,
        }
    }

    pub fn text(x: f32, y: f32, text: impl Into<String>, color: Color) -> Self {
        Self {
            kind: ChartGraphicKind::Text,
            x,
            y,
            width: 220.0,
            height: 20.0,
            text: Some(text.into()),
            color,
            stroke: None,
        }
    }

    pub fn line(x: f32, y: f32, width: f32, height: f32, color: Color) -> Self {
        Self {
            kind: ChartGraphicKind::Line,
            x,
            y,
            width,
            height,
            text: None,
            color,
            stroke: None,
        }
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }
}
