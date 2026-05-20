use super::title_block;
use crate::ui::actions::{execute_command, toggle_strict, ExecuteCommand, ToggleStrict};
use crate::ui::commands::UiCommand;
use crate::ui::components::{ActionButton, ButtonTone, KeyValueRow, TogglePill};
use crate::ui::state::{all_targets, target_label, UiState};
use crate::ui::theme::UiPalette;
use fission::prelude::*;

#[derive(Clone)]
pub(crate) struct DoctorScreen;

impl Widget<UiState> for DoctorScreen {
    fn build(&self, ctx: &mut BuildCtx<UiState>, view: &View<UiState>) -> Node {
        let palette = UiPalette::for_mode(view.state.theme_mode);
        let doctor_all = with_reducer!(ctx, ExecuteCommand(UiCommand::DoctorAll), execute_command);
        let strict = with_reducer!(ctx, ToggleStrict, toggle_strict);
        let mut target_checks = Vec::new();
        for target in all_targets() {
            let configured = view.state.targets.contains(&target);
            let action = with_reducer!(
                ctx,
                ExecuteCommand(UiCommand::DoctorTarget(target)),
                execute_command
            );
            target_checks.push(
                ActionButton::new(format!("Check {}", target_label(target)), action)
                    .tone(if configured {
                        ButtonTone::Primary
                    } else {
                        ButtonTone::Neutral
                    })
                    .width(22.0)
                    .build(ctx, view),
            );
        }

        Column {
            gap: Some(1.0),
            children: vec![
                title_block(
                    "Doctor",
                    "Check the Rust targets, platform SDKs, device tools, and local executables needed by each platform.",
                    palette.accent,
                    palette.muted,
                ),
                KeyValueRow::new("Project", view.state.project_dir.display().to_string())
                    .build(ctx, view),
                TogglePill::new("Strict exit status", view.state.strict, strict).build(ctx, view),
                ActionButton::new("Check all configured tooling", doctor_all)
                    .tone(ButtonTone::Primary)
                    .width(32.0)
                    .build(ctx, view),
                Text::new("Platform checks").color(palette.accent).into_node(),
                Column {
                    gap: Some(1.0),
                    children: target_checks,
                    ..Default::default()
                }
                .into_node(),
            ],
            ..Default::default()
        }
        .into_node()
    }
}
