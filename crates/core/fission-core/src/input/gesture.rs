use super::{ControllerContext, InputController};
use crate::event::{InputEvent, PointerEvent};
use fission_ir::NodeId;
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
                        } else {
                            ctx.gesture.target_node = None;
                        }
                    }
                    PointerEvent::Move { point } => {
                        if let Some(start) = ctx.gesture.start_point {
                            let dx = point.x - start.x;
                            let dy = point.y - start.y;
                            let dist_sq = dx*dx + dy*dy;
                            let threshold = 5.0 * 5.0; // 5px threshold
                            
                            if !ctx.gesture.is_panning && dist_sq > threshold {
                                ctx.gesture.is_panning = true;
                            }
                            
                            if ctx.gesture.is_panning {
                                let last = ctx.gesture.last_point.unwrap_or(start);
                                let delta = LayoutPoint { x: point.x - last.x, y: point.y - last.y };
                                ctx.gesture.last_point = Some(*point);
                                
                                if self.handle_pan_update(ctx, delta) {
                                    return true; 
                                }
                            }
                        }
                    }
                    PointerEvent::Up { .. } => {
                        let handled = ctx.gesture.is_panning;
                        ctx.gesture.start_point = None;
                        ctx.gesture.is_panning = false;
                        return handled; // Consume Up if it was a drag
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
    fn handle_pan_update(&self, ctx: &mut ControllerContext, delta: LayoutPoint) -> bool {
        // SCROLL DRAGGING LOGIC
        if let Some(target) = ctx.gesture.target_node {
            let mut current = Some(target);
            while let Some(id) = current {
                if let Some(node) = ctx.ir.nodes.get(&id) {
                    if let fission_ir::Op::Semantics(sem) = &node.op {
                        if sem.draggable {
                            // This node handles its own drag (e.g. Slider).
                            // Stop scroll propagation.
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
