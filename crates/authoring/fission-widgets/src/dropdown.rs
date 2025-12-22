use fission_core::action::{Action, ActionEnvelope, AppState};
use fission_core::ui::{Button, Text, TextContent};
use fission_core::{Node, Widget, BuildCtx, View};

#[derive(Default, Clone)]
pub struct DropDown {
    pub on_toggle: Option<ActionEnvelope>,
    pub options: Vec<String>,
    pub on_select: Option<ActionEnvelope>,
    pub selected: Option<String>,
}

impl<S: AppState + 'static> Widget<S> for DropDown {
    fn build(&self, _ctx: &mut BuildCtx<S>, _view: &View<S>) -> Node {
        let button_text = self.selected.as_deref().unwrap_or("Select an option");

        Button {
            child: Some(Box::new(
                Text {
                    content: TextContent::Literal(button_text.into()),
                    ..Default::default()
                }
                .into(),
            )),
            on_press: self.on_toggle.clone(),
            ..Default::default()
        }
        .into()
    }
}
