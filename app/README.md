# TotalDrop app (Flutter)

The app for Linux, Windows, macOS, Android and iOS lives here. It is a thin interface: every cryptographic and protocol decision happens in the Rust crates, reached through `flutter_rust_bridge`.

The Flutter project has not been generated yet. On a machine with Flutter installed:

```bash
cd app
flutter create --org app.totaldrop --project-name totaldrop_app --platforms linux,windows,macos,android,ios .
```

Rules for this directory:

- No cryptography in Dart. If the app needs a key, a hash or a signature, it asks the Rust core.
- No analytics, crash reporters or advertising SDKs, ever.
- Network access only through the Rust core, so privacy mode can route everything through Tor.

License: GPL-3.0-or-later.
