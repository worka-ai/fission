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
- `src/lib.rs` shared desktop/mobile entry helpers
- `src/app.rs` minimal counter app
- `assets/app-icon.png` seeded from `docs/fission_logo.png`
- `fission.toml` target manifest
- `platforms/<target>/README.md` target notes and prerequisites

## Verified flow

This branch now has a verified scaffolding smoke test for the desktop path:

```sh
cargo run -p fission-cli --bin fission -- init /tmp/demo-app --local-path "$PWD"
cargo run -p fission-cli --bin cargo-fission -- fission add-target web ios android --project-dir /tmp/demo-app
cd /tmp/demo-app
cargo check
```

That flow completes successfully today.

The repository now has two generated iOS simulator launch paths:

1. the checked-in `examples/mobile-smoke/` example
2. a CLI-generated app after `cargo fission add-target ios android web`

Direct example commands:

```sh
./examples/mobile-smoke/platforms/ios/run-sim.sh
```

Those scripts package and launch correctly, but the current runtime still renders a black frame on CoreSimulator because the simulator Metal device does not expose `DownlevelFlags(INDIRECT_EXECUTION)` for Vello.

On macOS, Android works end to end with the NDK toolchain environment configured:

```sh
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK="$ANDROID_HOME/ndk/24.0.8215888"
export ANDROID_TOOLCHAIN="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export CC_aarch64_linux_android="$ANDROID_TOOLCHAIN/aarch64-linux-android24-clang"
export AR_aarch64_linux_android="$ANDROID_TOOLCHAIN/llvm-ar"

cargo check -p fission-shell-mobile -p mobile-smoke --target aarch64-linux-android
```

Generated-app commands from the scaffolded project root:

```sh
./platforms/ios/run-sim.sh
# after exporting the Android env block from below
./platforms/android/run-emulator.sh
```

## Current target status

| Target | Scaffolded by CLI | Compile smoke in repo | Runnable today | Notes |
|---|---|---:|---:|---|
| Windows | yes | yes | yes | Uses the generated desktop entrypoint |
| macOS | yes | yes | yes | Uses the generated desktop entrypoint |
| Linux | yes | yes | yes | Uses the generated desktop entrypoint |
| iOS | yes | yes | no | simulator packaging/launch scripts are generated, but the current Vello path only produces a black frame on CoreSimulator because the simulator Metal device lacks `INDIRECT_EXECUTION` |
| Android | yes | yes | yes (emulator) | the checked-in mobile smoke example and a CLI-generated app both package, install, and launch through `platforms/android/run-emulator.sh` |
| Web | yes | no | no | `fission-shell-web` is still a placeholder; there is no runnable `web-smoke` example yet |

## Toolchains, env vars, and paths

Install the Rust targets:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-linux-android wasm32-unknown-unknown
```

iOS prerequisites:

- Xcode installed
- `xcrun --sdk iphonesimulator --show-sdk-path` must resolve an iPhoneSimulator SDK
- scaffold/launch command:

```sh
./examples/mobile-smoke/platforms/ios/run-sim.sh
```

Known blocker:

- the app currently launches but only renders a black frame inside CoreSimulator because `wgpu` / Vello still requires `DownlevelFlags(INDIRECT_EXECUTION)` there

Generated app command after `cargo fission add-target ios`:

```sh
./platforms/ios/run-sim.sh
```

Android prerequisites:

- Android SDK installed
- Android NDK installed
- the verified macOS paths on this branch were:
  - `ANDROID_HOME=$HOME/Library/Android/sdk`
  - `ANDROID_NDK=$ANDROID_HOME/ndk/24.0.8215888`
  - toolchain bin dir: `$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin`
- required env:

```sh
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK="$ANDROID_HOME/ndk/24.0.8215888"
export ANDROID_TOOLCHAIN="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export CC_aarch64_linux_android="$ANDROID_TOOLCHAIN/aarch64-linux-android24-clang"
export AR_aarch64_linux_android="$ANDROID_TOOLCHAIN/llvm-ar"
```

If your NDK uses a different host directory, replace `darwin-x86_64` with the matching prebuilt folder.

Android run command from the checked-in example:

```sh
./examples/mobile-smoke/platforms/android/run-emulator.sh
```

Android run command from a generated app:

```sh
./platforms/android/run-emulator.sh
```

Android emulator script controls:

- visible by default when it launches a fresh AVD
- `ANDROID_EMULATOR_HEADLESS=1` for background/CI runs
- `ANDROID_EMULATOR_RESTART=1` to kill an already-running hidden emulator and relaunch it visibly

WASM prerequisites:

- `rustup target add wasm32-unknown-unknown`
- `cargo install wasm-pack`
- current status: documentation only; the checked-in web shell/runtime path is still under `crates/shell/fission-shell-web`

Relevant paths:

- mobile shell: `crates/shell/fission-shell-mobile/`
- web shell: `crates/shell/fission-shell-web/`
- mobile smoke example: `examples/mobile-smoke/`
- target scaffolding docs in generated apps: `platforms/<target>/README.md`

## Immediate next work

1. replace the current Vello path on iOS simulator with a renderer/runtime path that does not require `INDIRECT_EXECUTION`
2. implement `fission-shell-web` with a WASM entrypoint and WebGPU surface
3. add a first-party `web-smoke` example once `fission-shell-web` exists
4. add first-party devtools hooks so the CLI can launch apps with widget-tree inspection enabled
