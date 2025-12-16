use anyhow::{anyhow, Result};
use fission_diagnostics::prelude as diag;
use downcast_rs::Downcast;
use fission_ir::CoreIR;
use lazy_static::lazy_static;
use serde_json;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;
use std::sync::Arc;

pub mod action;
pub mod diff;
pub mod env;
pub mod event;
pub mod hit_test;
pub mod lowering;
pub mod registry;
pub mod time;
pub mod ui;
pub mod view;

use crate::action::video::{
    VideoPause, VideoPlay, VideoSeek, VideoSetMuted, VideoSetRate, VideoSetVolume, VideoStop,
};
use crate::env::ActiveAnimation;
pub use action::{Action, ActionEnvelope, ActionId, AppState};
pub use env::{Env, InteractionStateMap, RuntimeState, ScrollStateMap};
pub use event::{InputEvent, KeyCode, KeyEvent, LifecycleEvent, PointerButton, PointerEvent};
pub use fission_ir::op;
pub use fission_ir::{EmbedKind, NodeId, Op, WidgetNodeId};
pub use fission_layout::{
    FlexDirection, LayoutOp, LayoutPoint, LayoutRect, LayoutSize, LayoutSnapshot, LayoutUnit, TextMeasurer,
};
use hit_test::{find_next_focus_node, hit_test, hit_test_with_scroll};
pub use lowering::{LoweringContext, NodeBuilder};
pub use registry::{
    ActionRegistry, AnimationPropertyId, AnimationRequest, AnimationStartValue, BuildCtx, Handler,
    VideoRegistration,
};
pub use time::{Clock, CurrentTime};
pub use ui::{Button, Column, CustomNode, Lower, LowerDyn, Node, Row, Text};
pub use view::{Selector, View, Widget};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tick {
    pub dt: CurrentTime,
}

impl Action for Tick {
    fn static_id() -> ActionId {
        *TICK_ACTION_ID
    }
}

lazy_static! {
    pub static ref TICK_ACTION_ID: ActionId = ActionId::from_name("fission_core::Tick");
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdvanceTo {
    pub time: CurrentTime,
}

impl Action for AdvanceTo {
    fn static_id() -> ActionId {
        *ADVANCE_TO_ACTION_ID
    }
}

lazy_static! {
    pub static ref ADVANCE_TO_ACTION_ID: ActionId = ActionId::from_name("fission_core::AdvanceTo");
}

pub type BoxedReducer = Box<
    dyn FnMut(&mut HashMap<TypeId, Box<dyn AppState>>, &ActionEnvelope, NodeId) -> Result<()>
        + Send
        + Sync,
>;

pub struct Runtime {
    reducers: HashMap<ActionId, Vec<BoxedReducer>>,
    app_states: HashMap<TypeId, Box<dyn AppState>>,
    pub runtime_state: RuntimeState,
    pub measurer: Option<Arc<dyn TextMeasurer>>,
}

impl Default for Runtime {
    fn default() -> Self {
        let mut runtime = Self {
            reducers: HashMap::new(),
            app_states: HashMap::new(),
            runtime_state: RuntimeState::default(),
            measurer: None,
        };

        runtime
            .add_app_state(Box::new(Clock::default()))
            .expect("Failed to add Clock state");

        runtime.register_base_reducers();

        runtime
    }
}

impl Runtime {
    pub fn with_measurer(mut self, measurer: Arc<dyn TextMeasurer>) -> Self {
        self.measurer = Some(measurer);
        self
    }

    fn approx_text_width(s: &str, font_size: f32) -> f32 {
        (s.chars().count() as f32) * font_size * 0.6
    }

    pub fn caret_from_point_in_text(&self, value: &str, font_size: f32, viewport_x: f32, viewport_w: f32, content_w: f32, scroll_offset: f32, point_x: f32) -> usize {
        let mut local_x = (point_x - viewport_x) + scroll_offset;
        if local_x <= 0.0 { return 0; }
        let max_x = content_w.max(viewport_w);
        if local_x >= max_x { return value.len(); }

        if let Some(measurer) = &self.measurer {
            let mut last_idx = 0;
            let mut last_w = 0.0;
            
            for (idx, _) in value.grapheme_indices(true) {
                // Optimization: skip measurement for index 0
                let w = if idx == 0 { 0.0 } else {
                    measurer.measure(&value[..idx], font_size, None).0
                };
                
                if w > local_x {
                    // Check midpoint between last char end (last_w) and this char end (w)
                    // If local_x is closer to last_w, pick last_idx.
                    if local_x < (last_w + w) / 2.0 {
                        return last_idx;
                    } else {
                        return idx;
                    }
                }
                last_idx = idx;
                last_w = w;
            }
            // Check last segment
            let (total_w, _) = measurer.measure(value, font_size, None);
            if local_x < (last_w + total_w) / 2.0 {
                return last_idx;
            } else {
                return value.len();
            }
        } else {
            // Fallback to approx if no measurer
            let mut acc = 0.0f32;
            let mut last_index = 0usize;
            for (idx, g) in value.grapheme_indices(true) {
                let w = Self::approx_text_width(g, font_size);
                if acc + w * 0.5 >= local_x { return idx; }
                acc += w;
                last_index = idx;
            }
            value.len()
        }
    }

    fn clamp_caret_to_value(value: &str, caret: usize) -> usize {
        if caret > value.len() { value.len() } else { caret }
    }

    fn prev_grapheme_boundary(value: &str, idx: usize) -> usize {
        let mut last = 0;
        for (pos, _) in value.grapheme_indices(true) {
            if pos >= idx { break; }
            last = pos;
        }
        last
    }

    fn next_grapheme_boundary(value: &str, idx: usize) -> usize {
        for (pos, _) in value.grapheme_indices(true) {
            if pos > idx { return pos; }
        }
        value.len()
    }

    fn delete_prev_grapheme(value: &str, caret: usize, sel: Option<(usize,usize)>) -> (String, usize) {
        if let Some((a,b)) = sel {
            let (s,e) = if a<=b {(a,b)} else {(b,a)};
            let mut out = String::with_capacity(value.len() - (e-s));
            out.push_str(&value[..s]);
            out.push_str(&value[e..]);
            return (out, s);
        }
        let at = caret.min(value.len());
        if at == 0 { return (value.to_string(), 0); }
        let prev = Self::prev_grapheme_boundary(value, at);
        let mut out = String::with_capacity(value.len() - (at-prev));
        out.push_str(&value[..prev]);
        out.push_str(&value[at..]);
        (out, prev)
    }

    fn prev_word_boundary(value: &str, idx: usize) -> usize {
        let mut at = idx.min(value.len());
        while at > 0 {
            let prev = Self::prev_grapheme_boundary(value, at);
            let ch = value[prev..].chars().next().unwrap_or('\0');
            if !ch.is_whitespace() { at = prev; break; }
            at = prev;
        }
        while at > 0 {
            let prev = Self::prev_grapheme_boundary(value, at);
            let ch = value[prev..].chars().next().unwrap_or('\0');
            if ch.is_alphanumeric() || ch == '_' { at = prev; } else { break; }
        }
        at
    }

    fn next_word_boundary(value: &str, idx: usize) -> usize {
        let mut at = idx.min(value.len());
        while at < value.len() {
            let next = Self::next_grapheme_boundary(value, at);
            let ch = value[at..].chars().next().unwrap_or('\0');
            if !ch.is_whitespace() { at = next; break; }
            at = next;
        }
        while at < value.len() {
            let next = Self::next_grapheme_boundary(value, at);
            let ch = value[at..].chars().next().unwrap_or('\0');
            if ch.is_alphanumeric() || ch == '_' { at = next; } else { break; }
            at = next;
        }
        at
    }

    fn insert_text(value: &str, caret: usize, sel: Option<(usize,usize)>, text: &str) -> (String, usize) {
        let (s,e) = sel.map(|(a,b)| if a<=b {(a,b)} else {(b,a)}).unwrap_or((caret, caret));
        let mut out = String::with_capacity(value.len() - (e-s) + text.len());
        out.push_str(&value[..s]);
        out.push_str(text);
        out.push_str(&value[e..]);
        (out, s + text.len())
    }

    fn find_scroll_row_and_text(ir: &CoreIR, root: NodeId) -> Option<(NodeId, NodeId)> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if let Some(n) = ir.nodes.get(&id) {
                if let Op::Layout(op::LayoutOp::Scroll { direction, .. }) = &n.op {
                    if *direction == op::FlexDirection::Row {
                        let mut q = vec![id];
                        while let Some(cid) = q.pop() {
                            if let Some(cn) = ir.nodes.get(&cid) {
                                if let Op::Paint(fission_ir::PaintOp::DrawText { .. }) = cn.op {
                                    return Some((id, cid));
                                }
                                for &gc in &cn.children { q.push(gc); }
                            }
                        }
                        return None;
                    }
                }
                for &c in &n.children { stack.push(c); }
            }
        }
        None
    }

    fn find_caret_in_scroll(ir: &CoreIR, scroll_id: NodeId) -> Option<NodeId> {
        let mut q = vec![scroll_id];
        while let Some(id) = q.pop() {
            if let Some(n) = ir.nodes.get(&id) {
                if let Op::Layout(op::LayoutOp::Box { width: Some(w), .. }) = &n.op {
                    if (*w - 2.0).abs() < 0.01 {
                        let mut has_paint = false;
                        for &cid in &n.children {
                            if let Some(cn) = ir.nodes.get(&cid) {
                                if let Op::Paint(fission_ir::PaintOp::DrawRect { .. }) = cn.op {
                                    has_paint = true;
                                    break;
                                }
                            }
                        }
                        if has_paint { return Some(id); }
                    }
                }
                for &c in &n.children { q.push(c); }
            }
        }
        None
    }

    fn auto_scroll_textinput(&mut self, text_root: NodeId, ir: &CoreIR, layout: &LayoutSnapshot) {
        if let Some((scroll_id, text_id)) = Self::find_scroll_row_and_text(ir, text_root) {
            if let Some(scroll_geom) = layout.get_node_geometry(scroll_id) {
                let viewport_x = scroll_geom.rect.origin.x;
                let viewport_w = scroll_geom.rect.size.width;
                let content_w = scroll_geom.content_size.width.max(viewport_w);
                let caret_left = if let Some(caret_id) = Self::find_caret_in_scroll(ir, scroll_id) {
                    layout.get_node_geometry(caret_id).map(|g| g.rect.origin.x).unwrap_or_else(|| {
                        layout.get_node_geometry(text_id).map(|g| g.rect.origin.x + g.rect.size.width).unwrap_or(viewport_x)
                    })
                } else {
                    layout.get_node_geometry(text_id).map(|g| g.rect.origin.x + g.rect.size.width).unwrap_or(viewport_x)
                };
                let caret_width = 2.0f32;
                let caret_right = caret_left + caret_width;
                let mut offset = self.runtime_state.scroll.get_offset(scroll_id);
                let margin_left = 2.0f32;
                let margin_right = 3.0f32; 
                let visible_left = (caret_left - offset) - viewport_x;
                let visible_right = (caret_right - offset) - viewport_x;
                let offset_before = offset;
                if visible_right > (viewport_w - margin_right) {
                    offset = (caret_right - (viewport_x + viewport_w - margin_right)).max(0.0);
                } else if visible_left < margin_left {
                    offset = (caret_left - (viewport_x + margin_left)).max(0.0);
                }
                let max_offset = (content_w - viewport_w).max(0.0);
                offset = offset.clamp(0.0, max_offset);
                self.runtime_state.scroll.set_offset(scroll_id, offset);

                let text_len = if let Some(node) = ir.nodes.get(&text_id) {
                    if let Op::Paint(fission_ir::PaintOp::DrawText { text, .. }) = &node.op { text.len() as u32 } else { 0 }
                } else { 0 };
                let line_h = layout.get_node_geometry(text_id).map(|g| g.rect.size.height).unwrap_or(0.0);
                let (caret_left_geom, caret_gap) = if let Some(caret_id) = Self::find_caret_in_scroll(ir, scroll_id) {
                    if let Some(cg) = layout.get_node_geometry(caret_id) {
                        let left = cg.rect.origin.x;
                        (left, left - caret_left)
                    } else { (0.0, 0.0) }
                } else { (0.0, 0.0) };
                diag::emit(
                    diag::DiagCategory::Layout,
                    diag::DiagLevel::Debug,
                    diag::DiagEventKind::TextInputAutoScroll {
                        scroll_id: scroll_id.as_u128(),
                        text_id: text_id.as_u128(),
                        text_len,
                        measured_w: layout.get_node_geometry(text_id).map(|g| g.rect.size.width).unwrap_or(0.0),
                        line_h,
                        viewport_x,
                        viewport_w,
                        content_w,
                        caret_abs_x: caret_left,
                        offset_before,
                        offset_after: offset,
                    },
                );
                if caret_gap.abs() > 0.5 {
                    diag::emit(
                        diag::DiagCategory::Input,
                        diag::DiagLevel::Debug,
                        diag::DiagEventKind::InputEvent { kind: format!("caret_gap: {:.2} (caret_left={:.2} caret_abs_x={:.2})", caret_gap, caret_left_geom, caret_left), target: Some(scroll_id.as_u128()), position: None },
                    );
                }
            }
        }
    }
    pub fn register_base_reducers(&mut self) {
        self.register_reducer::<Clock>(
            *TICK_ACTION_ID,
            |state: &mut Clock, action: &ActionEnvelope, _target| {
                let tick_action: Tick = serde_json::from_slice(&action.payload)
                    .map_err(|e| anyhow!("Failed to deserialize Tick: {}", e))?;
                state.advance_by(tick_action.dt)
            },
        )
        .expect("Failed to register Tick reducer");

        self.register_reducer::<Clock>(
            *ADVANCE_TO_ACTION_ID,
            |state: &mut Clock, action: &ActionEnvelope, _target| {
                let advance_action: AdvanceTo = serde_json::from_slice(&action.payload)
                    .map_err(|e| anyhow!("Failed to deserialize AdvanceTo: {}", e))?;
                state.set_to(advance_action.time)
            },
        )
        .expect("Failed to register AdvanceTo reducer");
    }

    pub fn clear_reducers(&mut self) {
        self.reducers.clear();
        self.register_base_reducers();
    }

    pub fn absorb_registry<S: AppState>(&mut self, registry: ActionRegistry<S>) {
        let new_reducers = registry.into_runtime_reducers();
        for (id, mut list) in new_reducers {
            self.reducers.entry(id).or_default().append(&mut list);
        }
    }

    pub fn clock(&self) -> &Clock {
        self.get_app_state::<Clock>()
            .expect("Clock state must always be present")
    }

    pub fn get_app_state<S: AppState + 'static>(&self) -> Option<&S> {
        self.app_states
            .get(&TypeId::of::<S>())
            .and_then(|s_box| s_box.downcast_ref::<S>())
    }

    pub fn get_app_state_mut<S: AppState + 'static>(&mut self) -> Option<&mut S> {
        self.app_states
            .get_mut(&TypeId::of::<S>())
            .and_then(|s_box| s_box.downcast_mut::<S>())
    }

    pub fn add_app_state<S: AppState + 'static>(&mut self, state: Box<S>) -> Result<()> {
        let type_id = TypeId::of::<S>();
        if self.app_states.insert(type_id, state).is_some() {
            anyhow::bail!("App state of this type already registered.");
        }
        Ok(())
    }

    pub fn register_reducer<S: AppState + 'static>(
        &mut self,
        action_id: ActionId,
        reducer_fn: fn(&mut S, &ActionEnvelope, NodeId) -> Result<()>,
    ) -> Result<()> {
        let state_type_id = TypeId::of::<S>();

        let boxed_reducer: BoxedReducer = Box::new(
            move |app_states: &mut HashMap<TypeId, Box<dyn AppState>>,
                  action: &ActionEnvelope,
                  target: NodeId|
                  -> Result<()> {
                if let Some(state_box) = app_states.get_mut(&state_type_id) {
                    let concrete_state = state_box.downcast_mut::<S>().ok_or_else(|| {
                        anyhow!("Failed to downcast AppState to concrete type for reducer")
                    })?;
                    reducer_fn(concrete_state, action, target)
                } else {
                    anyhow::bail!("Target AppState for reducer not found in runtime.");
                }
            },
        );

        self.reducers
            .entry(action_id)
            .or_default()
            .push(boxed_reducer);
        Ok(())
    }

    pub fn dispatch(&mut self, action: ActionEnvelope, target: NodeId) -> Result<()> {
        diag::emit(
            diag::DiagCategory::Input,
            diag::DiagLevel::Debug,
            diag::DiagEventKind::InputEvent { kind: "dispatch_start".into(), target: Some(target.as_u128()), position: None },
        );
        if action.id == VideoPlay::static_id() {
            let cmd: VideoPlay = serde_json::from_slice(&action.payload)
                .map_err(|e| anyhow!("Failed to deserialize VideoPlay: {}", e))?;
            if let Some(video_state) = self.runtime_state.video.states.get_mut(&cmd.target) {
                video_state.status = env::VideoStatus::Playing;
            }
            return Ok(());
        }

        if action.id == VideoPause::static_id() {
            let cmd: VideoPause = serde_json::from_slice(&action.payload)
                .map_err(|e| anyhow!("Failed to deserialize VideoPause: {}", e))?;
            if let Some(video_state) = self.runtime_state.video.states.get_mut(&cmd.target) {
                video_state.status = env::VideoStatus::Paused;
            }
            return Ok(());
        }

        if action.id == VideoStop::static_id() {
            let cmd: VideoStop = serde_json::from_slice(&action.payload)
                .map_err(|e| anyhow!("Failed to deserialize VideoStop: {}", e))?;
            if let Some(video_state) = self.runtime_state.video.states.get_mut(&cmd.target) {
                video_state.status = env::VideoStatus::Stopped;
                video_state.position_ms = 0;
                video_state.pending_seek = Some(0);
            }
            return Ok(());
        }

        if action.id == VideoSeek::static_id() {
            let cmd: VideoSeek = serde_json::from_slice(&action.payload)
                .map_err(|e| anyhow!("Failed to deserialize VideoSeek: {}", e))?;
            if let Some(video_state) = self.runtime_state.video.states.get_mut(&cmd.target) {
                video_state.position_ms = cmd.position_ms;
                video_state.pending_seek = Some(cmd.position_ms);
            }
            return Ok(());
        }

        if action.id == VideoSetRate::static_id() {
            let cmd: VideoSetRate = serde_json::from_slice(&action.payload)
                .map_err(|e| anyhow!("Failed to deserialize VideoSetRate: {}", e))?;
            if let Some(video_state) = self.runtime_state.video.states.get_mut(&cmd.target) {
                video_state.rate = cmd.rate;
            }
            return Ok(());
        }

        if action.id == VideoSetVolume::static_id() {
            let cmd: VideoSetVolume = serde_json::from_slice(&action.payload)
                .map_err(|e| anyhow!("Failed to deserialize VideoSetVolume: {}", e))?;
            if let Some(video_state) = self.runtime_state.video.states.get_mut(&cmd.target) {
                video_state.volume = cmd.volume.clamp(0.0, 1.0);
            }
            return Ok(());
        }

        if action.id == VideoSetMuted::static_id() {
            let cmd: VideoSetMuted = serde_json::from_slice(&action.payload)
                .map_err(|e| anyhow!("Failed to deserialize VideoSetMuted: {}", e))?;
            if let Some(video_state) = self.runtime_state.video.states.get_mut(&cmd.target) {
                video_state.muted = cmd.muted;
            }
            return Ok(());
        }

        let action_id = action.id;
        if let Some(reducers) = self.reducers.get_mut(&action_id) {
            diag::emit(
                diag::DiagCategory::Input,
                diag::DiagLevel::Debug,
                diag::DiagEventKind::InputEvent { kind: format!("reducers:{}", reducers.len()), target: Some(target.as_u128()), position: None },
            );
            let mut temp_reducers: Vec<BoxedReducer> = reducers.drain(..).collect();

            for reducer_wrapper in temp_reducers.iter_mut() {
                reducer_wrapper(&mut self.app_states, &action, target)?;
            }
            reducers.extend(temp_reducers);
        }
        diag::emit(
            diag::DiagCategory::Input,
            diag::DiagLevel::Debug,
            diag::DiagEventKind::InputEvent { kind: "dispatch_end".into(), target: Some(target.as_u128()), position: None },
        );
        Ok(())
    }

    pub fn tick(&mut self, dt: CurrentTime) -> Result<()> {
        let action = Tick { dt };
        let envelope: ActionEnvelope = action.into();
        self.dispatch(envelope, NodeId::derived(0, &[0]))?;

        let current_time = self.clock().current_time();

        let mut finished = Vec::new();
        for ((target, property), anim) in self.runtime_state.animation.active.iter_mut() {
            let elapsed = current_time.saturating_sub(anim.start_time);
            let progress = if anim.duration == 0 {
                1.0
            } else {
                (elapsed as f32 / anim.duration as f32).clamp(0.0, 1.0)
            };
            let value = anim.start_value + (anim.end_value - anim.start_value) * progress;
            self.runtime_state
                .animation
                .values
                .insert((*target, property.clone()), value);
            if progress >= 1.0 {
                finished.push((*target, property.clone()));
            }
        }

        for key in finished {
            self.runtime_state.animation.active.remove(&key);
        }

        Ok(())
    }

    pub fn enqueue_animation(&mut self, target: WidgetNodeId, request: AnimationRequest) {
        let key = (target, request.property.clone());
        let current_value = self
            .runtime_state
            .animation
            .values
            .get(&key)
            .copied()
            .unwrap_or_else(|| request.property.default_value());
        let start_value = match request.from {
            AnimationStartValue::Explicit(v) => v,
            AnimationStartValue::Current => current_value,
        };

        let anim = ActiveAnimation {
            target,
            property: request.property.clone(),
            start_value,
            end_value: request.to,
            start_time: self.clock().current_time(),
            duration: request.duration_ms,
        };

        self.runtime_state
            .animation
            .values
            .insert(key.clone(), start_value);
        self.runtime_state.animation.active.insert(key, anim);
    }

    pub fn sync_video_nodes(&mut self, registrations: &[VideoRegistration]) {
        let mut seen: HashSet<WidgetNodeId> = HashSet::new();

        for reg in registrations {
            seen.insert(reg.node_id);
            let entry = self
                .runtime_state
                .video
                .states
                .entry(reg.node_id)
                .or_insert_with(env::VideoState::default);
            entry.asset_source = reg.source.clone();
            entry.looped = reg.loop_playback;
            if reg.autoplay && entry.status == env::VideoStatus::Stopped {
                entry.status = env::VideoStatus::Playing;
            }
        }

        self.runtime_state
            .video
            .states
            .retain(|node_id, _| seen.contains(node_id));
    }

    pub fn handle_input(
        &mut self,
        event: InputEvent,
        ir: &CoreIR,
        layout: &LayoutSnapshot,
    ) -> Result<()> {
        match event {
            InputEvent::Pointer(PointerEvent::Scroll { point, delta }) => {
                if let Some(hit_node_id) =
                    hit_test_with_scroll(ir, layout, &self.runtime_state.scroll, point)
                {
                    let mut current_id = Some(hit_node_id);
                    while let Some(node_id) = current_id {
                        if let Some(node) = ir.nodes.get(&node_id) {
                            if let Op::Layout(op::LayoutOp::Scroll { direction, .. }) = &node.op {
                                let current_offset = self.runtime_state.scroll.get_offset(node_id);
                                let delta_val = match direction { op::FlexDirection::Row => delta.x, op::FlexDirection::Column => delta.y };
                                let mut new_offset = current_offset + delta_val;

                                // Clamping logic
                                if let Some(geom) = layout.get_node_geometry(node_id) {
                                    let max_offset = if matches!(direction, op::FlexDirection::Row) {
                                        (geom.content_size.width - geom.rect.width()).max(0.0)
                                    } else {
                                        (geom.content_size.height - geom.rect.height()).max(0.0)
                                    };
                                    new_offset = new_offset.clamp(0.0, max_offset);
                                }

                                self.runtime_state.scroll.set_offset(node_id, new_offset);
                                break;
                            }
                            current_id = node.parent;
                        } else {
                            break;
                        }
                    }
                }
            }
            InputEvent::Keyboard(KeyEvent::Down {
                key_code,
                modifiers,
            }) => match key_code {
                KeyCode::Tab => {
                    let reverse = (modifiers & 1) != 0;
                    let next = find_next_focus_node(ir, self.runtime_state.interaction.focused, reverse);
                    if next != self.runtime_state.interaction.focused {
                        self.runtime_state.ime_preedit = None;
                    }
                    self.runtime_state.interaction.set_focused(next);
                }
                KeyCode::Space => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let current_text = semantics.value.as_deref().unwrap_or("");
                                        let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                        let caret = Self::clamp_caret_to_value(current_text, st.caret);
                                        let sel = if st.caret != st.anchor { Some((st.anchor, st.caret)) } else { None };
                                        let (new_text, new_caret) = Self::insert_text(current_text, caret, sel, " ");
                                        
                                        if let Some(action_entry) = semantics.actions.entries.first() {
                                            let payload = serde_json::to_vec(&new_text).unwrap();
                                            let envelope = ActionEnvelope {
                                                id: ActionId::from_u128(action_entry.action_id),
                                                payload,
                                            };
                                            let res = self.dispatch(envelope, node_id);
                                            let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                            st.caret = new_caret; st.anchor = new_caret;
                                            self.auto_scroll_textinput(node_id, ir, layout);
                                            return res;
                                        }
                                        return Ok(());
                                    } else if let Some(action_entry) = semantics.actions.entries.first() {
                                        if let Some(payload) = &action_entry.payload_data {
                                            let envelope = ActionEnvelope {
                                                id: ActionId::from_u128(action_entry.action_id),
                                                payload: payload.clone(),
                                            };
                                            return self.dispatch(envelope, node_id);
                                        }
                                    }
                                }
                                current_id = node.parent;
                            } else { break; }
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if let Some(action_entry) = semantics.actions.entries.first() {
                                        if let Some(payload) = &action_entry.payload_data {
                                            let envelope = ActionEnvelope {
                                                id: ActionId::from_u128(action_entry.action_id),
                                                payload: payload.clone(),
                                            };
                                            return self.dispatch(envelope, node_id);
                                        }
                                    }
                                }
                                current_id = node.parent;
                            } else {
                                break;
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let current_text = semantics.value.as_deref().unwrap_or("");
                                        let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                        let caret = Self::clamp_caret_to_value(current_text, st.caret);
                                        let sel = if st.caret != st.anchor { Some((st.anchor, st.caret)) } else { None };
                                        let (new_text, new_caret) = Self::insert_text(current_text, caret, sel, &c.to_string());
                                        
                                        if let Some(action_entry) = semantics.actions.entries.first() {
                                            let payload = serde_json::to_vec(&new_text).unwrap();
                                            let envelope = ActionEnvelope {
                                                id: ActionId::from_u128(action_entry.action_id),
                                                payload,
                                            };
                                            let res = self.dispatch(envelope, node_id);
                                            let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                            // typing collapses selection
                                            st.caret = new_caret; st.anchor = new_caret;
                                            // Auto-scroll to keep caret visible using measured geometry
                                            self.auto_scroll_textinput(node_id, ir, layout);
                                            return res;
                                        }
                                    }
                                }
                                current_id = node.parent;
                            } else {
                                break;
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let current_text = semantics.value.as_deref().unwrap_or("");
                                        let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                        let caret = Self::clamp_caret_to_value(current_text, st.caret);
                                        let sel = if st.caret != st.anchor { Some((st.anchor, st.caret)) } else { None };
                                        let (new_text, new_caret) = if (modifiers & 2) != 0 && sel.is_none() {
                                            // Alt/Option+Backspace: delete previous word (coarse)
                                            let mut at = caret;
                                            // skip whitespace
                                            while at > 0 {
                                                let prev = Self::prev_grapheme_boundary(current_text, at);
                                                let ch = current_text[prev..].chars().next().unwrap_or('\0');
                                                if !ch.is_whitespace() { at = prev; break; }
                                                at = prev;
                                            }
                                            // delete word chars
                                            while at > 0 {
                                                let prev = Self::prev_grapheme_boundary(current_text, at);
                                                let ch = current_text[prev..].chars().next().unwrap_or('\0');
                                                if ch.is_alphanumeric() || ch == '_' { at = prev; } else { break; }
                                            }
                                            let mut out = String::with_capacity(current_text.len() - (caret - at));
                                            out.push_str(&current_text[..at]);
                                            out.push_str(&current_text[caret..]);
                                            (out, at)
                                        } else {
                                            Self::delete_prev_grapheme(current_text, caret, sel)
                                        };
                                        
                                        if let Some(action_entry) = semantics.actions.entries.first() {
                                            let payload = serde_json::to_vec(&new_text).unwrap();
                                            let envelope = ActionEnvelope {
                                                id: ActionId::from_u128(action_entry.action_id),
                                                payload,
                                            };
                                            let res = self.dispatch(envelope, node_id);
                                            let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                            st.caret = new_caret; st.anchor = new_caret;
                                            self.auto_scroll_textinput(node_id, ir, layout);
                                            return res;
                                        }
                                    }
                                }
                                current_id = node.parent;
                            } else {
                                break;
                            }
                        }
                    }
                }
                // Copy/Cut/Paste via Ctrl/Super modifiers
                KeyCode::Char(ch) if ((modifiers & 4) != 0) || ((modifiers & 8) != 0) => {
                    let lower = ch.to_ascii_lowercase();
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        if let Some(node) = ir.nodes.get(&focused_id) {
                            if let Op::Semantics(sem) = &node.op {
                                if sem.role == fission_ir::semantics::Role::TextInput {
                                    let value = sem.value.as_deref().unwrap_or("");
                                    let st = self.runtime_state.text_edit.get_mut_or_default(focused_id);
                                    let (s,e) = if st.caret <= st.anchor { (st.caret, st.anchor) } else { (st.anchor, st.caret) };
                                    match lower {
                                        'c' => {
                                            if s != e {
                                                self.runtime_state.clipboard = value[s..e].to_string();
                                            }
                                        }
                                        'x' => {
                                            if s != e {
                                                self.runtime_state.clipboard = value[s..e].to_string();
                                                if let Some(node) = ir.nodes.get(&focused_id) {
                                                    if let Op::Semantics(semantics) = &node.op {
                                                        if let Some(action_entry) = semantics.actions.entries.first() {
                                                            let mut out = String::with_capacity(value.len() - (e - s));
                                                            out.push_str(&value[..s]);
                                                            out.push_str(&value[e..]);
                                                            let payload = serde_json::to_vec(&out).unwrap();
                                                            let envelope = ActionEnvelope { id: ActionId::from_u128(action_entry.action_id), payload };
                                                            let _ = self.dispatch(envelope, focused_id);
                                                            self.runtime_state.text_edit.set_caret(focused_id, s, Some(s));
                                                            self.auto_scroll_textinput(focused_id, ir, layout);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        'v' => {
                                            if !self.runtime_state.clipboard.is_empty() {
                                                if let Some(node) = ir.nodes.get(&focused_id) {
                                                    if let Op::Semantics(semantics) = &node.op {
                                                        if let Some(action_entry) = semantics.actions.entries.first() {
                                                            let sel_opt = if s != e { Some((s, e)) } else { None };
                                                            let (new_text, new_caret) = Self::insert_text(value, st.caret, sel_opt, &self.runtime_state.clipboard);
                                                            let payload = serde_json::to_vec(&new_text).unwrap();
                                                            let envelope = ActionEnvelope { id: ActionId::from_u128(action_entry.action_id), payload };
                                                            let _ = self.dispatch(envelope, focused_id);
                                                            self.runtime_state.text_edit.set_caret(focused_id, new_caret, Some(new_caret));
                                                            self.auto_scroll_textinput(focused_id, ir, layout);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                KeyCode::Left => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let current_text = semantics.value.as_deref().unwrap_or("");
                                        let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                        let caret = Self::clamp_caret_to_value(current_text, st.caret);
                                        let prev = if (modifiers & 2) != 0 { // Alt/Option
                                            Self::prev_word_boundary(current_text, caret)
                                        } else {
                                            Self::prev_grapheme_boundary(current_text, caret)
                                        };
                                        // Shift extends selection: do not collapse anchor
                                        if (modifiers & 1) != 0 { st.caret = prev; } else { st.caret = prev; st.anchor = prev; }
                                        self.auto_scroll_textinput(node_id, ir, layout);
                                        break;
                                    }
                                }
                                current_id = node.parent;
                            } else { break; }
                        }
                    }
                }
                KeyCode::Right => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let current_text = semantics.value.as_deref().unwrap_or("");
                                        let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                        let caret = Self::clamp_caret_to_value(current_text, st.caret);
                                        let next = if (modifiers & 2) != 0 { // Alt/Option
                                            Self::next_word_boundary(current_text, caret)
                                        } else {
                                            Self::next_grapheme_boundary(current_text, caret)
                                        };
                                        if (modifiers & 1) != 0 { st.caret = next; } else { st.caret = next; st.anchor = next; }
                                        self.auto_scroll_textinput(node_id, ir, layout);
                                        break;
                                    }
                                }
                                current_id = node.parent;
                            } else { break; }
                        }
                    }
                }
                KeyCode::Home => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
        
                                        if (modifiers & 1) != 0 { st.caret = 0; } else { st.caret = 0; st.anchor = 0; }
                                        self.auto_scroll_textinput(node_id, ir, layout);
                                        break;
                                    }
                                }
                                current_id = node.parent;
                            } else { break; }
                        }
                    }
                }
                KeyCode::End => {
                    if let Some(focused_id) = self.runtime_state.interaction.focused {
                        let mut current_id = Some(focused_id);
                        while let Some(node_id) = current_id {
                            if let Some(node) = ir.nodes.get(&node_id) {
                                if let Op::Semantics(semantics) = &node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let value = semantics.value.as_deref().unwrap_or("");
                                        let end = value.len();
                                        let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                        if (modifiers & 1) != 0 { st.caret = end; } else { st.caret = end; st.anchor = end; }
                                        self.auto_scroll_textinput(node_id, ir, layout);
                                        break;
                                    }
                                }
                                current_id = node.parent;
                            } else { break; }
                        }
                    }
                }
                _ => {}
            },
            InputEvent::Ime(ime) => {
                // Minimal IME handling: commit inserts text at caret; preedit ignored for now
                match ime {
                    crate::event::ImeEvent::Commit { text } => {
                        if let Some(focused_id) = self.runtime_state.interaction.focused {
                            let mut current_id = Some(focused_id);
                            while let Some(node_id) = current_id {
                                if let Some(node) = ir.nodes.get(&node_id) {
                                    if let Op::Semantics(semantics) = &node.op {
                                        if semantics.role == fission_ir::semantics::Role::TextInput {
                                            let current_text = semantics.value.as_deref().unwrap_or("");
                                            let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                            let caret = Self::clamp_caret_to_value(current_text, st.caret);
                                            let sel = if st.caret != st.anchor { Some((st.anchor, st.caret)) } else { None };
                                            let (new_text, new_caret) = Self::insert_text(current_text, caret, sel, &text);
                                            if let Some(action_entry) = semantics.actions.entries.first() {
                                                let payload = serde_json::to_vec(&new_text).unwrap();
                                                let envelope = ActionEnvelope {
                                                    id: ActionId::from_u128(action_entry.action_id),
                                                    payload,
                                                };
                                                // Clear preedit and scroll to caret
                                                self.runtime_state.ime_preedit = None;
                                                let res = self.dispatch(envelope, node_id);
                                                let st = self.runtime_state.text_edit.get_mut_or_default(node_id);
                                                st.caret = new_caret; st.anchor = new_caret;
                                                self.auto_scroll_textinput(node_id, ir, layout);
                                                return res;
                                            }
                                        }
                                    }
                                    current_id = node.parent;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    crate::event::ImeEvent::Preedit { text } => {
                        if let Some(focused_id) = self.runtime_state.interaction.focused {
                            self.runtime_state.ime_preedit = Some((focused_id, text.clone()));
                            // Auto-scroll including preedit
                            if let Some(focused_node) = ir.nodes.get(&focused_id) {
                                if let Op::Semantics(semantics) = &focused_node.op {
                                    if semantics.role == fission_ir::semantics::Role::TextInput {
                                        let current_text = semantics.value.as_deref().unwrap_or("");
                                        self.auto_scroll_textinput(focused_id, ir, layout);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            InputEvent::Pointer(PointerEvent::Move { point, .. }) => {
                let hit = hit_test_with_scroll(ir, layout, &self.runtime_state.scroll, point);

                let mut new_hovered = std::collections::HashSet::new();
                if let Some(mut node_id) = hit {
                    loop {
                        new_hovered.insert(node_id);
                        if let Some(node) = ir.nodes.get(&node_id) {
                            if let Some(parent) = node.parent {
                                node_id = parent;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                self.runtime_state.interaction.hovered.clear();

                for id in new_hovered {
                    self.runtime_state.interaction.set_hovered(id, true);
                }

                // Drag-select: if focused TextInput and mouse is down beyond a small threshold, update caret based on x using approximate measurement
                if let Some(focused) = self.runtime_state.interaction.focused {
                    if let Some(node) = ir.nodes.get(&focused) {
                        if let Op::Semantics(sem) = &node.op {
                            if sem.role == fission_ir::semantics::Role::TextInput {
                                if !self.runtime_state.interaction.pressed.is_empty() {
                                    // Apply a 2px movement threshold to avoid accidental selection on tiny mouse moves
                                    let mut moved_enough = true;
                                    if let Some(start) = self.runtime_state.interaction.last_down_point {
                                        let dx = point.x - start.x;
                                        let dy = point.y - start.y;
                                        if dx*dx + dy*dy < 4.0 { moved_enough = false; }
                                    }
                                    if moved_enough {
                                        if let Some((scroll_id, _)) = Self::find_scroll_row_and_text(ir, focused) {
                                            if let Some(scroll_geom) = layout.get_node_geometry(scroll_id) {
                                                let value = sem.value.as_deref().unwrap_or("");
                                                let new_caret = self.caret_from_point_in_text(
                                                    value,
                                                    16.0,
                                                    scroll_geom.rect.origin.x,
                                                    scroll_geom.rect.size.width,
                                                    scroll_geom.content_size.width,
                                                    self.runtime_state.scroll.get_offset(scroll_id),
                                                    point.x,
                                                );
                                                let st = self.runtime_state.text_edit.get_mut_or_default(focused);
                                                st.caret = new_caret; // anchor preserved from pointer down
                                                self.auto_scroll_textinput(focused, ir, layout);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            InputEvent::Pointer(PointerEvent::Down { point, .. }) => {
                if let Some(hit_node_id) =
                    hit_test_with_scroll(ir, layout, &self.runtime_state.scroll, point)
                {
                    diag::emit(
                        diag::DiagCategory::Input,
                        diag::DiagLevel::Debug,
                        diag::DiagEventKind::InputEvent { kind: "pointer_down_hit".into(), target: Some(hit_node_id.as_u128()), position: Some((point.x, point.y)) },
                    );
                    let mut focus_candidate = Some(hit_node_id);
                    while let Some(node_id) = focus_candidate {
                        if let Some(node) = ir.nodes.get(&node_id) {
                            if let Op::Semantics(s) = &node.op {
                                if s.focusable {
                                    if Some(node_id) != self.runtime_state.interaction.focused {
                                        self.runtime_state.ime_preedit = None;
                                    }
                                    self.runtime_state.interaction.set_focused(Some(node_id));
                                    // If focusing a text input, initialize caret to end
                                    if s.role == fission_ir::semantics::Role::TextInput {
                                        let value = s.value.as_deref().unwrap_or("");
                                        self.runtime_state.text_edit.set_caret(node_id, value.len(), None);
                                        // On pointer down inside text, set both caret and anchor based on x (coarse start/end)
                                        if let Some((scroll_id, _)) = Self::find_scroll_row_and_text(ir, node_id) {
                                            if let Some(scroll_geom) = layout.get_node_geometry(scroll_id) {
                                                let caret = self.caret_from_point_in_text(
                                                    value,
                                                    16.0,
                                                    scroll_geom.rect.origin.x,
                                                    scroll_geom.rect.size.width,
                                                    scroll_geom.content_size.width,
                                                    self.runtime_state.scroll.get_offset(scroll_id),
                                                    point.x,
                                                );
                                                self.runtime_state.text_edit.set_caret(node_id, caret, Some(caret));
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                            focus_candidate = node.parent;
                        } else {
                            break;
                        }
                    }
                    if focus_candidate.is_none() {
                        // self.runtime_state.interaction.set_focused(None);
                    }

                    let mut current_pressed_id = Some(hit_node_id);
                    while let Some(node_id) = current_pressed_id {
                        self.runtime_state.interaction.set_pressed(node_id, true);
                        if let Some(node) = ir.nodes.get(&node_id) {
                            current_pressed_id = node.parent;
                        } else {
                            break;
                        }
                    }
                    // Record pointer down location (for move threshold)
                    self.runtime_state.interaction.last_down_point = Some(point);
                } else {
                    self.runtime_state.interaction.set_focused(None);
                }
            }
            InputEvent::Pointer(PointerEvent::Up { point, .. }) => {
                if let Some(hit_node_id) =
                    hit_test_with_scroll(ir, layout, &self.runtime_state.scroll, point)
                {
                    // Clear pressed state
                    self.runtime_state.interaction.pressed.clear();

                    // Dispatch at most one action on pointer up, closest semantics first
                    let mut current_id = Some(hit_node_id);
                    while let Some(node_id) = current_id {
                        if let Some(node) = ir.nodes.get(&node_id) {
                            if let Op::Semantics(semantics) = &node.op {
                                if semantics.role == fission_ir::semantics::Role::TextInput {
                                    // TextInput only takes focus on click, no action dispatch
                                } else if let Some(action_entry) = semantics.actions.entries.first() {
                                    if let Some(payload) = &action_entry.payload_data {
                                        let envelope = ActionEnvelope {
                                            id: ActionId::from_u128(action_entry.action_id),
                                            payload: payload.clone(),
                                        };
                                        diag::emit(
                                            diag::DiagCategory::Input,
                                            diag::DiagLevel::Debug,
                                            diag::DiagEventKind::InputEvent { kind: "pointer_up_dispatch".into(), target: Some(node_id.as_u128()), position: Some((point.x, point.y)) },
                                        );
                                        return self.dispatch(envelope, node_id);
                                    }
                                    // If no payload (dynamic action), ignore for click.
                                }
                            }
                            current_id = node.parent;
                        } else {
                            break;
                        }
                    }

                    // Clear drag start on pointer up
                    self.runtime_state.interaction.last_down_point = None;
                }
            }
            InputEvent::Pointer(PointerEvent::Up { point: _, .. }) => {
                self.runtime_state.interaction.pressed.clear();
                self.runtime_state.interaction.last_down_point = None;
            }
            _ => {}
        }
        Ok(())
    }
}