//! Versioned protocol types for Fission developer tooling.
//!
//! This crate does not start servers, inspect Rust code, or depend on any Fission
//! renderer. It is the stable schema shared by shell instrumentation, the CLI,
//! tests, trace viewers, and future IDE plugins.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const FDTP_SCHEMA_VERSION: u16 = 1;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ShellTarget {
    Desktop,
    Web,
    Android,
    Ios,
    Terminal,
    StaticSite,
    ServerSite,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SnapshotKind {
    WidgetTree,
    CoreIr,
    Layout,
    Semantics,
    DisplayList,
    HitTest,
    Performance,
    Logs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DevSessionId(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct DevFrameId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DevViewport {
    pub logical_width: f32,
    pub logical_height: f32,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
}

impl DevViewport {
    pub fn logical(width: f32, height: f32) -> Self {
        Self {
            logical_width: width,
            logical_height: height,
            physical_width: width.max(0.0).round() as u32,
            physical_height: height.max(0.0).round() as u32,
            scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceProvenance {
    pub crate_name: String,
    pub module_path: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DevtoolsCapabilities {
    pub widget_tree: bool,
    pub core_ir: bool,
    pub layout: bool,
    pub display_list: bool,
    pub semantics: bool,
    pub hit_test: bool,
    pub actions: bool,
    pub reducers: bool,
    pub effects: bool,
    pub resources: bool,
    pub jobs: bool,
    pub services: bool,
    pub capabilities: bool,
    pub network: bool,
    pub performance: bool,
    pub memory: bool,
    pub app_size: bool,
    pub screenshots: bool,
    pub test_recording: bool,
    pub visual_preview: bool,
    pub shell_specific: Vec<String>,
}

impl DevtoolsCapabilities {
    pub fn runtime_baseline() -> Self {
        Self {
            widget_tree: true,
            core_ir: true,
            layout: true,
            display_list: false,
            semantics: true,
            hit_test: true,
            actions: true,
            reducers: true,
            effects: true,
            resources: true,
            jobs: true,
            services: true,
            capabilities: true,
            network: false,
            performance: true,
            memory: false,
            app_size: false,
            screenshots: true,
            test_recording: true,
            visual_preview: false,
            shell_specific: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRef {
    pub kind: SnapshotKind,
    pub id: String,
    pub node_count: usize,
    pub byte_len: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DevFrame {
    pub schema_version: u16,
    pub session_id: Option<DevSessionId>,
    pub frame_id: DevFrameId,
    pub sequence: u64,
    pub shell: ShellTarget,
    pub viewport: DevViewport,
    pub widget_tree_ref: Option<SnapshotRef>,
    pub core_ir_ref: Option<SnapshotRef>,
    pub layout_ref: Option<SnapshotRef>,
    pub display_list_ref: Option<SnapshotRef>,
    pub semantics_ref: Option<SnapshotRef>,
    pub performance_ref: Option<SnapshotRef>,
    pub diagnostics_ref: Option<SnapshotRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DevtoolsFrameSnapshot {
    pub frame: DevFrame,
    pub capabilities: DevtoolsCapabilities,
    pub widget_tree: Option<WidgetTreeSnapshot>,
    pub core_ir: Option<CoreIrSnapshot>,
    pub layout: Option<LayoutSnapshotPayload>,
    pub semantics: Option<SemanticsSnapshot>,
    pub performance: Option<FramePerformanceSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetTreeSnapshot {
    pub root: Option<u64>,
    pub nodes: Vec<WidgetTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetTreeNode {
    pub ordinal: u64,
    pub widget_id: Option<String>,
    pub kind: String,
    pub debug_label: Option<String>,
    pub children: Vec<u64>,
    pub properties: BTreeMap<String, String>,
    pub source: Option<SourceProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreIrSnapshot {
    pub root: Option<String>,
    pub nodes: Vec<CoreIrNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreIrNode {
    pub id: String,
    pub op_tag: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub hash: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DevPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DevSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DevRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutSnapshotPayload {
    pub viewport: DevSize,
    pub nodes: Vec<LayoutNodeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutNodeSnapshot {
    pub id: String,
    pub parent: Option<String>,
    pub rect: DevRect,
    pub content_size: DevSize,
    pub constraints: Option<BoxConstraintsSnapshot>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BoxConstraintsSnapshot {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticsSnapshot {
    pub nodes: Vec<SemanticsNodeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticsNodeSnapshot {
    pub id: String,
    pub role: String,
    pub label: Option<String>,
    pub value: Option<String>,
    pub focusable: bool,
    pub enabled: bool,
    pub selected: bool,
    pub checked: Option<bool>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FramePerformanceSample {
    pub sequence: u64,
    pub renderer: Option<String>,
    pub total_ms: f64,
    pub build_ms: Option<f64>,
    pub lower_ms: Option<f64>,
    pub layout_ms: Option<f64>,
    pub paint_ms: Option<f64>,
    pub raster_ms: Option<f64>,
    pub present_ms: Option<f64>,
    pub input_latency_ms: Option<f64>,
    pub widget_count: usize,
    pub core_node_count: usize,
    pub layout_node_count: usize,
    pub paint_op_count: Option<usize>,
}

impl FramePerformanceSample {
    pub fn fps(&self) -> Option<f64> {
        if self.total_ms > 0.0 {
            Some(1000.0 / self.total_ms)
        } else {
            None
        }
    }

    pub fn slowest_known_stage(&self) -> Option<(&'static str, f64)> {
        [
            ("build", self.build_ms),
            ("lower", self.lower_ms),
            ("layout", self.layout_ms),
            ("paint", self.paint_ms),
            ("raster", self.raster_ms),
            ("present", self.present_ms),
        ]
        .into_iter()
        .filter_map(|(name, value)| value.map(|value| (name, value)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceOverlayState {
    pub enabled: bool,
    pub frame_budget_ms: f64,
    pub last_frame_ms: f64,
    pub fps: Option<f64>,
    pub slowest_stage: Option<String>,
    pub widget_count: usize,
    pub core_node_count: usize,
    pub layout_node_count: usize,
}

impl PerformanceOverlayState {
    pub fn from_sample(
        enabled: bool,
        frame_budget_ms: f64,
        sample: &FramePerformanceSample,
    ) -> Self {
        Self {
            enabled,
            frame_budget_ms,
            last_frame_ms: sample.total_ms,
            fps: sample.fps(),
            slowest_stage: sample
                .slowest_known_stage()
                .map(|(name, duration)| format!("{name} {duration:.2}ms")),
            widget_count: sample.widget_count,
            core_node_count: sample.core_node_count,
            layout_node_count: sample.layout_node_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceManifest {
    pub schema_version: u16,
    pub created_unix_ms: u64,
    pub app_name: Option<String>,
    pub target: ShellTarget,
    pub frames: Vec<String>,
    pub redaction_summary: Vec<String>,
}
