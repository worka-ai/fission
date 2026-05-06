# Fission CLI and target status

## Commands

Create a new app:

```sh
fission init my-app
```

Create a new app against a local Fission checkout:

```sh
fission init my-app --local-path /path/to/fission
```

Add more platform targets:

```sh
cargo fission add-target web ios android --project-dir my-app
```

The generated project contains:

- `src/main.rs` desktop entrypoint
- `src/app.rs` minimal counter app
- `fission.toml` target manifest
- `platforms/<target>/README.md` target placeholders

## Verified flow

This branch now has a verified scaffolding smoke test for the desktop path:

```sh
cargo run -p fission-cli --bin fission -- init /tmp/demo-app --local-path "$PWD"
cargo run -p fission-cli --bin cargo-fission -- fission add-target web ios android --project-dir /tmp/demo-app
cd /tmp/demo-app
cargo check
```

That flow completes successfully today.

## Current target status

| Target | Scaffolded by CLI | Runnable today | Notes |
|---|---|---:|---|
| Windows | yes | yes | Uses the generated desktop entrypoint |
| macOS | yes | yes | Uses the generated desktop entrypoint |
| Linux | yes | yes | Uses the generated desktop entrypoint |
| Web | yes | no | `fission-shell-web` is still a placeholder |
| iOS | yes | no | `fission-shell-mobile` is still a placeholder |
| Android | yes | no | `fission-shell-mobile` is still a placeholder |

## Immediate next work

1. implement `fission-shell-web` with a WASM entrypoint and WebGPU surface
2. implement `fission-shell-mobile` runtime setup for iOS and Android
3. teach `fission add-target` to generate real per-platform entrypoints once those shells exist
4. add first-party devtools hooks so the CLI can launch apps with widget-tree inspection enabled
