use fission_core::devtools::{inspect_core_ir, inspect_widget_tree, PerformanceOverlay};
use fission_core::ui::{Button, Column, Text, Widget, WidgetIdExt};

#[test]
fn widget_tree_snapshot_preserves_authoring_structure_and_ids() {
    let explicit = fission_ir::WidgetId::explicit("counter.increment");
    let tree: Widget = Column {
        children: vec![
            Text::new("Counter").into(),
            Button {
                child: Some(Text::new("Increment").into()),
                ..Default::default()
            }
            .id(explicit),
        ],
        ..Default::default()
    }
    .into();

    let snapshot = inspect_widget_tree(&tree);

    let root = snapshot.root.expect("root ordinal");
    assert_eq!(root, 0);
    assert_eq!(snapshot.nodes[root as usize].kind, "Column");
    assert_eq!(snapshot.nodes[root as usize].children, vec![1, 2]);
    assert_eq!(snapshot.nodes[1].kind, "Text");
    assert_eq!(snapshot.nodes[2].kind, "Button");
    let explicit_string = explicit.as_u128().to_string();
    assert_eq!(
        snapshot.nodes[2].widget_id.as_deref(),
        Some(explicit_string.as_str())
    );
    assert_eq!(snapshot.nodes[2].children, vec![3]);
    assert_eq!(snapshot.nodes[3].kind, "Text");
}

#[test]
fn core_ir_snapshot_is_stable_and_source_linkable() {
    let tree: Widget = Button {
        child: Some(Text::new("Save").into()),
        ..Default::default()
    }
    .into();
    let ir = fission_core::internal::lower_widget_to_ir(&tree);
    let snapshot = inspect_core_ir(&ir);

    assert!(!snapshot.nodes.is_empty());
    assert!(snapshot
        .nodes
        .iter()
        .any(|node| node.op_tag == "layout" || node.op_tag == "semantics"));
    assert!(snapshot
        .nodes
        .windows(2)
        .all(|pair| pair[0].id <= pair[1].id));
}

#[test]
fn performance_overlay_is_a_normal_widget_tree() {
    let sample = fission_core::devtools::FramePerformanceSample {
        sequence: 4,
        renderer: Some("vello".into()),
        total_ms: 12.5,
        frame_interval_ms: Some(16.67),
        build_ms: Some(2.0),
        lower_ms: Some(1.5),
        layout_ms: Some(3.0),
        paint_ms: Some(4.0),
        raster_ms: None,
        present_ms: None,
        input_latency_ms: None,
        widget_count: 5,
        core_node_count: 20,
        layout_node_count: 10,
        paint_op_count: None,
    };
    let state = fission_core::devtools::PerformanceOverlayState::from_sample(true, 16.0, &sample);
    let overlay: Widget = PerformanceOverlay::new(state).into();
    let snapshot = inspect_widget_tree(&overlay);

    assert_eq!(snapshot.nodes[0].kind, "Container");
    assert!(snapshot.nodes.iter().any(|node| node.kind == "Text"));
    assert!(snapshot.nodes.iter().any(|node| {
        node.debug_label
            .as_deref()
            .unwrap_or_default()
            .contains("Fission performance")
    }));
}
