# mobile-smoke

Shared mobile smoke example for the current `fission-shell-mobile` path.

## What it proves

- the shared runtime launches on the host through `MobileApp`
- the same example packages, installs, and launches on the iOS Simulator
- the same example cross-compiles for Android when the NDK toolchain env is set

## Commands

Desktop preview:

```sh
cargo run -p mobile-smoke
```

iOS simulator smoke:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
xcrun --sdk iphonesimulator --show-sdk-path
./examples/mobile-smoke/platforms/ios/run-sim.sh
```

Optional test-control port:

```sh
FISSION_TEST_CONTROL_PORT=48711 ./examples/mobile-smoke/platforms/ios/run-sim.sh
curl http://127.0.0.1:48711/health
```

Android compile smoke on macOS:

```sh
rustup target add aarch64-linux-android
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK="$ANDROID_HOME/ndk/24.0.8215888"
export ANDROID_TOOLCHAIN="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export CC_aarch64_linux_android="$ANDROID_TOOLCHAIN/aarch64-linux-android24-clang"
export AR_aarch64_linux_android="$ANDROID_TOOLCHAIN/llvm-ar"

cargo check -p fission-shell-mobile -p mobile-smoke --target aarch64-linux-android
```

If your NDK uses a different host prebuilt directory, replace `darwin-x86_64` with the matching
folder on your machine.
