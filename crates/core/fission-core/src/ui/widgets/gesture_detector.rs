use crate::lowering::{LoweringContext, NodeBuilder};
use crate::ui::traits::Lower;
use crate::ui::Node;
use crate::ActionEnvelope;
use fission_ir::{
    ActionEntry, NodeId, Op, Semantics, 
    semantics::{ActionTrigger, Role},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureDetector {
    pub id: Option<NodeId>,
    pub child: Box<Node>,
    pub on_tap: Option<ActionEnvelope>,
    pub on_double_tap: Option<ActionEnvelope>,
    pub on_long_press: Option<ActionEnvelope>,
    pub on_drag_start: Option<ActionEnvelope>,
    pub on_drag_update: Option<ActionEnvelope>,
    pub on_drag_end: Option<ActionEnvelope>,
    pub on_hover_enter: Option<ActionEnvelope>,
    pub on_hover_exit: Option<ActionEnvelope>,
}

impl Default for GestureDetector {
    fn default() -> Self {
        Self {
            id: None,
            child: Box::new(crate::ui::widgets::spacer::Spacer::default().into_node()),
            on_tap: None,
            on_double_tap: None,
            on_long_press: None,
            on_drag_start: None,
            on_drag_update: None,
            on_drag_end: None,
            on_hover_enter: None,
            on_hover_exit: None,
        }
    }
}

impl GestureDetector {
    pub fn new(child: Node) -> Self {
        Self {
            child: Box::new(child),
            ..Default::default()
        }
    }

    pub fn into_node(self) -> crate::ui::Node {
        crate::ui::Node::GestureDetector(self)
    }
}

impl Lower for GestureDetector {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let id = self.id.unwrap_or_else(|| cx.next_node_id());
        
        // Lower child
        let child_id = self.child.lower(cx);
        
        // Build Semantics
        let mut semantics = Semantics {
            role: Role::Generic, // Generic container? Or specific role?
            // "GestureDetector" isn't a role. It decorates.
            // If the child has semantics, we wrap it?
            // Yes, Semantics node wraps child layout node.
            // If child is also Semantics, we have nested Semantics.
            // Runtime needs to handle bubbling or priority.
            // Currently runtime `handle_input` finds *closest* (deepest) semantics.
            
            label: None,
            value: None,
            actions: Default::default(),
            focusable: self.on_tap.is_some(), // Tappable implies focusable usually?
            multiline: false,
            masked: false,
            input_mask: None,
            ime_preedit_range: None,
            checked: None,
            disabled: false,
            draggable: self.on_drag_start.is_some() || self.on_drag_update.is_some(),
            scrollable_x: false,
            scrollable_y: false,
            min_value: None,
            max_value: None,
            current_value: None,
        };

        if let Some(a) = &self.on_tap {
            semantics.actions.entries.push(ActionEntry {
                trigger: ActionTrigger::Default, // Tap
                action_id: a.id.as_u128(),
                payload_data: Some(a.payload.clone()),
            });
        }
        
        // TODO: Map other triggers when ActionEntry supports them (I added them to ActionTrigger enum!)
        // I added: DragStart, DragUpdate, DragEnd, HoverEnter, HoverExit.
        
        if let Some(a) = &self.on_drag_start {
            semantics.actions.entries.push(ActionEntry {
                trigger: ActionTrigger::DragStart,
                action_id: a.id.as_u128(),
                payload_data: Some(a.payload.clone()),
            });
        }
        
        if let Some(a) = &self.on_drag_update {
            semantics.actions.entries.push(ActionEntry {
                trigger: ActionTrigger::DragUpdate,
                action_id: a.id.as_u128(),
                payload_data: Some(a.payload.clone()),
            });
        }
        
        if let Some(a) = &self.on_drag_end {
            semantics.actions.entries.push(ActionEntry {
                trigger: ActionTrigger::DragEnd,
                action_id: a.id.as_u128(),
                payload_data: Some(a.payload.clone()),
            });
        }
        
        if let Some(a) = &self.on_hover_enter {
            semantics.actions.entries.push(ActionEntry {
                trigger: ActionTrigger::HoverEnter,
                action_id: a.id.as_u128(),
                payload_data: Some(a.payload.clone()),
            });
        }
        
        if let Some(a) = &self.on_hover_exit {
            semantics.actions.entries.push(ActionEntry {
                trigger: ActionTrigger::HoverExit,
                action_id: a.id.as_u128(),
                payload_data: Some(a.payload.clone()),
            });
        }

        let mut node = NodeBuilder::new(id, Op::Semantics(semantics));
        node.add_child(child_id);
        node.build(cx)
    }
}
