use anyhow::Result;
use fission_ir::{FlexDirection as IrFlexDirection, NodeId, Op, PaintOp};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use taffy::node::Node as TaffyNodeId;
use taffy::prelude::*;

pub use fission_ir::{FlexDirection, LayoutOp};

pub type LayoutUnit = f32;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct LayoutPoint {
    pub x: LayoutUnit,
    pub y: LayoutUnit,
}

impl LayoutPoint {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub fn new(x: LayoutUnit, y: LayoutUnit) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct LayoutSize {
    pub width: LayoutUnit,
    pub height: LayoutUnit,
}

impl LayoutSize {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };
    pub fn new(width: LayoutUnit, height: LayoutUnit) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutRect {
    pub origin: LayoutPoint,
    pub size: LayoutSize,
}

impl LayoutRect {
    pub fn new(x: LayoutUnit, y: LayoutUnit, width: LayoutUnit, height: LayoutUnit) -> Self {
        Self {
            origin: LayoutPoint { x, y },
            size: LayoutSize { width, height },
        }
    }

    pub fn x(&self) -> LayoutUnit {
        self.origin.x
    }
    pub fn y(&self) -> LayoutUnit {
        self.origin.y
    }
    pub fn width(&self) -> LayoutUnit {
        self.size.width
    }
    pub fn height(&self) -> LayoutUnit {
        self.size.height
    }

    pub fn right(&self) -> LayoutUnit {
        self.origin.x + self.size.width
    }
    pub fn bottom(&self) -> LayoutUnit {
        self.origin.y + self.size.height
    }

    pub fn contains(&self, p: LayoutPoint) -> bool {
        p.x >= self.x() && p.x < self.right() && p.y >= self.y() && p.y < self.bottom()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeGeometry {
    pub rect: LayoutRect,
    pub content_size: LayoutSize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LayoutSnapshot {
    pub nodes: HashMap<NodeId, LayoutNodeGeometry>,
    pub viewport_size: LayoutSize,
}

impl LayoutSnapshot {
    pub fn new(viewport_size: LayoutSize) -> Self {
        Self {
            nodes: HashMap::new(),
            viewport_size,
        }
    }

    pub fn get_node_geometry(&self, node_id: NodeId) -> Option<&LayoutNodeGeometry> {
        self.nodes.get(&node_id)
    }

    pub fn get_node_rect(&self, node_id: NodeId) -> Option<LayoutRect> {
        self.nodes.get(&node_id).map(|g| g.rect)
    }
}

#[derive(Debug, Clone)]
pub struct LayoutInputNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub op: LayoutOp,
    pub children_ids: Vec<NodeId>,
    pub debug_name: String,
    pub width: Option<LayoutUnit>,
    pub height: Option<LayoutUnit>,
    pub flex_grow: LayoutUnit,
    pub flex_shrink: LayoutUnit,
    pub text_content: Option<String>,
    pub font_size: Option<f32>,
}

pub trait TextMeasurer: Send + Sync {
    fn measure(&self, text: &str, font_size: f32, available_width: Option<f32>) -> (f32, f32);
}

pub struct LayoutEngine {
    measurer: Option<Arc<dyn TextMeasurer>>,
    taffy: Taffy,
    taffy_map: HashMap<NodeId, TaffyNodeId>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            measurer: None,
            taffy: Taffy::new(),
            taffy_map: HashMap::new(),
        }
    }

    pub fn with_measurer(mut self, measurer: Arc<dyn TextMeasurer>) -> Self {
        self.measurer = Some(measurer);
        self
    }

    pub fn update(&mut self, input_nodes: &[LayoutInputNode], dirty_set: &HashSet<NodeId>) {
        let node_map: HashMap<NodeId, &LayoutInputNode> =
            input_nodes.iter().map(|n| (n.id, n)).collect();

        // 1. Cleanup removed nodes
        let mut to_remove = Vec::new();
        for id in self.taffy_map.keys() {
            if !node_map.contains_key(id) {
                to_remove.push(*id);
            }
        }
        for id in to_remove {
            let t_id = self.taffy_map.remove(&id).unwrap();
            self.taffy.remove(t_id).ok();
        }

        let mut ensure_exists = |id: NodeId| {
            if !self.taffy_map.contains_key(&id) {
                let t_id = self.taffy.new_leaf(Style::default()).unwrap();
                self.taffy_map.insert(id, t_id);
            }
        };

        for id in dirty_set {
            if node_map.contains_key(id) {
                ensure_exists(*id);
            }
        }

        // 3. Update properties and children
        for id in dirty_set {
            if let Some(node) = node_map.get(id) {
                let t_id = *self.taffy_map.get(id).unwrap();

                // Style
                let style = self.compute_style(node);
                self.taffy.set_style(t_id, style).unwrap();

                // Measure
                if let (Some(text), Some(size), Some(measurer)) =
                    (&node.text_content, node.font_size, &self.measurer)
                {
                    let text = text.clone();
                    let font_size = size;
                    let measurer = measurer.clone();

                    self.taffy
                        .set_measure(
                            t_id,
                            Some(taffy::node::MeasureFunc::Boxed(Box::new(
                                move |_known_dims, available_space| {
                                    let avail_width = match available_space.width {
                                        AvailableSpace::Definite(w) => Some(w),
                                        AvailableSpace::MaxContent => None,
                                        AvailableSpace::MinContent => Some(0.0),
                                    };
                                    let (w, h) = measurer.measure(&text, font_size, avail_width);
                                    taffy::geometry::Size {
                                        width: w,
                                        height: h,
                                    }
                                },
                            ))),
                        )
                        .unwrap();
                } else {
                    self.taffy.set_measure(t_id, None).unwrap();
                }

                // Children
                let mut child_t_ids = Vec::new();
                for cid in &node.children_ids {
                    if !self.taffy_map.contains_key(cid) {
                        let t_id = self.taffy.new_leaf(Style::default()).unwrap();
                        self.taffy_map.insert(*cid, t_id);
                    }
                    child_t_ids.push(*self.taffy_map.get(cid).unwrap());
                }

                self.taffy.set_children(t_id, &child_t_ids).unwrap();
            }
        }
    }

    fn compute_style(&self, node: &LayoutInputNode) -> Style {
        let mut style = Style::default();
        style.flex_grow = node.flex_grow;
        style.flex_shrink = node.flex_shrink;

        match &node.op {
            LayoutOp::Box {
                width,
                height,
                padding,
            } => {
                style.display = Display::Flex;
                style.align_items = Some(AlignItems::Center);
                style.justify_content = Some(JustifyContent::Center);
                style.padding = taffy::geometry::Rect {
                    left: points(padding[0]),
                    right: points(padding[1]),
                    top: points(padding[2]),
                    bottom: points(padding[3]),
                };
                style.size = taffy::geometry::Size {
                    width: width.map(Dimension::Points).unwrap_or(Dimension::Auto),
                    height: height.map(Dimension::Points).unwrap_or(Dimension::Auto),
                };
            }
            LayoutOp::Flex {
                direction, padding, ..
            } => {
                style.display = Display::Flex;
                style.flex_direction = match direction {
                    IrFlexDirection::Row => taffy::style::FlexDirection::Row,
                    IrFlexDirection::Column => taffy::style::FlexDirection::Column,
                };
                style.align_items = Some(AlignItems::Center);
                style.padding = taffy::geometry::Rect {
                    left: points(padding[0]),
                    right: points(padding[1]),
                    top: points(padding[2]),
                    bottom: points(padding[3]),
                };
                style.size = taffy::geometry::Size {
                    width: node.width.map(Dimension::Points).unwrap_or(Dimension::Auto),
                    height: node
                        .height
                        .map(Dimension::Points)
                        .unwrap_or(Dimension::Auto),
                };
            }
            LayoutOp::Scroll {
                direction, padding, ..
            } => {
                style.display = Display::Flex;
                style.flex_direction = match direction {
                    IrFlexDirection::Row => taffy::style::FlexDirection::Row,
                    IrFlexDirection::Column => taffy::style::FlexDirection::Column,
                };
                style.padding = taffy::geometry::Rect {
                    left: points(padding[0]),
                    right: points(padding[1]),
                    top: points(padding[2]),
                    bottom: points(padding[3]),
                };
                style.size = taffy::geometry::Size {
                    width: Dimension::Auto,
                    height: Dimension::Auto,
                };
            }
            LayoutOp::Embed { .. } => {
                style.display = Display::Flex;
                style.size = taffy::geometry::Size {
                    width: node.width.map(Dimension::Points).unwrap_or(Dimension::Auto),
                    height: node
                        .height
                        .map(Dimension::Points)
                        .unwrap_or(Dimension::Auto),
                };
            }
            LayoutOp::AbsoluteFill => {
                style.display = Display::Flex;
                style.position = Position::Absolute;
                style.inset = taffy::geometry::Rect {
                    left: points(0.0),
                    right: points(0.0),
                    top: points(0.0),
                    bottom: points(0.0),
                };
                style.size = taffy::geometry::Size {
                    width: Dimension::Auto,
                    height: Dimension::Auto,
                };
            }
            _ => {
                style.display = Display::Flex;
            }
        }
        style
    }

    pub fn compute_layout(
        &mut self,
        input_nodes: &[LayoutInputNode],
        root_node_id: NodeId,
        viewport_size: LayoutSize,
    ) -> Result<LayoutSnapshot> {
        let node_map: HashMap<NodeId, &LayoutInputNode> =
            input_nodes.iter().map(|n| (n.id, n)).collect();

        if let Some(root_taffy_id) = self.taffy_map.get(&root_node_id) {
            self.taffy
                .compute_layout(
                    *root_taffy_id,
                    taffy::geometry::Size {
                        width: taffy::style::AvailableSpace::Definite(viewport_size.width),
                        height: taffy::style::AvailableSpace::Definite(viewport_size.height),
                    },
                )
                .map_err(|e| anyhow::anyhow!("Taffy layout error: {:?}", e))?;

            let mut geometries = HashMap::new();
            self.extract_geometry_recursive(
                root_node_id,
                LayoutPoint::ZERO,
                &node_map,
                &mut geometries,
            );

            let mut snapshot = LayoutSnapshot::new(viewport_size);
            snapshot.nodes = geometries;
            Ok(snapshot)
        } else {
            Err(anyhow::anyhow!(
                "Root node layout not found. Did you call update()?"
            ))
        }
    }

    fn extract_geometry_recursive(
        &self,
        node_id: NodeId,
        parent_absolute_pos: LayoutPoint,
        node_map: &HashMap<NodeId, &LayoutInputNode>,
        geometries: &mut HashMap<NodeId, LayoutNodeGeometry>,
    ) {
        let taffy_id = self.taffy_map.get(&node_id).unwrap();
        let layout = self.taffy.layout(*taffy_id).unwrap();
        let node = node_map.get(&node_id).unwrap();

        let absolute_x = parent_absolute_pos.x + layout.location.x;
        let absolute_y = parent_absolute_pos.y + layout.location.y;

        let mut width = layout.size.width;
        let mut height = layout.size.height;
        let content_width = width;
        let content_height = height;

        if let LayoutOp::Scroll { .. } = &node.op {
            if let Some(w) = node.width {
                width = w;
            }
            if let Some(h) = node.height {
                height = h;
            }
        }

        let rect = LayoutRect::new(absolute_x, absolute_y, width, height);
        let content_size = LayoutSize::new(content_width, content_height);

        geometries.insert(node_id, LayoutNodeGeometry { rect, content_size });

        for child_id in &node.children_ids {
            self.extract_geometry_recursive(
                *child_id,
                LayoutPoint::new(absolute_x, absolute_y),
                node_map,
                geometries,
            );
        }
    }
}
