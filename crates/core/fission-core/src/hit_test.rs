use fission_ir::{CoreIR, NodeId};
use fission_layout::{LayoutSnapshot, LayoutPoint};

pub fn hit_test(
    ir: &CoreIR,
    layout: &LayoutSnapshot,
    point: LayoutPoint,
) -> Option<NodeId> {
    // We need to find the topmost node that contains the point.
    // Assuming standard paint order (pre-order traversal), the last node in traversal 
    // that contains the point is the one on top (if we ignore explicit z-index for now).
    
    // However, hit testing usually proceeds in reverse paint order (front-to-back)
    // to find the first match.
    
    // Since we don't have a reverse iterator for the tree handy without recursion,
    // let's traverse standard order and keep the last match.
    
    let mut last_hit: Option<NodeId> = None;
    
    if let Some(root) = ir.root {
        hit_test_recursive(root, ir, layout, point, &mut last_hit);
    }
    
    last_hit
}

fn hit_test_recursive(
    node_id: NodeId,
    ir: &CoreIR,
    layout: &LayoutSnapshot,
    point: LayoutPoint,
    last_hit: &mut Option<NodeId>,
) {
    // 1. Check if point is inside this node's rect
    // My LayoutEngine implementation stored: `geometry.rect.origin = offset;` where offset was absolute accumulation.
    // So `rect` in snapshot is in Global coordinates (relative to root).
    // This is correct for global hit testing.
    
    let is_hit = if let Some(geom) = layout.get_node_geometry(node_id) {
        geom.rect.contains(point)
    } else {
        false
    };

    if is_hit {
        *last_hit = Some(node_id);
    }

    // 2. Recurse into children
    if let Some(node) = ir.nodes.get(&node_id) {
        // Children are typically drawn on top of parents, so recurse after checking parent.
        // This ensures the deepest, thus top-most, hit is found.
        for child_id in &node.children {
            hit_test_recursive(*child_id, ir, layout, point, last_hit);
        }
    }
}