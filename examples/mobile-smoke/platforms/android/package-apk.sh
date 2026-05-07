#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROJECT_DIR=$(cd -- "$SCRIPT_DIR/../.." && pwd)
REPO_ROOT=$(cd -- "$PROJECT_DIR/../.." && pwd)
ICON_SOURCE="${FISSION_APP_ICON:-$REPO_ROOT/docs/fission_logo.png}"
TARGET="${ANDROID_TARGET_TRIPLE:-aarch64-linux-android}"
PACKAGE_NAME="mobile-smoke"
LIB_NAME="mobile_smoke"
PROFILE="${ANDROID_PROFILE:-debug}"
APP_ID="ai.worka.fission.mobile.smoke"
ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
ANDROID_NDK="${ANDROID_NDK:-$ANDROID_HOME/ndk/24.0.8215888}"
ANDROID_TOOLCHAIN="${ANDROID_TOOLCHAIN:-$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin}"
CC_aarch64_linux_android="${CC_aarch64_linux_android:-$ANDROID_TOOLCHAIN/aarch64-linux-android24-clang}"
AR_aarch64_linux_android="${AR_aarch64_linux_android:-$ANDROID_TOOLCHAIN/llvm-ar}"
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER:-$CC_aarch64_linux_android}"
CARGO_TARGET_AARCH64_LINUX_ANDROID_AR="${CARGO_TARGET_AARCH64_LINUX_ANDROID_AR:-$AR_aarch64_linux_android}"
export ANDROID_HOME ANDROID_NDK ANDROID_TOOLCHAIN CC_aarch64_linux_android AR_aarch64_linux_android
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER CARGO_TARGET_AARCH64_LINUX_ANDROID_AR

BUILD_TOOLS=$(find "$ANDROID_HOME/build-tools" -maxdepth 1 -mindepth 1 -type d | sort -V | tail -1)
ANDROID_JAR=$(find "$ANDROID_HOME/platforms" -maxdepth 2 -path '*/android.jar' | sort -V | tail -1)
AAPT="$BUILD_TOOLS/aapt"
ZIPALIGN="$BUILD_TOOLS/zipalign"
APKSIGNER="$BUILD_TOOLS/apksigner"

BUILD_ARGS=(build --manifest-path "$PROJECT_DIR/Cargo.toml" --lib --target "$TARGET" --package "$PACKAGE_NAME")
ARTIFACT_DIR=debug
if [[ "$PROFILE" == "release" ]]; then
  BUILD_ARGS+=(--release)
  ARTIFACT_DIR=release
fi

cargo "${BUILD_ARGS[@]}"
TARGET_DIR=$(python3 - <<'PY' "$PROJECT_DIR/Cargo.toml"
import json
import subprocess
import sys

manifest = sys.argv[1]
metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--manifest-path", manifest, "--format-version", "1", "--no-deps"]
    )
)
print(metadata["target_directory"])
PY
)

SO_PATH="$TARGET_DIR/$TARGET/$ARTIFACT_DIR/lib$LIB_NAME.so"
BUILD_DIR="$SCRIPT_DIR/build/$PROFILE"
APK_ROOT="$BUILD_DIR/apk-root"
UNALIGNED_APK="$BUILD_DIR/$PACKAGE_NAME-unaligned.apk"
ALIGNED_APK="$BUILD_DIR/$PACKAGE_NAME-aligned.apk"
SIGNED_APK="$BUILD_DIR/$PACKAGE_NAME.apk"
KEYSTORE="${ANDROID_DEBUG_KEYSTORE:-$HOME/.android/debug.keystore}"

rm -rf "$APK_ROOT"
mkdir -p "$APK_ROOT/lib/arm64-v8a" "$APK_ROOT/res/drawable-nodpi" "$BUILD_DIR"
cp "$SO_PATH" "$APK_ROOT/lib/arm64-v8a/lib$LIB_NAME.so"
cp "$ICON_SOURCE" "$APK_ROOT/res/drawable-nodpi/app_icon.png"

"$AAPT" package -f -F "$UNALIGNED_APK" -M "$SCRIPT_DIR/AndroidManifest.xml" -S "$APK_ROOT/res" -I "$ANDROID_JAR"
(cd "$APK_ROOT" && zip -qr "$UNALIGNED_APK" lib)
"$ZIPALIGN" -f 4 "$UNALIGNED_APK" "$ALIGNED_APK"

if [[ ! -f "$KEYSTORE" ]]; then
  mkdir -p "$(dirname "$KEYSTORE")"
  keytool -genkeypair -v \
    -keystore "$KEYSTORE" \
    -storepass android \
    -alias androiddebugkey \
    -keypass android \
    -dname "CN=Android Debug,O=Android,C=US" \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 >/dev/null 2>&1
fi

"$APKSIGNER" sign \
  --ks "$KEYSTORE" \
  --ks-pass pass:android \
  --key-pass pass:android \
  --out "$SIGNED_APK" \
  "$ALIGNED_APK"

printf '%s\n' "$SIGNED_APK"
