use anyhow::Result;
use fission_core::ui::{Grid, GridItem, Node, TextInput};
use fission_core::{BuildCtx, View, Widget, op::GridTrack, WidgetNodeId, NodeId};
use fission_widgets::{HStack, VStack, MenuButton, MenuItem};
use fission_shell_desktop::Pipeline;
use fission_core::env::Env;
use fission_core::lowering::LoweringContext;
use fission_core::Runtime;
use fission_layout::{LayoutEngine, TextMeasurer, LineMetric};
use fission_render::{DisplayList, Renderer};
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
struct AppState { open: bool }
impl fission_core::action::AppState for AppState {}

struct MockRenderer;
impl Renderer for MockRenderer {
    fn render(&mut self, _dl: &DisplayList) -> Result<()> { Ok(()) }
}

struct MockMeasurer;
impl TextMeasurer for MockMeasurer {
    fn measure(&self, _text: &str, _font_size: f32, _available_width: Option<f32>) -> (f32, f32) { (80.0, 20.0) }
    fn hit_test(&self, _text: &str, _font_size: f32, _available_width: Option<f32>, _x: f32, _y: f32) -> usize { 0 }
    fn get_line_metrics(&self, _text: &str, _font_size: f32, _available_width: Option<f32>) -> Vec<LineMetric> { vec![] }
}

struct Root;
impl Widget<AppState> for Root {
    fn build(&self, ctx: &mut BuildCtx<AppState>, view: &View<AppState>) -> Node {
        Grid {
            columns: vec![GridTrack::Points(220.0), GridTrack::Points(380.0), GridTrack::Fr(1.0)],
            rows: vec![GridTrack::Fr(1.0)],
            children: vec![
                GridItem::new(
                    VStack { spacing: Some(0.0), children: vec![
                        HStack { spacing: Some(8.0), children: vec![
                            // A stable reference input whose geometry must not change
                            TextInput { width: Some(200.0), ..Default::default() }.into(),
                            // The menu under test
                            MenuButton {
                                id: WidgetNodeId::explicit("filter_menu"),
                                label: "Filter".into(),
                                is_open: view.state.open,
                                on_toggle: None,
                                on_dismiss: None,
                                items: vec![
                                    MenuItem { label: "All".into(), on_select: None },
                                    MenuItem { label: "Unread".into(), on_select: None },
                                ],
                            }.build(ctx, view),
                        ]}.build(ctx, view),
                    ]}.build(ctx, view)
                ).cell(1, 2).into(),
            ],
            ..Default::default()
        }.into()
    }
}

#[test]
fn flyout_does_not_shift_content() -> Result<()> {
    let env = Env::default();
    let mut runtime = Runtime::default();
    runtime.add_app_state(Box::new(AppState { open: false }))?;
    let mut layout = LayoutEngine::new().with_measurer(Arc::new(MockMeasurer));
    let mut pipe = Pipeline::new();

    // Frame 1: closed
    let (node_tree, portals) = {
        let state = runtime.get_app_state::<AppState>().unwrap();
        let view = View::new(state, &runtime.runtime_state, &env, pipe.last_snapshot.as_ref());
        let mut ctx = BuildCtx::new();
        let node = Root.build(&mut ctx, &view);
        (node, ctx.take_portals())
    };
    let final_root = if portals.is_empty() {
        node_tree
    } else {
        use fission_core::ui::{Overlay, ZStack};
        Node::Overlay(Overlay { id: None, content: Box::new(node_tree), overlay: Box::new(Node::ZStack(ZStack { children: portals, ..Default::default() })) })
    };
    let mut cx = LoweringContext::new(&env, &runtime.runtime_state, None, pipe.last_snapshot.as_ref());
    let root_id = final_root.lower(&mut cx);
    cx.ir.root = Some(root_id);
    let ir1 = cx.ir;
    let viewport = fission_layout::LayoutSize { width: 1024.0, height: 768.0 };
    let _ = pipe.render(ir1.clone(), viewport, &mut layout, &runtime.runtime_state.scroll, &mut MockRenderer, &runtime.runtime_state.video)?;
    let snap1 = pipe.last_snapshot.clone().expect("snap1");

    // Record geometry of a stable node (the TextInput)
    let input_id = NodeId::derived(WidgetNodeId::explicit("filter_menu").as_u128(), &[]);
    let anchor_rect1 = snap1.get_node_rect(input_id).expect("anchor rect1");

    // Frame 2: open
    if let Some(state) = runtime.get_app_state_mut::<AppState>() { state.open = true; }
    let (node_tree2, portals2) = {
        let state = runtime.get_app_state::<AppState>().unwrap();
        let view = View::new(state, &runtime.runtime_state, &env, pipe.last_snapshot.as_ref());
        let mut ctx = BuildCtx::new();
        let node = Root.build(&mut ctx, &view);
        (node, ctx.take_portals())
    };
    let final_root2 = if portals2.is_empty() {
        node_tree2
    } else {
        use fission_core::ui::{Overlay, ZStack};
        Node::Overlay(Overlay { id: None, content: Box::new(node_tree2), overlay: Box::new(Node::ZStack(ZStack { children: portals2, ..Default::default() })) })
    };
    let mut cx2 = LoweringContext::new(&env, &runtime.runtime_state, None, pipe.last_snapshot.as_ref());
    let root_id2 = final_root2.lower(&mut cx2);
    cx2.ir.root = Some(root_id2);
    let ir2 = cx2.ir;
    let _ = pipe.render(ir2.clone(), viewport, &mut layout, &runtime.runtime_state.scroll, &mut MockRenderer, &runtime.runtime_state.video)?;
    let snap2 = pipe.last_snapshot.clone().expect("snap2");

    let anchor_rect2 = snap2.get_node_rect(input_id).expect("anchor rect2");

    // Assert anchor rect unchanged (opening flyout must not affect normal layout)
    assert!((anchor_rect1.x() - anchor_rect2.x()).abs() < 1.0, "anchor x changed when opening flyout");
    assert!((anchor_rect1.y() - anchor_rect2.y()).abs() < 1.0, "anchor y changed when opening flyout");
    assert!((anchor_rect1.width() - anchor_rect2.width()).abs() < 1.0, "anchor w changed when opening flyout");
    assert!((anchor_rect1.height() - anchor_rect2.height()).abs() < 1.0, "anchor h changed when opening flyout");

    Ok(())
}

