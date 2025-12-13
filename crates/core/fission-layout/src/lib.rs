use fission_ir::{NodeId, Op};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use fission_ir::{LayoutOp, FlexDirection}; // Make LayoutOp and FlexDirection public

pub type LayoutUnit = f32; // Layout operates in logical units, often floats

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)] // Derive Default here
pub struct LayoutPoint {
    pub x: LayoutUnit,
    pub y: LayoutUnit,
}

impl LayoutPoint {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub fn new(x: LayoutUnit, y: LayoutUnit) -> Self { Self { x, y } }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)] // Derive Default here
pub struct LayoutSize {
    pub width: LayoutUnit,
    pub height: LayoutUnit,
}

impl LayoutSize {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };
    pub fn new(width: LayoutUnit, height: LayoutUnit) -> Self { Self { width, height } }
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

    pub fn contains(&self, p: LayoutPoint) -> bool {
        p.x >= self.x() && p.x < self.right() &&
        p.y >= self.y() && p.y < self.bottom()
    }
}

// Represents the computed layout geometry for a single node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeGeometry {
    pub rect: LayoutRect,
    // Add more geometry fields as defined in 08-5 (baselines, paint_bounds, clip_bounds)
    // For MVP, just the rect.
}

// The immutable, canonical record of all computed layout geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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

// Constraints for a node during layout
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutConstraints {
    pub min_width: LayoutUnit,
    pub max_width: LayoutUnit,
    pub min_height: LayoutUnit,
    pub max_height: LayoutUnit,
}

impl LayoutConstraints {
    pub fn new(min_width: LayoutUnit, max_width: LayoutUnit, min_height: LayoutUnit, max_height: LayoutUnit) -> Self {
        Self { min_width, max_width, min_height, max_height }
    }

    pub fn tight_for_size(size: LayoutSize) -> Self {
        Self { min_width: size.width, max_width: size.width, min_height: size.height, max_height: size.height }
    }

    pub fn loosen(&self) -> Self {
        Self { min_width: 0.0, max_width: self.max_width, min_height: 0.0, max_height: self.max_height }
    }

    pub fn clamp_width(&self, width: LayoutUnit) -> LayoutUnit {
        width.max(self.min_width).min(self.max_width)
    }

    pub fn clamp_height(&self, height: LayoutUnit) -> LayoutUnit {
        height.max(self.min_height).min(self.max_height)
    }

    pub fn clamp_size(&self, size: LayoutSize) -> LayoutSize {
        LayoutSize { 
            width: self.clamp_width(size.width), 
            height: self.clamp_height(size.height) 
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutInputNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub op: LayoutOp, // Now holds the LayoutOp enum
    pub children_ids: Vec<NodeId>,
    pub debug_name: String, // For easier debugging
    // Layout properties extracted from Op
    pub width: Option<LayoutUnit>,
    pub height: Option<LayoutUnit>,
    pub flex_grow: LayoutUnit,
    pub flex_shrink: LayoutUnit,
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self { Self }

    // The main layout computation entry point.
    // It takes a list of nodes and their relationships, applies constraints,
    // and produces a LayoutSnapshot.
    pub fn compute_layout(
        &self,
        input_nodes: &[LayoutInputNode],
        viewport_size: LayoutSize,
    ) -> Result<LayoutSnapshot> {
        let mut snapshot = LayoutSnapshot::new(viewport_size);
        let mut node_geometries = HashMap::new();

        let node_map: HashMap<NodeId, &LayoutInputNode> = input_nodes.iter().map(|n| (n.id, n)).collect();

        // Find the root nodes (those with no parent in the input, or not found in parent_map)
        let root_node_id = input_nodes.first().map(|n| n.id).ok_or_else(|| anyhow::anyhow!("No root node provided for layout."))?;
        
        let root_constraints = LayoutConstraints::new(0.0, viewport_size.width, 0.0, viewport_size.height);

        let _ = self.layout_node(
            root_node_id,
            LayoutPoint::ZERO, // Root is at (0,0) absolute
            root_constraints,
            &node_map,
            &mut node_geometries,
        )?;
        
        snapshot.nodes = node_geometries;
        Ok(snapshot)
    }

    // Recursive layout function for a single node.
    fn layout_node(
        &self,
        node_id: NodeId,
        offset: LayoutPoint, // Absolute position accumulated from parent
        constraints: LayoutConstraints,
        node_map: &HashMap<NodeId, &LayoutInputNode>,
        node_geometries: &mut HashMap<NodeId, LayoutNodeGeometry>,
    ) -> Result<LayoutSize> {
        let node = node_map.get(&node_id)
            .ok_or_else(|| anyhow::anyhow!("Node not found in map during layout: {:?}", node_id))?;

        let mut final_size = LayoutSize::ZERO;

        match &node.op {
            LayoutOp::Box { width: op_width, height: op_height } => {
                let desired_width = op_width.unwrap_or(constraints.max_width);
                let desired_height = op_height.unwrap_or(constraints.max_height);

                final_size = LayoutSize {
                    width: constraints.clamp_width(desired_width),
                    height: constraints.clamp_height(desired_height),
                };

                // For a Box, children are positioned inside it. Accumulate offset for children.
                for &child_id in &node.children_ids {
                    let _child_size = self.layout_node(child_id, offset, constraints.loosen(), node_map, node_geometries)?;
                }
            }
            LayoutOp::Flex { direction, flex_grow: _, flex_shrink: _ } => {
                let mut children_initial_sizes_and_nodes = Vec::new();

                // First pass: measure children with loose constraints
                for &child_id in &node.children_ids {
                    let child_node = node_map.get(&child_id).unwrap();
                    // For Flex, children get loose constraints initially on the main axis
                    // For MVP, just assume children are also Boxes for now
                    let child_constraints = match direction {
                        FlexDirection::Row => LayoutConstraints::new(0.0, constraints.max_width, 0.0, constraints.max_height),
                        FlexDirection::Column => LayoutConstraints::new(0.0, constraints.max_width, 0.0, constraints.max_height),
                    };
                    let child_size = self.layout_node(child_id, LayoutPoint::ZERO, child_constraints, node_map, node_geometries)?;
                    children_initial_sizes_and_nodes.push((child_id, child_size, child_node));
                }

                let mut total_fixed_main_size = 0.0;
                let mut total_flex_grow_factor = 0.0;
                let mut total_flex_shrink_factor = 0.0;

                // Calculate initial main size and flex factors
                for (_, child_size, child_node) in &children_initial_sizes_and_nodes {
                    let main_size = match direction {
                        FlexDirection::Row => child_size.width,
                        FlexDirection::Column => child_size.height,
                    };
                    total_fixed_main_size += main_size;
                    total_flex_grow_factor += child_node.flex_grow;
                    total_flex_shrink_factor += child_node.flex_shrink;
                }

                let remaining_main_space = match direction {
                    FlexDirection::Row => constraints.max_width - total_fixed_main_size,
                    FlexDirection::Column => constraints.max_height - total_fixed_main_size,
                };

                let mut current_main_pos = 0.0;
                let mut final_cross_size: LayoutUnit = 0.0;

                // Second pass: distribute remaining space and position children
                for (child_id, child_initial_size, child_node) in children_initial_sizes_and_nodes {
                    let mut child_final_main_size = match direction {
                        FlexDirection::Row => child_initial_size.width,
                        FlexDirection::Column => child_initial_size.height,
                    };
                    let mut child_final_cross_size = match direction {
                        FlexDirection::Row => child_initial_size.height,
                        FlexDirection::Column => child_initial_size.width,
                    };

                    if remaining_main_space > 0.0 && child_node.flex_grow > 0.0 && total_flex_grow_factor > 0.0 {
                        child_final_main_size += (remaining_main_space * child_node.flex_grow) / total_flex_grow_factor;
                    } else if remaining_main_space < 0.0 && child_node.flex_shrink > 0.0 && total_flex_shrink_factor > 0.0 {
                        child_final_main_size += (remaining_main_space * child_node.flex_shrink) / total_flex_shrink_factor;
                    }
                    
                    // Position child relative to parent's origin
                    let child_offset = match direction {
                        FlexDirection::Row => LayoutPoint::new(offset.x + current_main_pos, offset.y),
                        FlexDirection::Column => LayoutPoint::new(offset.x, offset.y + current_main_pos),
                    };

                    let child_geometry = node_geometries.entry(child_id).or_insert(LayoutNodeGeometry { rect: LayoutRect::new(0.0, 0.0, 0.0, 0.0) });
                    match direction {
                        FlexDirection::Row => {
                            child_geometry.rect.origin = child_offset;
                            child_geometry.rect.size.width = child_final_main_size;
                            child_geometry.rect.size.height = child_final_cross_size;
                            final_cross_size = final_cross_size.max(child_final_cross_size);
                        }
                        FlexDirection::Column => {
                            child_geometry.rect.origin = child_offset;
                            child_geometry.rect.size.width = child_final_cross_size;
                            child_geometry.rect.size.height = child_final_main_size;
                            final_cross_size = final_cross_size.max(child_final_cross_size);
                        }
                    }

                    current_main_pos += child_final_main_size;
                }

                final_size = match direction {
                    FlexDirection::Row => LayoutSize {
                        width: constraints.clamp_width(current_main_pos),
                        height: constraints.clamp_height(final_cross_size),
                    },
                    FlexDirection::Column => LayoutSize {
                        width: constraints.clamp_width(final_cross_size),
                        height: constraints.clamp_height(current_main_pos),
                    },
                };
            }
            LayoutOp::Grid => {
                // Placeholder for Grid layout
                final_size = LayoutSize {
                    width: constraints.clamp_width(node.width.unwrap_or(constraints.max_width)),
                    height: constraints.clamp_height(node.height.unwrap_or(constraints.max_height)),
                };
                 for &child_id in &node.children_ids {
                    let _child_size = self.layout_node(child_id, offset, constraints.loosen(), node_map, node_geometries)?;
                }
            }
            LayoutOp::Stack => {
                // Placeholder for Stack layout
                final_size = LayoutSize {
                    width: constraints.clamp_width(node.width.unwrap_or(constraints.max_width)),
                    height: constraints.clamp_height(node.height.unwrap_or(constraints.max_height)),
                };
                 for &child_id in &node.children_ids {
                    let _child_size = self.layout_node(child_id, offset, constraints.loosen(), node_map, node_geometries)?;
                }
            }
            LayoutOp::Align => {
                // Placeholder for Align layout
                final_size = LayoutSize {
                    width: constraints.clamp_width(node.width.unwrap_or(constraints.max_width)),
                    height: constraints.clamp_height(node.height.unwrap_or(constraints.max_height)),
                };
                 for &child_id in &node.children_ids {
                    let _child_size = self.layout_node(child_id, offset, constraints.loosen(), node_map, node_geometries)?;
                }
            }
        }

        // Store this node's final size and position
        let geometry = node_geometries.entry(node_id).or_insert(LayoutNodeGeometry { rect: LayoutRect::new(0.0, 0.0, 0.0, 0.0) });
        geometry.rect.origin = offset;
        geometry.rect.size = final_size;

        Ok(final_size)
    }
}
