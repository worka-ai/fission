use fission_core::ui::{Button, ButtonVariant, Container, Node, Text, TextContent, Positioned};
use fission_core::{BuildCtx, View, Widget, ActionEnvelope, WidgetNodeId, NodeId};
use fission_core::op::{Color, BoxShadow};
use crate::stack::VStack;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectItem<V> {
    pub label: String,
    pub value: V,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Select<V> {
    pub id: WidgetNodeId,
    pub value: V,
    // For now, simple string-based select or generic if we can serialize it.
    // Making Widget generic over V requires V: Serialize + Deserialize + PartialEq + Clone.
    // For simplicity, let's assume V is String for now or usize.
    // But `Widget<S>` trait doesn't constrain V.
    // We'll make `Select` struct generic but specialized impl or assume simple types.
    // Let's stick to `Select` taking `items: Vec<(String, V)>` and `on_change: ActionEnvelope`.
    // The action payload will need to carry the value.
    // Fission Action system payload is `Vec<u8>`.
    // We can't easily make a generic Widget without trait bounds on V spreading everywhere.
    // Let's implement `Select` for String values for now.
    pub items: Vec<(String, String)>,
    pub on_change: Option<ActionEnvelope>, // Action(String)
    pub is_open: bool,
    pub on_toggle: Option<ActionEnvelope>,
}

impl<S: fission_core::AppState> Widget<S> for Select<String> {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let tokens = &view.env.theme.tokens;
        let anchor_id = NodeId::derived(self.id.as_u128(), &[]);

        let selected_label = self.items.iter()
            .find(|(_, v)| v == &self.value)
            .map(|(l, _)| l.clone())
            .unwrap_or_else(|| "Select...".into());

        // Trigger
        let trigger = Button {
            id: Some(anchor_id),
            variant: ButtonVariant::Outline,
            child: Some(Box::new(
                Text { 
                    content: TextContent::Literal(selected_label), 
                    color: Some(tokens.colors.primary),
                    ..Default::default() 
                }.into()
            )),
            on_press: self.on_toggle.clone(),
            ..Default::default()
        }.into();

        // Dropdown
        if self.is_open {
            if let Some(rect) = view.get_rect(self.id) {
                let x = rect.origin.x;
                let y = rect.bottom() + 4.0;

                let mut options = Vec::new();
                for (label, val) in &self.items {
                    // We need to bind action with payload `val`.
                    // But `on_change` is already an envelope.
                    // We can't easily "clone and modify payload" of an envelope without knowing the Action type.
                    // The caller should provide a way to generate envelopes?
                    // Or `Select` takes `fn(V) -> ActionEnvelope`? No, closures not allowed.
                    // `Select` takes `ActionId` and we serialize `val`?
                    // We'll assume `on_change` is a "template" and we replace payload?
                    // This is unsafe.
                    
                    // Better: `Select` takes `on_change: Box<dyn Fn(String) -> ActionEnvelope>`. Not serializable.
                    
                    // The idiomatic Fission way:
                    // `items` contains `(Label, ActionEnvelope)`.
                    // Caller constructs the envelope for each item.
                    
                    // So `Select` doesn't need to know about V.
                    // `Select` just displays `label` and dispatches `action`.
                }
            }
        }
        
        trigger
    }
}
