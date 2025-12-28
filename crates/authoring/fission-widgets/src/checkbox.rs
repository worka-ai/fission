use fission_core::action::ActionEnvelope;
use fission_core::{BuildCtx, View, Widget, Node, NodeBuilder, NodeId, Op, LowerDyn, LoweringContext};
use fission_core::op::{PaintOp, LayoutOp, Fill, Stroke, Color};
use std::sync::Arc;

#[derive(Default, Clone, Debug)]
pub struct Checkbox {
    pub checked: bool,
    pub on_toggle: Option<ActionEnvelope>,
    pub label: Option<String>,
}

impl<S: fission_core::AppState> Widget<S> for Checkbox {
    fn build(&self, _ctx: &mut BuildCtx<S>, _view: &View<S>) -> Node {
        Node::Custom(fission_core::CustomNode {
            debug_tag: "Checkbox".into(),
            lowerer: Some(Arc::new(CheckboxLowerer {
                checked: self.checked,
                on_toggle: self.on_toggle.clone(),
                label: self.label.clone(),
            })),
        })
    }
}

#[derive(Debug)]
struct CheckboxLowerer {
    checked: bool,
    on_toggle: Option<ActionEnvelope>,
    label: Option<String>,
}

impl LowerDyn for CheckboxLowerer {
    fn lower_dyn(&self, cx: &mut LoweringContext) -> NodeId {
        let tokens = &cx.env.theme.tokens;
        let size = 18.0;
        let radius = tokens.radii.small;
        let border_color = tokens.colors.text_secondary; // Unchecked border
        let active_color = tokens.colors.primary;
        let text_color = tokens.colors.text_primary;

        // Square indicator
        let square_id = cx.next_node_id();
        
        let bg_paint = if self.checked {
            Op::Paint(PaintOp::DrawRect {
                fill: Some(Fill { color: active_color }),
                stroke: None,
                corner_radius: radius,
                shadow: None,
            })
        } else {
            Op::Paint(PaintOp::DrawRect {
                fill: None,
                stroke: Some(Stroke { color: border_color, width: 1.5 }),
                corner_radius: radius,
                shadow: None,
            })
        };
        
        let bg_node = NodeBuilder::new(cx.next_node_id(), bg_paint).build(cx);
        
        // Checkmark (simplified as inner rect for now, TODO: DrawPath/Icon)
        let check_node = if self.checked {
            let check = NodeBuilder::new(cx.next_node_id(), Op::Paint(PaintOp::DrawRect {
                fill: Some(Fill { color: tokens.colors.on_primary }),
                stroke: None,
                corner_radius: 1.0,
                shadow: None,
            })).build(cx);
            // Center checkmark
            let mut check_box = NodeBuilder::new(cx.next_node_id(), Op::Layout(LayoutOp::Box {
                width: Some(10.0), height: Some(10.0), 
                min_width: None, max_width: None, min_height: None, max_height: None, padding: [0.0;4]
            }));
            check_box.add_child(check);
            Some(check_box.build(cx))
        } else {
            None
        };

        let mut square_box = NodeBuilder::new(
            square_id,
            Op::Layout(LayoutOp::Box { width: Some(size), height: Some(size), min_width: None, max_width: None, min_height: None, max_height: None, padding: [0.0; 4] }),
        );
        square_box.add_child(bg_node);
        if let Some(c) = check_node { square_box.add_child(c); }
        let square_final = square_box.build(cx);

        // Optional label
        let label_id = if let Some(text) = &self.label {
            let text_id = NodeBuilder::new(
                cx.next_node_id(),
                Op::Paint(PaintOp::DrawText { 
                    text: text.clone(), 
                    size: tokens.typography.body_medium_size, 
                    color: text_color, 
                    underline: false, 
                    caret_index: None 
                }),
            )
            .build(cx);
            let mut layout_builder = NodeBuilder::new(
                cx.next_node_id(),
                Op::Layout(LayoutOp::Box { width: None, height: None, min_width: None, max_width: None, min_height: None, max_height: None, padding: [tokens.spacing.s, 0.0, 0.0, 0.0] }), // Left padding
            );
            layout_builder.add_child(text_id);
            Some(layout_builder.build(cx))
        } else { None };

        // Row container
        let mut row = NodeBuilder::new(
            cx.next_node_id(),
            Op::Layout(LayoutOp::Flex { direction: fission_core::FlexDirection::Row, flex_grow: 0.0, flex_shrink: 0.0, padding: [0.0; 4], gap: None }),
        );
        row.add_child(square_final);
        if let Some(l) = label_id { row.add_child(l); }
        let row_id = row.build(cx);

        // Semantics wrapper
        let mut semantics = fission_ir::Semantics {
            role: fission_ir::Role::Checkbox,
            label: self.label.clone(),
            value: Some(if self.checked { "true".into() } else { "false".into() }),
            actions: fission_ir::ActionSet::default(),
            focusable: true,
            multiline: false,
            masked: false,
            input_mask: None,
            ime_preedit_range: None,
            checked: Some(self.checked),
            disabled: false,
        };
        if let Some(action) = &self.on_toggle {
            semantics
                .actions
                .entries
                .push(fission_ir::ActionEntry { action_id: action.id.as_u128(), payload_data: Some(action.payload.clone()) });
        }
        let mut sem = NodeBuilder::new(cx.next_node_id(), Op::Semantics(semantics));
        sem.add_child(row_id);
        sem.build(cx)
    }
}
