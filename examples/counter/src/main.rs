use fission_widgets::{Button, Text, Row};
use fission_core::{Action, AppState, ActionId};
use fission_macros::Action;
use fission_shell_desktop::DesktopApp;
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CounterState {
    value: i32,
}

impl AppState for CounterState {}

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Increment;

fn main() -> anyhow::Result<()> {
    let app = DesktopApp::<CounterState, _>::new(
        Row {
            children: vec![
                Text { value: "Count:".into(), ..Default::default() }.into(),
                Button { 
                    on_press: Some(Increment.into()), 
                    child: Some(Box::new(Text { value: "Inc".into(), ..Default::default() }.into())),
                    ..Default::default() 
                }.into(),
            ],
            ..Default::default()
        }
    );

    app.run()
}
