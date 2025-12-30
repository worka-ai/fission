use fission_core::ui::{Container, Node, Text};
use fission_core::{BuildCtx, View, Widget};
use fission_core::op::Color;
use crate::stack::{VStack, HStack};
use crate::Icon;
use fission_icons::material;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AlertKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub kind: AlertKind,
    pub title: String,
    pub description: Option<String>,
}

impl<S: fission_core::AppState> Widget<S> for Alert {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let tokens = &view.env.theme.tokens;
        
        let (icon_path, bg_color, text_color) = match self.kind {
            AlertKind::Info => (
                material::action::info::regular(), 
                Color { r: 229, g: 246, b: 253, a: 255 }, // Light Blue
                Color { r: 1, g: 67, b: 97, a: 255 }      // Dark Blue
            ),
            AlertKind::Success => (
                material::action::check_circle::regular(),
                Color { r: 237, g: 247, b: 237, a: 255 }, // Light Green
                Color { r: 30, g: 70, b: 32, a: 255 }     // Dark Green
            ),
            AlertKind::Warning => (
                material::action::report_problem::regular(),
                Color { r: 255, g: 244, b: 229, a: 255 }, // Light Orange
                Color { r: 102, g: 60, b: 0, a: 255 }     // Dark Orange
            ),
            AlertKind::Error => (
                material::alert::error::regular(),
                Color { r: 253, g: 236, b: 234, a: 255 }, // Light Red
                Color { r: 97, g: 26, b: 21, a: 255 }     // Dark Red
            ),
        };

        let mut text_stack = vec![
            Text::new(self.title.clone())
                // .weight(Bold)
                .color(text_color)
                .into_node()
        ];
        
        if let Some(desc) = &self.description {
            text_stack.push(
                Text::new(desc.clone())
                    .color(text_color)
                    .size(12.0)
                    .into_node()
            );
        }

        Container::new(
            HStack {
                spacing: Some(12.0),
                children: vec![
                    Icon::svg(icon_path).color(text_color).size(24.0).into_node(),
                    VStack {
                        spacing: Some(4.0),
                        children: text_stack,
                    }.into_node()
                ]
            }.into_node()
        )
        .bg(bg_color)
        .border_radius(tokens.radii.medium)
        .padding_all(12.0)
        .into_node()
    }
}
