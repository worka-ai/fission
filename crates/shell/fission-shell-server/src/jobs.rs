use anyhow::{anyhow, Result as AnyResult};
use fission_core::{JobRef, JobSpec};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerJobCtx {
    pub req_id: u64,
    pub resource_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerJobError {
    pub payload: Option<Vec<u8>>,
    pub message: Option<String>,
}

impl ServerJobError {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            payload: None,
            message: Some(message.into()),
        }
    }

    pub fn typed<E: Serialize>(error: &E) -> Self {
        Self {
            payload: serde_json::to_vec(error).ok(),
            message: None,
        }
    }
}

type JobHandler =
    dyn Fn(Vec<u8>, ServerJobCtx) -> std::result::Result<Vec<u8>, ServerJobError> + Send + Sync;

#[derive(Clone, Default)]
pub struct ServerJobRegistry {
    handlers: BTreeMap<String, Arc<JobHandler>>,
}

impl ServerJobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_job<J, F>(mut self, job: JobRef<J>, handler: F) -> Self
    where
        J: JobSpec,
        F: Fn(J::Request, ServerJobCtx) -> std::result::Result<J::Ok, J::Err>
            + Send
            + Sync
            + 'static,
        J::Err: Serialize,
    {
        self.handlers.insert(
            job.name.to_string(),
            Arc::new(move |payload, ctx| {
                let request = serde_json::from_slice::<J::Request>(&payload)
                    .map_err(|error| ServerJobError::message(error.to_string()))?;
                match handler(request, ctx) {
                    Ok(value) => serde_json::to_vec(&value)
                        .map_err(|error| ServerJobError::message(error.to_string())),
                    Err(error) => Err(ServerJobError::typed(&error)),
                }
            }),
        );
        self
    }

    pub fn has_job(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    pub fn run(
        &self,
        name: &str,
        payload: Vec<u8>,
        ctx: ServerJobCtx,
    ) -> std::result::Result<Vec<u8>, ServerJobError> {
        let Some(handler) = self.handlers.get(name) else {
            return Err(ServerJobError::message(format!(
                "server job `{name}` is not registered"
            )));
        };
        handler(payload, ctx)
    }

    pub fn require_job(&self, name: &str) -> AnyResult<()> {
        if self.has_job(name) {
            Ok(())
        } else {
            Err(anyhow!("server job `{name}` is not registered"))
        }
    }
}
