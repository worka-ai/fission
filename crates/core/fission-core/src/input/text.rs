use super::{ControllerContext, InputController};
use crate::event::{InputEvent, KeyCode, KeyEvent, PointerEvent};
use crate::ActionEnvelope;
use crate::ActionId;
use fission_diagnostics::prelude as diag;
use fission_ir::{op::{self, LayoutOp, Op}, NodeId};
use fission_layout::LayoutSnapshot;
use serde_json;
use unicode_segmentation::UnicodeSegmentation;

pub struct TextInputController;

impl InputController for TextInputController {
    fn handle_event(&mut self, ctx: &mut ControllerContext, event: &InputEvent) -> bool {
        match event {
            InputEvent::Keyboard(KeyEvent::Down { key_code, modifiers }) => {
                self.handle_key(ctx, key_code.clone(), *modifiers)
            }
            InputEvent::Ime(ime) => {
                self.handle_ime(ctx, ime)
            }
            InputEvent::Pointer(PointerEvent::Down { point, .. }) => {
                if let Some(focused_id) = ctx.interaction.focused {
                    if let Some(node) = ctx.ir.nodes.get(&focused_id) {
                        if let Op::Semantics(sem) = &node.op {
                            if sem.role == fission_ir::semantics::Role::TextInput {
                                if let Some((scroll_id, _)) = Self::find_scroll_row_and_text(ctx.ir, focused_id) {
                                    if let Some(scroll_geom) = ctx.layout.get_node_geometry(scroll_id) {
                                        let value = sem.value.as_deref().unwrap_or("");
                                        let offset = ctx.scroll.get_offset(scroll_id);
                                        let caret = caret_from_point_in_text(
                                            ctx.measurer,
                                            value,
                                            16.0,
                                            scroll_geom.rect.origin.x,
                                            scroll_geom.rect.size.width,
                                            scroll_geom.content_size.width,
                                            offset,
                                            point.x,
                                        );
                                        let st = ctx.text_edit.get_mut_or_default(focused_id);
                                        st.caret = caret;
                                        st.anchor = caret;
                                    }
                                }
                                return true; 
                            }
                        }
                    }
                }
                false
            }
            InputEvent::Pointer(PointerEvent::Move { point, .. }) => {
                if let Some(focused_id) = ctx.interaction.focused {
                    if let Some(node) = ctx.ir.nodes.get(&focused_id) {
                        if let Op::Semantics(sem) = &node.op {
                            if sem.role == fission_ir::semantics::Role::TextInput {
                                if !ctx.interaction.pressed.is_empty() {
                                    let mut moved_enough = true;
                                    if let Some(start) = ctx.interaction.last_down_point {
                                        let dx = point.x - start.x;
                                        let dy = point.y - start.y;
                                        if dx*dx + dy*dy < 4.0 { moved_enough = false; }
                                    }
                                    if moved_enough {
                                        if let Some((scroll_id, _)) = Self::find_scroll_row_and_text(ctx.ir, focused_id) {
                                            if let Some(scroll_geom) = ctx.layout.get_node_geometry(scroll_id) {
                                                let value = sem.value.as_deref().unwrap_or("");
                                                let offset = ctx.scroll.get_offset(scroll_id);
                                                let new_caret = caret_from_point_in_text(
                                                    ctx.measurer,
                                                    value,
                                                    16.0,
                                                    scroll_geom.rect.origin.x,
                                                    scroll_geom.rect.size.width,
                                                    scroll_geom.content_size.width,
                                                    offset,
                                                    point.x,
                                                );
                                                let st = ctx.text_edit.get_mut_or_default(focused_id);
                                                st.caret = new_caret; 
                                                Self::auto_scroll_textinput(ctx, focused_id);
                                            }
                                        }
                                    }
                                }
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => false
        }
    }
}

impl TextInputController {
    fn handle_key(&mut self, ctx: &mut ControllerContext, key_code: KeyCode, modifiers: u8) -> bool {
        let focused_id = if let Some(id) = ctx.interaction.focused { id } else { return false; };
        
        let mut semantics = None;
        let mut current_id = Some(focused_id);
        while let Some(node_id) = current_id {
            if let Some(node) = ctx.ir.nodes.get(&node_id) {
                if let Op::Semantics(s) = &node.op {
                    if s.role == fission_ir::semantics::Role::TextInput {
                        semantics = Some(s);
                        break;
                    }
                }
                current_id = node.parent;
            } else { break; }
        }
        
        let semantics = if let Some(s) = semantics { s } else { return false; };
        
        let value = semantics.value.as_deref().unwrap_or("").to_string();
        let st = ctx.text_edit.get_mut_or_default(focused_id);
        let caret = Self::clamp_caret_to_value(&value, st.caret);
        st.caret = caret;
        st.anchor = Self::clamp_caret_to_value(&value, st.anchor);
        
        let sel = if st.caret != st.anchor { Some((st.anchor, st.caret)) } else { None };

        match key_code {
            KeyCode::Space => {
                let (new_text, new_caret) = Self::insert_text(&value, st.caret, sel, " ");
                self.dispatch_change(ctx, semantics, focused_id, new_text, new_caret);
                true
            }
            KeyCode::Char(ch) if ((modifiers & 4) != 0) || ((modifiers & 8) != 0) => {
                let lower = ch.to_ascii_lowercase();
                let (s, e) = if st.caret <= st.anchor { (st.caret, st.anchor) } else { (st.anchor, st.caret) };
                match lower {
                    'c' => {
                        if s != e {
                            let txt = value[s..e].to_string();
                            if let Some(cb) = ctx.clipboard {
                                cb.set_text(&txt);
                            }
                        }
                    }
                    'x' => {
                        if s != e {
                            let txt = value[s..e].to_string();
                            if let Some(cb) = ctx.clipboard {
                                cb.set_text(&txt);
                            }
                            let mut out = String::with_capacity(value.len() - (e - s));
                            out.push_str(&value[..s]);
                            out.push_str(&value[e..]);
                            self.dispatch_change(ctx, semantics, focused_id, out, s);
                        }
                    }
                    'v' => {
                        let text_to_paste = if let Some(cb) = ctx.clipboard {
                            cb.get_text().unwrap_or_default()
                        } else {
                            String::new()
                        };
                        if !text_to_paste.is_empty() {
                            let (new_text, new_caret) = Self::insert_text(&value, st.caret, sel, &text_to_paste);
                            self.dispatch_change(ctx, semantics, focused_id, new_text, new_caret);
                        }
                    }
                    _ => {}
                }
                true
            }
            KeyCode::Char(c) => {
                let (new_text, new_caret) = Self::insert_text(&value, st.caret, sel, &c.to_string());
                self.dispatch_change(ctx, semantics, focused_id, new_text, new_caret);
                true
            }
            KeyCode::Backspace => {
                let (new_text, new_caret) = if (modifiers & 2) != 0 && sel.is_none() {
                    let mut at = caret;
                    while at > 0 {
                        let prev = Self::prev_grapheme_boundary(&value, at);
                        let ch = value[prev..].chars().next().unwrap_or('\0');
                        if !ch.is_whitespace() { at = prev; break; }
                        at = prev;
                    }
                    while at > 0 {
                        let prev = Self::prev_grapheme_boundary(&value, at);
                        let ch = value[prev..].chars().next().unwrap_or('\0');
                        if ch.is_alphanumeric() || ch == '_' { at = prev; } else { break; }
                    }
                    let mut out = String::with_capacity(value.len() - (caret - at));
                    out.push_str(&value[..at]);
                    out.push_str(&value[caret..]);
                    (out, at)
                } else {
                    Self::delete_prev_grapheme(&value, caret, sel)
                };
                self.dispatch_change(ctx, semantics, focused_id, new_text, new_caret);
                true
            }
            KeyCode::Left => {
                let prev = if (modifiers & 2) != 0 {
                    Self::prev_word_boundary(&value, caret)
                } else {
                    Self::prev_grapheme_boundary(&value, caret)
                };
                if (modifiers & 1) != 0 { st.caret = prev; } else { st.caret = prev; st.anchor = prev; }
                Self::auto_scroll_textinput(ctx, focused_id);
                true
            }
            KeyCode::Right => {
                let next = if (modifiers & 2) != 0 {
                    Self::next_word_boundary(&value, caret)
                } else {
                    Self::next_grapheme_boundary(&value, caret)
                };
                if (modifiers & 1) != 0 { st.caret = next; } else { st.caret = next; st.anchor = next; }
                Self::auto_scroll_textinput(ctx, focused_id);
                true
            }
            KeyCode::Home => {
                if (modifiers & 1) != 0 { st.caret = 0; } else { st.caret = 0; st.anchor = 0; }
                Self::auto_scroll_textinput(ctx, focused_id);
                true
            }
            KeyCode::End => {
                let end = value.len();
                if (modifiers & 1) != 0 { st.caret = end; } else { st.caret = end; st.anchor = end; }
                Self::auto_scroll_textinput(ctx, focused_id);
                true
            }
            _ => false,
        }
    }

    fn handle_ime(&mut self, ctx: &mut ControllerContext, ime: &crate::event::ImeEvent) -> bool {
        match ime {
            crate::event::ImeEvent::Commit { text } => {
                if let Some(focused_id) = ctx.interaction.focused {
                    // Reuse generic char logic or insert logic?
                    // For now, let's map it to insertion manually to avoid duplication?
                    // Actually handle_key Char(c) is single char. Commit can be string.
                    // Copy-paste logic from lib.rs Ime commit:
                    if let Some(node) = ctx.ir.nodes.get(&focused_id) {
                        if let Op::Semantics(semantics) = &node.op {
                            if semantics.role == fission_ir::semantics::Role::TextInput {
                                let value = semantics.value.as_deref().unwrap_or("").to_string();
                                let st = ctx.text_edit.get_mut_or_default(focused_id);
                                let caret = Self::clamp_caret_to_value(&value, st.caret);
                                let sel = if st.caret != st.anchor { Some((st.anchor, st.caret)) } else { None };
                                let (new_text, new_caret) = Self::insert_text(&value, caret, sel, text);
                                self.dispatch_change(ctx, semantics, focused_id, new_text, new_caret);
                                *ctx.ime_preedit = None;
                                return true;
                            }
                        }
                    }
                }
            }
            crate::event::ImeEvent::Preedit { text } => {
                if let Some(focused_id) = ctx.interaction.focused {
                    *ctx.ime_preedit = Some((focused_id, text.clone()));
                    Self::auto_scroll_textinput(ctx, focused_id);
                    return true;
                }
            }
        }
        false
    }

    fn dispatch_change(&self, ctx: &mut ControllerContext, semantics: &fission_ir::Semantics, node_id: NodeId, new_text: String, new_caret: usize) {
        if let Some(action_entry) = semantics.actions.entries.first() {
            let payload = serde_json::to_vec(&new_text).unwrap();
            let envelope = ActionEnvelope {
                id: ActionId::from_u128(action_entry.action_id),
                payload,
            };
            ctx.dispatched_actions.push((node_id, envelope));
            
            let st = ctx.text_edit.get_mut_or_default(node_id);
            st.caret = new_caret;
            st.anchor = new_caret; 
            
            Self::auto_scroll_textinput(ctx, node_id);
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

    fn insert_text(value: &str, caret: usize, sel: Option<(usize,usize)>, text: &str) -> (String, usize) {
        let (s,e) = sel.map(|(a,b)| if a<=b {(a,b)} else {(b,a)}).unwrap_or((caret, caret));
        let mut out = String::with_capacity(value.len() - (e-s) + text.len());
        out.push_str(&value[..s]);
        out.push_str(text);
        out.push_str(&value[e..]);
        (out, s + text.len())
    }

    fn find_scroll_row_and_text(ir: &fission_ir::CoreIR, root: NodeId) -> Option<(NodeId, NodeId)> {
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

    fn find_caret_in_scroll(ir: &fission_ir::CoreIR, scroll_id: NodeId) -> Option<NodeId> {
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

    fn approx_text_width(s: &str, font_size: f32) -> f32 {
        (s.chars().count() as f32) * font_size * 0.6
    }

    fn auto_scroll_textinput(ctx: &mut ControllerContext, text_root: NodeId) {
        if let Some((scroll_id, text_id)) = Self::find_scroll_row_and_text(ctx.ir, text_root) {
            if let Some(scroll_geom) = ctx.layout.get_node_geometry(scroll_id) {
                let viewport_x = scroll_geom.rect.origin.x;
                let viewport_w = scroll_geom.rect.size.width;
                let content_w = scroll_geom.content_size.width.max(viewport_w);
                let caret_left = if let Some(caret_id) = Self::find_caret_in_scroll(ctx.ir, scroll_id) {
                    ctx.layout.get_node_geometry(caret_id).map(|g| g.rect.origin.x).unwrap_or_else(|| {
                        ctx.layout.get_node_geometry(text_id).map(|g| g.rect.origin.x + g.rect.size.width).unwrap_or(viewport_x)
                    })
                } else {
                    ctx.layout.get_node_geometry(text_id).map(|g| g.rect.origin.x + g.rect.size.width).unwrap_or(viewport_x)
                };
                let caret_width = 2.0f32;
                let caret_right = caret_left + caret_width;
                let mut offset = ctx.scroll.get_offset(scroll_id);
                let margin_left = 2.0f32;
                let margin_right = 3.0f32; 
                let visible_left = (caret_left - offset) - viewport_x;
                let visible_right = (caret_right - offset) - viewport_x;
                if visible_right > (viewport_w - margin_right) {
                    offset = (caret_right - (viewport_x + viewport_w - margin_right)).max(0.0);
                } else if visible_left < margin_left {
                    offset = (caret_left - (viewport_x + margin_left)).max(0.0);
                }
                let max_offset = (content_w - viewport_w).max(0.0);
                offset = offset.clamp(0.0, max_offset);
                ctx.scroll.set_offset(scroll_id, offset);
            }
        }
    }
}

pub fn caret_from_point_in_text(
    measurer: Option<&std::sync::Arc<dyn fission_layout::TextMeasurer>>,
    value: &str, 
    font_size: f32, 
    viewport_x: f32, viewport_w: f32, content_w: f32, scroll_offset: f32, point_x: f32
) -> usize {
    let mut local_x = (point_x - viewport_x) + scroll_offset;
    if local_x <= 0.0 { return 0; }
    let max_x = content_w.max(viewport_w);
    if local_x >= max_x { return value.len(); }

    if let Some(measurer) = measurer {
        let mut last_idx = 0;
        let mut last_w = 0.0;
        
        for (idx, _) in value.grapheme_indices(true) {
            let w = if idx == 0 { 0.0 } else {
                measurer.measure(&value[..idx], font_size, None).0
            };
            
            if w > local_x {
                if local_x < (last_w + w) / 2.0 {
                    return last_idx;
                } else {
                    return idx;
                }
            }
            last_idx = idx;
            last_w = w;
        }
        let (total_w, _) = measurer.measure(value, font_size, None);
        if local_x < (last_w + total_w) / 2.0 {
            return last_idx;
        } else {
            return value.len();
        }
    } else {
        let mut acc = 0.0f32;
        let mut last_index = 0usize;
        for (idx, g) in value.grapheme_indices(true) {
            let w = TextInputController::approx_text_width(g, font_size);
            if acc + w * 0.5 >= local_x { return idx; }
            acc += w;
            last_index = idx;
        }
        value.len()
    }
}