use fission_widgets::{Button, Text, Row};
use fission_core::{Action, AppState, ActionId, op::Color as IrColor}; // Import IrColor from fission_core
use fission_macros::Action;
use fission_shell_desktop::DesktopApp;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static; 
use anyhow; 

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
                Text { 
                    value: "Count:".into(), 
                    width: Some(100.0),
                    height: Some(50.0),
                    font_size: Some(20.0),
                    color: Some(IrColor::BLACK),
                    ..Default::default() 
                }.into(),
                Button { 
                    on_press: Some(Increment.into()), 
                    child: Some(Box::new(Text { 
                        value: "Inc".into(), 
                        width: Some(80.0),
                        height: Some(40.0),
                        font_size: Some(20.0),
                        color: Some(IrColor::WHITE),
                        ..Default::default() 
                    }.into())),
                    width: Some(100.0),
                    height: Some(60.0),
                    ..Default::default() 
                }.into(),
            ],
            ..Default::default()
        }
    );

    app.run()
}