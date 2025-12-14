pub mod node_id;
pub mod op;
pub mod semantics; // New module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use node_id::NodeId;
pub use op::{Op, StructuralOp, LayoutOp, PaintOp, FlexDirection, Color};
pub use semantics::{Role, Semantics, ActionSet, ActionEntry}; // Added ActionEntry

pub const IR_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreNode {
    pub id: NodeId,
    pub op: Op,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CoreIR {
    pub nodes: HashMap<NodeId, CoreNode>,
    pub root: Option<NodeId>,
}

impl CoreIR {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: NodeId, op: Op, children: Vec<NodeId>) {
        // Store the node
        let mut core_node = CoreNode {
            id,
            op,
            children: children.clone(),
            parent: None, 
        };
        self.nodes.insert(id, core_node);

        // Update parent pointers in children
        for child_id in children {
            if let Some(child_node) = self.nodes.get_mut(&child_id) {
                child_node.parent = Some(id);
            }
        }
    }
    
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
    }
}
