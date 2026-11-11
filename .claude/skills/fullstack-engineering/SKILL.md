---
name: fullstack-engineering
description: Use when changing TotalDrop's servers and Workers (server/), CI (.github/), the site's functions (site/functions/), the Linux desktop app (linux/, packaging/) or the CLI (crates/totaldrop-cli). APIs, limits, Cloudflare, deployment, Qt/QML and packaging.
---

# Full-stack engineering (TotalDrop)

Own a change from the wire to the pixel. Apply
`software-engineering-discipline` for process and `security-engineering-analyst`
for anything a stranger can reach.

## Servers and Workers (`server/`)

- Each service has two implementations: the Cloudflare Worker
  (`server/*-worker`, TypeScript) and the self-hosted Rust server
  (`server/totaldrop-*`). A behaviour change lands in both; the end-to-end
  tests (`cargo test -p totaldrop-cli --test end_to_end`) run against each
  in `.github/workflows/server.yml`.
- The Rust servers depend only on `totaldrop-proto`, never on `-core` or
  `-crypto`.
- Every resource has a limit: message size checked before parsing, bytes and
  connections per client, time. Write the limit down in
  `docs/protocol-v0.1-notes.md` and test just over it.
- What the server sees and stores is part of the privacy policy
  (`site/public/privacy.html`, every language). If it changes, the policy
  changes before the feature ships.
- No request logs with addresses; aggregate counters only.
- Deploys run only from CI (`.github/workflows/deploy.yml`) with the
  maintainer's approval; the Cloudflare connector cannot deploy. Rollback:
  `docs/surec.md`, "Dağıtım".

## CI (`.github/`)

- Actions pinned to full commit SHAs, least `permissions`, secrets only in
  protected environments, never echoed.
- A changed check gets its local copy in `.pre-commit-config-extra.yaml` in
  the same PR (`docs/gelistirme.md`).
- Packages are built only in CI. A release build without the release key
  fails; it never falls back to a debug key.

## Linux desktop (`linux/`, `packaging/`)

- Qt 6, QML and CXX-Qt. The UI follows the system theme live on KDE Plasma,
  Hyprland and Omarchy (`docs/tasarim.md`, table in section 1). Every UI
  change is tried on all three, light and dark, with screenshots in the PR.
- QML component tests: `linux/tests/qml/run.sh`; new components are added to
  `stub/qmldir`.
- Packages: deb, rpm, Arch and the Flatpak (`packaging/flatpak`, decision
  0014). A setting lives in the package recipe, not only in a build script.

## CLI (`crates/totaldrop-cli`)

- Text from the other side is neutralised before it reaches the terminal
  (control characters made visible).
- Exit codes and messages are stable and translated (`l10n/cli`).

## Review checklist

1. Is every input validated and every size bounded, before parsing?
2. What happens under retry, concurrency and partial failure?
3. What does the server store and log, and for how long?
4. Do both server implementations and their tests agree?
5. Can it be deployed and rolled back from CI alone?
