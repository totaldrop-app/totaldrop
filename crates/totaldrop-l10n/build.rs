//! Compiles the ARB files under `l10n/` and `l10n/status.json` into the
//! crate, so programs never read translations from disk at run time.

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../l10n");
    let mut out = String::from(
        "/// (domain, ARB file contents)\npub(crate) static FILES: &[(&str, &str)] = &[\n",
    );
    for domain in ["app", "cli"] {
        let dir = root.join(domain);
        println!("cargo::rerun-if-changed={}", dir.display());
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
            .unwrap_or_default();
        paths.retain(|p| p.extension().is_some_and(|e| e == "arb"));
        paths.sort();
        for p in paths {
            println!("cargo::rerun-if-changed={}", p.display());
            let _ = writeln!(
                out,
                "    ({domain:?}, include_str!({:?})),",
                p.display().to_string()
            );
        }
    }
    out.push_str("];\n");
    let status = root.join("status.json");
    println!("cargo::rerun-if-changed={}", status.display());
    let _ = writeln!(
        out,
        "pub(crate) static STATUS: &str = include_str!({:?});",
        status.display().to_string()
    );
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("files.rs");
    if let Err(e) = std::fs::write(&dest, out) {
        println!("cargo::error=cannot write {}: {e}", dest.display());
    }
}
