use fission_ir::{CoreIR, NodeId};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct FrameDiff {
    pub dirty_structural: HashSet<NodeId>,
}

pub fn diff_ir(prev: &CoreIR, next: &CoreIR) -> FrameDiff {
    let mut diff = FrameDiff::default();

    for (id, next_node) in &next.nodes {
        if let Some(prev_node) = prev.nodes.get(id) {
            if prev_node.hash != next_node.hash {
                diff.dirty_structural.insert(*id);
            }
        } else {
            diff.dirty_structural.insert(*id);
        }
    }

    diff
}
