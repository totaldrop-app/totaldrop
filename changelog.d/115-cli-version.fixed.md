- **Core, command line:** `totaldrop --version` printed 0.1.0-beta.2 in
  the Linux 0.2.0-beta.1 and 0.3.0-beta.1 packages, although the command
  line had since gained the relay, privacy mode (`--relay-only`),
  translations and a clean stop with Ctrl-C. The command line and the core
  libraries are now 0.2.0-beta.1; the self-hosted servers and the Android
  app's Rust library share that version and move with it (#115, #146).
