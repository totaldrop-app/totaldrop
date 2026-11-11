---
name: fullstack-engineering
description: "Use when changing TotalDrop's servers and Workers (server/), CI (.github/), the site's functions (site/functions/), the Linux desktop app (linux/, packaging/) or the CLI (crates/totaldrop-cli). APIs, limits, Cloudflare, deployment, Qt/QML and packaging."
---

# Full-stack engineering (TotalDrop)

Own a change from the wire to the pixel. Apply
`software-engineering-discipline` for process and `security-engineering-analyst`
for anything a stranger can reach.

## Servers and Workers (`server/`)

- Each service has two implementations: the Cloudflare Worker
  (`server/*-worker`, TypeScript) and the self-hosted Rust server
  (`server/totaldrop-*`). A behaviour change lands in both; the same
  end-to-end tests (`crates/totaldrop-cli/tests/end_to_end.rs`) run against
  the Rust servers in process and against the Workers in
  `.github/workflows/server.yml`.
- The Rust servers depend only on `totaldrop-proto`, never on `-core` or
  `-crypto`.
- Every resource has a limit: message size checked before parsing, bytes and
  connections per client, time. Write the limit down in
  `docs/protocol-v0.1-notes.md` and test just over it.
- What the server sees and stores is part of the privacy policy
  (`site/public/privacy.html`, every language). If it changes, the policy
  changes before the feature ships.
- No request logs and no addresses stored; per-client counters for rate
  limits live only in memory.
- Deploys run only from CI (`.github/workflows/deploy.yml`) with the
  maintainer's approval, never from a session. Rollback:
  `docs/surec.md`, "Dağıtım".

## CI (`.github/`)

- Actions pinned to full commit SHAs, least `permissions`, secrets only in
  protected environments, never echoed.
- A changed check gets its local copy in `.pre-commit-config-extra.yaml` in
  the same PR (`docs/gelistirme.md`).
- Packages are built only in CI. In CI a release build without the release
  key fails (`TOTALDROP_REQUIRE_RELEASE_KEY`); keep that guard.

## Linux desktop (`linux/`, `packaging/`)

- Qt 6, QML and CXX-Qt. The UI follows the system theme live on KDE Plasma,
  Hyprland and Omarchy (`docs/tasarim.md`, table in section 1). Every UI
  change is tried on all three, light and dark, and in Arabic and Japanese,
  with screenshots in the PR. The mark's motion matches the site
  (`Mark.qml`, `docs/tasarim.md`, "Motion").
- QML component tests: `linux/tests/qml/run.sh`; a component that gets a
  test is added to `stub/qmldir` and to the copy line in `run.sh`.
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
