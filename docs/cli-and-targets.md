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

Diagnose local SDKs, emulators, browsers, and Rust targets:

```sh
cargo fission doctor web ios android --project-dir my-app
```

The generated project contains:

- `src/main.rs` desktop entrypoint
- `src/lib.rs` shared desktop/mobile/web entry helpers
- `src/app.rs` minimal counter app
- `assets/app-icon.png` seeded from `docs/fission_logo.png`
- `fission.toml` target manifest
- `platforms/<target>/README.md` target notes and prerequisites
- target smoke scripts such as `platforms/web/test-browser.sh`, `platforms/ios/test-sim.sh`, and `platforms/android/test-emulator.sh`

## Verified flow

This branch has a verified scaffolding smoke path for desktop, web, iOS simulator, and Android emulator scaffolding:

```sh
cargo run -p fission-cli --bin fission -- init /tmp/demo-app --local-path "$PWD"
cargo run -p fission-cli --bin cargo-fission -- fission add-target web ios android --project-dir /tmp/demo-app
cargo run -p fission-cli --bin fission -- doctor web ios android --project-dir /tmp/demo-app
cd /tmp/demo-app
cargo check
```

Generated-app commands from the scaffolded project root:

```sh
./platforms/ios/run-sim.sh
./platforms/ios/test-sim.sh
./platforms/android/run-emulator.sh
./platforms/android/test-emulator.sh
./platforms/web/run-browser.sh
./platforms/web/test-browser.sh
```

The repository also keeps checked-in smoke examples:

```sh
./examples/mobile-smoke/platforms/ios/run-sim.sh
./examples/mobile-smoke/platforms/ios/test-sim.sh
./examples/mobile-smoke/platforms/android/run-emulator.sh
./examples/mobile-smoke/platforms/android/test-emulator.sh
./examples/web-smoke/platforms/web/run-browser.sh
./examples/web-smoke/platforms/web/test-browser.sh
```

## Current target status

| Target | Scaffolded by CLI | Compile smoke in repo | Runnable today | Notes |
|---|---|---:|---:|---|
| Windows | yes | yes | yes | Uses the generated desktop entrypoint |
| macOS | yes | yes | yes | Uses the generated desktop entrypoint |
| Linux | yes | yes | yes | Uses the generated desktop entrypoint |
| iOS | yes | yes | yes (simulator) | simulator app bundles are generated and can be health-checked through test control |
| Android | yes | yes | yes (emulator) | package scripts auto-detect SDK, NDK, toolchain, platform, and build-tools where possible |
| Web | yes | yes | yes (browser) | `wasm-pack` builds the app and `test-browser.sh` launches Chrome/Chromium with CDP enabled |

## Toolchains, env vars, and paths

Install the Rust targets:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-linux-android wasm32-unknown-unknown
```

Run doctor before platform work:

```sh
cargo fission doctor web ios android --project-dir .
```

### iOS

Required tools:

- macOS with Xcode installed
- iPhoneSimulator SDK visible through `xcrun --sdk iphonesimulator --show-sdk-path`
- Rust targets `aarch64-apple-ios` and `aarch64-apple-ios-sim`

Commands:

```sh
./examples/mobile-smoke/platforms/ios/run-sim.sh
./examples/mobile-smoke/platforms/ios/test-sim.sh
```

Generated app command after `cargo fission add-target ios`:

```sh
./platforms/ios/run-sim.sh
./platforms/ios/test-sim.sh
```

The generated iOS script opens the Simulator app by default. Set `IOS_SIM_HEADLESS=1` for CI or background-only runs.

### Android

Required tools:

- Android SDK
- Android NDK
- Rust target `aarch64-linux-android`

The generated package script detects `ANDROID_HOME`, the latest installed NDK, the matching NDK LLVM prebuilt host directory, the latest installed Android platform, and build-tools. Override these only when the auto-detected value is wrong:

- `ANDROID_HOME` or `ANDROID_SDK_ROOT`
- `ANDROID_NDK`
- `ANDROID_TOOLCHAIN`
- `ANDROID_MIN_API_LEVEL` (default: `24`)
- `ANDROID_TARGET_API_LEVEL` (default: latest installed platform)
- `ANDROID_BUILD_TOOLS`

Android emulator controls:

- `ANDROID_EMULATOR_HEADLESS=1` for background/CI runs
- `ANDROID_EMULATOR_RESTART=1` to kill an already-running hidden emulator and relaunch it visibly
- `ANDROID_EMULATOR_API_LEVEL`, `ANDROID_AVD_NAME`, or `ANDROID_SYSTEM_IMAGE` to pick a specific emulator runtime

Commands:

```sh
./examples/mobile-smoke/platforms/android/run-emulator.sh
./examples/mobile-smoke/platforms/android/test-emulator.sh
```

Generated app command after `cargo fission add-target android`:

```sh
./platforms/android/run-emulator.sh
./platforms/android/test-emulator.sh
```

### Web / WASM

Required tools:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
node --version # Node 22+ is required by the CDP smoke test
cargo fission doctor web --project-dir .
```

The browser test script uses Node.js plus Chrome/Chromium's DevTools Protocol endpoint. It starts a transient server, fails on browser runtime or console errors, and waits for a non-empty canvas. Set `FISSION_CHROME=/path/to/chrome` if the browser cannot be auto-detected.

Commands:

```sh
./examples/web-smoke/platforms/web/run-browser.sh
./examples/web-smoke/platforms/web/test-browser.sh
```

Generated app command after `cargo fission add-target web`:

```sh
./platforms/web/run-browser.sh
./platforms/web/test-browser.sh
```

Relevant paths:

- CLI: `crates/tools/fission-cli/`
- mobile shell: `crates/shell/fission-shell-mobile/`
- web shell: `crates/shell/fission-shell-web/`
- mobile smoke example: `examples/mobile-smoke/`
- web smoke example: `examples/web-smoke/`
- target scaffolding docs in generated apps: `platforms/<target>/README.md`

## Immediate next work

1. add browser-side semantic test control so web can use the same interaction tooling as desktop/mobile
2. validate iOS on physical devices after simulator coverage
3. extend generated host projects from smoke packaging toward release packaging
4. add first-party devtools hooks so the CLI can launch apps with widget-tree inspection enabled
