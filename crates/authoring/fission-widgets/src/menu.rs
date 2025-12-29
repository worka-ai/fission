use fission_core::ui::{Button, ButtonVariant, Container, Node, Text, TextContent, Positioned};
use fission_core::{BuildCtx, View, Widget, ActionEnvelope, WidgetNodeId, NodeId};
use fission_core::op::{Color, BoxShadow};
use crate::stack::VStack;
use crate::flyout;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub on_select: Option<ActionEnvelope>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuButton {
    pub id: WidgetNodeId,
    pub label: String,
    pub items: Vec<MenuItem>,
    pub is_open: bool,
    pub on_toggle: Option<ActionEnvelope>,
    pub on_dismiss: Option<ActionEnvelope>,
}

impl<S: fission_core::AppState> Widget<S> for MenuButton {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let tokens = &view.env.theme.tokens;
        let anchor_id = NodeId::derived(self.id.as_u128(), &[]);

        // Trigger Button
        let trigger = Button {
            id: Some(anchor_id),
            variant: ButtonVariant::Outline,
            child: Some(Box::new(
                Text { 
                    content: TextContent::Literal(self.label.clone()), 
                    color: Some(tokens.colors.primary),
                    ..Default::default() 
                }.into()
            )),
            on_press: self.on_toggle.clone(),
            ..Default::default()
        }.into();

        // Menu Overlay
        if self.is_open {
            // Build menu content and place it with explicit coordinates below the anchor.
            let mut menu_items = Vec::new();
                for item in &self.items {
                    menu_items.push(
                        Button {
                            variant: ButtonVariant::Ghost,
                            child: Some(Box::new(
                                Container::new(
                                    Text { content: TextContent::Literal(item.label.clone()), ..Default::default() }.into()
                                )
                                .padding_all(8.0)
                                .into_node()
                            )),
                            on_press: item.on_select.clone(),
                            width: Some(150.0), // Fixed width for menu items for now
                            ..Default::default()
                        }.into()
                    );
                }

                let content_node = Container::new(
                    VStack {
                        spacing: Some(0.0),
                        children: menu_items,
                    }
                    .build(ctx, view),
                )
                .bg(Color::WHITE)
                .border(tokens.colors.border, 1.0)
                .shadow(BoxShadow {
                    color: Color { r: 0, g: 0, b: 0, a: 50 },
                    blur_radius: 10.0,
                    offset: (0.0, 4.0),
                })
                .padding_all(4.0)
                .into_node();

                // Use engine-level Flyout for robust positioning
            // Use engine-level Flyout for robust positioning using previous snapshot
            let flyout_node = flyout(anchor_id, content_node);
            ctx.register_portal(flyout_node);

            // Backdrop? click-outside can be implemented as a transparent portal layer.
        }

        trigger
    }
}
