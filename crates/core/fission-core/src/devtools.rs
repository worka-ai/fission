//! Developer-tooling snapshot and overlay helpers.
//!
//! The functions in this module are inert until a shell, test harness, or CLI
//! asks for a snapshot. They do not open sockets or retain history by themselves.

use crate::ui::{Column, Container, Row, Text, Widget};
use crate::Env;
use fission_devtools_protocol as proto;
use fission_ir::{op::Color, CoreIR, Op};
use fission_layout::LayoutSnapshot;

pub use proto::{
    CoreIrNode, CoreIrSnapshot, DevFrame, DevFrameId, DevRect, DevSessionId, DevSize, DevViewport,
    DevtoolsCapabilities, DevtoolsFrameSnapshot, FramePerformanceSample, LayoutNodeSnapshot,
    LayoutSnapshotPayload, PerformanceOverlayState, SemanticsNodeSnapshot, SemanticsSnapshot,
    ShellTarget, SnapshotKind, SnapshotRef, WidgetTreeNode, WidgetTreeSnapshot,
    FDTP_SCHEMA_VERSION,
};

#[derive(Debug, Clone)]
pub struct DevtoolsRuntimeState {
    pub performance_overlay_enabled: bool,
    pub frame_budget_ms: f64,
    pub latest_performance: Option<FramePerformanceSample>,
}

impl Default for DevtoolsRuntimeState {
    fn default() -> Self {
        Self {
            performance_overlay_enabled: false,
            frame_budget_ms: 16.666_667,
            latest_performance: None,
        }
    }
}

impl DevtoolsRuntimeState {
    pub fn record_frame(&mut self, sample: FramePerformanceSample) {
        self.latest_performance = Some(sample);
    }

    pub fn overlay_state(&self) -> Option<PerformanceOverlayState> {
        self.latest_performance.as_ref().map(|sample| {
            PerformanceOverlayState::from_sample(
                self.performance_overlay_enabled,
                self.frame_budget_ms,
                sample,
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceOverlay {
    pub state: PerformanceOverlayState,
}

impl PerformanceOverlay {
    pub fn new(state: PerformanceOverlayState) -> Self {
        Self { state }
    }
}

impl From<PerformanceOverlay> for Widget {
    fn from(overlay: PerformanceOverlay) -> Self {
        let accent = Color {
            r: 145,
            g: 255,
            b: 216,
            a: 255,
        };
        let secondary = Color {
            r: 220,
            g: 229,
            b: 246,
            a: 255,
        };
        let panel = Color {
            r: 4,
            g: 12,
            b: 32,
            a: 232,
        };
        let border = Color {
            r: 58,
            g: 234,
            b: 181,
            a: 190,
        };
        let fps = overlay
            .state
            .fps
            .map(|value| format!("{value:.0} fps"))
            .unwrap_or_else(|| "fps --".to_string());
        let slowest = overlay
            .state
            .slowest_stage
            .clone()
            .unwrap_or_else(|| "stage --".to_string());
        Container::new(Column {
            children: vec![
                Text::new("Fission performance")
                    .size(12.0)
                    .weight(700)
                    .color(accent)
                    .into(),
                Row {
                    children: vec![
                        Text::new(fps)
                            .size(11.0)
                            .color(fission_ir::op::Color::WHITE)
                            .into(),
                        Text::new(format!("{:.2}ms", overlay.state.last_frame_ms))
                            .size(11.0)
                            .color(fission_ir::op::Color::WHITE)
                            .into(),
                    ],
                    gap: Some(8.0),
                    ..Default::default()
                }
                .into(),
                Text::new(slowest).size(11.0).color(secondary).into(),
                Text::new(format!(
                    "widgets {} / ir {} / layout {}",
                    overlay.state.widget_count,
                    overlay.state.core_node_count,
                    overlay.state.layout_node_count
                ))
                .size(11.0)
                .color(secondary)
                .into(),
            ],
            gap: Some(4.0),
            ..Default::default()
        })
        .padding_all(10.0)
        .bg_fill(fission_ir::op::Fill::Solid(panel))
        .border(border, 1.0)
        .border_radius(10.0)
        .width(220.0)
        .into()
    }
}

pub fn inspect_widget_tree(widget: &Widget) -> WidgetTreeSnapshot {
    let mut nodes = Vec::new();
    let root = Some(collect_widget_node(widget, &mut nodes));
    WidgetTreeSnapshot { root, nodes }
}

fn collect_widget_node(widget: &Widget, nodes: &mut Vec<WidgetTreeNode>) -> u64 {
    let ordinal = nodes.len() as u64;
    nodes.push(WidgetTreeNode {
        ordinal,
        widget_id: widget.devtools_explicit_id().map(widget_id_string),
        kind: widget.kind_name().to_string(),
        debug_label: widget.devtools_debug_label(),
        children: Vec::new(),
        properties: widget.devtools_properties(),
        source: None,
    });
    let children = widget
        .devtools_children()
        .into_iter()
        .map(|child| collect_widget_node(child, nodes))
        .collect::<Vec<_>>();
    if let Some(node) = nodes.get_mut(ordinal as usize) {
        node.children = children;
    }
    ordinal
}

pub fn inspect_core_ir(ir: &CoreIR) -> CoreIrSnapshot {
    let mut nodes = ir
        .nodes
        .values()
        .map(|node| CoreIrNode {
            id: widget_id_string(node.id),
            op_tag: op_tag(&node.op).to_string(),
            parent: node.parent.map(widget_id_string),
            children: node
                .children
                .iter()
                .copied()
                .map(widget_id_string)
                .collect(),
            hash: node.hash,
        })
        .collect::<Vec<_>>();
    nodes.sort_by_key(|node| node.id.parse::<u128>().unwrap_or(0));
    CoreIrSnapshot {
        root: ir.root.map(widget_id_string),
        nodes,
    }
}

pub fn inspect_layout(ir: Option<&CoreIR>, layout: &LayoutSnapshot) -> LayoutSnapshotPayload {
    let mut nodes = layout
        .nodes
        .iter()
        .map(|(id, geometry)| {
            let constraints =
                layout
                    .get_node_constraints(*id)
                    .map(|c| proto::BoxConstraintsSnapshot {
                        min_width: finite_f32(c.min_w),
                        max_width: finite_f32(c.max_w),
                        min_height: finite_f32(c.min_h),
                        max_height: finite_f32(c.max_h),
                    });
            LayoutNodeSnapshot {
                id: widget_id_string(*id),
                parent: ir
                    .and_then(|ir| ir.nodes.get(id))
                    .and_then(|node| node.parent)
                    .map(widget_id_string),
                rect: DevRect {
                    x: geometry.rect.origin.x,
                    y: geometry.rect.origin.y,
                    width: geometry.rect.size.width,
                    height: geometry.rect.size.height,
                },
                content_size: DevSize {
                    width: geometry.content_size.width,
                    height: geometry.content_size.height,
                },
                constraints,
            }
        })
        .collect::<Vec<_>>();
    nodes.sort_by_key(|node| node.id.parse::<u128>().unwrap_or(0));
    LayoutSnapshotPayload {
        viewport: DevSize {
            width: layout.viewport_size.width,
            height: layout.viewport_size.height,
        },
        nodes,
    }
}

pub fn inspect_semantics(ir: &CoreIR) -> SemanticsSnapshot {
    let mut nodes = ir
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            let Op::Semantics(semantics) = &node.op else {
                return None;
            };
            Some(SemanticsNodeSnapshot {
                id: widget_id_string(*id),
                role: format!("{:?}", semantics.role),
                label: semantics.label.clone(),
                value: semantics.value.clone(),
                focusable: semantics.focusable,
                enabled: !semantics.disabled,
                selected: false,
                checked: semantics.checked,
                actions: semantics
                    .actions
                    .entries
                    .iter()
                    .map(|entry| format!("{:?}", entry.trigger))
                    .collect(),
            })
        })
        .collect::<Vec<_>>();
    nodes.sort_by_key(|node| node.id.parse::<u128>().unwrap_or(0));
    SemanticsSnapshot { nodes }
}

pub fn frame_snapshot(
    sequence: u64,
    shell: ShellTarget,
    viewport: DevViewport,
    widget_tree: Option<WidgetTreeSnapshot>,
    core_ir: Option<CoreIrSnapshot>,
    layout: Option<LayoutSnapshotPayload>,
    semantics: Option<SemanticsSnapshot>,
    performance: Option<FramePerformanceSample>,
) -> DevtoolsFrameSnapshot {
    let widget_tree_ref = widget_tree.as_ref().map(|snapshot| {
        snapshot_ref(
            SnapshotKind::WidgetTree,
            sequence,
            snapshot.nodes.len(),
            snapshot,
        )
    });
    let core_ir_ref = core_ir.as_ref().map(|snapshot| {
        snapshot_ref(
            SnapshotKind::CoreIr,
            sequence,
            snapshot.nodes.len(),
            snapshot,
        )
    });
    let layout_ref = layout.as_ref().map(|snapshot| {
        snapshot_ref(
            SnapshotKind::Layout,
            sequence,
            snapshot.nodes.len(),
            snapshot,
        )
    });
    let semantics_ref = semantics.as_ref().map(|snapshot| {
        snapshot_ref(
            SnapshotKind::Semantics,
            sequence,
            snapshot.nodes.len(),
            snapshot,
        )
    });
    let performance_ref = performance
        .as_ref()
        .map(|snapshot| snapshot_ref(SnapshotKind::Performance, sequence, 1, snapshot));

    DevtoolsFrameSnapshot {
        frame: DevFrame {
            schema_version: FDTP_SCHEMA_VERSION,
            session_id: None,
            frame_id: DevFrameId(sequence),
            sequence,
            shell,
            viewport,
            widget_tree_ref,
            core_ir_ref,
            layout_ref,
            display_list_ref: None,
            semantics_ref,
            performance_ref,
            diagnostics_ref: None,
        },
        capabilities: DevtoolsCapabilities::runtime_baseline(),
        widget_tree,
        core_ir,
        layout,
        semantics,
        performance,
    }
}

fn snapshot_ref<T: serde::Serialize>(
    kind: SnapshotKind,
    sequence: u64,
    node_count: usize,
    payload: &T,
) -> SnapshotRef {
    let json = serde_json::to_vec(payload).unwrap_or_default();
    SnapshotRef {
        kind,
        id: format!("frame-{sequence}-{:?}", kind).to_ascii_lowercase(),
        node_count,
        byte_len: json.len(),
    }
}

fn op_tag(op: &Op) -> &'static str {
    match op {
        Op::Layout(_) => "layout",
        Op::Paint(_) => "paint",
        Op::Semantics(_) => "semantics",
        Op::Structural(_) => "structural",
    }
}

fn widget_id_string(id: fission_ir::WidgetId) -> String {
    id.as_u128().to_string()
}

fn finite_f32(value: f32) -> Option<f32> {
    value.is_finite().then_some(value)
}

pub fn default_desktop_viewport(env: &Env) -> DevViewport {
    DevViewport::logical(env.viewport_size.width, env.viewport_size.height)
}

pub fn performance_sample_from_runtime(
    sequence: u64,
    renderer: Option<String>,
    total_ms: f64,
    widget_count: usize,
    ir: Option<&CoreIR>,
    layout: Option<&LayoutSnapshot>,
) -> FramePerformanceSample {
    FramePerformanceSample {
        sequence,
        renderer,
        total_ms,
        build_ms: None,
        lower_ms: None,
        layout_ms: None,
        paint_ms: None,
        raster_ms: None,
        present_ms: None,
        input_latency_ms: None,
        widget_count,
        core_node_count: ir.map(|ir| ir.nodes.len()).unwrap_or(0),
        layout_node_count: layout.map(|layout| layout.nodes.len()).unwrap_or(0),
        paint_op_count: None,
    }
}
