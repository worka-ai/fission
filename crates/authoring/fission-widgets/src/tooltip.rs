use fission_core::ui::{Container, Node, Text, TextContent, Positioned};
use fission_core::{BuildCtx, View, Widget, WidgetNodeId, NodeId};
use fission_core::op::Color;
use crate::flyout;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tooltip {
    pub id: WidgetNodeId,
    pub child: Box<Node>,
    pub text: String,
}

impl<S: fission_core::AppState> Widget<S> for Tooltip {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let node_id: NodeId = self.id.into();
        
        let trigger = Container::new(*self.child.clone())
            .id(node_id)
            .into_node();

        let is_hovered = view.runtime.interaction.is_hovered(node_id);

        if is_hovered {
            let tooltip_node = Container::new(
                    Text::new(self.text.clone())
                        .color(Color::WHITE)
                        .size(12.0)
                        .into_node(),
                )
                .bg(Color { r: 50, g: 50, b: 50, a: 255 })
                .border_radius(4.0)
                .padding_all(4.0)
                .into_node();
            let flyout_node = crate::flyout(node_id, tooltip_node);
            ctx.register_portal(flyout_node);
        }

        trigger
    }
}
