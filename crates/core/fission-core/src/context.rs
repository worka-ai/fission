use crate::action::{Action, ActionEnvelope, ActionId, AppState};
use crate::effect::{ActionInput, Effect, EffectEnvelope, SystemEffect, EffectPayload};
use crate::NodeId;
use crate::ActionRegistry;
use std::collections::HashMap;
use std::marker::PhantomData;
use serde::Serialize;

pub struct ReducerContext<'a, 'b, 'c, S: AppState> {
    pub effects: &'a mut Effects<'b, S>,
    pub input: &'c ActionInput,
}

pub struct Effects<'a, S: AppState> {
    pub out: Vec<EffectEnvelope>,
    next_req_id: u64,
    pub(crate) registry: Option<&'a mut ActionRegistry<S>>, 
    _phantom: PhantomData<S>,
}

impl<'a, S: AppState> Effects<'a, S> {
    pub fn new(next_req_id: u64, registry: &'a mut ActionRegistry<S>) -> Self {
        Self {
            out: Vec::new(),
            next_req_id,
            registry: Some(registry),
            _phantom: PhantomData,
        }
    }

    pub fn new_headless(next_req_id: u64) -> Self {
        Self {
            out: Vec::new(),
            next_req_id,
            registry: None,
            _phantom: PhantomData,
        }
    }

    pub fn add(&mut self, effect: SystemEffect) -> u64 {
        let req_id = self.next_req_id;
        self.next_req_id += 1;
        
        self.out.push(EffectEnvelope {
            req_id,
            effect: Effect::System(effect),
            on_ok: None,
            on_err: None,
        });
        req_id
    }

    pub fn system_effect(&mut self, effect: SystemEffect) -> EffectBuilder<'_, 'a, S> {
        let req_id = self.next_req_id;
        self.next_req_id += 1;
        
        let index = self.out.len();
        self.out.push(EffectEnvelope {
            req_id,
            effect: Effect::System(effect),
            on_ok: None,
            on_err: None,
        });
        
        EffectBuilder {
            effects: self,
            index,
        }
    }

    pub fn http_get(&mut self, url: impl Into<String>) -> EffectBuilder<'_, 'a, S> {
        self.system_effect(SystemEffect::HttpGet { 
            url: url.into(),
            headers: HashMap::new() 
        })
    }

    pub fn file_read(&mut self, path: impl Into<String>) -> EffectBuilder<'_, 'a, S> {
        self.system_effect(SystemEffect::FileRead { 
            path: path.into()
        })
    }

    pub fn cancel(&mut self, req_id: u64) {
        self.system_effect(SystemEffect::Cancel { req_id });
    }

    pub fn release_resource(&mut self, resource_id: u64) {
        self.system_effect(SystemEffect::ReleaseResource { resource_id });
    }
}

pub struct EffectBuilder<'a, 'b, S: AppState> {
    effects: &'a mut Effects<'b, S>,
    index: usize,
}

impl<'a, 'b, S: AppState> EffectBuilder<'a, 'b, S> {
    pub fn on_ok<A: Action, H>(self, action: ActionEnvelope) -> Self {
        self.effects.out[self.index].on_ok = Some(action);
        self
    }

    pub fn on_err(self, action: ActionEnvelope) -> Self {
        self.effects.out[self.index].on_err = Some(action);
        self
    }

    pub fn dispatch(self) {
        // Drop
    }
}
