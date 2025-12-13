use fission_test::TestHarness;
use fission_core::{Action, ActionId, AppState, ActionEnvelope};
use fission_ir::NodeId;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use anyhow::Result;
use fission_widgets::Text;

// --- Custom State ---
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct MyAppState {
    counter: i32,
}

impl AppState for MyAppState { }

// --- Custom Action ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Increment;

impl Action for Increment {
    fn static_id() -> ActionId { *INCREMENT_ID }
}

lazy_static! {
    static ref INCREMENT_ID: ActionId = ActionId::from_name("test::Increment");
}

// --- Reducer ---
fn increment_reducer(state: &mut MyAppState, _action: &ActionEnvelope, _target: NodeId) -> Result<()> {
    state.counter += 1;
    Ok(())
}

#[test]
fn test_end_to_end_flow() -> Result<()> {
    let mut harness = TestHarness::new()
        .with_app_state(MyAppState::default())
        .with_root_widget(Text { value: "Initial".into(), ..Default::default() })
        .register_reducer::<MyAppState>(*INCREMENT_ID, increment_reducer);

    harness.dispatch(Increment)?;

    let state = harness.runtime.get_app_state::<MyAppState>().expect("State missing");
    assert_eq!(state.counter, 1);

    assert_eq!(harness.current_time(), 0);
    harness.tick(16)?;
    assert_eq!(harness.current_time(), 16);

    harness.pump()?;

    let display_list = harness.get_last_display_list().expect("Display list should be produced");
    assert_eq!(display_list.bounds.width(), 800.0);
    
    let snapshot = harness.last_snapshot.expect("Snapshot missing");
    assert!(!snapshot.nodes.is_empty(), "Snapshot should contain nodes from the widget tree");

    Ok(())
}
