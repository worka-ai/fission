use fission_ir::{NodeId, Op, CoreIR, LayoutOp};
use fission_layout::LayoutInputNode;
use std::fmt::Debug;

// Context passed down during the desugaring phase.
// It builds the CoreIR.
pub struct LoweringContext {
    pub next_node_id_seed: u128,
    pub ir: CoreIR,
}

impl LoweringContext {
    pub fn new() -> Self {
        LoweringContext {
            next_node_id_seed: 0,
            ir: CoreIR::new(),
        }
    }

    pub fn next_node_id(&mut self) -> NodeId {
        self.next_node_id_seed += 1;
        NodeId::derived(0, &[self.next_node_id_seed as u32])
    }

    pub fn add_node(&mut self, node_id: NodeId, op: Op, children: Vec<NodeId>) {
        self.ir.add_node(node_id, op, children);
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

// The trait that Authoring Widgets must implement to convert themselves into Core IR.
pub trait Desugar: Send + Sync + Debug {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId;
}

pub fn build_layout_tree(ir: &CoreIR) -> Vec<LayoutInputNode> {
    let mut input_nodes = Vec::new();
    
    // We need to traverse the CoreIR and produce LayoutInputNodes.
    // LayoutEngine currently expects a flat list but with parent_id info.
    // CoreIR has parent info if we set it, or we can infer from children.
    
    for (id, node) in &ir.nodes {
        // Only layout ops participate in layout tree? 
        // For MVP, assume all nodes map 1:1.
        // If Op is Structural, we might need to skip or handle differently?
        // Let's assume Structural ops (Group) are transparent or just layout boxes for now.
        
        let layout_op = match &node.op {
            Op::Layout(op) => op.clone(),
            Op::Structural(_) => LayoutOp::Box, // Treat groups as Boxes for now
            Op::Paint(_) => LayoutOp::Box, // Treat paint nodes as Boxes
        };

        input_nodes.push(LayoutInputNode {
            id: *id,
            parent_id: node.parent, // Parent might be None if not set
            op: layout_op,
            children_ids: node.children.clone(),
            debug_name: format!("{:?}", node.id),
        });
    }
    
    // Fix up parent pointers if they are missing in CoreIR but implied by children
    // (Our simple add_node implementation didn't set parent pointers)
    // A more efficient way is to do a traversal from root.
    
    // Let's do a quick pass to set parents based on children relationships
    // Since LayoutInputNode is being built, we can just build a map first.
    // But LayoutInputNode owns data.
    
    // For MVP, just returning the list is fine if LayoutEngine doesn't strictly require valid parent_ids for the dummy implementation.
    // The dummy engine iterated linearly.
    // But `08-layout-system.md` implies tree traversal.
    // Let's rely on `children_ids` which `LayoutEngine` uses?
    // Actually `LayoutEngine::compute_layout` in my dummy impl used `parent_children_map` derived from `parent_id`.
    // So I DO need to set `parent_id` correctly.
    
    // Let's populate parent_ids from children_ids.
    let mut parent_map = std::collections::HashMap::new();
    for (id, node) in &ir.nodes {
        for child in &node.children {
            parent_map.insert(*child, *id);
        }
    }
    
    for node in &mut input_nodes {
        if let Some(parent) = parent_map.get(&node.id) {
            node.parent_id = Some(*parent);
        }
    }

    input_nodes
}
