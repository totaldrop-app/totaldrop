# 0023: On-device QR code display and CameraX + zxing-cpp scanner on Android

- **Status:** Proposed (#29)
- **Context:** When two devices sit next to each other, typing or dictating
  a transfer code (`7-harbor-rocket`) on a phone keyboard is slower and more
  error-prone than pointing the receiver's camera at the sender's screen
  (#29). The Linux app already renders a QR code for
  `https://totaldrop.app/r#<code>` using the `qrcode` Rust crate and reads
  QR images locally. Android needs both sides: rendering the transfer QR
  code on `SendStage.waiting` and scanning a QR code from `ReceiveView`,
  without adding Google Play Services, ML Kit, analytics, or cloud calls,
  and without excluding devices that have no camera.
- **Options:**
  1. **Google Play Services Code Scanner / ML Kit Barcode Scanning.** Zero
     camera UI to maintain, but pulls in proprietary Play Services libraries,
     fails on de-Googled Android distributions (GrapheneOS, CalyxOS,
     LineageOS without GApps), and violates TotalDrop's rule against
     proprietary SDKs.
  2. **Pure-Rust frame decoding over `flutter_rust_bridge` (`rqrr`).** Keeps
     decoding in memory-safe Rust, but copies camera Y-plane buffers across
     Platform Channels / FFI on every frame and lacks `zxing-cpp`'s mature
     binarizer for angled, inverted, or glare-heavy phone screens.
  3. **Jetpack CameraX + `io.github.zxing-cpp:android` (`Apache-2.0`) in a
     dedicated unexported `QrScanActivity`, with strict link parsing in Rust
     (`TransferCode::from_qr`) and QR matrix generation via `qrcode` in
     `app/rust`.**
- **Decision:** Option 3.
  - **Send QR generation:** `app/rust` reuses the workspace's existing
    `qrcode = 0.14` (`default-features = false`, `MIT OR Apache-2.0`) crate
    to encode `TransferCode::qr_link()` (`https://totaldrop.app/r#<code>`,
    where the code stays in the URL fragment and never reaches a server).
    Flutter draws the matrix dark-on-white with a four-module quiet zone and
    a zero-elevation tap-to-enlarge dialog that auto-dismisses as soon as
    the transfer leaves `SendStage.waiting`.
  - **Receive camera scanner:** `QrScanActivity` (`android:exported="false"`)
    uses Jetpack CameraX (`androidx.camera:*:1.4.2`, `Apache-2.0`) and
    `io.github.zxing-cpp:android:3.1.1` (`Apache-2.0`). Camera hardware is
    declared optional (`android.hardware.camera` and
    `android.hardware.camera.any` both `android:required="false"`), and
    `android.permission.CAMERA` is requested only when the person taps
    **Scan QR code** on `ReceiveView`. Denying permission or lacking a
    camera leaves manual typing and clipboard paste working calmly.
  - **Security & threat-model boundaries:**
    - `zxing-cpp` runs with `formats = setOf(QR_CODE)`,
      `textMode = TextMode.PLAIN`, and `maxNumberOfSymbols = 4`. Unrelated
      QR codes in view (Wi-Fi credentials, payment codes, arbitrary URLs)
      are ignored so they do not close the camera; only candidates carrying
      a TotalDrop receive link prefix (`https://totaldrop.app/r#…`,
      `https://totaldrop.app/r/#…`, or `totaldrop://r/…`) and bounded to
      `MAX_LINK_BYTES` (`1024` bytes) are returned to Dart.
    - All code parsing and validation remain in Rust
      (`TransferCode::from_qr`, covered by `fuzz/fuzz_targets/code.rs`),
      which rejects bare codes, `http://`, query strings, and malformed
      words.
    - Per threat model `T19`, scanning fills the normalized code into the
      Receive input field without connecting automatically; the person still
      taps **Connect** and verifies the four comparison words before
      accepting any offer.
- **Consequences:**
  - The APK includes CameraX and `zxing-cpp`'s native `libzxingcpp_android.so`
    (`Apache-2.0`, compatible with `GPL-3.0-or-later`). Because `zxing-cpp`
    is C++ behind JNI, updates are reviewed with native-code discipline and
    restricted strictly to `QR_CODE` + `TextMode.PLAIN`.
  - Camera frames are analyzed in memory and closed immediately; nothing is
    written to disk or sent over the network.
  - Before tagging the next Android release (`0.3.0-beta.8`), the published
    privacy policy (`site/privacy/`, after #511 merges) and threat model
    (`docs/tehdit-modeli.md`, after #417 merges) must be updated to record
    optional on-device camera access and the `zxing-cpp` JNI decoder (#541).
