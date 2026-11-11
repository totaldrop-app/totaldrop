# 0001: One repository, a folder per platform

- **Status:** Accepted, 27 October 2026
- **Context:** TotalDrop will speak the same protocol on Linux, Android,
  Windows, macOS and iOS. Updating one platform must not break the others.
- **Decision:** One repository and one `main` branch. Each platform has its
  own folder, version number, tag (`linux-v…`, `android-v…`) and CI
  workflow. The shared core (`crates/`) changes once and reaches every
  platform. No long-lived platform branches are opened.
- **Why:** A security fix is made in one place and is not forgotten on any
  platform. A separate version per platform means they do not all have to
  be released at the same time.
- **Consequences:** A change in the core runs every platform's CI and is not
  merged until all of them pass. Details:
  [gelistirme.md](../gelistirme.md), [versioning.md](../versioning.md).
