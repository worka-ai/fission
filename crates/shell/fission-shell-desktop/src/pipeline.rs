use anyhow::Result;
use fission_core::diff::diff_ir;
use fission_core::env::{AnimationStateMap, Env, VideoStateMap, WebStateMap};
use fission_core::registry::AnimationPropertyId;
use fission_core::lowering::build_layout_tree;
use fission_core::{LayoutPoint, ScrollStateMap};
use fission_diagnostics::prelude as diag;
use fission_diagnostics::{SnapshotBlob, SnapshotKind, SnapshotProvider};
use fission_ir::{CompositeScalar, CoreIR, EmbedKind, FlexDirection, LayoutOp, NodeId, Op, WidgetNodeId};
use fission_layout::{LayoutEngine, LayoutInputNode, LayoutRect, LayoutSnapshot, LayoutSize};
use fission_render::{
    BoxShadow, Color as RenderColor, DisplayList, DisplayOp, Fill, Renderer, Stroke,
};
use fission_shell::VideoSurfaceFrame;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::Instant;

fn render_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("FISSION_RENDER_TRACE").is_ok())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InvalidationSet {
    pub build: bool,
    pub layout: bool,
    pub paint: bool,
    pub composite: bool,
}

impl InvalidationSet {
    pub fn mark_build(&mut self) {
        self.build = true;
        self.layout = true;
        self.paint = true;
        self.composite = true;
    }

    pub fn mark_layout(&mut self) {
        self.layout = true;
        self.paint = true;
        self.composite = true;
    }

    pub fn mark_paint(&mut self) {
        self.paint = true;
        self.composite = true;
    }

    pub fn mark_composite(&mut self) {
        self.composite = true;
    }

    pub fn merge(&mut self, other: Self) {
        self.build |= other.build;
        self.layout |= other.layout;
        self.paint |= other.paint;
        self.composite |= other.composite;
    }

    pub fn any(self) -> bool {
        self.build || self.layout || self.paint || self.composite
    }
}

#[derive(Debug, Clone)]
struct BoundaryCacheEntry {
    hash: u64,
    ops: Vec<DisplayOp>,
}

#[derive(Debug, Clone)]
struct OpacityBinding {
    op_index: usize,
    scalar: CompositeScalar,
}

#[derive(Debug, Clone)]
struct TransformBinding {
    op_index: usize,
    rect: LayoutRect,
    translate_x: Option<CompositeScalar>,
    translate_y: Option<CompositeScalar>,
    scale: Option<CompositeScalar>,
    rotation: Option<CompositeScalar>,
}

#[derive(Debug, Clone)]
struct ScrollBinding {
    op_index: usize,
    node_id: NodeId,
    direction: FlexDirection,
}

#[derive(Debug, Clone, Default)]
struct RetainedDynamicOps {
    opacity: Vec<OpacityBinding>,
    transform: Vec<TransformBinding>,
    scroll: Vec<ScrollBinding>,
}

pub struct Pipeline {
    pub prev_ir: Option<CoreIR>,
    pub last_snapshot: Option<LayoutSnapshot>,
    pub paint_cache: HashMap<NodeId, (u64, Vec<DisplayOp>)>,
    pub boundary_cache: HashMap<NodeId, BoundaryCacheEntry>,
    pub last_scroll_offsets: HashMap<NodeId, u32>,
    pub video_surfaces: Vec<VideoSurfaceFrame>,
    pub scene_3d_surfaces: Vec<(WidgetNodeId, LayoutRect, Vec<u8>)>,
    pub last_viewport: Option<LayoutRect>,
    pub layout_invariant_violation_count: u32,
    pub layout_full_rebuild_count: u32,
    retained_display_list: Option<DisplayList>,
    retained_dynamic_ops: RetainedDynamicOps,
    layout_input_nodes: Vec<LayoutInputNode>,
    pending_layout_dirty: HashSet<NodeId>,
    pending_layout_full: bool,
    compositor_animation_keys: HashSet<(WidgetNodeId, AnimationPropertyId)>,
    runtime_dynamic_subtrees: HashMap<NodeId, bool>,
}

pub struct PipelineStats {
    pub dirty_nodes: usize,
    pub layout_updates: usize,
    pub paint_misses: usize,
    pub paint_hits: usize,
    pub video_surfaces: usize,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            prev_ir: None,
            last_snapshot: None,
            paint_cache: HashMap::new(),
            boundary_cache: HashMap::new(),
            last_scroll_offsets: HashMap::new(),
            video_surfaces: Vec::new(),
            scene_3d_surfaces: Vec::new(),
            last_viewport: None,
            layout_invariant_violation_count: 0,
            layout_full_rebuild_count: 0,
            retained_display_list: None,
            retained_dynamic_ops: RetainedDynamicOps::default(),
            layout_input_nodes: Vec::new(),
            pending_layout_dirty: HashSet::new(),
            pending_layout_full: true,
            compositor_animation_keys: HashSet::new(),
            runtime_dynamic_subtrees: HashMap::new(),
        }
    }

    pub fn take_video_surfaces(&mut self) -> Vec<VideoSurfaceFrame> {
        std::mem::take(&mut self.video_surfaces)
    }

    pub fn invalidate_layout_all(&mut self) {
        self.pending_layout_full = true;
    }

    pub fn replace_ir(&mut self, next_ir: CoreIR, env: &Env) -> InvalidationSet {
        let mut invalidation = InvalidationSet::default();
        let mut rebuild_layout_tree = self.prev_ir.is_none();

        if let Some(prev_ir) = &self.prev_ir {
            let diff = diff_ir(prev_ir, &next_ir);
            if !diff.dirty_layout.is_empty() {
                invalidation.mark_layout();
                self.pending_layout_dirty.extend(diff.dirty_layout.iter().copied());
            }
            if !diff.dirty_paint.is_empty() {
                invalidation.mark_paint();
            }
            if !diff.dirty_composite.is_empty() {
                invalidation.mark_composite();
            }
            rebuild_layout_tree = rebuild_layout_tree || invalidation.layout;
        } else {
            invalidation.mark_build();
            self.pending_layout_full = true;
        }

        if rebuild_layout_tree {
            self.layout_input_nodes = build_layout_tree(&next_ir, env);
        }

        if invalidation.layout {
            self.pending_layout_full |= self.prev_ir.is_none();
            self.clear_render_caches();
        } else if invalidation.paint || invalidation.composite {
            self.clear_render_caches();
        }

        self.prev_ir = Some(next_ir);
        self.refresh_retained_metadata();
        invalidation
    }

    pub fn classify_animation_updates(
        &self,
        changed: &[(WidgetNodeId, AnimationPropertyId)],
    ) -> InvalidationSet {
        let mut invalidation = InvalidationSet::default();
        for key in changed {
            if self.compositor_animation_keys.contains(key) {
                invalidation.mark_composite();
            } else {
                invalidation.mark_build();
            }
        }
        invalidation
    }

    pub fn ensure_layout(
        &mut self,
        viewport: LayoutRect,
        layout_engine: &mut LayoutEngine,
        scroll_map: &ScrollStateMap,
    ) -> Result<usize> {
        let viewport_changed = self.last_viewport.map(|v| v != viewport).unwrap_or(true);
        let needs_full = self.pending_layout_full || self.last_snapshot.is_none() || viewport_changed;

        if !needs_full && self.pending_layout_dirty.is_empty() {
            self.last_viewport = Some(viewport);
            return Ok(0);
        }

        let start_layout = Instant::now();
        let dirty_nodes: HashSet<NodeId> = if needs_full {
            self.layout_full_rebuild_count = self.layout_full_rebuild_count.saturating_add(1);
            self.layout_input_nodes.iter().map(|n| n.id).collect()
        } else {
            self.pending_layout_dirty.clone()
        };

        layout_engine.update(&self.layout_input_nodes, &dirty_nodes);

        let root_id = self
            .prev_ir
            .as_ref()
            .and_then(|ir| ir.root)
            .expect("no root in IR");
        let snapshot = layout_engine.compute_layout(&self.layout_input_nodes, root_id, viewport.size, &|id| {
            scroll_map.get_offset(id)
        })?;
        self.last_snapshot = Some(snapshot);
        self.last_viewport = Some(viewport);
        self.pending_layout_dirty.clear();
        self.pending_layout_full = false;
        self.clear_render_caches();

        let duration = start_layout.elapsed().as_nanos() as u64;
        diag::emit(
            diag::DiagCategory::Layout,
            diag::DiagLevel::Debug,
            diag::DiagEventKind::LayoutSummary {
                nodes: self.layout_input_nodes.len() as u32,
                dirty_count: dirty_nodes.len() as u32,
                full_rebuild: needs_full,
                duration_ns: duration,
            },
        );

        Ok(dirty_nodes.len())
    }

    pub fn render_current(
        &mut self,
        viewport_size: LayoutSize,
        renderer: &mut dyn Renderer,
        scroll_map: &ScrollStateMap,
        animation_map: &AnimationStateMap,
        video_map: &VideoStateMap,
        _web_map: &WebStateMap,
    ) -> Result<PipelineStats> {
        let viewport = LayoutRect::new(0.0, 0.0, viewport_size.width, viewport_size.height);
        let mut stats = PipelineStats {
            dirty_nodes: self.pending_layout_dirty.len(),
            layout_updates: 0,
            paint_misses: 0,
            paint_hits: 0,
            video_surfaces: 0,
        };

        let ir = self.prev_ir.as_ref().expect("ir missing before render");
        let snapshot = self
            .last_snapshot
            .as_ref()
            .expect("snapshot missing before render");

        self.video_surfaces.clear();
        self.scene_3d_surfaces.clear();
        if let Some(root) = ir.root {
            collect_video_surfaces(
                root,
                ir,
                snapshot,
                video_map,
                scroll_map,
                LayoutPoint::ZERO,
                &mut self.video_surfaces,
                &mut self.scene_3d_surfaces,
            );
        }
        stats.video_surfaces = self.video_surfaces.len();

        if self.retained_display_list.is_none() {
            if render_trace_enabled() {
                eprintln!("[pipeline] rebuilding retained display list");
            }
            let mut display_list = DisplayList {
                ops: Vec::new(),
                bounds: viewport,
            };

            if let Some(root) = ir.root {
                let mut visited = HashSet::new();
                display_list.ops = generate_display_list_recursive(
                    root,
                    ir,
                    snapshot,
                    scroll_map,
                    animation_map,
                    &mut self.paint_cache,
                    &mut self.boundary_cache,
                    &self.runtime_dynamic_subtrees,
                    &mut stats.paint_misses,
                    &mut stats.paint_hits,
                    LayoutPoint::ZERO,
                    true,
                    &mut visited,
                );
                self.retained_dynamic_ops = collect_retained_dynamic_ops(
                    root,
                    ir,
                    snapshot,
                    scroll_map,
                    animation_map,
                    &self.runtime_dynamic_subtrees,
                );
            }

            self.retained_display_list = Some(display_list);
        }

        self.patch_retained_display_list(scroll_map, animation_map);
        let display_list = self
            .retained_display_list
            .as_ref()
            .expect("retained display list missing before render");

        diag::emit(
            diag::DiagCategory::Layout,
            diag::DiagLevel::Debug,
            diag::DiagEventKind::PaintSummary {
                segments_reused: stats.paint_hits as u32,
                segments_regenerated: stats.paint_misses as u32,
                paint_ops_total: display_list.ops.len() as u32,
            },
        );

        self.last_scroll_offsets = scroll_map
            .offsets
            .iter()
            .map(|(id, offset)| (*id, offset.to_bits()))
            .collect();

        renderer.render(display_list)?;
        Ok(stats)
    }

    pub fn render(
        &mut self,
        next_ir: CoreIR,
        viewport_size: LayoutSize,
        layout_engine: &mut LayoutEngine,
        scroll_map: &ScrollStateMap,
        renderer: &mut dyn Renderer,
        video_map: &VideoStateMap,
        web_map: &WebStateMap,
        env: &Env,
    ) -> Result<PipelineStats> {
        self.replace_ir(next_ir, env);
        let viewport = LayoutRect::new(0.0, 0.0, viewport_size.width, viewport_size.height);
        let layout_updates = self.ensure_layout(viewport, layout_engine, scroll_map)?;
        let mut stats = self.render_current(
            viewport_size,
            renderer,
            scroll_map,
            &AnimationStateMap::default(),
            video_map,
            web_map,
        )?;
        stats.layout_updates = layout_updates;
        Ok(stats)
    }

    fn refresh_retained_metadata(&mut self) {
        self.compositor_animation_keys.clear();
        self.runtime_dynamic_subtrees.clear();
        self.boundary_cache.clear();

        let Some(ir) = self.prev_ir.as_ref() else {
            return;
        };

        for node in ir.nodes.values() {
            if let Some(target) = node
                .composite
                .opacity
                .as_ref()
                .and_then(|value| value.animation_target)
            {
                self.compositor_animation_keys
                    .insert((target, AnimationPropertyId::Opacity));
            }
            if let Some(target) = node
                .composite
                .translate_x
                .as_ref()
                .and_then(|value| value.animation_target)
            {
                self.compositor_animation_keys
                    .insert((target, AnimationPropertyId::TranslateX));
            }
            if let Some(target) = node
                .composite
                .translate_y
                .as_ref()
                .and_then(|value| value.animation_target)
            {
                self.compositor_animation_keys
                    .insert((target, AnimationPropertyId::TranslateY));
            }
            if let Some(target) = node
                .composite
                .scale
                .as_ref()
                .and_then(|value| value.animation_target)
            {
                self.compositor_animation_keys
                    .insert((target, AnimationPropertyId::Scale));
            }
            if let Some(target) = node
                .composite
                .rotation
                .as_ref()
                .and_then(|value| value.animation_target)
            {
                self.compositor_animation_keys
                    .insert((target, AnimationPropertyId::Rotation));
            }
        }

        if let Some(root) = ir.root {
            let mut memo = HashMap::new();
            let _ = self.compute_runtime_dynamic_subtree(root, ir, &mut memo);
            self.runtime_dynamic_subtrees = memo;
        }
    }

    fn compute_runtime_dynamic_subtree(
        &self,
        node_id: NodeId,
        ir: &CoreIR,
        memo: &mut HashMap<NodeId, bool>,
    ) -> bool {
        if let Some(cached) = memo.get(&node_id) {
            return *cached;
        }

        let Some(node) = ir.nodes.get(&node_id) else {
            memo.insert(node_id, false);
            return false;
        };

        let mut dynamic = matches!(node.op, Op::Layout(LayoutOp::Scroll { .. }));
        dynamic |= node
            .composite
            .opacity
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some();
        dynamic |= node
            .composite
            .translate_x
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some();
        dynamic |= node
            .composite
            .translate_y
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some();
        dynamic |= node
            .composite
            .scale
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some();
        dynamic |= node
            .composite
            .rotation
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some();

        for child in &node.children {
            dynamic |= self.compute_runtime_dynamic_subtree(*child, ir, memo);
        }

        memo.insert(node_id, dynamic);
        dynamic
    }

    fn clear_render_caches(&mut self) {
        if render_trace_enabled() {
            eprintln!(
                "[pipeline] clear_render_caches layout_full={} dirty_layout={} retained_was_present={}",
                self.pending_layout_full,
                self.pending_layout_dirty.len(),
                self.retained_display_list.is_some()
            );
        }
        self.paint_cache.clear();
        self.boundary_cache.clear();
        self.retained_display_list = None;
        self.retained_dynamic_ops = RetainedDynamicOps::default();
    }

    fn patch_retained_display_list(
        &mut self,
        scroll_map: &ScrollStateMap,
        animation_map: &AnimationStateMap,
    ) {
        let Some(display_list) = self.retained_display_list.as_mut() else {
            return;
        };

        for binding in &self.retained_dynamic_ops.opacity {
            let alpha = resolve_scalar_value(&binding.scalar, animation_map, AnimationPropertyId::Opacity);
            if let Some(DisplayOp::OpacityLayer { alpha: current, .. }) =
                display_list.ops.get_mut(binding.op_index)
            {
                *current = alpha;
            }
        }

        for binding in &self.retained_dynamic_ops.transform {
            let translate_x = binding
                .translate_x
                .as_ref()
                .map(|scalar| resolve_scalar_value(scalar, animation_map, AnimationPropertyId::TranslateX))
                .unwrap_or(0.0);
            let translate_y = binding
                .translate_y
                .as_ref()
                .map(|scalar| resolve_scalar_value(scalar, animation_map, AnimationPropertyId::TranslateY))
                .unwrap_or(0.0);
            let scale = binding
                .scale
                .as_ref()
                .map(|scalar| resolve_scalar_value(scalar, animation_map, AnimationPropertyId::Scale))
                .unwrap_or(1.0);
            let rotation = binding
                .rotation
                .as_ref()
                .map(|scalar| resolve_scalar_value(scalar, animation_map, AnimationPropertyId::Rotation))
                .unwrap_or(0.0);

            if let Some(DisplayOp::Transform(matrix)) = display_list.ops.get_mut(binding.op_index) {
                *matrix = composite_transform_matrix(
                    binding.rect,
                    translate_x,
                    translate_y,
                    scale,
                    rotation,
                );
            }
        }

        for binding in &self.retained_dynamic_ops.scroll {
            let offset = scroll_map.get_offset(binding.node_id);
            let translation = match binding.direction {
                FlexDirection::Row => LayoutPoint::new(-offset, 0.0),
                FlexDirection::Column => LayoutPoint::new(0.0, -offset),
            };
            if let Some(DisplayOp::Translate(current)) = display_list.ops.get_mut(binding.op_index) {
                *current = translation;
            }
        }
    }

}

fn generate_display_list_recursive(
    node_id: NodeId,
    ir: &CoreIR,
    snapshot: &LayoutSnapshot,
    scroll_map: &ScrollStateMap,
    animation_map: &AnimationStateMap,
    paint_cache: &mut HashMap<NodeId, (u64, Vec<DisplayOp>)>,
    boundary_cache: &mut HashMap<NodeId, BoundaryCacheEntry>,
    runtime_dynamic_subtrees: &HashMap<NodeId, bool>,
    miss_count: &mut usize,
    hit_count: &mut usize,
    accumulated_offset: LayoutPoint,
    scene_cache_allowed: bool,
    visited: &mut HashSet<NodeId>,
) -> Vec<DisplayOp> {
    if !visited.insert(node_id) {
        return Vec::new();
    }

    generate_display_list_segment(
        node_id,
        ir,
        snapshot,
        scroll_map,
        animation_map,
        paint_cache,
        boundary_cache,
        runtime_dynamic_subtrees,
        miss_count,
        hit_count,
        accumulated_offset,
        scene_cache_allowed,
        visited,
    )
}

fn generate_display_list_segment(
    node_id: NodeId,
    ir: &CoreIR,
    snapshot: &LayoutSnapshot,
    scroll_map: &ScrollStateMap,
    animation_map: &AnimationStateMap,
    paint_cache: &mut HashMap<NodeId, (u64, Vec<DisplayOp>)>,
    boundary_cache: &mut HashMap<NodeId, BoundaryCacheEntry>,
    runtime_dynamic_subtrees: &HashMap<NodeId, bool>,
    miss_count: &mut usize,
    hit_count: &mut usize,
    accumulated_offset: LayoutPoint,
    scene_cache_allowed: bool,
    visited: &mut HashSet<NodeId>,
) -> Vec<DisplayOp> {

    let (Some(node), Some(geom)) = (ir.nodes.get(&node_id), snapshot.nodes.get(&node_id)) else {
        return Vec::new();
    };

    let translated_rect = translate_rect(geom.rect, accumulated_offset);
    // Treat every runtime-static subtree as a retained paint boundary.
    //
    // Explicit repaint boundaries still matter semantically, but for CPU cost
    // we cannot afford to walk and rebuild large static branches on every
    // compositor-only animation tick. If a subtree has no runtime dynamics
    // (scroll offset, compositor animation target, etc.), its display ops are
    // stable for a given resolved layout rect and can be reused directly.
    let can_use_boundary_cache = !runtime_dynamic_subtrees
        .get(&node_id)
        .copied()
        .unwrap_or(false);

    let scene_cache_key = boundary_hash(node, translated_rect);
    let can_cache_scene = scene_cache_allowed && can_use_boundary_cache && node.parent.is_some();
    if can_cache_scene {
        if let Some(entry) = boundary_cache.get(&node_id) {
            if entry.hash == scene_cache_key {
                *hit_count += 1;
                if entry.ops.is_empty() {
                    return Vec::new();
                }
                return vec![DisplayOp::CachedScene {
                    cache_key: scene_cache_key,
                    bounds: translated_rect,
                    list: Box::new(DisplayList {
                        ops: entry.ops.clone(),
                        bounds: translated_rect,
                    }),
                }];
            }
        }

        let raw_ops = generate_display_list_segment(
            node_id,
            ir,
            snapshot,
            scroll_map,
            animation_map,
            paint_cache,
            boundary_cache,
            runtime_dynamic_subtrees,
            miss_count,
            hit_count,
            accumulated_offset,
            false,
            visited,
        );
        if raw_ops.is_empty() {
            boundary_cache.insert(
                node_id,
                BoundaryCacheEntry {
                    hash: scene_cache_key,
                    ops: raw_ops,
                },
            );
            return Vec::new();
        }
        boundary_cache.insert(
            node_id,
            BoundaryCacheEntry {
                hash: scene_cache_key,
                ops: raw_ops.clone(),
            },
        );
        return vec![DisplayOp::CachedScene {
            cache_key: scene_cache_key,
            bounds: translated_rect,
            list: Box::new(DisplayList {
                ops: raw_ops,
                bounds: translated_rect,
            }),
        }];
    } else if can_use_boundary_cache {
        if let Some(entry) = boundary_cache.get(&node_id) {
            if entry.hash == scene_cache_key {
                *hit_count += 1;
                return entry.ops.clone();
            }
        }
    }

    let mut segment = Vec::new();
    let mut child_offset = accumulated_offset;
    let mut pushed_state = false;
    let mut clip_already_applied = false;

    let composite_opacity = resolve_composite_scalar(
        node.composite.opacity.as_ref(),
        animation_map,
        AnimationPropertyId::Opacity,
    );
    let composite_tx = resolve_composite_scalar(
        node.composite.translate_x.as_ref(),
        animation_map,
        AnimationPropertyId::TranslateX,
    );
    let composite_ty = resolve_composite_scalar(
        node.composite.translate_y.as_ref(),
        animation_map,
        AnimationPropertyId::TranslateY,
    );
    let composite_scale = resolve_composite_scalar(
        node.composite.scale.as_ref(),
        animation_map,
        AnimationPropertyId::Scale,
    )
    .unwrap_or(1.0);
    let composite_rotation = resolve_composite_scalar(
        node.composite.rotation.as_ref(),
        animation_map,
        AnimationPropertyId::Rotation,
    )
    .unwrap_or(0.0);

    let has_composite_transform = composite_tx.unwrap_or(0.0).abs() > 0.001
        || composite_ty.unwrap_or(0.0).abs() > 0.001
        || (composite_scale - 1.0).abs() > 0.001
        || composite_rotation.abs() > 0.001;
    let has_opacity_layer = composite_opacity
        .map(|value| (value - 1.0).abs() > 0.001)
        .unwrap_or(false);
    let needs_dynamic_opacity = node
        .composite
        .opacity
        .as_ref()
        .and_then(|value| value.animation_target)
        .is_some();
    let needs_dynamic_transform = node
        .composite
        .translate_x
        .as_ref()
        .and_then(|value| value.animation_target)
        .is_some()
        || node
            .composite
            .translate_y
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some()
        || node
            .composite
            .scale
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some()
        || node
            .composite
            .rotation
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some();
    let emit_opacity_layer = has_opacity_layer || needs_dynamic_opacity;
    let emit_composite_transform = has_composite_transform || needs_dynamic_transform;
    let has_runtime_clip = node.composite.clip_to_bounds;

    match &node.op {
        Op::Layout(LayoutOp::Scroll { direction, .. }) => {
            let offset = scroll_map.get_offset(node_id);
            segment.push(DisplayOp::Save);
            segment.push(DisplayOp::ClipRect(translated_rect));
            clip_already_applied = true;
            match direction {
                fission_ir::FlexDirection::Row => {
                    segment.push(DisplayOp::Translate(LayoutPoint::new(-offset, 0.0)));
                    child_offset = LayoutPoint::new(accumulated_offset.x - offset, accumulated_offset.y);
                }
                fission_ir::FlexDirection::Column => {
                    segment.push(DisplayOp::Translate(LayoutPoint::new(0.0, -offset)));
                    child_offset = LayoutPoint::new(accumulated_offset.x, accumulated_offset.y - offset);
                }
            }
            pushed_state = true;
        }
        Op::Layout(LayoutOp::Clip { .. }) => {
            segment.push(DisplayOp::Save);
            segment.push(DisplayOp::ClipRect(translated_rect));
            clip_already_applied = true;
            pushed_state = true;
        }
        Op::Layout(LayoutOp::Transform { transform }) => {
            segment.push(DisplayOp::Save);
            segment.push(DisplayOp::Transform(*transform));
            pushed_state = true;
        }
        _ => {
            if has_runtime_clip || emit_opacity_layer || emit_composite_transform {
                segment.push(DisplayOp::Save);
                pushed_state = true;
            }
            if has_runtime_clip {
                segment.push(DisplayOp::ClipRect(translated_rect));
                clip_already_applied = true;
            }
        }
    }

    if has_runtime_clip && !clip_already_applied {
        if !pushed_state {
            segment.push(DisplayOp::Save);
            pushed_state = true;
        }
        segment.push(DisplayOp::ClipRect(translated_rect));
    }

    if emit_opacity_layer {
        segment.push(DisplayOp::OpacityLayer {
            alpha: composite_opacity.unwrap_or(1.0),
            bounds: translated_rect,
        });
    }

    if emit_composite_transform {
        segment.push(DisplayOp::Transform(composite_transform_matrix(
            translated_rect,
            composite_tx.unwrap_or(0.0),
            composite_ty.unwrap_or(0.0),
            composite_scale,
            composite_rotation,
        )));
    }

    let local_hash = local_paint_hash(node);
    if let Some((cached_hash, cached_ops)) = paint_cache.get(&node_id) {
        if *cached_hash == local_hash {
            *hit_count += 1;
            segment.extend(cached_ops.clone());
        } else {
            *miss_count += 1;
            let ops = build_local_paint_ops(node_id, node, translated_rect);
            paint_cache.insert(node_id, (local_hash, ops.clone()));
            segment.extend(ops);
        }
    } else {
        *miss_count += 1;
        let ops = build_local_paint_ops(node_id, node, translated_rect);
        paint_cache.insert(node_id, (local_hash, ops.clone()));
        segment.extend(ops);
    }

    for child in &node.children {
        segment.extend(generate_display_list_recursive(
            *child,
            ir,
            snapshot,
            scroll_map,
            animation_map,
            paint_cache,
            boundary_cache,
            runtime_dynamic_subtrees,
            miss_count,
            hit_count,
            child_offset,
            scene_cache_allowed,
            visited,
        ));
    }

    if pushed_state {
        segment.push(DisplayOp::Restore);
    }

    if can_use_boundary_cache {
        boundary_cache.insert(
            node_id,
            BoundaryCacheEntry {
                hash: scene_cache_key,
                ops: segment.clone(),
            },
        );
    }

    segment
}

fn collect_retained_dynamic_ops(
    root: NodeId,
    ir: &CoreIR,
    snapshot: &LayoutSnapshot,
    scroll_map: &ScrollStateMap,
    animation_map: &AnimationStateMap,
    runtime_dynamic_subtrees: &HashMap<NodeId, bool>,
) -> RetainedDynamicOps {
    let mut bindings = RetainedDynamicOps::default();
    let mut visited = HashSet::new();
    let mut op_index = 0usize;
    collect_retained_dynamic_ops_recursive(
        root,
        ir,
        snapshot,
        scroll_map,
        animation_map,
        runtime_dynamic_subtrees,
        LayoutPoint::ZERO,
        true,
        &mut op_index,
        &mut bindings,
        &mut visited,
    );
    bindings
}

fn collect_retained_dynamic_ops_recursive(
    node_id: NodeId,
    ir: &CoreIR,
    snapshot: &LayoutSnapshot,
    scroll_map: &ScrollStateMap,
    animation_map: &AnimationStateMap,
    runtime_dynamic_subtrees: &HashMap<NodeId, bool>,
    accumulated_offset: LayoutPoint,
    scene_cache_allowed: bool,
    op_index: &mut usize,
    bindings: &mut RetainedDynamicOps,
    visited: &mut HashSet<NodeId>,
) {
    if !visited.insert(node_id) {
        return;
    }

    let (Some(node), Some(geom)) = (ir.nodes.get(&node_id), snapshot.nodes.get(&node_id)) else {
        return;
    };

    let translated_rect = translate_rect(geom.rect, accumulated_offset);
    let mut child_offset = accumulated_offset;
    let mut pushed_state = false;
    let mut clip_already_applied = false;
    let can_use_boundary_cache = !runtime_dynamic_subtrees
        .get(&node_id)
        .copied()
        .unwrap_or(false);
    let can_cache_scene = scene_cache_allowed && can_use_boundary_cache && node.parent.is_some();
    if can_cache_scene {
        *op_index += 1; // CachedScene
        return;
    }

    let composite_opacity = resolve_composite_scalar(
        node.composite.opacity.as_ref(),
        animation_map,
        AnimationPropertyId::Opacity,
    );
    let composite_tx = resolve_composite_scalar(
        node.composite.translate_x.as_ref(),
        animation_map,
        AnimationPropertyId::TranslateX,
    );
    let composite_ty = resolve_composite_scalar(
        node.composite.translate_y.as_ref(),
        animation_map,
        AnimationPropertyId::TranslateY,
    );
    let composite_scale = resolve_composite_scalar(
        node.composite.scale.as_ref(),
        animation_map,
        AnimationPropertyId::Scale,
    )
    .unwrap_or(1.0);
    let composite_rotation = resolve_composite_scalar(
        node.composite.rotation.as_ref(),
        animation_map,
        AnimationPropertyId::Rotation,
    )
    .unwrap_or(0.0);

    let has_composite_transform = composite_tx.unwrap_or(0.0).abs() > 0.001
        || composite_ty.unwrap_or(0.0).abs() > 0.001
        || (composite_scale - 1.0).abs() > 0.001
        || composite_rotation.abs() > 0.001;
    let has_opacity_layer = composite_opacity
        .map(|value| (value - 1.0).abs() > 0.001)
        .unwrap_or(false);
    let needs_dynamic_opacity = node
        .composite
        .opacity
        .as_ref()
        .and_then(|value| value.animation_target)
        .is_some();
    let needs_dynamic_transform = node
        .composite
        .translate_x
        .as_ref()
        .and_then(|value| value.animation_target)
        .is_some()
        || node
            .composite
            .translate_y
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some()
        || node
            .composite
            .scale
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some()
        || node
            .composite
            .rotation
            .as_ref()
            .and_then(|value| value.animation_target)
            .is_some();
    let emit_opacity_layer = has_opacity_layer || needs_dynamic_opacity;
    let emit_composite_transform = has_composite_transform || needs_dynamic_transform;
    let has_runtime_clip = node.composite.clip_to_bounds;

    match &node.op {
        Op::Layout(LayoutOp::Scroll { direction, .. }) => {
            let offset = scroll_map.get_offset(node_id);
            *op_index += 1; // Save
            *op_index += 1; // ClipRect
            clip_already_applied = true;
            bindings.scroll.push(ScrollBinding {
                op_index: *op_index,
                node_id,
                direction: *direction,
            });
            *op_index += 1; // Translate
            child_offset = match direction {
                FlexDirection::Row => LayoutPoint::new(accumulated_offset.x - offset, accumulated_offset.y),
                FlexDirection::Column => LayoutPoint::new(accumulated_offset.x, accumulated_offset.y - offset),
            };
            pushed_state = true;
        }
        Op::Layout(LayoutOp::Clip { .. }) => {
            *op_index += 1; // Save
            *op_index += 1; // ClipRect
            clip_already_applied = true;
            pushed_state = true;
        }
        Op::Layout(LayoutOp::Transform { .. }) => {
            *op_index += 1; // Save
            *op_index += 1; // Transform
            pushed_state = true;
        }
        _ => {
            if has_runtime_clip || emit_opacity_layer || emit_composite_transform {
                *op_index += 1; // Save
                pushed_state = true;
            }
            if has_runtime_clip {
                *op_index += 1; // ClipRect
                clip_already_applied = true;
            }
        }
    }

    if has_runtime_clip && !clip_already_applied {
        if !pushed_state {
            *op_index += 1; // Save
            pushed_state = true;
        }
        *op_index += 1; // ClipRect
    }

    if emit_opacity_layer {
        if let Some(scalar) = node.composite.opacity.as_ref() {
            if scalar.animation_target.is_some() {
                bindings.opacity.push(OpacityBinding {
                    op_index: *op_index,
                    scalar: scalar.clone(),
                });
            }
        }
        *op_index += 1; // OpacityLayer
    }

    if emit_composite_transform {
        if needs_dynamic_transform {
            bindings.transform.push(TransformBinding {
                op_index: *op_index,
                rect: translated_rect,
                translate_x: node.composite.translate_x.clone(),
                translate_y: node.composite.translate_y.clone(),
                scale: node.composite.scale.clone(),
                rotation: node.composite.rotation.clone(),
            });
        }
        *op_index += 1; // Transform
    }

    *op_index += local_paint_op_count(node);

    for child in &node.children {
        collect_retained_dynamic_ops_recursive(
            *child,
            ir,
            snapshot,
            scroll_map,
            animation_map,
            runtime_dynamic_subtrees,
            child_offset,
            scene_cache_allowed,
            op_index,
            bindings,
            visited,
        );
    }

    if pushed_state {
        *op_index += 1; // Restore
    }
}

fn local_paint_op_count(node: &fission_ir::CoreNode) -> usize {
    match &node.op {
        Op::Paint(
            fission_ir::PaintOp::DrawRect { .. }
            | fission_ir::PaintOp::DrawText { .. }
            | fission_ir::PaintOp::DrawRichText { .. }
            | fission_ir::PaintOp::DrawPath { .. }
            | fission_ir::PaintOp::DrawSvg { .. },
        ) => 1,
        _ => 0,
    }
}

fn push_video_surface(
    video_surfaces: &mut Vec<VideoSurfaceFrame>,
    widget_id: WidgetNodeId,
    rect: LayoutRect,
    video_map: &VideoStateMap,
) {
    if let Some(state) = video_map.states.get(&widget_id) {
        let surface_id = state.surface_id.unwrap_or(0);
        video_surfaces.push(VideoSurfaceFrame {
            widget_id,
            surface_id,
            rect,
        });
    }
}

fn collect_video_surfaces(
    node_id: NodeId,
    ir: &CoreIR,
    snapshot: &LayoutSnapshot,
    video_map: &VideoStateMap,
    scroll_map: &ScrollStateMap,
    accumulated_offset: LayoutPoint,
    video_surfaces: &mut Vec<VideoSurfaceFrame>,
    scene_3d_surfaces: &mut Vec<(WidgetNodeId, LayoutRect, Vec<u8>)>,
) {
    let mut visited = HashSet::new();
    collect_video_surfaces_with_visited(
        node_id,
        ir,
        snapshot,
        video_map,
        scroll_map,
        accumulated_offset,
        video_surfaces,
        scene_3d_surfaces,
        &mut visited,
    );
}

fn collect_video_surfaces_with_visited(
    node_id: NodeId,
    ir: &CoreIR,
    snapshot: &LayoutSnapshot,
    video_map: &VideoStateMap,
    scroll_map: &ScrollStateMap,
    accumulated_offset: LayoutPoint,
    video_surfaces: &mut Vec<VideoSurfaceFrame>,
    scene_3d_surfaces: &mut Vec<(WidgetNodeId, LayoutRect, Vec<u8>)>,
    visited: &mut HashSet<NodeId>,
) {
    if !visited.insert(node_id) {
        return;
    }
    if let (Some(node), Some(geom)) = (ir.nodes.get(&node_id), snapshot.nodes.get(&node_id)) {
        let mut child_offset = accumulated_offset;
        if let Op::Layout(LayoutOp::Scroll { direction, .. }) = &node.op {
            let offset = scroll_map.get_offset(node_id);
            child_offset = match direction {
                fission_ir::FlexDirection::Row => {
                    LayoutPoint::new(accumulated_offset.x - offset, accumulated_offset.y)
                }
                fission_ir::FlexDirection::Column => {
                    LayoutPoint::new(accumulated_offset.x, accumulated_offset.y - offset)
                }
            };
        }

        if let Op::Layout(LayoutOp::Embed {
            kind: EmbedKind::Video,
            widget_id,
            ..
        }) = &node.op
        {
            let translated_rect = translate_rect(geom.rect, accumulated_offset);
            push_video_surface(video_surfaces, *widget_id, translated_rect, video_map);
        } else if let Op::Layout(LayoutOp::Embed {
            kind: EmbedKind::Custom(payload),
            widget_id,
            ..
        }) = &node.op
        {
            let translated_rect = translate_rect(geom.rect, accumulated_offset);
            scene_3d_surfaces.push((*widget_id, translated_rect, payload.clone()));
        }

        for child in &node.children {
            collect_video_surfaces_with_visited(
                *child,
                ir,
                snapshot,
                video_map,
                scroll_map,
                child_offset,
                video_surfaces,
                scene_3d_surfaces,
                visited,
            );
        }
    }
}

fn local_paint_hash(node: &fission_ir::CoreNode) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    node.op.hash(&mut hasher);
    hasher.finish()
}

fn boundary_hash(node: &fission_ir::CoreNode, rect: LayoutRect) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    node.hash.hash(&mut hasher);
    rect.origin.x.to_bits().hash(&mut hasher);
    rect.origin.y.to_bits().hash(&mut hasher);
    rect.size.width.to_bits().hash(&mut hasher);
    rect.size.height.to_bits().hash(&mut hasher);
    hasher.finish()
}

fn build_local_paint_ops(
    node_id: NodeId,
    node: &fission_ir::CoreNode,
    rect: LayoutRect,
) -> Vec<DisplayOp> {
    let mut ops = Vec::new();
    match &node.op {
        Op::Paint(fission_ir::PaintOp::DrawRect {
            fill,
            stroke,
            corner_radius,
            shadow,
        }) => {
            ops.push(DisplayOp::DrawRect {
                rect,
                fill: fill.as_ref().map(map_fill),
                stroke: stroke.as_ref().map(map_stroke),
                corner_radius: *corner_radius,
                shadow: shadow.as_ref().map(|s| BoxShadow {
                    color: RenderColor {
                        r: s.color.r,
                        g: s.color.g,
                        b: s.color.b,
                        a: s.color.a,
                    },
                    blur_radius: s.blur_radius,
                    offset: s.offset,
                }),
                bounds: rect,
                node_id: Some(node_id),
            });
        }
        Op::Paint(fission_ir::PaintOp::DrawText {
            text,
            size,
            color,
            underline,
            caret_index,
        }) => {
            ops.push(DisplayOp::DrawText {
                text: text.clone(),
                position: rect.origin,
                size: *size,
                color: RenderColor {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: color.a,
                },
                bounds: rect,
                node_id: Some(node_id),
                underline: *underline,
                caret_index: *caret_index,
            });
        }
        Op::Paint(fission_ir::PaintOp::DrawRichText { runs, caret_index }) => {
            let render_runs = runs
                .iter()
                .map(|r| fission_render::TextRun {
                    text: r.text.clone(),
                    style: fission_render::TextStyle {
                        font_size: r.style.font_size,
                        color: RenderColor {
                            r: r.style.color.r,
                            g: r.style.color.g,
                            b: r.style.color.b,
                            a: r.style.color.a,
                        },
                        underline: r.style.underline,
                        background_color: r.style.background_color.map(|c| RenderColor {
                            r: c.r,
                            g: c.g,
                            b: c.b,
                            a: c.a,
                        }),
                    },
                })
                .collect();

            ops.push(DisplayOp::DrawRichText {
                runs: render_runs,
                position: rect.origin,
                bounds: rect,
                node_id: Some(node_id),
                caret_index: *caret_index,
            });
        }
        Op::Paint(fission_ir::PaintOp::DrawPath { path, fill, stroke }) => {
            ops.push(DisplayOp::DrawPath {
                path: path.clone(),
                fill: fill.as_ref().map(map_fill),
                stroke: stroke.as_ref().map(map_stroke),
                bounds: rect,
                node_id: Some(node_id),
            });
        }
        Op::Paint(fission_ir::PaintOp::DrawSvg { content, fill, stroke }) => {
            ops.push(DisplayOp::DrawSvg {
                content: content.clone(),
                fill: fill.as_ref().map(map_fill),
                stroke: stroke.as_ref().map(map_stroke),
                bounds: rect,
                node_id: Some(node_id),
            });
        }
        _ => {}
    }
    ops
}

fn resolve_composite_scalar(
    scalar: Option<&fission_ir::CompositeScalar>,
    animation_map: &AnimationStateMap,
    property: AnimationPropertyId,
) -> Option<f32> {
    let scalar = scalar?;
    Some(resolve_scalar_value(scalar, animation_map, property))
}

fn resolve_scalar_value(
    scalar: &fission_ir::CompositeScalar,
    animation_map: &AnimationStateMap,
    property: AnimationPropertyId,
) -> f32 {
    scalar
        .animation_target
        .and_then(|target| animation_map.values.get(&(target, property)).copied())
        .unwrap_or(scalar.base)
}

fn composite_transform_matrix(
    rect: LayoutRect,
    translate_x: f32,
    translate_y: f32,
    scale: f32,
    rotation: f32,
) -> [f32; 16] {
    let center_x = rect.origin.x + rect.size.width * 0.5;
    let center_y = rect.origin.y + rect.size.height * 0.5;

    let to_center = translation_matrix(center_x, center_y);
    let from_center = translation_matrix(-center_x, -center_y);
    let scale_matrix = scale_matrix(scale);
    let rotation_matrix = rotation_z_matrix(rotation);
    let animated_translate = translation_matrix(translate_x, translate_y);

    multiply_matrix(
        animated_translate,
        multiply_matrix(
            to_center,
            multiply_matrix(rotation_matrix, multiply_matrix(scale_matrix, from_center)),
        ),
    )
}

fn translation_matrix(tx: f32, ty: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        tx, ty, 0.0, 1.0,
    ]
}

fn scale_matrix(scale: f32) -> [f32; 16] {
    [
        scale, 0.0, 0.0, 0.0,
        0.0, scale, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn rotation_z_matrix(radians: f32) -> [f32; 16] {
    let sin = radians.sin();
    let cos = radians.cos();
    [
        cos, sin, 0.0, 0.0,
        -sin, cos, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn multiply_matrix(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut out = [0.0; 16];
    for row in 0..4 {
        for col in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[row * 4 + k] * b[k * 4 + col];
            }
            out[row * 4 + col] = sum;
        }
    }
    out
}

fn scroll_offsets_changed(prev: &HashMap<NodeId, u32>, scroll_map: &ScrollStateMap) -> bool {
    if prev.len() != scroll_map.offsets.len() {
        return true;
    }

    scroll_map
        .offsets
        .iter()
        .any(|(id, offset)| prev.get(id).copied() != Some(offset.to_bits()))
}

impl SnapshotProvider for Pipeline {
    fn snapshot(&self, kind: SnapshotKind) -> Option<SnapshotBlob> {
        match kind {
            SnapshotKind::Layout => self.last_snapshot.as_ref().and_then(|snap| {
                serde_json::to_string_pretty(snap)
                    .ok()
                    .map(|json| SnapshotBlob { kind, json })
            }),
        }
    }
}

fn map_fill(f: &fission_ir::op::Fill) -> Fill {
    match f {
        fission_ir::op::Fill::Solid(c) => Fill::Solid(RenderColor {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }),
        fission_ir::op::Fill::LinearGradient { start, end, stops } => Fill::LinearGradient {
            start: *start,
            end: *end,
            stops: stops
                .iter()
                .map(|(o, c)| (*o, RenderColor {
                    r: c.r,
                    g: c.g,
                    b: c.b,
                    a: c.a,
                }))
                .collect(),
        },
        fission_ir::op::Fill::RadialGradient { center, radius, stops } => Fill::RadialGradient {
            center: *center,
            radius: *radius,
            stops: stops
                .iter()
                .map(|(o, c)| (*o, RenderColor {
                    r: c.r,
                    g: c.g,
                    b: c.b,
                    a: c.a,
                }))
                .collect(),
        },
    }
}

fn map_stroke(s: &fission_ir::op::Stroke) -> Stroke {
    Stroke {
        fill: map_fill(&s.fill),
        width: s.width,
        dash_array: s.dash_array.clone(),
        line_cap: match s.line_cap {
            fission_ir::op::LineCap::Butt => fission_render::LineCap::Butt,
            fission_ir::op::LineCap::Round => fission_render::LineCap::Round,
            fission_ir::op::LineCap::Square => fission_render::LineCap::Square,
        },
        line_join: match s.line_join {
            fission_ir::op::LineJoin::Miter => fission_render::LineJoin::Miter,
            fission_ir::op::LineJoin::Round => fission_render::LineJoin::Round,
            fission_ir::op::LineJoin::Bevel => fission_render::LineJoin::Bevel,
        },
    }
}

fn translate_rect(rect: LayoutRect, offset: LayoutPoint) -> LayoutRect {
    LayoutRect {
        origin: LayoutPoint::new(rect.origin.x + offset.x, rect.origin.y + offset.y),
        size: rect.size,
    }
}

#[cfg(test)]
mod tests {
    use super::{scroll_offsets_changed, InvalidationSet, Pipeline};
    use fission_core::env::Env;
    use fission_core::registry::AnimationPropertyId;
    use fission_ir::{CompositeScalar, CompositeStyle, CoreIR, LayoutOp, NodeId, Op, WidgetNodeId};
    use fission_core::ScrollStateMap;
    use std::collections::HashMap;

    #[test]
    fn unchanged_scroll_offsets_do_not_invalidate_cache() {
        let id = NodeId::derived(1, &[0]);
        let mut prev = HashMap::new();
        prev.insert(id, 12.5f32.to_bits());
        let mut scroll = ScrollStateMap::default();
        scroll.set_offset(id, 12.5);
        assert!(!scroll_offsets_changed(&prev, &scroll));
    }

    #[test]
    fn changed_scroll_offsets_invalidate_cache() {
        let id = NodeId::derived(2, &[0]);
        let mut prev = HashMap::new();
        prev.insert(id, 0.0f32.to_bits());
        let mut scroll = ScrollStateMap::default();
        scroll.set_offset(id, 4.0);
        assert!(scroll_offsets_changed(&prev, &scroll));
    }

    #[test]
    fn compositor_bound_opacity_animation_is_composite_only() {
        let mut ir = CoreIR::new();
        let child = NodeId::derived(10, &[1]);
        let root = NodeId::derived(10, &[0]);
        ir.add_node(child, Op::Layout(LayoutOp::AbsoluteFill), vec![]);
        ir.add_node_with_composite(
            root,
            Op::Structural(fission_ir::StructuralOp::Group { stable_hash: 1 }),
            CompositeStyle {
                opacity: Some(CompositeScalar::new(0.0).animated(WidgetNodeId::explicit("fade"))),
                ..Default::default()
            },
            vec![child],
        );
        ir.set_root(root);

        let mut pipeline = Pipeline::new();
        pipeline.replace_ir(ir, &Env::default());
        let invalidation = pipeline.classify_animation_updates(&[(
            WidgetNodeId::explicit("fade"),
            AnimationPropertyId::Opacity,
        )]);
        assert_eq!(
            invalidation,
            InvalidationSet {
                build: false,
                layout: false,
                paint: false,
                composite: true,
            }
        );
    }

    #[test]
    fn unbound_custom_animation_requires_build() {
        let pipeline = Pipeline::new();
        let invalidation = pipeline.classify_animation_updates(&[(
            WidgetNodeId::explicit("custom"),
            AnimationPropertyId::custom("phase"),
        )]);
        assert!(invalidation.build);
        assert!(invalidation.layout);
    }
}
