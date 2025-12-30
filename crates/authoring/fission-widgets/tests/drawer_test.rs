use fission_core::{BuildCtx, View, Widget, WidgetNodeId, NodeId, AppState};
use fission_core::ui::{Node, Text};
use fission_widgets::{Drawer, DrawerSide, Container};
use std::sync::Arc;

#[derive(Default, Clone, Debug)]
struct State;
impl AppState for State {}

#[test]
fn test_drawer_registers_portal_when_open() {
    let mut runtime = fission_core::Runtime::default();
    runtime.add_app_state(Box::new(State)).unwrap();
    
    let mut ctx = BuildCtx::<State>::new();
    let env = fission_core::Env::default();
    let state = runtime.get_app_state::<State>().unwrap();
    let view = View::new(state, &runtime.runtime_state, &env, None);
    
    let drawer = Drawer {
        id: WidgetNodeId::explicit("drawer"),
        side: DrawerSide::Left,
        is_open: true,
        on_dismiss: None,
        content: Box::new(Text::new("Drawer Content").into_node()),
        width: Some(250.0),
    };
    
    let _ = drawer.build(&mut ctx, &view);
    
    let portals = ctx.take_portals();
    assert_eq!(portals.len(), 1, "Drawer should register a portal when open");
}
