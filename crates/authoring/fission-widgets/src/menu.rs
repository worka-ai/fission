use fission_core::ui::{Button, ButtonVariant, Container, Node, Text, TextContent, Positioned};
use fission_core::{BuildCtx, View, Widget, ActionEnvelope, WidgetNodeId, NodeId};
use fission_core::op::{Color, BoxShadow};
use crate::stack::VStack;
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
            if let Some(rect) = view.get_rect(self.id) {
                // Position: Bottom-Left aligned with anchor
                // Visual coordinates (LayoutSnapshot is visual)
                let x = rect.origin.x;
                let y = rect.bottom() + 4.0;

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

                let content = Positioned {
                    left: Some(x),
                    top: Some(y),
                    child: Some(Box::new(
                        Container::new(
                            VStack {
                                spacing: Some(0.0),
                                children: menu_items,
                            }.build(ctx, view)
                        )
                        .bg(Color::WHITE)
                        .border(tokens.colors.border, 1.0)
                        .shadow(BoxShadow { 
                            color: Color { r:0, g:0, b:0, a:50 }, 
                            blur_radius: 10.0, 
                            offset: (0.0, 4.0) 
                        })
                        .padding_all(4.0)
                        .into_node()
                    )),
                    ..Default::default()
                }.build(ctx, view);

                ctx.register_portal(content);
                
                // Backdrop? 
                // We should add a transparent full-screen button behind menu to dismiss?
                // For now, assume click-outside logic handles it (not yet implemented fully).
                // Or explicit close button.
            }
        }

        trigger
    }
}
