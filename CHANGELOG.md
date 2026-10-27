# Changelog

All notable changes to TotalDrop are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/) as described in
[docs/versioning.md](docs/versioning.md).

## [Unreleased]

### Fixed

- **Core:** a transfer no longer fails when a firewall blocks incoming
  connections on the sending device (as Omarchy's ufw does by default). Both
  sides now listen and dial; the first connection that comes up is used.

## [android-0.1.0-beta.3] - 2026-10-27

### Changed

- **Android:** while sending, the mark plays the website's animation: the
  file travels the channel and a ring spreads from the drop.

## [linux-0.1.0-beta.2] - 2026-10-27

### Added

- **Linux:** first desktop app, Qt 6 and QML, native Wayland. Send with the
  desktop's file chooser or by drag and drop, a code and a QR code; receive
  by typing the code with word completion, pasting it, reading a QR code from
  an image or opening a TotalDrop link. Desktop notifications when a
  transfer ends in the background.
- **Linux:** follows the desktop's look: the Omarchy theme (live), the
  portal's colour scheme and accent, the Plasma palette with Breeze controls
  and icons, and Hyprland's corner rounding.
- **Linux:** separate packages for Arch, Debian/Ubuntu (.deb) and
  Fedora (.rpm).
- **Site:** `totaldrop.app/r#code`, the page TotalDrop QR codes and links
  open: it shows the code and opens the app with it. The code stays in the
  browser. The download page offers the Linux packages.

## [0.1.0-beta.2] - 2026-10-27

### Changed

- New mark, "Dalga": a drop at the centre, waves spreading from it, and the
  file leaving through a channel in the waves. It replaces the orbit mark in
  the app, the launcher icon and the site. While sending, the file travels
  the channel; with reduced motion the mark stays still.

## [0.1.0-beta.1] - 2026-10-27

First beta. Protocol v0 with the clarifications in
[docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md).

### Added

- Transfer by code between two devices that can reach each other directly
  (same machine or same network), end to end encrypted:
  CPace over ristretto255, Noise XXpsk3 with an ML-KEM-768 hybrid step,
  QUIC pinned to single-use raw public keys, STREAM-encrypted chunks
  verified with BLAKE3.
- Command-line client `totaldrop send` / `totaldrop receive`.
- Rendezvous server in Rust (self-hosting) and on Cloudflare Workers,
  serving `rendezvous.totaldrop.app`.
- Android app (Flutter, Material 3, follows the system colours), in English
  and Turkish.

### Security

- Issues found by the first internal security review were fixed before
  this release; see the commit "Fix issues from the security review".
