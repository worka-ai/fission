pub mod node_id;
pub mod op;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use node_id::NodeId;
pub use op::{Op, StructuralOp, LayoutOp, PaintOp};

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
        // We'll fix up parents later or allow caller to set them
        let node = CoreNode {
            id,
            op,
            children: children.clone(),
            parent: None, 
        };
        self.nodes.insert(id, node);
        
        // Fix up parent pointers for children (if they exist)
        // Note: this assumes children are already added or will be updated. 
        // Ideally we do a separate pass or require adding leaves first?
        // Actually, just storing children IDs is enough for top-down traversal. 
        // Parent pointers are useful for bottom-up but maybe not strictly required for initial build.
    }
    
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
    }
}