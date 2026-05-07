# fission-shell-mobile

Mobile shell for the Fission UI framework (iOS and Android).

`fission-shell-mobile` provides the current mobile bootstrap layer for running Fission
applications on iOS and Android. In this branch it is backed by the shared
`fission-shell-winit` runtime while the first dedicated mobile lifecycle and packaging
work is being built out.

## Status

Current branch status:

- host desktop preview: verified
- Android emulator smoke: verified with the Android SDK + NDK env configured
- iOS simulator packaging/launcher generation: implemented
- iOS simulator runtime: currently blocked because the Vello path only renders a black frame on CoreSimulator when `INDIRECT_EXECUTION` is unavailable
- touch, safe-area, soft-keyboard, and mobile-specific lifecycle hooks: still in progress

## Verified commands

Desktop preview of the shared UI path:

```sh
cargo run -p mobile-smoke
```

iOS simulator scaffold/launch:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
xcrun --sdk iphonesimulator --show-sdk-path
./examples/mobile-smoke/platforms/ios/run-sim.sh
```

Known blocker:

- the current renderer path logs `wgpu` / Vello validation errors on CoreSimulator and only renders a black frame when `DownlevelFlags(INDIRECT_EXECUTION)` is missing, so the simulator host project exists but does not yet render successfully

Android emulator smoke on macOS:

```sh
rustup target add aarch64-linux-android
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK="$ANDROID_HOME/ndk/24.0.8215888"
export ANDROID_TOOLCHAIN="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export CC_aarch64_linux_android="$ANDROID_TOOLCHAIN/aarch64-linux-android24-clang"
export AR_aarch64_linux_android="$ANDROID_TOOLCHAIN/llvm-ar"

./examples/mobile-smoke/platforms/android/run-emulator.sh
```

If your NDK uses a different host prebuilt directory, replace `darwin-x86_64` with the matching
folder on your machine.

## Current scope

- `MobileApp` wrapper for the shared `fission-shell-winit` runtime
- Android `android_main` entry support
- iOS simulator app-bundle packaging through `examples/mobile-smoke/platforms/ios/`
- Android emulator packaging/launcher scripts through `examples/mobile-smoke/platforms/android/`
- host-side screenshot/test-control transport via `FISSION_TEST_CONTROL_PORT`
- smoke coverage through `examples/mobile-smoke/`

## Next work

- replace the current Vello path on iOS simulator with a renderer/runtime path that does not require `INDIRECT_EXECUTION`
- iOS device packaging/signing beyond the simulator path
- touch and gesture input mapping to Fission `InputEvent` types
- safe-area insets and display-cutout awareness
- soft keyboard / IME handling

More setup detail lives in `../../../docs/platform-smoke-tests.md`.
