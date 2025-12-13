use anyhow::Result;
use fission_core::{Action, ActionId, AppState, CurrentTime, Runtime, Tick, AdvanceTo, TICK_ACTION_ID, ADVANCE_TO_ACTION_ID};
use fission_ir::NodeId;
use serde::{Deserialize, Serialize};
use std::any::Any;
use lazy_static::lazy_static;
use downcast_rs::Downcast; // Import Downcast trait for test file

// --- Test AppState --- //
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterState {
    pub count: i32,
}

impl AppState for CounterState {
    // as_any and as_any_mut are provided by the Downcast trait now.
}

// --- Test Action --- //
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Increment {
    pub by: i32,
}

impl Action for Increment {
    fn id(&self) -> ActionId {
        *INCREMENT_ACTION_ID
    }
}

lazy_static! {
    static ref INCREMENT_ACTION_ID: ActionId = ActionId::from_name("test_app::Increment");
}

// --- Test Reducer --- //
fn counter_reducer(state: &mut CounterState, action: &dyn Action, _target: NodeId) -> Result<()> {
    let inc_action = action.downcast_ref::<Increment>().ok_or_else(|| anyhow::anyhow!("Invalid Increment action"))?;
    state.count += inc_action.by;
    Ok(())
}

#[test]
fn test_runtime_init_and_clock_default() {
    let runtime = Runtime::default();
    assert_eq!(runtime.clock().current_time(), 0);
}

#[test]
fn test_add_get_app_state() -> Result<()> {
    let mut runtime = Runtime::default();
    runtime.add_app_state(Box::new(CounterState::default()))?;

    let counter = runtime.get_app_state::<CounterState>().expect("CounterState should be present");
    assert_eq!(counter.count, 0);

    Ok(())
}

#[test]
fn test_dispatch_custom_action_and_state_update() -> Result<()> {
    let mut runtime = Runtime::default();
    runtime.add_app_state(Box::new(CounterState::default()))?;
    runtime.register_reducer::<CounterState>(*INCREMENT_ACTION_ID, counter_reducer)?;

    let target_node_id = NodeId::derived(0, &[1]);
    runtime.dispatch(Box::new(Increment { by: 5 }), target_node_id)?;

    let counter = runtime.get_app_state::<CounterState>().expect("CounterState should be updated");
    assert_eq!(counter.count, 5);

    Ok(())
}

#[test]
fn test_dispatch_tick_action() -> Result<()> {
    let mut runtime = Runtime::default();
    assert_eq!(runtime.clock().current_time(), 0);

    runtime.dispatch(Box::new(Tick { dt: 100 }), NodeId::derived(0, &[0]))?;
    assert_eq!(runtime.clock().current_time(), 100);

    runtime.dispatch(Box::new(Tick { dt: 50 }), NodeId::derived(0, &[0]))?;
    assert_eq!(runtime.clock().current_time(), 150);

    Ok(())
}

#[test]
fn test_dispatch_advance_to_action() -> Result<()> {
    let mut runtime = Runtime::default();
    assert_eq!(runtime.clock().current_time(), 0);

    runtime.dispatch(Box::new(AdvanceTo { time: 500 }), NodeId::derived(0, &[0]))?;
    assert_eq!(runtime.clock().current_time(), 500);

    // Advance to a later time
    runtime.dispatch(Box::new(AdvanceTo { time: 700 }), NodeId::derived(0, &[0]))?;
    assert_eq!(runtime.clock().current_time(), 700);

    Ok(())
}

#[test]
fn test_clock_cannot_go_backward() -> Result<()> {
    let mut runtime = Runtime::default();
    runtime.dispatch(Box::new(AdvanceTo { time: 500 }), NodeId::derived(0, &[0]))?;
    assert_eq!(runtime.clock().current_time(), 500);

    let res = runtime.dispatch(Box::new(AdvanceTo { time: 400 }), NodeId::derived(0, &[0]));
    assert!(res.is_err());
    assert_eq!(runtime.clock().current_time(), 500); // Clock should remain unchanged

    Ok(()) // Test passes if the error is caught
}

#[test]
fn test_add_duplicate_app_state_fails() -> Result<()> {
    let mut runtime = Runtime::default();
    runtime.add_app_state(Box::new(CounterState::default()))?;
    let res = runtime.add_app_state(Box::new(CounterState::default()));
    assert!(res.is_err(), "Should not be able to add duplicate AppState types");
    Ok(())
}

#[test]
fn test_multiple_reducers_for_same_action() -> Result<()> {
    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct OtherState { value: String }
    impl AppState for OtherState {
        // as_any and as_any_mut are provided by the Downcast trait now.
    }

    fn first_reducer(state: &mut CounterState, action: &dyn Action, _target: NodeId) -> Result<()> {
        let inc_action = action.downcast_ref::<Increment>().unwrap();
        state.count += inc_action.by * 2;
        Ok(())
    }

    fn second_reducer(state: &mut OtherState, action: &dyn Action, _target: NodeId) -> Result<()> {
        let inc_action = action.downcast_ref::<Increment>().unwrap();
        state.value = format!("Processed: {}", inc_action.by);
        Ok(())
    }

    let mut runtime = Runtime::default();
    runtime.add_app_state(Box::new(CounterState::default()))?;
    runtime.add_app_state(Box::new(OtherState::default()))?;
    runtime.register_reducer::<CounterState>(*INCREMENT_ACTION_ID, first_reducer)?;
    runtime.register_reducer::<OtherState>(*INCREMENT_ACTION_ID, second_reducer)?;

    let target_node_id = NodeId::derived(0, &[1]);
    runtime.dispatch(Box::new(Increment { by: 3 }), target_node_id)?;

    let counter_state = runtime.get_app_state::<CounterState>().unwrap();
    assert_eq!(counter_state.count, 6); // 3 * 2

    let other_state = runtime.get_app_state::<OtherState>().unwrap();
    assert_eq!(other_state.value, "Processed: 3");

    Ok(())
}
