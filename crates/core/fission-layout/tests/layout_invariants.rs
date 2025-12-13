use anyhow::Result;
use fission_ir::NodeId;
use fission_layout::{
    LayoutEngine, LayoutInputNode, LayoutPoint, LayoutRect, LayoutSize, LayoutSnapshot,
};
use fission_ir::LayoutOp;

#[test]
fn test_layout_rect_properties() {
    let rect = LayoutRect::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(rect.x(), 10.0);
    assert_eq!(rect.y(), 20.0);
    assert_eq!(rect.width(), 100.0);
    assert_eq!(rect.height(), 50.0);
    assert_eq!(rect.right(), 110.0);
    assert_eq!(rect.bottom(), 70.0);
}

#[test]
fn test_simple_box_layout() -> Result<()> {
    let engine = LayoutEngine::new();
    let viewport_size = LayoutSize { width: 800.0, height: 600.0 };

    let node_id_1 = NodeId::derived(0, &[1]);
    let node_1 = LayoutInputNode {
        id: node_id_1,
        parent_id: None,
        op: LayoutOp::Box,
        children_ids: vec![],
        debug_name: "box1".into(),
    };

    let node_id_2 = NodeId::derived(0, &[2]);
    let node_2 = LayoutInputNode {
        id: node_id_2,
        parent_id: Some(node_id_1),
        op: LayoutOp::Box,
        children_ids: vec![],
        debug_name: "box2".into(),
    };

    let input_nodes = vec![node_1, node_2];
    let snapshot = engine.compute_layout(&input_nodes, viewport_size)?;

    let geom_1 = snapshot.get_node_geometry(node_id_1).expect("Node 1 geometry missing");
    assert_eq!(geom_1.rect, LayoutRect::new(10.0, 10.0, 100.0, 50.0));

    let geom_2 = snapshot.get_node_geometry(node_id_2).expect("Node 2 geometry missing");
    // The dummy layout engine just stacks them, not considering parent-child relationship
    assert_eq!(geom_2.rect, LayoutRect::new(10.0, 70.0, 100.0, 50.0)); 

    Ok(())
}

#[test]
fn test_layout_determinism() -> Result<()> {
    let engine = LayoutEngine::new();
    let viewport_size = LayoutSize { width: 800.0, height: 600.0 };

    let node_id_1 = NodeId::derived(0, &[1]);
    let node_1 = LayoutInputNode {
        id: node_id_1,
        parent_id: None,
        op: LayoutOp::Box,
        children_ids: vec![],
        debug_name: "box1".into(),
    };

    let node_id_2 = NodeId::derived(0, &[2]);
    let node_2 = LayoutInputNode {
        id: node_id_2,
        parent_id: Some(node_id_1),
        op: LayoutOp::Flex,
        children_ids: vec![],
        debug_name: "flex1".into(),
    };

    let input_nodes_a = vec![node_1.clone(), node_2.clone()];
    let input_nodes_b = vec![node_1.clone(), node_2.clone()];

    let snapshot_a = engine.compute_layout(&input_nodes_a, viewport_size)?;
    let snapshot_b = engine.compute_layout(&input_nodes_b, viewport_size)?;

    assert_eq!(snapshot_a, snapshot_b, "Layout snapshots must be identical for identical inputs");

    Ok(())
}
