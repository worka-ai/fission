use fission_core::ui::{Button, Text, Column, Node, CustomNode, TextContent};
use fission_core::{AppState, ActionEnvelope, op::Color as IrColor, LoweringContext, LowerDyn, FlexDirection, Widget, View, BuildCtx, Selector}; 
use fission_macros::Action;
use fission_shell_desktop::DesktopApp;
use serde::{Serialize, Deserialize};
use anyhow;
use std::sync::Arc;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CounterState {
    value: i32,
}

impl AppState for CounterState {}

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Increment;

fn on_increment(state: &mut CounterState, _action: Increment) {
    state.value += 1;
    println!("Counter incremented to: {}", state.value);
}

struct CounterVM {
    label: String,
}

impl Selector<CounterState> for CounterVM {
    type Output = CounterVM;

    fn select(view: &View<CounterState>) -> Self::Output {
        CounterVM {
            label: format!("Count: {}", view.state.value),
        }
    }
}

struct CounterApp;

impl Widget<CounterState> for CounterApp {
    fn build(&self, ctx: &mut BuildCtx<CounterState>, view: &View<CounterState>) -> Node {
        let vm = view.select::<CounterVM>();

        Column {
            children: vec![
                Text { 
                    content: TextContent::Literal(vm.label),
                    font_size: Some(24.0),
                    ..Default::default() 
                }.into(),
                
                Button { 
                    on_press: Some(ctx.bind(Increment, on_increment)), 
                    child: Some(Box::new(Text { 
                        content: TextContent::Literal("Increment".into()), 
                        color: Some(IrColor::WHITE),
                        ..Default::default() 
                    }.into())),
                    width: Some(120.0),
                    ..Default::default() 
                }.into(),
            ],
            ..Default::default()
        }.into()
    }
}

fn main() -> anyhow::Result<()> {
    let app = DesktopApp::new(CounterApp);
    app.run()
}
