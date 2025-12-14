use crate::{node::Node, Desugar, LoweringContext, WidgetNodeId};
use fission_core::ActionEnvelope;
use fission_ir::{ActionEntry, ActionSet, LayoutOp, NodeId, Op, Role, Semantics};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub id: Option<WidgetNodeId>,
    pub child: Option<Box<Node>>,
    pub on_press: Option<ActionEnvelope>,
    pub semantics: Option<Semantics>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Button {
    fn should_attach_semantics(&self) -> bool {
        self.semantics.is_some() || self.on_press.is_some()
    }

    fn build_semantics(&self) -> Option<Semantics> {
        if !self.should_attach_semantics() {
            return None;
        }

        let mut semantics = self
            .semantics
            .clone()
            .unwrap_or_else(default_button_semantics);

        if let Some(action_envelope) = &self.on_press {
            semantics.actions.entries.push(ActionEntry {
                action_id: action_envelope.id.as_u128(),
                payload_data: Some(action_envelope.payload.clone()),
            });
        }

        Some(semantics)
    }
}

impl Desugar for Button {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let mut child_ids = Vec::new();
        if let Some(child) = &self.child {
            child_ids.push(child.desugar(cx));
        }

        let attach_semantics = self.should_attach_semantics();

        let layout_id;
        let mut semantics_id = None;

        if attach_semantics {
            layout_id = cx.next_node_id();
            semantics_id = Some(self.id.unwrap_or_else(|| cx.next_node_id()));
        } else {
            layout_id = self.id.unwrap_or_else(|| cx.next_node_id());
        }

        cx.add_node(
            layout_id,
            Op::Layout(LayoutOp::Box {
                width: self.width,
                height: self.height,
            }),
            child_ids,
        );

        if let Some(semantics_op) = self.build_semantics() {
            let semantics_id = semantics_id.unwrap_or_else(|| cx.next_node_id());
            cx.add_node(semantics_id, Op::Semantics(semantics_op), vec![layout_id]);
            return semantics_id;
        }

        layout_id
    }
}

fn default_button_semantics() -> Semantics {
    Semantics {
        role: Role::Button,
        label: None,
        value: None,
        actions: ActionSet::default(),
        focusable: true,
    }
}
