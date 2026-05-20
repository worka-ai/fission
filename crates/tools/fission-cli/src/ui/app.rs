use super::components::AppShell;
use super::screens::ActiveScreen;
use super::state::UiState;
use fission::prelude::*;

#[derive(Clone)]
pub(crate) struct CliUiApp;

impl Widget<UiState> for CliUiApp {
    fn build(&self, ctx: &mut BuildCtx<UiState>, view: &View<UiState>) -> Node {
        let content = ActiveScreen.build(ctx, view);
        AppShell { content }.build(ctx, view)
    }
}
