use fission_core::{BuildCtx, View, Widget, WidgetNodeId, NodeId, AppState};
use fission_core::ui::{Node, Text};
use fission_widgets::{MenuButton, MenuItem, Toast, ToastKind, Tooltip};
use fission_core::env::Env;
use std::sync::Arc;

#[derive(Default, Clone, Debug)]
struct State {
    menu_open: bool,
    toast_visible: bool,
}
impl AppState for State {}

#[test]
fn test_menu_button_registers_portal_when_open() {
    let mut runtime = fission_core::Runtime::default();
    runtime.add_app_state(Box::new(State { menu_open: true, toast_visible: false })).unwrap();
    
    let mut ctx = BuildCtx::<State>::new();
    let env = Env::default();
    let state = runtime.get_app_state::<State>().unwrap();
    let view = View::new(state, &runtime.runtime_state, &env, None);
    
    let menu_button = MenuButton {
        id: WidgetNodeId::explicit("test_menu"),
        label: "Menu".into(),
        items: vec![
            MenuItem { label: "Item 1".into(), icon: None, on_select: None },
        ],
        is_open: true,
        on_toggle: None,
    };
    
    let _ = menu_button.build(&mut ctx, &view);
    
    let portals = ctx.take_portals();
    assert_eq!(portals.len(), 1, "MenuButton should register a portal when open");
}

#[test]
fn test_toast_renders_content() {
    let mut runtime = fission_core::Runtime::default();
    runtime.add_app_state(Box::new(State::default())).unwrap();
    
    let mut ctx = BuildCtx::<State>::new();
    let env = Env::default();
    let state = runtime.get_app_state::<State>().unwrap();
    let view = View::new(state, &runtime.runtime_state, &env, None);
    
    let toast = Toast {
        id: WidgetNodeId::explicit("test_toast"),
        kind: ToastKind::Success,
        message: "Operation completed".into(),
        on_close: None,
    };
    
    let node = toast.build(&mut ctx, &view);
    
    // Toast is a direct widget, it returns a Container node
    if let Node::Container(_) = node {
        // ok
    } else {
        panic!("Toast build should return a Container node");
    }
}
