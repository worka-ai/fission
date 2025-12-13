use serde::{Deserialize, Serialize};
use crate::semantics::Semantics;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] 
pub enum Op {
    Structural(StructuralOp),
    Layout(LayoutOp),
    Paint(PaintOp),
    Semantics(Semantics), // Added Semantics variant
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructuralOp {
    Group,
    Scope,
    Fragment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] 
pub enum LayoutOp {
    Box { 
        width: Option<f32>,
        height: Option<f32>,
    },
    Flex { 
        direction: FlexDirection,
        flex_grow: f32,
        flex_shrink: f32,
    },
    Grid,
    Stack,
    Align,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaintOp {
    DrawRect,
    DrawText,
    DrawImage,
    DrawPath,
    PaintGroup,
}