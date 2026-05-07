# Platform smoke tests

This document records the current reproducible target setup for the Fission repository.

## Current status

| Target | Example / shell | Status | Notes |
|---|---|---|---|
| Desktop | `examples/mobile-smoke` + `fission-shell-mobile` | runnable | `cargo run -p mobile-smoke` uses the shared winit + Vello path on the host |
| iOS | `examples/mobile-smoke` + `fission-shell-mobile` | scaffolded, not runnable end to end | the checked-in example and a CLI-generated app both build and launch a simulator app bundle, but the current Vello path only produces a black frame on CoreSimulator because the simulator Metal device lacks `INDIRECT_EXECUTION` |
| Android | `examples/mobile-smoke` + `fission-shell-mobile` | runnable on emulator | requires Android SDK + NDK env vars; both the checked-in example and a CLI-generated app package, install, and launch through `run-emulator.sh` |
| Web/WASM | `crates/shell/fission-shell-web` | not runnable yet | toolchain setup is documented, but there is no checked-in web shell/runtime or `web-smoke` example yet |

## Rust targets

Install the Rust targets once:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-linux-android wasm32-unknown-unknown
```

## iOS

Required tools:

- Xcode
- iPhoneSimulator SDK visible through `xcrun`

Sanity check:

```sh
xcrun --sdk iphonesimulator --show-sdk-path
```

Scaffold/launch command:

```sh
./examples/mobile-smoke/platforms/ios/run-sim.sh
```

Optional test-control port:

```sh
FISSION_TEST_CONTROL_PORT=48711 ./examples/mobile-smoke/platforms/ios/run-sim.sh
curl http://127.0.0.1:48711/health
```

Current blocker:

- the simulator runtime currently logs `wgpu` / Vello validation errors for missing `DownlevelFlags(INDIRECT_EXECUTION)`
- the app stays up and exposes test control, but the rendered output is a black frame
- this means the iOS host-project generation is in place, but the current renderer path is not yet simulator-safe

Relevant paths:

- shell: `crates/shell/fission-shell-mobile/`
- example: `examples/mobile-smoke/`
- example iOS host files:
  - `examples/mobile-smoke/platforms/ios/Info.plist`
  - `examples/mobile-smoke/platforms/ios/package-sim.sh`
  - `examples/mobile-smoke/platforms/ios/run-sim.sh`

## Android

Required tools:

- Android SDK
- Android NDK

Verified environment on this branch:

```sh
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK="$ANDROID_HOME/ndk/24.0.8215888"
export ANDROID_TOOLCHAIN="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export CC_aarch64_linux_android="$ANDROID_TOOLCHAIN/aarch64-linux-android24-clang"
export AR_aarch64_linux_android="$ANDROID_TOOLCHAIN/llvm-ar"
```

Smoke command:

```sh
./examples/mobile-smoke/platforms/android/run-emulator.sh
```

Optional test-control port:

```sh
FISSION_TEST_CONTROL_PORT=48761 ./examples/mobile-smoke/platforms/android/run-emulator.sh
curl http://127.0.0.1:48761/health
```

Notes:

- the host prebuilt directory may be `darwin-arm64`, `darwin-x86_64`, or a Linux variant on other machines
- the script launches a visible emulator when it boots a fresh AVD
- set `ANDROID_EMULATOR_HEADLESS=1` for background/CI runs
- set `ANDROID_EMULATOR_RESTART=1` if a hidden emulator is already running and you want the script to relaunch it visibly

Relevant paths:

- shell: `crates/shell/fission-shell-mobile/`
- example: `examples/mobile-smoke/`

## Web / WASM

Required tools today:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Current state:

- `crates/shell/fission-shell-web/` exists, but is still a placeholder
- there is no checked-in `web-smoke` example yet
- `fission add-target web` only scaffolds the target and records it in `fission.toml`

This means the web path is not yet reproducibly runnable for third-party developers. The toolchain
instructions are here now so the eventual shell/example can land against a documented setup.

## Generated app smoke

A newly scaffolded app was also verified in this branch:

```sh
fission init /tmp/demo-app --local-path "$PWD"
cargo fission add-target ios android web --project-dir /tmp/demo-app
cd /tmp/demo-app
./platforms/ios/run-sim.sh
# after exporting the Android env block from the Android section above
./platforms/android/run-emulator.sh
```

The generated app now gets both Android launcher/package scripts and iOS simulator packaging scripts. Android is runnable on the emulator. iOS is still blocked at runtime by the Vello/CoreSimulator issue above.
