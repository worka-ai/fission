use super::{ControllerContext, InputController};
use crate::event::{InputEvent, PointerEvent};
use crate::{ActionEnvelope, ActionId, ActionInput};
use fission_ir::{NodeId, Op, semantics::ActionTrigger};
use fission_layout::LayoutPoint;

pub struct GestureController;

impl InputController for GestureController {
    fn handle_event(&mut self, ctx: &mut ControllerContext, event: &InputEvent) -> bool {
        match event {
            InputEvent::Pointer(pe) => {
                match pe {
                    PointerEvent::Down { point, .. } => {
                        ctx.gesture.start_point = Some(*point);
                        ctx.gesture.last_point = Some(*point);
                        ctx.gesture.is_panning = false;
                        
                        if let Some(hit) = crate::hit_test::hit_test_with_scroll(ctx.ir, ctx.layout, ctx.scroll, *point) {
                            ctx.gesture.target_node = Some(hit);
                            
                            // Dispatch DragStart if draggable
                            // Note: DragStart might fire only after threshold? 
                            // Usually "Down" fires DragStart immediately or waits.
                            // But Tap also fires on Down/Up sequence.
                            // Let's fire DragStart only when panning starts.
                            // For now, Down is just tracking.
                        } else {
                            ctx.gesture.target_node = None;
                        }
                    }
                    PointerEvent::Move { point } => {
                        if let Some(start) = ctx.gesture.start_point {
                            let dx = point.x - start.x;
                            let dy = point.y - start.y;
                            let dist_sq = dx*dx + dy*dy;
                            let threshold = 5.0 * 5.0; 
                            
                            if !ctx.gesture.is_panning && dist_sq > threshold {
                                ctx.gesture.is_panning = true;
                                // Dispatch DragStart now
                                if let Some(target) = ctx.gesture.target_node {
                                    self.dispatch_trigger(ctx, target, ActionTrigger::DragStart, *point, None);
                                }
                            }
                            
                            if ctx.gesture.is_panning {
                                let last = ctx.gesture.last_point.unwrap_or(start);
                                let delta = LayoutPoint { x: point.x - last.x, y: point.y - last.y };
                                ctx.gesture.last_point = Some(*point);
                                
                                // Try dispatching DragUpdate
                                let dispatched = if let Some(target) = ctx.gesture.target_node {
                                    self.dispatch_trigger(ctx, target, ActionTrigger::DragUpdate, *point, Some(delta))
                                } else { false };
                                
                                if dispatched {
                                    return true;
                                }
                                
                                // Fallback to Scroll Panning if DragUpdate not handled
                                if self.handle_pan_update(ctx, delta) {
                                    return true; 
                                }
                            }
                        }
                    }
                    PointerEvent::Up { point, .. } => {
                        let mut handled = false;
                        if ctx.gesture.is_panning {
                            if let Some(target) = ctx.gesture.target_node {
                                self.dispatch_trigger(ctx, target, ActionTrigger::DragEnd, *point, None);
                            }
                            handled = true;
                        } else {
                            // Tap
                            if let Some(target) = ctx.gesture.target_node {
                                // Verify we are still over the target? Or loose tap?
                                // Usually Tap fires if Up is inside target bounds.
                                // hit_test check:
                                if let Some(up_hit) = crate::hit_test::hit_test_with_scroll(ctx.ir, ctx.layout, ctx.scroll, *point) {
                                    if up_hit == target || self.is_descendant(ctx, up_hit, target) || self.is_descendant(ctx, target, up_hit) {
                                        // Dispatch Tap (Default)
                                        // NOTE: `runtime.rs` Pointer::Up also dispatches.
                                        // We should consume it here if we want GestureController to own it.
                                        // But `runtime.rs` logic is fallback.
                                        // If we return `true` (handled), runtime skips fallback?
                                        // `handle_input` calls `gesture_controller` first.
                                        // If it returns true, `handle_input` returns Ok.
                                        // BUT `runtime.rs` logic is *after* the controller block.
                                        // So yes, returning true here suppresses `runtime.rs` default click logic.
                                        
                                        if self.dispatch_trigger(ctx, target, ActionTrigger::Default, *point, None) {
                                            handled = true;
                                        }
                                    }
                                }
                            }
                        }
                        
                        ctx.gesture.start_point = None;
                        ctx.gesture.is_panning = false;
                        return handled; 
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }
}

impl GestureController {
    fn is_descendant(&self, ctx: &ControllerContext, child: NodeId, ancestor: NodeId) -> bool {
        let mut curr = Some(child);
        while let Some(id) = curr {
            if id == ancestor { return true; }
            if let Some(node) = ctx.ir.nodes.get(&id) {
                curr = node.parent;
            } else { break; }
        }
        false
    }

    fn dispatch_trigger(&self, ctx: &mut ControllerContext, start_node: NodeId, trigger: ActionTrigger, point: LayoutPoint, delta: Option<LayoutPoint>) -> bool {
        let mut current_id = Some(start_node);
        while let Some(node_id) = current_id {
            if let Some(node) = ctx.ir.nodes.get(&node_id) {
                if let Op::Semantics(sem) = &node.op {
                    for entry in &sem.actions.entries {
                        if entry.trigger == trigger {
                            let envelope = ActionEnvelope {
                                id: ActionId::from_u128(entry.action_id),
                                payload: entry.payload_data.clone().unwrap_or_default(),
                            };
                            
                            let input = crate::ActionInput::Pointer { 
                                x: point.x, 
                                y: point.y, 
                                delta_x: delta.map(|d| d.x).unwrap_or(0.0), 
                                delta_y: delta.map(|d| d.y).unwrap_or(0.0), 
                            };
                            
                            ctx.dispatched_actions.push((node_id, envelope, input));
                            return true; // Stop bubbling once handled
                        }
                    }
                }
                current_id = node.parent;
            } else { break; }
        }
        false
    }

    fn handle_pan_update(&self, ctx: &mut ControllerContext, delta: LayoutPoint) -> bool {
        // SCROLL DRAGGING LOGIC
        if let Some(target) = ctx.gesture.target_node {
            let mut current = Some(target);
            while let Some(id) = current {
                if let Some(node) = ctx.ir.nodes.get(&id) {
                    if let fission_ir::Op::Semantics(sem) = &node.op {
                        if sem.draggable {
                            // If marked draggable but didn't handle DragUpdate (checked before),
                            // maybe it handles DragEnd only?
                            // Do we stop scroll?
                            // Usually yes.
                            return false; 
                        }
                    }
                    if let fission_ir::Op::Layout(fission_ir::op::LayoutOp::Scroll { direction, .. }) = &node.op {
                        let current_offset = ctx.scroll.get_offset(id);
                        let move_val = match direction {
                            fission_ir::op::FlexDirection::Row => -delta.x,
                            fission_ir::op::FlexDirection::Column => -delta.y,
                        };
                        
                        let mut new_offset = current_offset + move_val;
                        
                        if let Some(geom) = ctx.layout.get_node_geometry(id) {
                            let max_offset = if matches!(direction, fission_ir::op::FlexDirection::Row) {
                                (geom.content_size.width - geom.rect.width()).max(0.0)
                            } else {
                                (geom.content_size.height - geom.rect.height()).max(0.0)
                            };
                            new_offset = new_offset.clamp(0.0, max_offset);
                        }
                        
                        ctx.scroll.set_offset(id, new_offset);
                        return true;
                    }
                    current = node.parent;
                } else { break; }
            }
        }
        false
    }
}