use anyhow::{anyhow, Result};
use fission_ir::{NodeId, CoreIR, Op};
use fission_layout::{LayoutSnapshot, LayoutPoint};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use lazy_static::lazy_static;
use downcast_rs::Downcast;

pub mod action;
pub mod time;
pub mod lowering;
pub mod event;
pub mod hit_test;

pub use action::{Action, ActionId, AppState, ActionEnvelope};
pub use time::{Clock, CurrentTime};
pub use lowering::{Desugar, LoweringContext};
pub use event::{InputEvent, PointerEvent, PointerButton, KeyEvent, KeyCode, LifecycleEvent};
use hit_test::hit_test;

// Concrete Action implementations for clock control
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tick {
    pub dt: CurrentTime,
}

impl Action for Tick {
    fn static_id() -> ActionId {
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
    fn static_id() -> ActionId {
        *ADVANCE_TO_ACTION_ID
    }
}

lazy_static! {
    pub static ref ADVANCE_TO_ACTION_ID: ActionId = ActionId::from_name("fission_core::AdvanceTo");
}

pub type BoxedReducer = Box<dyn FnMut(&mut HashMap<TypeId, Box<dyn AppState>>, &ActionEnvelope, NodeId) -> Result<()> + Send + Sync>;

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
        
        runtime.add_app_state(Box::new(Clock::default())).expect("Failed to add Clock state");

        runtime.register_reducer::<Clock>(*TICK_ACTION_ID, |state: &mut Clock, action: &ActionEnvelope, _target| {
            let tick_action: Tick = serde_json::from_slice(&action.payload).map_err(|e| anyhow!("Failed to deserialize Tick: {}", e))?;
            state.advance_by(tick_action.dt)
        }).expect("Failed to register Tick reducer");

        runtime.register_reducer::<Clock>(*ADVANCE_TO_ACTION_ID, |state: &mut Clock, action: &ActionEnvelope, _target| {
            let advance_action: AdvanceTo = serde_json::from_slice(&action.payload).map_err(|e| anyhow!("Failed to deserialize AdvanceTo: {}", e))?;
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
        self.app_states.get(&TypeId::of::<S>()).and_then(|s_box| s_box.downcast_ref::<S>())
    }

    pub fn get_app_state_mut<S: AppState + 'static>(&mut self) -> Option<&mut S> {
        self.app_states.get_mut(&TypeId::of::<S>()).and_then(|s_box| s_box.downcast_mut::<S>())
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
        reducer_fn: fn(&mut S, &ActionEnvelope, NodeId) -> Result<()>,
    ) -> Result<()> {
        let state_type_id = TypeId::of::<S>();

        let boxed_reducer: BoxedReducer = Box::new(
            move |app_states: &mut HashMap<TypeId, Box<dyn AppState>>, action: &ActionEnvelope, target: NodeId| -> Result<()> {
                if let Some(state_box) = app_states.get_mut(&state_type_id) {
                    let concrete_state = state_box.downcast_mut::<S>()
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

    pub fn dispatch(&mut self, action: ActionEnvelope, target: NodeId) -> Result<()> {
        let action_id = action.id;
        if let Some(reducers) = self.reducers.get_mut(&action_id) {
            let mut temp_reducers: Vec<BoxedReducer> = reducers.drain(..).collect();

            for reducer_wrapper in temp_reducers.iter_mut() {
                reducer_wrapper(&mut self.app_states, &action, target)?;
            }
            reducers.extend(temp_reducers);
        }
        Ok(())
    }

    pub fn handle_input(
        &mut self,
        event: InputEvent,
        ir: &CoreIR,
        layout: &LayoutSnapshot,
    ) -> Result<()> {
        match event {
            InputEvent::Pointer(PointerEvent::Down { point, .. }) => {
                if let Some(hit_node_id) = hit_test(ir, layout, point) {
                    let mut current_id = Some(hit_node_id);
                    while let Some(node_id) = current_id {
                        if let Some(node) = ir.nodes.get(&node_id) {
                            if let Op::Semantics(semantics) = &node.op {
                                // Now ActionEntry contains the ActionId and Payload (serialized).
                                // We can reconstruct the ActionEnvelope directly!
                                if let Some(action_entry) = semantics.actions.entries.first() {
                                    if let Some(payload) = &action_entry.payload_data {
                                        // We have everything needed for the Envelope
                                        let envelope = ActionEnvelope {
                                            id: ActionId::from_u128(action_entry.action_id),
                                            payload: payload.clone(),
                                        };
                                        // Dispatch to target node (the semantics node, or original hit node? 
                                        // Usually target is the element that handled it)
                                        // Let's say target is the semantics node ID.
                                        println!("Dispatching action {:?} from input", envelope.id);
                                        return self.dispatch(envelope, node_id); 
                                    } else {
                                        println!("ActionEntry found but no payload data.");
                                    }
                                }
                            }
                            current_id = node.parent;
                        } else {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
