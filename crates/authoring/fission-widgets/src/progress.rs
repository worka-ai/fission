use fission_core::ui::{Container, Grid, GridItem, Node};
use fission_core::op::GridTrack;
use fission_core::{BuildCtx, View, Widget};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ProgressBar {
    pub value: f32, // 0.0 to 1.0
}

impl<S: fission_core::AppState> Widget<S> for ProgressBar {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        let tokens = &view.env.theme.tokens;
        let value = self.value.clamp(0.0, 1.0);
        let height = 4.0; // from theme? or constant

        // Track background
        let track_color = tokens.colors.surface; // or darker?
        // Actually, usually a dedicated track color or opacity.
        let track_color = fission_core::op::Color { r: 230, g: 230, b: 230, a: 255 }; // Light gray for now

        let fill_color = tokens.colors.primary;

        // Use Grid to achieve percentage width
        // Col 1: Fill (Percent)
        // Col 2: Remaining (Fr 1)
        // Note: If value is 0, Col 1 is 0.
        // If value is 1, Col 1 is 100%. Col 2 is 0? Fr(1) takes remaining.
        
        let pct = value * 100.0;
        
        // We wrap Grid in a Container to set the background (Track).
        // Then Grid item is the Fill.
        
        Container::new(
            Grid {
                columns: vec![GridTrack::Percent(pct), GridTrack::Fr(1.0)],
                rows: vec![GridTrack::Points(height)],
                children: vec![
                    GridItem::new(
                        Container::new(fission_core::ui::Row::default().into())
                            .bg(fill_color)
                            .border_radius(height / 2.0)
                            .into_node()
                    ).cell(1, 1).into()
                ],
                ..Default::default()
            }.into()
        )
        .bg(track_color)
        .border_radius(height / 2.0)
        .height(height)
        .into_node()
    }
}
