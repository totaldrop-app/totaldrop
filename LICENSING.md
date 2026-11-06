# Licensing

TotalDrop uses three licenses, each chosen for what the code is for.

| Code | License | Why |
| --- | --- | --- |
| Protocol libraries (`crates/totaldrop-crypto`, `crates/totaldrop-proto`, `crates/totaldrop-core`, `crates/totaldrop-l10n`, `fuzz/`) and the translations (`l10n/`), which `totaldrop-l10n` compiles into itself | MIT OR Apache-2.0, at your option | Anyone, including other projects, can build a compatible client without restrictions. An open protocol only matters if it is easy to implement. |
| Apps (`crates/totaldrop-cli`, `app/`, `linux/`) | GPL-3.0-or-later; in `app/`, the bundled fonts keep their SIL Open Font License 1.1 and Cargokit (`app/rust_builder/cargokit/`) its MIT OR Apache-2.0 | The TotalDrop apps, and anything derived from them, stay free software. |
| Web site (`site/`) | GPL-3.0-or-later; the bundled fonts keep their SIL Open Font License 1.1, and the team page's profile pictures (`site/public/team/avatars/`) belong to the people they show (`LicenseRef-TotalDrop-Team-Avatars`) | Same as the apps. |
| Servers (`server/`) | AGPL-3.0-or-later | Anyone who runs a modified server for others must publish their changes, so a hosted fork cannot quietly weaken privacy. |
| Everything else: Linux packaging (`packaging/`), tools, documents, CI | GPL-3.0-or-later | Same as the apps. |

The full texts are in [`LICENSES/`](LICENSES/), named by their SPDX identifiers as the [REUSE](https://reuse.software/) layout does: `MIT.txt`, `Apache-2.0.txt`, `GPL-3.0-or-later.txt`, `AGPL-3.0-or-later.txt` and, for the fonts, `OFL-1.1.txt`; the team page's pictures: `LicenseRef-TotalDrop-Team-Avatars.txt`.

Every file says its copyright and license. Source files start with two lines:

```
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
```

with the identifier of their directory from the table above (comment style as the language needs). [`REUSE.toml`](REUSE.toml) covers everything else by directory: files that cannot carry a comment (images, JSON, translations), documents, build and configuration files (`Cargo.toml`, workflows, Dockerfiles, package recipes, HTML and XML), generated files, and code copied from elsewhere: Cargokit (Matej Knopp, MIT OR Apache-2.0) and the fonts. CI runs `reuse lint` on every change, and `tools/check-spdx.sh`, which fails when a source file lacks the two lines. Copied code keeps its own copyright and license: give it its own entry in `REUSE.toml`, never the project's lines.

The licenses cover the code, not the TotalDrop name and mark. The mark's files (logo, icons) are distributed under the license of their directory like everything else; the name and mark rules only say how they may be used to identify a product, and add no terms to the licenses. A modified version is published under its own name; unmodified packages, links, articles and compatible clients can use the name as described in [docs/marka.md](docs/marka.md) (in Turkish) and on [totaldrop.app/brand](https://totaldrop.app/brand#use), in every language of the site.

Unless you state otherwise, any contribution you submit is licensed under the license of the directory it goes into, without additional terms.
