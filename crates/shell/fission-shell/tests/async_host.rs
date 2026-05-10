use fission_core::{
    BoxFuture, JobCtx, JobRef, JobSpec, ServiceCtx, ServiceRunner, ServiceSlot, ServiceSpec,
    ServiceType,
};
use fission_shell::async_host::{AsyncMessage, AsyncRegistry, ServiceControlMessage};
use serde::{Deserialize, Serialize};
use std::sync::{mpsc, Arc};
use std::time::Duration;

#[derive(Debug)]
struct EchoJob;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EchoRequest {
    value: String,
}

impl JobSpec for EchoJob {
    type Request = EchoRequest;
    type Ok = String;
    type Err = String;
    const NAME: &'static str = "echo-job";
}

const ECHO_JOB: JobRef<EchoJob> = JobRef::new("echo-job");

#[derive(Debug)]
struct SyncService;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SyncConfig {
    prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum SyncCommand {
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SyncCommandOk;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SyncCommandErr {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum SyncEvent {
    Connected,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SyncStartErr {
    message: String,
}

impl ServiceSpec for SyncService {
    type Config = SyncConfig;
    type Command = SyncCommand;
    type CommandOk = SyncCommandOk;
    type CommandErr = SyncCommandErr;
    type Event = SyncEvent;
    type StartErr = SyncStartErr;
    const NAME: &'static str = "sync-service";
}

const SYNC_TYPE: ServiceType<SyncService> = ServiceType::new("sync-service");

struct SyncRunner;

impl ServiceRunner<SyncService> for SyncRunner {
    fn on_command(
        &mut self,
        command: SyncCommand,
        ctx: ServiceCtx<SyncService>,
    ) -> BoxFuture<Result<SyncCommandOk, SyncCommandErr>> {
        Box::pin(async move {
            if matches!(command, SyncCommand::Ping) {
                let _ = ctx.emit(SyncEvent::Pong).await;
            }
            Ok(SyncCommandOk)
        })
    }

    fn on_stop(self: Box<Self>, _ctx: ServiceCtx<SyncService>) -> BoxFuture<()> {
        Box::pin(async {})
    }
}

#[test]
fn registered_jobs_emit_typed_results() {
    let mut registry = AsyncRegistry::new();
    registry.register_job(ECHO_JOB, |request: EchoRequest, _ctx: JobCtx| async move {
        Ok::<_, String>(format!("echo:{}", request.value))
    });

    let (tx, rx) = mpsc::channel();
    let spawned = registry.spawn_job(
        ECHO_JOB.name,
        7,
        serde_json::to_vec(&EchoRequest {
            value: "hello".into(),
        })
        .unwrap(),
        None,
        None,
        None,
        &tx,
        Arc::new(|| {}),
    );

    assert!(spawned);
    let message = rx.recv_timeout(Duration::from_secs(1)).expect("job result");
    match message {
        AsyncMessage::JobOk {
            req_id, payload, ..
        } => {
            assert_eq!(req_id, 7);
            let ok: String = serde_json::from_slice(&payload).unwrap();
            assert_eq!(ok, "echo:hello");
        }
        other => panic!("unexpected message: {:?}", other),
    }
}

#[test]
fn registered_services_start_accept_commands_and_stop() {
    let mut registry = AsyncRegistry::new();
    registry.register_service(
        SYNC_TYPE,
        |config: SyncConfig, ctx: ServiceCtx<SyncService>| async move {
            let _ = ctx.emit(SyncEvent::Connected).await;
            let _ = config.prefix;
            Ok::<_, SyncStartErr>(Box::new(SyncRunner) as Box<dyn ServiceRunner<SyncService>>)
        },
    );

    let (tx, rx) = mpsc::channel();
    let handle = registry
        .spawn_service(
            SYNC_TYPE.name,
            ServiceSlot::singleton(SYNC_TYPE).slot_key(),
            3,
            serde_json::to_vec(&SyncConfig {
                prefix: "demo".into(),
            })
            .unwrap(),
            None,
            &tx,
            Arc::new(|| {}),
        )
        .expect("service handle");

    let first = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("started or event");
    let second = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("started or event");
    assert!(
        matches!(first, AsyncMessage::ServiceStarted { .. })
            || matches!(second, AsyncMessage::ServiceStarted { .. })
    );
    assert!(
        matches!(first, AsyncMessage::ServiceEvent { .. })
            || matches!(second, AsyncMessage::ServiceEvent { .. })
    );

    handle
        .control_tx
        .send(ServiceControlMessage::Command {
            req_id: 9,
            payload: serde_json::to_vec(&SyncCommand::Ping).unwrap(),
            on_ok: None,
            on_err: None,
        })
        .unwrap();

    let third = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("command event");
    let fourth = rx.recv_timeout(Duration::from_secs(1)).expect("command ok");
    assert!(
        matches!(third, AsyncMessage::ServiceEvent { .. })
            || matches!(fourth, AsyncMessage::ServiceEvent { .. })
    );
    assert!(
        matches!(third, AsyncMessage::ServiceCommandOk { req_id: 9, .. })
            || matches!(fourth, AsyncMessage::ServiceCommandOk { req_id: 9, .. })
    );

    handle.control_tx.send(ServiceControlMessage::Stop).unwrap();
    let stopped = rx.recv_timeout(Duration::from_secs(1)).expect("stopped");
    assert!(matches!(
        stopped,
        AsyncMessage::ServiceStopped { instance_id: 3, .. }
    ));
}
