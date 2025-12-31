use fission_core::ui::{Container, Row, Column, Text, Node};
use fission_core::{AppState, BuildCtx, View, Widget};
use fission_test::TestHarness;

#[derive(Debug, Default, Clone)]
struct State;
impl AppState for State {}

#[test]
fn text_wrap_increases_layout_height() {
    // Regression: text could render wrapped (paint) but be measured as 1-line (layout),
    // causing it to overlap the next line.
    //
    // We assert that when the text is width-constrained below its full width, its
    // layout height grows beyond a single line (MockTextMeasurer uses 20px line height).

    struct Root;
    impl Widget<State> for Root {
        fn build(&self, _ctx: &mut BuildCtx<State>, _view: &View<State>) -> Node {
            let long = "This is a very long subject line that should wrap into multiple lines";
            Container::new(
                Row::default()
                    .children(vec![
                        Container::new(fission_core::ui::widgets::spacer::Spacer::default().into_node())
                            .width(40.0)
                            .height(40.0)
                            .into_node(),
                        Container::new(
                            Column::default()
                                .children(vec![
                                    Text::new(long).into_node(),
                                    Text::new("Preview").into_node(),
                                ])
                                .into_node(),
                        )
                        .flex_grow(1.0)
                        .into_node(),
                        Text::new("10:00 AM").into_node(),
                    ])
                    .into_node(),
            )
            .width(300.0)
            .into_node()
        }
    }

    let mut h = TestHarness::new(State).with_root_widget(Root);
    h.pump().unwrap();

    let snap = h.last_snapshot.as_ref().unwrap();
    let ir = h.last_ir.as_ref().unwrap();

    let mut subject_rect = None;
    for (id, node) in &ir.nodes {
        if let fission_ir::Op::Paint(fission_ir::PaintOp::DrawText { text, .. }) = &node.op {
            if text.starts_with("This is a very long subject") {
                subject_rect = Some(snap.get_node_rect(*id).unwrap());
                break;
            }
        }
    }
    let subject_rect = subject_rect.expect("subject text node not found");

    // Full width with mock measurer: len * 10.0
    let full_w = "This is a very long subject line that should wrap into multiple lines".len() as f32 * 10.0;
    assert!(subject_rect.width() < full_w, "expected subject to be width-constrained");
    assert!(subject_rect.height() > 20.0, "expected wrapped height > 20, got {}", subject_rect.height());
}

