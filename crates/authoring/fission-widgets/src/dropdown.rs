use fission::prelude::*;

#[derive(Default)]
pub struct DropDown {
    // TODO: add fields
}

impl Widget<AppState> for DropDown {
    fn build(&self, _ctx: &mut BuildCtx<AppState>, _view: &View<AppState>) -> Node {
        // TODO: implement build
        Text::new("DropDown").into()
    }
}
