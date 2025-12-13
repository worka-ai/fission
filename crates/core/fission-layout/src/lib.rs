use fission_ir::{NodeId, LayoutOp};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type LayoutUnit = f32; // Layout operates in logical units, often floats

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutPoint {
    pub x: LayoutUnit,
    pub y: LayoutUnit,
}

impl LayoutPoint {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutSize {
    pub width: LayoutUnit,
    pub height: LayoutUnit,
}

impl LayoutSize {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };
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

    pub fn x(&self) -> LayoutUnit { self.origin.x }
    pub fn y(&self) -> LayoutUnit { self.origin.y }
    pub fn width(&self) -> LayoutUnit { self.size.width }
    pub fn height(&self) -> LayoutUnit { self.size.height }

    pub fn right(&self) -> LayoutUnit { self.origin.x + self.size.width }
    pub fn bottom(&self) -> LayoutUnit { self.origin.y + self.size.height }
}

// Represents the computed layout geometry for a single node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeGeometry {
    pub rect: LayoutRect,
    // Add more geometry fields as defined in 08-5 (baselines, paint_bounds, clip_bounds)
    // For MVP, just the rect.
}

// The immutable, canonical record of all computed layout geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    pub nodes: HashMap<NodeId, LayoutNodeGeometry>,
    pub viewport_size: LayoutSize,
    // Add version, rounding policy, etc. later.
}

impl LayoutSnapshot {
    pub fn new(viewport_size: LayoutSize) -> Self {
        Self { nodes: HashMap::new(), viewport_size }
    }

    pub fn get_node_geometry(&self, node_id: NodeId) -> Option<&LayoutNodeGeometry> {
        self.nodes.get(&node_id)
    }

    pub fn get_node_rect(&self, node_id: NodeId) -> Option<LayoutRect> {
        self.nodes.get(&node_id).map(|g| g.rect)
    }
}

// Dummy structure representing a node that the layout engine needs to process.
// In reality, this would come from a Core IR tree.
#[derive(Debug, Clone)]
pub struct LayoutInputNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub op: LayoutOp,
    pub children_ids: Vec<NodeId>,
    pub debug_name: String, // For easier debugging
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self { Self }

    // This is a simplified layout function for now.
    // It takes a flat list of nodes and processes them.
    // A real layout engine would traverse a tree structure.
    pub fn compute_layout(
        &self,
        input_nodes: &[LayoutInputNode],
        viewport_size: LayoutSize,
    ) -> Result<LayoutSnapshot> {
        let mut snapshot = LayoutSnapshot::new(viewport_size);
        let mut node_geometries = HashMap::new();

        // For MVP, just a very simple linear layout or fixed positioning based on ops.
        // This will be replaced by actual constraint solving later.
        let mut current_y = 0.0;
        let mut parent_children_map: HashMap<NodeId, Vec<&LayoutInputNode>> = HashMap::new();
        for node in input_nodes {
            if let Some(parent_id) = node.parent_id {
                parent_children_map.entry(parent_id).or_default().push(node);
            }
        }

        // A very rudimentary tree traversal (depth-first for now)
        let mut stack: Vec<(&LayoutInputNode, LayoutPoint)> = Vec::new();
        // Find root nodes (no parent_id or parent not in map) - simplified for tests
        // For this first test, we'll just process the input nodes linearly as if they are children of a single root

        // Process nodes linearly for now, assuming they are direct children under the viewport
        for node in input_nodes {
            let rect = match node.op {
                LayoutOp::Box => {
                    // Dummy fixed size for a Box
                    LayoutRect::new(10.0, current_y + 10.0, 100.0, 50.0)
                }
                LayoutOp::Flex => {
                    // Dummy flex item, will expand later
                    LayoutRect::new(10.0, current_y + 10.0, 150.0, 80.0)
                }
                _ => LayoutRect::new(0.0, 0.0, 0.0, 0.0), // Placeholder for other ops
            };
            node_geometries.insert(node.id, LayoutNodeGeometry { rect });
            current_y = rect.bottom();
        }

        snapshot.nodes = node_geometries;
        Ok(snapshot)
    }
}
