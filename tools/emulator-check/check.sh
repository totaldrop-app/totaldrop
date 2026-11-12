#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Throwaway emulator check for #480, #489 and #498, run by
# .github/workflows/emulator-check.yml inside android-emulator-runner.
# The working directory is the repository at the PR's head; the debug APK
# and the CLI are already built. Screenshots, UI dumps and notes go to
# shots/. Nothing here fails the job: the screenshots are the result.

set -u
PR="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$PWD/shots"
mkdir -p "$OUT"
NOTES="$OUT/notes.txt"
APK=app/build/app/outputs/flutter-apk/app-debug.apk
CLI="$PWD/target/release/totaldrop"
PKG=app.totaldrop
DATA="${RUNNER_TEMP:-/tmp}/emulator-send"
FILE="$DATA/totaldrop-test.bin"
L=en
SEND_PID=
CODE=

note() { echo "$*" | tee -a "$NOTES"; }

# A string from the app's ARB file for language $L.
label() {
  python3 -c 'import json,sys; print(json.load(open(f"l10n/app/app_{sys.argv[1]}.arb"))[sys.argv[2]])' "$L" "$1"
}

shot() { adb exec-out screencap -p > "$OUT/$1.png"; }

dump() {
  rm -f "$OUT/$1.xml"
  timeout 30 adb shell uiautomator dump /sdcard/ui.xml > /dev/null 2>&1 &&
    adb pull /sdcard/ui.xml "$OUT/$1.xml" > /dev/null 2>&1
}

# Screenshot, dump, and every text on screen into the notes.
look() {
  shot "$1"
  if dump "$1"; then
    note "--- $1"
    python3 "$HERE/ui.py" texts "$OUT/$1.xml" | tee -a "$NOTES"
  else
    note "--- $1: no UI dump (timed out)"
  fi
}

tap() {
  local xy
  dump _tap || return 1
  xy=$(python3 "$HERE/ui.py" find "$OUT/_tap.xml" "$1") || return 1
  # shellcheck disable=SC2086
  adb shell input tap $xy
}

wait_for() {
  local i
  for i in $(seq "${2:-15}"); do
    dump _wait && python3 "$HERE/ui.py" find "$OUT/_wait.xml" "$1" > /dev/null && return 0
    sleep 2
  done
  return 1
}

launch() {
  adb shell am force-stop "$PKG"
  adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 > /dev/null 2>&1
  sleep 8
  # First run may ask about background limits; skip it as a person would.
  local skip
  skip=$(label setupBackgroundSkip)
  if wait_for "$skip" 2; then
    tap "$skip"
    sleep 2
  fi
}

link() {
  adb shell am start -W -a android.intent.action.VIEW -d "'$1'" "$PKG" > /dev/null
  sleep 4
}

# Restart Android with another system language and region.
set_locale() {
  adb shell am force-stop "$PKG"
  # A settings change (font_scale) stores the locale in system_locales,
  # which then wins over the property at start: set both.
  adb shell "settings put system system_locales $1; setprop persist.sys.locale $1; setprop ctl.restart zygote"
  sleep 20
  adb wait-for-device
  local i
  for i in $(seq 90); do
    adb shell dumpsys activity activities 2> /dev/null | grep -q 'ResumedActivity' && break
    sleep 2
  done
  sleep 10
  adb shell wm dismiss-keyguard > /dev/null 2>&1
  note "system locale: $(adb shell getprop persist.sys.locale | tr -d '\r'), config: $(adb shell am get-config 2> /dev/null | grep -m1 '^config:' | tr -d '\r')"
}

start_send() {
  local name="$1" i
  "$CLI" send "$FILE" > "$OUT/cli-$name.out" 2> "$OUT/cli-$name.err" &
  SEND_PID=$!
  CODE=
  for i in $(seq 90); do
    CODE=$(sed -n 's/^totaldrop receive //p' "$OUT/cli-$name.out" | tr -d '\r')
    [ -n "$CODE" ] && return 0
    sleep 1
  done
  note "$name: the CLI printed no code"
  return 1
}

stop_send() {
  [ -n "$SEND_PID" ] && kill "$SEND_PID" 2> /dev/null
  [ -n "$SEND_PID" ] && wait "$SEND_PID" 2> /dev/null
  SEND_PID=
  adb shell am force-stop "$PKG"
  adb shell "rm -rf /storage/emulated/0/Download/TotalDrop/* /storage/emulated/0/Android/data/$PKG/files/*" 2> /dev/null
}

# On the Receive screen with the code in the box: connect, take the offer,
# and take screenshots while the file comes in.
receive() {
  local name="$1" accept i found=
  accept=$(label receiveAccept)
  tap "$(label receiveConnect)" || adb shell input keyevent 66
  # Languages whose translation is not reviewed show Accept in English
  # (safeL10n, l10n/status.json).
  for i in $(seq 40); do
    dump _wait || { sleep 2; continue; }
    if python3 "$HERE/ui.py" find "$OUT/_wait.xml" "$accept" > /dev/null; then
      found=1
      break
    fi
    if python3 "$HERE/ui.py" find "$OUT/_wait.xml" Accept > /dev/null; then
      accept=Accept
      found=1
      break
    fi
    sleep 2
  done
  if [ -n "$found" ]; then
    look "$name-offer"
    tap "$accept"
    local i
    for i in 1 2 3 4 5 6; do
      shot "$name-receiving-$i"
      sleep 1.5
    done
    look "$name-receiving-dump"
  else
    look "$name-no-offer"
  fi
}

# One transfer from the CLI on the runner into the app, code typed.
transfer() {
  local name="$1"
  launch
  tap "$(label homeReceiveTitle)"
  sleep 3
  if start_send "$name"; then
    adb shell input text "$CODE"
    sleep 1
    receive "$name"
  fi
  stop_send
}

# --- setup -------------------------------------------------------------

note "PR #$PR at $(git rev-parse HEAD)"
adb root > /dev/null 2>&1
sleep 3
adb wait-for-device
adb install -r "$APK" > /dev/null || {
  note "install failed"
  exit 1
}
adb shell pm grant "$PKG" android.permission.POST_NOTIFICATIONS 2> /dev/null
# Battery optimisation off, so the first-run page has nothing to set up.
adb shell dumpsys deviceidle whitelist +"$PKG" > /dev/null
mkdir -p "$DATA"
head -c 512M /dev/urandom > "$FILE"
adb logcat -c
note "android $(adb shell getprop ro.build.version.release | tr -d '\r'), $(adb shell wm size | tr -d '\r')"

case "$PR" in
  498)
    launch
    look 498-01-home
    link "totaldrop://r/7-harbor-rocket"
    look 498-02-scheme-link
    link "https://totaldrop.app/r#4-maple-orbit"
    look 498-03-https-link
    link "totaldrop://r/"
    look 498-04-empty-link-keeps-box
    adb shell input keyevent 4
    sleep 2
    look 498-04b-after-back
    link "totaldrop://r/7-harbor"
    look 498-05-short-link-says-not-a-code
    # A link while a file comes in: the screen stays, linkBusy shows.
    adb shell am force-stop "$PKG"
    if start_send 498-busy; then
      link "totaldrop://r/$CODE"
      look 498-06-transfer-link
      tap "$(label receiveConnect)" || adb shell input keyevent 66
      if wait_for "$(label receiveAccept)" 40; then
        tap "$(label receiveAccept)"
        sleep 3
        shot 498-07-receiving
        link "totaldrop://r/7-harbor-rocket"
        look 498-08-link-while-receiving
      else
        look 498-07-no-offer
      fi
    fi
    stop_send
    note "--- pm get-app-links (debug key, so not verified here)"
    adb shell pm get-app-links "$PKG" | tee -a "$NOTES"
    ;;
  480)
    adb shell settings put system font_scale 2.0
    note "font_scale $(adb shell settings get system font_scale | tr -d '\r')"
    for mode in light dark; do
      if [ "$mode" = dark ]; then adb shell cmd uimode night yes; else adb shell cmd uimode night no; fi
      launch
      look "480-en-$mode-01-home"
      tap "$(label settings)" && sleep 3 && look "480-en-$mode-02-settings"
      adb shell input keyevent 4
      sleep 2
      tap "$(label homeSendTitle)" && sleep 3 && look "480-en-$mode-03-send"
      adb shell input keyevent 4
      sleep 2
      launch
      tap "$(label homeReceiveTitle)" && sleep 3
      adb shell input text 7-harbor
      adb shell input keyevent 66
      sleep 2
      look "480-en-$mode-04-receive-not-a-code"
      transfer "480-en-$mode-05"
    done
    adb shell cmd uimode night no
    set_locale ja-JP
    L=ja
    transfer 480-ja-light-06
    set_locale ar-EG
    L=ar
    transfer 480-ar-light-07
    ;;
  480-locales)
    adb shell settings put system font_scale 2.0
    note "font_scale $(adb shell settings get system font_scale | tr -d '\r')"
    for loc in ja-JP ar-EG; do
      set_locale "$loc"
      L=${loc%%-*}
      launch
      look "480-$loc-01-home"
      tap "$(label homeReceiveTitle)" && sleep 3
      adb shell input text 7-harbor
      adb shell input keyevent 66
      sleep 2
      look "480-$loc-02-receive-not-a-code"
      transfer "480-$loc-03"
    done
    ;;
  489-locales)
    for loc in ar-EG ja-JP de-DE; do
      set_locale "$loc"
      L=${loc%%-*}
      transfer "489-$loc"
    done
    ;;
  489)
    transfer 489-en-US
    for loc in tr-TR ar-EG ja-JP; do
      set_locale "$loc"
      L=${loc%%-*}
      transfer "489-$loc"
    done
    # Android 13+ per-app language: app language and device region differ.
    set_locale tr-TR
    adb shell cmd locale set-app-locales "$PKG" --locales en-US
    note "app locales: $(adb shell cmd locale get-app-locales "$PKG" | tr -d '\r')"
    L=en
    transfer 489-system-tr-TR-app-en
    set_locale en-US
    adb shell cmd locale set-app-locales "$PKG" --locales tr-TR
    note "app locales: $(adb shell cmd locale get-app-locales "$PKG" | tr -d '\r')"
    L=tr
    transfer 489-system-en-US-app-tr
    ;;
esac

adb logcat -d > "$OUT/logcat.txt" 2> /dev/null
adb logcat -d -s flutter:V > "$OUT/logcat-flutter.txt" 2> /dev/null
note "--- route errors: $(grep -c 'generator for route\|initial route' "$OUT/logcat-flutter.txt")"
note "--- Flutter overflow lines in logcat: $(grep -c 'overflowed by' "$OUT/logcat.txt")"
grep -m 20 'overflowed by\|EXCEPTION CAUGHT' "$OUT/logcat.txt" | tee -a "$NOTES"
rm -f "$OUT"/_tap.xml "$OUT"/_wait.xml
exit 0
