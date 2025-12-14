use anyhow;
use fission_core::{op::Color as IrColor, Action, ActionId, AppState, BuildCtx}; // Import BuildCtx
use fission_macros::Action;
use fission_shell_desktop::DesktopApp;
use fission_widgets::{Button, Row, Text};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CounterState {
    value: i32,
}

impl AppState for CounterState {}

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Increment;

// // The handler function. Pure logic.
// fn on_increment(state: &mut CounterState, _action: Increment) {
//     state.value += 1;
//     println!("Counter incremented to: {}", state.value);
// }

fn ui(ctx: &mut BuildCtx<CounterState>) -> fission_widgets::Node {
    Row {
        children: vec![
            Text {
                value: "Count:".into(),
                width: Some(100.0),
                height: Some(50.0),
                font_size: Some(20.0),
                color: Some(IrColor::BLACK),
                ..Default::default()
            }
            .into(),
            Button {
                // Bind the handler here. returns ActionEnvelope.
                on_press: Some(ctx.bind(Increment, |state, _action| {
                    state.value += 1;
                    println!("Counter incremented to: {}", state.value);
                } /*on_increment*/)),
                child: Some(Box::new(
                    Text {
                        value: "Inc".into(),
                        width: Some(80.0),
                        height: Some(40.0),
                        font_size: Some(20.0),
                        color: Some(IrColor::WHITE),
                        ..Default::default()
                    }
                    .into(),
                )),
                width: Some(100.0),
                height: Some(60.0),
                ..Default::default()
            }
            .into(),
        ],
        ..Default::default()
    }
    .into()
}

fn main() -> anyhow::Result<()> {
    // Use the builder pattern
    let app = DesktopApp::<CounterState, _>::build(|ctx| ui(ctx));

    app.run()
}
