# Licensing

TotalDrop uses three licenses, each chosen for what the code is for.

| Code | License | Why |
| --- | --- | --- |
| Protocol libraries (`crates/totaldrop-crypto`, `crates/totaldrop-proto`, `crates/totaldrop-core`, `fuzz/`) | MIT OR Apache-2.0, at your option | Anyone, including other projects, can build a compatible client without restrictions. An open protocol only matters if it is easy to implement. |
| Apps (`crates/totaldrop-cli`, `app/`) | GPL-3.0-or-later | The TotalDrop apps, and anything derived from them, stay free software. |
| Servers (`server/`) | AGPL-3.0-or-later | Anyone who runs a modified server for others must publish their changes, so a hosted fork cannot quietly weaken privacy. |

The full texts are in `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-GPL-3.0` and `LICENSE-AGPL-3.0`.

Unless you state otherwise, any contribution you submit is licensed under the license of the directory it goes into, without additional terms.
