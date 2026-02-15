use crate::stack::HStack;
use fission_core::ui::{Container, Node};
use fission_core::{
    AnimationPropertyId, AnimationRequest, AnimationStartValue, BuildCtx, View, Widget,
    WidgetNodeId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spinner {
    pub id: WidgetNodeId,
    pub color: Option<fission_core::op::Color>,
}

impl<S: fission_core::AppState> Widget<S> for Spinner {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let tokens = &view.env.theme.tokens;
        let color = self.color.unwrap_or(tokens.colors.primary);
        let dot_size = 10.0;

        let mut dots = Vec::new();

        for i in 0..3 {
            // Generate stable sub-ID for animation
            // Hacking a sub-ID by XORing? Or using a deterministic derivation if available.
            // WidgetNodeId doesn't expose derivation.
            // But we can create a new explicit one if we assume `id` is unique.
            // Or hash it.
            // Let's assume we can construct one.
            let sub_id_u128 = self.id.as_u128() ^ (i as u128 + 1);
            let sub_id = WidgetNodeId::from_u128(sub_id_u128);

            // Request Animation
            ctx.anim_for(sub_id).request(AnimationRequest {
                property: AnimationPropertyId::Opacity,
                from: AnimationStartValue::Explicit(0.3),
                to: 1.0,
                duration_ms: 600,
                repeat: true,
                delay_ms: i as u64 * 200,
            });

            // Apply animated value
            let opacity = view.animation_value(sub_id, &AnimationPropertyId::Opacity);
            let mut dot_color = color;
            dot_color.a = (opacity * 255.0) as u8;

            dots.push(
                Container::new(fission_core::ui::Row::default().into())
                    .size(dot_size, dot_size)
                    .bg(dot_color)
                    .border_radius(dot_size / 2.0)
                    .into_node(),
            );
        }

        HStack {
            spacing: Some(6.0),
            children: dots,
        }
        .build(ctx, view)
    }
}
