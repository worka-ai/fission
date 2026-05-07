#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
ADB="$ANDROID_HOME/platform-tools/adb"
EMULATOR_BIN="$ANDROID_HOME/emulator/emulator"
AVDMANAGER="$ANDROID_HOME/cmdline-tools/latest/bin/avdmanager"
AVD_NAME="${ANDROID_AVD_NAME:-FissionApi32Arm64}"
SYSTEM_IMAGE="${ANDROID_SYSTEM_IMAGE:-system-images;android-32;google_apis;arm64-v8a}"
DEVICE_PORT="${ANDROID_TEST_CONTROL_DEVICE_PORT:-48761}"
HOST_PORT="${FISSION_TEST_CONTROL_PORT:-48761}"
HEADLESS="${ANDROID_EMULATOR_HEADLESS:-0}"
RESTART_EMULATOR="${ANDROID_EMULATOR_RESTART:-0}"

if ! "$AVDMANAGER" list avd | grep -q "Name: $AVD_NAME"; then
  echo "no" | "$AVDMANAGER" create avd -n "$AVD_NAME" -k "$SYSTEM_IMAGE" --abi "google_apis/arm64-v8a" --device "pixel_5"
fi

RUNNING_EMULATOR=$("$ADB" devices | awk '/^emulator-.*device$/ { print $1; exit }')
if [[ -n "$RUNNING_EMULATOR" && "$RESTART_EMULATOR" == "1" ]]; then
  "$ADB" -s "$RUNNING_EMULATOR" emu kill >/dev/null || true
  until ! "$ADB" devices | grep -q '^emulator-'; do
    sleep 1
  done
  RUNNING_EMULATOR=""
fi

if [[ -z "$RUNNING_EMULATOR" ]]; then
  EMULATOR_ARGS=(-avd "$AVD_NAME" -gpu "${ANDROID_EMULATOR_GPU:-swiftshader_indirect}" -no-audio)
  if [[ "$HEADLESS" == "1" ]]; then
    EMULATOR_ARGS+=(-no-window)
  fi
  printf 'Launching emulator %s (%s)\n' "$AVD_NAME" "$([[ "$HEADLESS" == "1" ]] && echo headless || echo visible)"
  "$EMULATOR_BIN" "${EMULATOR_ARGS[@]}" >/tmp/fission-android-emulator.log 2>&1 &
  "$ADB" wait-for-device
  until "$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r' | grep -q '^1$'; do
    sleep 1
  done
else
  printf 'Using existing emulator %s\n' "$RUNNING_EMULATOR"
  if [[ "$HEADLESS" != "1" ]]; then
    printf 'If the window is not visible, restart with ANDROID_EMULATOR_RESTART=1 to relaunch a visible emulator.\n'
  fi
fi

APK=$("$SCRIPT_DIR/package-apk.sh")
"$ADB" install -r "$APK"
"$ADB" forward "tcp:$HOST_PORT" "tcp:$DEVICE_PORT"
"$ADB" shell am start -n ai.worka.fission.mobile.smoke/android.app.NativeActivity >/dev/null
printf 'APK=%s\n' "$APK"
