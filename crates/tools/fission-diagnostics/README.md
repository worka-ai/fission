# fission-diagnostics

Structured diagnostics and telemetry for the Fission rendering pipeline.

This crate provides a global, thread-safe diagnostics system that emits structured JSON events covering every stage of the Fission frame lifecycle: diffing, layout, painting, rasterization, animation, input handling, and invariant violations. It is designed for pipeline debugging, performance profiling, and automated testing.

## Architecture

The diagnostics system uses a global singleton (`OnceCell<RwLock<DiagnosticsInner>>`) initialized once at startup. All emission goes through the `emit()` function, which checks category filters and level thresholds before writing to the configured sink.

```
Application
  |
  +-- init_from_env() or init(config)
  |      |
  |      v
  |   DiagnosticsConfig
  |     +-- enabled_categories: BTreeSet<DiagCategory>
  |     +-- min_level: DiagLevel
  |     +-- sink: DiagSink
  |     +-- sampling: f32
  |
  +-- emit(category, level, event_kind)
  |      |
  |      v
  |   DiagnosticsInner::should_emit()  -- checks category + level
  |      |
  |      v
  |   SinkImpl::write()  -- serializes DiagEvent to JSONL
  |
  +-- begin_frame() / end_frame()  -- frame lifecycle markers
```

## Categories

Each diagnostic event belongs to exactly one `DiagCategory`:

| Category | Events |
|----------|--------|
| `Frame` | `FrameStart`, `FrameEnd` with per-frame statistics |
| `Diff` | `DiffSummary` -- node counts for created, removed, changed nodes |
| `Layout` | `LayoutSummary` -- node count, dirty count, full rebuild flag, duration |
| `Paint` | `PaintSummary`, `PaintNode`, `PaintNodeRect` -- segment reuse and per-node details |
| `Raster` | `RasterSummary` -- tile cache hit/miss rates |
| `Input` | `InputEvent` -- pointer, keyboard, scroll, and text input events |
| `Animation` | `AnimationSummary` -- active, started, replaced, ended counts |
| `Media` | `MediaSummary`, `MediaEvent` -- video/audio node tracking |
| `Semantics` | Reserved for accessibility tree events |
| `Invariants` | `InvariantViolation` -- layout/paint consistency checks |
| `Test` | Reserved for test harness events |

## Severity levels

`DiagLevel` controls filtering granularity: `Error` > `Warn` > `Info` > `Debug` > `Trace`. The `allows()` method returns true when a given event level is at or above the configured minimum.

## Configuration via environment variables

The `init_from_env()` function reads four environment variables:

| Variable | Values | Default |
|----------|--------|---------|
| `FISSION_DIAG` | Comma-separated category names, or `*` for all | (empty -- nothing enabled) |
| `FISSION_DIAG_LEVEL` | `error`, `warn`, `info`, `debug`, `trace` | `warn` |
| `FISSION_DIAG_SINK` | `stdout`, `file:/path/to/log.jsonl`, `disabled` | `stdout` |
| `FISSION_DIAG_SAMPLING` | Float 0.0-1.0 | `1.0` |

Example:

```sh
FISSION_DIAG=layout,paint,frame FISSION_DIAG_LEVEL=debug FISSION_DIAG_SINK=file:/tmp/fission.jsonl cargo run
```

## Sinks

| Sink | Description |
|------|-------------|
| `DiagSink::Stdout` | Writes each event as a single JSON line to stdout. |
| `DiagSink::File(path)` | Appends JSONL to the specified file. |
| `DiagSink::RingBuffer(cap)` | Keeps the last `cap` events in memory (useful for in-app inspector). |
| `DiagSink::Disabled` | Suppresses all output. |

## Key types

- **`DiagEvent`** -- The top-level event envelope. Contains `schema_version` (always 1), `timestamp_ns`, `frame_no`, `category`, `level`, and the `event` payload.
- **`DiagEventKind`** -- Tagged enum with all concrete event payloads. Serialized with `#[serde(tag = "kind", content = "payload")]`.
- **`FrameStats`** -- Summary statistics attached to `FrameEnd`: dirty nodes, layout updates, paint hits/misses, video surfaces.
- **`SnapshotProvider`** / **`SnapshotBlob`** -- Trait for components that can produce a JSON snapshot of their internal state (currently `SnapshotKind::Layout` only).

## Usage

```rust
use fission_diagnostics::prelude::*;

// Initialize from environment variables
init_from_env();

// Or initialize programmatically
use fission_diagnostics::{DiagnosticsConfig, DiagSink, DiagLevel, DiagCategory};
let config = DiagnosticsConfig {
    enabled_categories: [DiagCategory::Layout, DiagCategory::Paint].into_iter().collect(),
    min_level: DiagLevel::Debug,
    sink: DiagSink::Stdout,
    sampling: 1.0,
};
fission_diagnostics::init(config);

// Emit events
begin_frame(None);
emit(DiagCategory::Layout, DiagLevel::Debug, DiagEventKind::LayoutSummary {
    nodes: 1200,
    dirty_count: 3,
    full_rebuild: false,
    duration_ns: 450_000,
});
end_frame(FrameStats::default());
```

## Output format

Each event is a single JSON line (JSONL). Example:

```json
{"schema_version":1,"timestamp_ns":16666667,"frame_no":1,"category":"layout","level":"debug","kind":"LayoutSummary","payload":{"nodes":1200,"dirty_count":3,"full_rebuild":false,"duration_ns":450000}}
```
