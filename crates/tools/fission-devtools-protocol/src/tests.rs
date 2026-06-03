use super::*;

#[test]
fn performance_overlay_state_derives_fps_and_slowest_stage() {
    let sample = FramePerformanceSample {
        sequence: 7,
        renderer: Some("vello".into()),
        total_ms: 8.5,
        frame_interval_ms: Some(20.0),
        build_ms: Some(3.0),
        lower_ms: Some(4.0),
        layout_ms: Some(2.0),
        paint_ms: Some(9.0),
        raster_ms: Some(1.0),
        present_ms: None,
        input_latency_ms: None,
        widget_count: 12,
        core_node_count: 34,
        layout_node_count: 21,
        paint_op_count: Some(55),
    };

    let state = PerformanceOverlayState::from_sample(true, 16.0, &sample);

    assert_eq!(state.enabled, true);
    assert_eq!(state.fps, Some(50.0));
    assert_eq!(state.last_frame_ms, 20.0);
    assert_eq!(state.last_render_ms, 8.5);
    assert_eq!(state.slowest_stage.as_deref(), Some("paint 9.00ms"));
    assert_eq!(state.widget_count, 12);
}

#[test]
fn frame_snapshot_payload_serializes_schema_version() {
    let snapshot = DevtoolsFrameSnapshot {
        frame: DevFrame {
            schema_version: FDTP_SCHEMA_VERSION,
            session_id: None,
            frame_id: DevFrameId(1),
            sequence: 1,
            shell: ShellTarget::Desktop,
            viewport: DevViewport::logical(320.0, 240.0),
            widget_tree_ref: None,
            core_ir_ref: None,
            layout_ref: None,
            display_list_ref: None,
            semantics_ref: None,
            performance_ref: None,
            diagnostics_ref: None,
        },
        capabilities: DevtoolsCapabilities::runtime_baseline(),
        widget_tree: None,
        core_ir: None,
        layout: None,
        semantics: None,
        performance: None,
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(json.contains("\"schema_version\":1"));
    assert!(json.contains("\"widget_tree\":true"));
}
