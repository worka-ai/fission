use anyhow::{anyhow, Result};
use fission_ir::NodeId;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use lazy_static::lazy_static;
use downcast_rs::Downcast; // Import Downcast trait

pub mod action;
pub mod time;

pub use action::{Action, ActionId, AppState};
pub use time::{Clock, CurrentTime};

// Concrete Action implementations for clock control
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tick {
    pub dt: CurrentTime,
}

impl Action for Tick {
    fn id(&self) -> ActionId {
        *TICK_ACTION_ID
    }
}

lazy_static! {
    pub static ref TICK_ACTION_ID: ActionId = ActionId::from_name("fission_core::Tick");
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdvanceTo {
    pub time: CurrentTime,
}

impl Action for AdvanceTo {
    fn id(&self) -> ActionId {
        *ADVANCE_TO_ACTION_ID
    }
}

lazy_static! {
    pub static ref ADVANCE_TO_ACTION_ID: ActionId = ActionId::from_name("fission_core::AdvanceTo");
}

// Type alias for a boxed reducer closure that can be stored dynamically
pub type BoxedReducer = Box<dyn FnMut(&mut HashMap<TypeId, Box<dyn AppState>>, &dyn Action, NodeId) -> Result<()> + Send + Sync>;

pub struct Runtime {
    reducers: HashMap<ActionId, Vec<BoxedReducer>>,
    app_states: HashMap<TypeId, Box<dyn AppState>>,
}

impl Default for Runtime {
    fn default() -> Self {
        let mut runtime = Self {
            reducers: HashMap::new(),
            app_states: HashMap::new(),
        };
        
        // Add the clock as an AppState managed by the runtime
        // Note: Clock implements AppState, so it can be managed this way.
        runtime.add_app_state(Box::new(Clock::default())).expect("Failed to add Clock state");

        // Register internal clock reducers
        runtime.register_reducer::<Clock>(*TICK_ACTION_ID, |state: &mut Clock, action, _target| {
            let tick_action = action.downcast_ref::<Tick>().ok_or_else(|| anyhow!("Invalid Tick action"))?;
            state.advance_by(tick_action.dt)
        }).expect("Failed to register Tick reducer");

        runtime.register_reducer::<Clock>(*ADVANCE_TO_ACTION_ID, |state: &mut Clock, action, _target| {
            let advance_action = action.downcast_ref::<AdvanceTo>().ok_or_else(|| anyhow!("Invalid AdvanceTo action"))?;
            state.set_to(advance_action.time)
        }).expect("Failed to register AdvanceTo reducer");
        
        runtime
    }
}

impl Runtime {
    pub fn clock(&self) -> &Clock {
        self.get_app_state::<Clock>().expect("Clock state must always be present")
    }

    pub fn get_app_state<S: AppState + 'static>(&self) -> Option<&S> {
        self.app_states.get(&TypeId::of::<S>()).and_then(|s_box| s_box.as_ref().downcast_ref::<S>())
    }

    pub fn get_app_state_mut<S: AppState + 'static>(&mut self) -> Option<&mut S> {
        self.app_states.get_mut(&TypeId::of::<S>()).and_then(|s_box| s_box.as_mut().downcast_mut::<S>())
    }

    pub fn add_app_state<S: AppState + 'static>(&mut self, state: Box<S>) -> Result<()> {
        let type_id = TypeId::of::<S>();
        if self.app_states.insert(type_id, state).is_some() {
            anyhow::bail!("App state of this type already registered.");
        }
        Ok(())
    }

    pub fn register_reducer<S: AppState + 'static>(
        &mut self,
        action_id: ActionId,
        reducer_fn: fn(&mut S, &dyn Action, NodeId) -> Result<()>,
    ) -> Result<()> {
        let state_type_id = TypeId::of::<S>();

        let boxed_reducer: BoxedReducer = Box::new(
            move |app_states: &mut HashMap<TypeId, Box<dyn AppState>>, action: &dyn Action, target: NodeId| -> Result<()> {
                if let Some(state_box) = app_states.get_mut(&state_type_id) {
                    let concrete_state = state_box.as_mut().downcast_mut::<S>()
                        .ok_or_else(|| anyhow!("Failed to downcast AppState to concrete type for reducer"))?;
                    reducer_fn(concrete_state, action, target)
                } else {
                    anyhow::bail!("Target AppState for reducer not found in runtime.");
                }
            },
        );

        self.reducers.entry(action_id).or_default().push(boxed_reducer);
        Ok(())
    }

    pub fn dispatch(&mut self, action: Box<dyn Action>, target: NodeId) -> Result<()> {
        let action_id = action.id();
        if let Some(reducers) = self.reducers.get_mut(&action_id) {
            let mut temp_reducers: Vec<BoxedReducer> = reducers.drain(..).collect();

            for reducer_wrapper in temp_reducers.iter_mut() {
                reducer_wrapper(&mut self.app_states, action.as_ref(), target)?;
            }
            reducers.extend(temp_reducers);

        } else {
            // No reducers for this action, which might be acceptable for some actions.
            // Or, we could log a warning/error here if all actions are expected to have reducers.
        }
        Ok(())
    }
}