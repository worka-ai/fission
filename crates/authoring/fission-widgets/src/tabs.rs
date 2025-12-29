use fission_core::ui::{Button, ButtonVariant, Container, Node, Text, TextContent};
use fission_core::{BuildCtx, View, Widget, ActionEnvelope};
use fission_core::op::Color;
use crate::stack::{HStack, VStack};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabItem {
    pub title: String,
    pub content: Node,
    pub on_select: Option<ActionEnvelope>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Tabs {
    pub selected_index: usize,
    pub tabs: Vec<TabItem>,
}

impl<S: fission_core::AppState> Widget<S> for Tabs {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let tokens = &view.env.theme.tokens;
        
        // Tab Headers
        let mut headers = Vec::new();
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_selected = i == self.selected_index;
            
            // Styled Tab Button
            let btn = Button {
                variant: ButtonVariant::Ghost,
                child: Some(Box::new(
                    Container::new(
                        Text {
                            content: TextContent::Literal(tab.title.clone()),
                            color: Some(if is_selected { tokens.colors.primary } else { tokens.colors.text_secondary }),
                            ..Default::default()
                        }.into()
                    )
                    .padding_all(tokens.spacing.m)
                    // Bottom border if selected
                    .border(
                        if is_selected { tokens.colors.primary } else { Color { r:0,g:0,b:0,a:0 } }, // Transparent if not selected
                        if is_selected { 2.0 } else { 0.0 }
                    )
                    .into_node()
                )),
                on_press: tab.on_select.clone(),
                ..Default::default()
            }.into();
            headers.push(btn);
        }

        // Tab Content
        let content = if let Some(tab) = self.tabs.get(self.selected_index) {
            tab.content.clone()
        } else {
            Container::new(fission_core::ui::Row::default().into()).into_node()
        };

        VStack {
            spacing: Some(0.0),
            children: vec![
                HStack { spacing: Some(0.0), children: headers }.build(ctx, view),
                content,
            ]
        }.build(ctx, view)
    }
}