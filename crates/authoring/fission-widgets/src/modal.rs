use fission_core::ui::{Button, ButtonVariant, Container, Node, Text, TextContent, ZStack};
use fission_core::{BuildCtx, View, Widget, ActionEnvelope, WidgetNodeId, NodeId};
use fission_core::op::{Color, BoxShadow};
use crate::stack::{VStack, HStack};
use crate::{Icon, Portal};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Modal {
    pub id: WidgetNodeId,
    pub title: String,
    pub content: Box<Node>,
    pub is_open: bool,
    pub on_dismiss: Option<ActionEnvelope>,
    pub actions: Vec<ModalAction>,
    pub width: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModalAction {
    pub label: String,
    pub on_press: Option<ActionEnvelope>,
    pub is_primary: bool,
}

impl<S: fission_core::AppState> Widget<S> for Modal {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        if !self.is_open {
            return fission_core::ui::widgets::Spacer::default().into_node();
        }

        let tokens = &view.env.theme.tokens;
        let node_id = NodeId::derived(self.id.as_u128(), &[]);

        // Dimmed backdrop
        let backdrop = Container::new(fission_core::ui::widgets::Spacer::default().into_node())
            .bg(Color { r: 0, g: 0, b: 0, a: 128 }) // 50% opacity black
            .into_node();
        
        // Backdrop click handling? 
        // fission-core Button only wraps children.
        // We can make the backdrop a Ghost Button that fills available space.
        // For now, assume backdrop is just visual.
        // We'll wrap it in a Button if we want click-to-dismiss.
        let backdrop_btn = Button {
            variant: ButtonVariant::Ghost,
            child: Some(Box::new(backdrop)),
            on_press: self.on_dismiss.clone(),
            ..Default::default()
        }.into_node();

        // Modal Content
        let mut action_buttons = Vec::new();
        for action in &self.actions {
            action_buttons.push(
                Button {
                    variant: if action.is_primary { ButtonVariant::Filled } else { ButtonVariant::Outline },
                    child: Some(Box::new(Text::new(action.label.clone())
                        .color(if action.is_primary { tokens.colors.on_primary } else { tokens.colors.primary })
                        .into_node())),
                    on_press: action.on_press.clone(),
                    ..Default::default()
                }.into_node()
            );
        }

        let modal_card = Container::new(
            VStack {
                spacing: Some(16.0),
                children: vec![
                    // Header
                    HStack {
                        spacing: Some(8.0),
                        children: vec![
                            Text::new(self.title.clone())
                                .size(20.0)
                                .into_node(),
                            fission_core::ui::widgets::Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                            Button {
                                variant: ButtonVariant::Ghost,
                                child: Some(Box::new(Icon::svg(fission_icons::material::navigation::close::regular()).size(20.0).into_node())),
                                on_press: self.on_dismiss.clone(),
                                ..Default::default()
                            }.into_node(),
                        ]
                    }.into_node(),
                    
                    // Content
                    *self.content.clone(),
                    
                    // Footer Actions
                    HStack {
                        spacing: Some(8.0),
                        // Align right using spacer
                        children: vec![
                            fission_core::ui::widgets::Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                        ].into_iter().chain(action_buttons).collect(),
                    }.into_node(),
                ]
            }.into_node()
        )
        .bg(tokens.colors.surface)
        .border_radius(tokens.radii.large)
        .shadow(tokens.elevations.level3.unwrap_or(BoxShadow {
            color: Color { r: 0, g: 0, b: 0, a: 60 },
            blur_radius: 16.0,
            offset: (0.0, 8.0),
        }))
        .width(self.width.unwrap_or(400.0))
        .padding_all(24.0)
        .into_node();

        // Center the modal
        // We use a ZStack for the portal root (which fills window).
        // Layer 1: Backdrop (fills window).
        // Layer 2: Modal (centered).
        
        let root = Container::new(
            ZStack {
                children: vec![
                    // Layer 1: Absolute fill backdrop
                    fission_core::ui::Positioned {
                        left: Some(0.0), right: Some(0.0), top: Some(0.0), bottom: Some(0.0),
                        child: Some(Box::new(backdrop_btn)),
                        ..Default::default()
                    }.into_node(),
                    
                    // Layer 2: Modal Card (Flex item, centered by parent Container)
                    modal_card
                ],
                id: None,
            }.into_node()
        )
        .into_node();
        
        let positioned_root = fission_core::ui::Positioned {
            left: Some(0.0), right: Some(0.0), top: Some(0.0), bottom: Some(0.0),
            child: Some(Box::new(root)),
            ..Default::default()
        }.into_node();

        ctx.register_portal(positioned_root);

        // Return empty node for the widget tree position
        fission_core::ui::widgets::Spacer::default().into_node()
    }
}
