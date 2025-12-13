use anyhow::Result;
use fission_ir::NodeId;
use fission_layout::{
    LayoutEngine, LayoutInputNode, LayoutPoint, LayoutRect, LayoutSize, LayoutSnapshot, LayoutOp, LayoutConstraints
};
use fission_ir::FlexDirection;

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
    let node_id_2 = NodeId::derived(0, &[2]);

    let node_1 = LayoutInputNode {
        id: node_id_1,
        parent_id: None,
        op: LayoutOp::Box { width: Some(100.0), height: Some(50.0) },
        children_ids: vec![node_id_2], // Correctly set child relationship
        debug_name: "box1".into(),
        width: Some(100.0), height: Some(50.0), flex_grow: 0.0, flex_shrink: 0.0,
    };

    let node_2 = LayoutInputNode {
        id: node_id_2,
        parent_id: Some(node_id_1),
        op: LayoutOp::Box { width: Some(150.0), height: Some(80.0) },
        children_ids: vec![],
        debug_name: "box2".into(),
        width: Some(150.0), height: Some(80.0), flex_grow: 0.0, flex_shrink: 0.0,
    };

    let input_nodes = vec![node_1.clone(), node_2.clone()];
    let snapshot = engine.compute_layout(&input_nodes, viewport_size)?;

    let geom_1 = snapshot.get_node_geometry(node_id_1).expect("Node 1 geometry missing");
    assert_eq!(geom_1.rect, LayoutRect::new(0.0, 0.0, 100.0, 50.0));

    let geom_2 = snapshot.get_node_geometry(node_id_2).expect("Node 2 geometry missing");
    // Child of a Box is currently laid out at parent's origin with loosened constraints
    // Since node_1 is at (0,0), node_2 is relative to that.
    // My layout algorithm calculates absolute positions relative to the offset passed down.
    // For box layout, I passed `offset` (parent's pos) to child.
    // So if node_1 is at (0,0), node_2 should be at (0,0) (relative to root) but 150x80.
    assert_eq!(geom_2.rect, LayoutRect::new(0.0, 0.0, 150.0, 80.0)); 

    Ok(())
}

#[test]
fn test_layout_determinism() -> Result<()> {
    let engine = LayoutEngine::new();
    let viewport_size = LayoutSize { width: 800.0, height: 600.0 };

    let node_id_1 = NodeId::derived(0, &[1]);
    let node_id_2 = NodeId::derived(0, &[2]);

    let node_1 = LayoutInputNode {
        id: node_id_1,
        parent_id: None,
        op: LayoutOp::Box { width: Some(100.0), height: Some(50.0) },
        children_ids: vec![node_id_2],
        debug_name: "root_box".into(),
        width: Some(100.0), height: Some(50.0), flex_grow: 0.0, flex_shrink: 0.0,
    };

    let node_2 = LayoutInputNode {
        id: node_id_2,
        parent_id: Some(node_id_1),
        op: LayoutOp::Flex { direction: FlexDirection::Row, flex_grow: 0.0, flex_shrink: 0.0 },
        children_ids: vec![],
        debug_name: "flex_child".into(),
        width: None, height: None, flex_grow: 0.0, flex_shrink: 0.0,
    };

    let input_nodes_a = vec![node_1.clone(), node_2.clone()];
    let input_nodes_b = vec![node_1.clone(), node_2.clone()];

    let snapshot_a = engine.compute_layout(&input_nodes_a, viewport_size)?;
    let snapshot_b = engine.compute_layout(&input_nodes_b, viewport_size)?;

    assert_eq!(snapshot_a, snapshot_b, "Layout snapshots must be identical for identical inputs");

    Ok(())
}

#[test]
fn test_flex_row_fixed_children() -> Result<()> {
    let engine = LayoutEngine::new();
    let viewport_size = LayoutSize::new(300.0, 100.0);

    let root_id = NodeId::derived(0, &[1]);
    let child1_id = NodeId::derived(root_id.as_u128(), &[1]);
    let child2_id = NodeId::derived(root_id.as_u128(), &[2]);

    let root = LayoutInputNode {
        id: root_id,
        parent_id: None,
        op: LayoutOp::Flex { direction: FlexDirection::Row, flex_grow: 0.0, flex_shrink: 0.0 },
        children_ids: vec![child1_id, child2_id],
        debug_name: "root_flex".into(),
        width: None, height: None, flex_grow: 0.0, flex_shrink: 0.0,
    };
    let child1 = LayoutInputNode {
        id: child1_id,
        parent_id: Some(root_id),
        op: LayoutOp::Box { width: Some(50.0), height: Some(50.0) },
        children_ids: vec![],
        debug_name: "child1".into(),
        width: Some(50.0), height: Some(50.0), flex_grow: 0.0, flex_shrink: 0.0,
    };
    let child2 = LayoutInputNode {
        id: child2_id,
        parent_id: Some(root_id),
        op: LayoutOp::Box { width: Some(100.0), height: Some(50.0) },
        children_ids: vec![],
        debug_name: "child2".into(),
        width: Some(100.0), height: Some(50.0), flex_grow: 0.0, flex_shrink: 0.0,
    };

    let input_nodes = vec![root.clone(), child1.clone(), child2.clone()];
    let snapshot = engine.compute_layout(&input_nodes, viewport_size)?;

    let geom_root = snapshot.get_node_geometry(root_id).unwrap();
    assert_eq!(geom_root.rect.size.width, 150.0); // 50 + 100
    assert_eq!(geom_root.rect.size.height, 50.0);
    assert_eq!(geom_root.rect.origin, LayoutPoint::ZERO);

    let geom_child1 = snapshot.get_node_geometry(child1_id).unwrap();
    assert_eq!(geom_child1.rect, LayoutRect::new(0.0, 0.0, 50.0, 50.0));

    let geom_child2 = snapshot.get_node_geometry(child2_id).unwrap();
    assert_eq!(geom_child2.rect, LayoutRect::new(50.0, 0.0, 100.0, 50.0));

    Ok(())
}

#[test]
fn test_flex_row_grow_children() -> Result<()> {
    let engine = LayoutEngine::new();
    let viewport_size = LayoutSize::new(300.0, 100.0);

    let root_id = NodeId::derived(0, &[1]);
    let child1_id = NodeId::derived(root_id.as_u128(), &[1]);
    let child2_id = NodeId::derived(root_id.as_u128(), &[2]);

    let root = LayoutInputNode {
        id: root_id,
        parent_id: None,
        op: LayoutOp::Flex { direction: FlexDirection::Row, flex_grow: 0.0, flex_shrink: 0.0 },
        children_ids: vec![child1_id, child2_id],
        debug_name: "root_flex".into(),
        width: None, height: None, flex_grow: 0.0, flex_shrink: 0.0,
    };
    let child1 = LayoutInputNode {
        id: child1_id,
        parent_id: Some(root_id),
        op: LayoutOp::Box { width: Some(50.0), height: Some(50.0) },
        children_ids: vec![],
        debug_name: "child1".into(),
        width: Some(50.0), height: Some(50.0), flex_grow: 1.0, flex_shrink: 0.0,
    };
    let child2 = LayoutInputNode {
        id: child2_id,
        parent_id: Some(root_id),
        op: LayoutOp::Box { width: Some(50.0), height: Some(50.0) },
        children_ids: vec![],
        debug_name: "child2".into(),
        width: Some(50.0), height: Some(50.0), flex_grow: 1.0, flex_shrink: 0.0,
    };

    let input_nodes = vec![root.clone(), child1.clone(), child2.clone()];
    let snapshot = engine.compute_layout(&input_nodes, viewport_size)?;

    let geom_root = snapshot.get_node_geometry(root_id).unwrap();
    assert_eq!(geom_root.rect.size.width, 300.0); // Fills viewport
    assert_eq!(geom_root.rect.size.height, 50.0);

    let geom_child1 = snapshot.get_node_geometry(child1_id).unwrap();
    assert_eq!(geom_child1.rect, LayoutRect::new(0.0, 0.0, 150.0, 50.0)); // 50 + (300-100)/2

    let geom_child2 = snapshot.get_node_geometry(child2_id).unwrap();
    assert_eq!(geom_child2.rect, LayoutRect::new(150.0, 0.0, 150.0, 50.0));

    Ok(())
}
