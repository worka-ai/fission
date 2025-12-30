use fission_core::ui::{Button, ButtonVariant, Container, Node, Text, TextContent};
use fission_core::{BuildCtx, View, Widget, ActionEnvelope};
use fission_core::op::Color;
use crate::stack::HStack;
use crate::Icon;
use fission_icons::material;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub label: String,
    pub on_click: Option<ActionEnvelope>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub items: Vec<BreadcrumbItem>,
}

impl<S: fission_core::AppState> Widget<S> for Breadcrumb {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let tokens = &view.env.theme.tokens;
        let mut children = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            let is_last = i == self.items.len() - 1;
            
            if i > 0 {
                children.push(
                    Icon::svg(material::navigation::chevron_right::regular())
                        .size(16.0)
                        .color(tokens.colors.text_secondary)
                        .into_node()
                );
            }

            if is_last || item.on_click.is_none() {
                children.push(
                    Text::new(item.label.clone())
                        .color(if is_last { tokens.colors.text_primary } else { tokens.colors.text_secondary })
                        // .weight(if is_last { Bold } else { Normal })
                        .into_node()
                );
            } else {
                children.push(
                    Button {
                        variant: ButtonVariant::Ghost,
                        child: Some(Box::new(
                            Text::new(item.label.clone())
                                .color(tokens.colors.text_secondary)
                                .into_node()
                        )),
                        on_press: item.on_click.clone(),
                        ..Default::default()
                    }.into_node()
                );
            }
        }

        HStack {
            spacing: Some(8.0),
            children,
        }.build(ctx, view)
    }
}
