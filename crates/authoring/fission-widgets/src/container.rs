use fission_core::action::AppState;
use fission_core::registry::BuildCtx;
use fission_core::ui::{Text, TextContent};
use fission_core::view::View;
use fission_core::{Node, Widget};
use fission_ir::{
    Color,
    op::{Op, PaintOp, LayoutOp},
};

#[derive(Default)]
pub struct Container {
    pub child: Option<Node>,
    pub background: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
}

impl Container {
    pub fn new(child: Node) -> Self {
        Self {
            child: Some(child),
            ..Default::default()
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }
}

impl<S: AppState + 'static> Widget<S> for Container {
    fn build(&self, _ctx: &mut BuildCtx<S>, _view: &View<S>) -> Node {
        self.child.clone().unwrap_or_else(|| {
            Text {
                content: TextContent::Literal("".into()),
                ..Default::default()
            }
            .into()
        })
    }
}
