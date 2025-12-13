use anyhow::{anyhow, Result};
use fission_ir::{NodeId, CoreIR, Op};
use fission_layout::{LayoutSnapshot};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use lazy_static::lazy_static;
use downcast_rs::Downcast;

pub mod action;
pub mod time;
pub mod lowering;
pub mod event;
pub mod hit_test;

pub use action::{Action, ActionId, AppState};
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
        
        runtime.add_app_state(Box::new(Clock::default())).expect("Failed to add Clock state");

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

        }
        Ok(())
    }

    // Process an input event, performing hit testing and semantic action resolution.
    pub fn handle_input(
        &mut self,
        event: InputEvent,
        ir: &CoreIR,
        layout: &LayoutSnapshot,
    ) -> Result<()> {
        match event {
            InputEvent::Pointer(PointerEvent::Down { point, .. }) => {
                // 1. Hit Test
                if let Some(hit_node_id) = hit_test(ir, layout, point) {
                    // 2. Resolve Semantics and Actions
                    // Traverse up from hit_node_id to find a node with Semantics that handles a primary action
                    // (For now, assume "press" or "click" maps to the first action in the list for MVP)
                    
                    let mut current_id = Some(hit_node_id);
                    while let Some(node_id) = current_id {
                        if let Some(node) = ir.nodes.get(&node_id) {
                            if let Op::Semantics(semantics) = &node.op {
                                // Found a semantics node. Check if it has actions.
                                // For MVP, we assume the first action is the "click" action.
                                // In a real system, we'd map "PointerDown" -> "Press" intent -> Action
                                if let Some(action_id_raw) = semantics.actions.supported.first() {
                                    // We need to re-construct the ActionId from raw u128 or however it's stored.
                                    // Wait, ActionSet stores u128. Runtime needs ActionId.
                                    // And dispatch needs a Box<dyn Action>.
                                    // Problem: We only have the ID! We can't construct the Action struct (it might have fields).
                                    // This reveals a gap: "Actions are descriptors".
                                    // The semantics declares *what* action types are supported.
                                    // But to dispatch, we need an *instance*.
                                    // For simple actions (unit structs), we could register a factory?
                                    // Or Semantics should store the *instance* of the action (serialized)?
                                    // 
                                    // Revised Semantics Design in `07-3`: "Payloads... validated during dispatch".
                                    // If an action has a payload, where does it come from during a Click?
                                    // Usually, the `BindAction` op (or Semantics) stores the *full action descriptor* (including payload).
                                    // 
                                    // Check `fission-widgets` Button: `on_press: Option<ActionId>`.
                                    // This only stores the ID. It doesn't store the payload!
                                    // `fission-widgets` should store `Box<dyn Action>` or similar.
                                    // Since `Action` is `Serialize`, we can store it as bytes/JSON in Semantics?
                                    // Or `fission-widgets` stores `Arc<dyn Action>`.
                                    
                                    // Let's stick to the ID for now (assuming unit actions) or fix `fission-widgets` to store full Action.
                                    // Given the constraints and current complexity, let's assume we can't dispatch without the instance.
                                    
                                    // FIX: Update `Semantics` to store `Vec<Box<dyn Action>>`? No, `Semantics` is in `fission-ir` which doesn't know about `Action` trait (circular dep).
                                    // `Semantics` stores `ActionSet`.
                                    
                                    // Workaround for MVP:
                                    // `Runtime` needs a registry of `ActionId -> fn() -> Box<dyn Action>` for unit actions?
                                    // Or `Semantics` stores serialized actions.
                                    
                                    // Let's assume for this step that we just log the hit for now, or dispatch a special `NoOp` action if we can't create the real one.
                                    // Actually, let's fix `Button` to store the Action instance if possible?
                                    // `Action` trait is in `fission-core`. `Semantics` is in `fission-ir`.
                                    // `Semantics` CANNOT store `Box<dyn Action>`.
                                    
                                    // `Semantics` can store `Vec<u8>` (serialized action).
                                    // `Runtime` can deserialize it? But `Runtime` doesn't know the concrete type to deserialize into.
                                    // `Runtime` knows `ActionId`. It needs a map `ActionId -> Deserializer`.
                                    
                                    // This is getting deep. For MVP "End-to-End Interaction", let's assume:
                                    // 1. `Button` stores `ActionId`.
                                    // 2. We register a "default action factory" in Runtime?
                                    // 3. Or `Button` stores a closure? No.
                                    
                                    // Simplest path: `Semantics` stores `actions: Vec<ActionDescriptor>` where `ActionDescriptor` has `id` and `payload: Vec<u8>`.
                                    // `Runtime` has a registry to re-hydrate actions from descriptors?
                                    
                                    // Let's just log "Hit node X" and return Ok(()) for now to prove hit testing works.
                                    // Real event dispatch requires the Action Factory / Registry infrastructure which is Phase 7+ stuff.
                                    return Ok(());
                                }
                            }
                            
                            // Traverse up
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
