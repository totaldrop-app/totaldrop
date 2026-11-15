#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Single-source component versioning, native package/store projections,
// deterministic changelog.d bump planning, and release-tag verification
// (docs/versioning.md, docs/kararlar/0026-automated-component-versioning.md).
//
//   node tools/version.mjs check [--tag <tag>] [--rev <rev>] [--main-ref <ref>]
//       Verifies versions.json, every manifest/lockfile/recipe in the checkout,
//       and (with --tag) release-tag ancestry, SemVer monotonicity, build-code
//       monotonicity, and bundled CLI invariants.
//   node tools/version.mjs plan [<component>] [--json]
//       Inspects changelog.d/ and merged release tags to compute the next
//       SemVer and integer build number for one or all components.
//   node tools/version.mjs bump <component> [--level <lvl> | --set <ver>]
//       [--build <n>] [--stage pre-1.0|stable] [--date YYYY-MM-DD]
//       Atomically updates versions.json and every manifest/lockfile/recipe
//       for <component>, then runs check to verify consistency.
//   node tools/version.mjs project <component> [--format <fmt>]
//       Prints deterministic platform/store version strings derived from
//       versions.json (semver, android, deb, rpm, arch, appstream, msix, apple).

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { parseName, splitAreas } from "./changelog.mjs";

export const CHANNELS = ["alpha", "beta", "rc"];
export const KNOWN_COMPONENTS = ["core", "cli", "android", "linux", "server", "mail", "windows", "macos", "ios"];
export const BUILD_REQUIRED = new Set(["android", "windows", "macos", "ios"]);
export const CORE_CRATES = ["totaldrop-crypto", "totaldrop-proto", "totaldrop-l10n", "totaldrop-core"];
export const CORE_LOCK_CRATES = [...CORE_CRATES, "totaldrop-cli"];
export const SERVER_CRATES = ["totaldrop-rendezvous", "totaldrop-relay"];
export const MAX_PLAY_VERSION_CODE = 2_100_000_000;
export const MAX_MSIX_PART = 65_535;

const SEMVER_RE =
  /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-(alpha|beta|rc)\.([1-9][0-9]*))?$/;
const TAG_RE = /^([a-z]+)-v(.+)$/;

/** Parses a TotalDrop SemVer 2.0 string into its numeric parts and channel, or returns null. */
export function parseSemver(raw) {
  if (typeof raw !== "string") return null;
  const m = SEMVER_RE.exec(raw);
  if (!m) return null;
  const major = Number(m[1]);
  const minor = Number(m[2]);
  const patch = Number(m[3]);
  const pre = m[5] !== undefined ? Number(m[5]) : null;
  if (
    !Number.isSafeInteger(major) ||
    !Number.isSafeInteger(minor) ||
    !Number.isSafeInteger(patch) ||
    (pre !== null && !Number.isSafeInteger(pre))
  ) {
    return null;
  }
  return {
    raw,
    major,
    minor,
    patch,
    channel: m[4] ?? null,
    pre,
  };
}

/** Formats parsed SemVer parts back into a canonical SemVer 2.0 string. */
export function formatSemver({ major, minor, patch, channel = null, pre = null }) {
  const base = `${major}.${minor}.${patch}`;
  return channel ? `${base}-${channel}.${pre}` : base;
}

/** Compares two valid TotalDrop SemVer 2.0 strings: -1 if a < b, 0 if equal, 1 if a > b. */
export function compareSemver(a, b) {
  const pa = typeof a === "string" ? parseSemver(a) : a;
  const pb = typeof b === "string" ? parseSemver(b) : b;
  if (!pa || !pb) throw new Error(`invalid SemVer comparison: "${a}" vs "${b}"`);
  for (const k of ["major", "minor", "patch"]) {
    if (pa[k] !== pb[k]) return pa[k] < pb[k] ? -1 : 1;
  }
  if (pa.channel === null && pb.channel === null) return 0;
  if (pa.channel === null) return 1;
  if (pb.channel === null) return -1;
  const ca = CHANNELS.indexOf(pa.channel);
  const cb = CHANNELS.indexOf(pb.channel);
  if (ca !== cb) return ca < cb ? -1 : 1;
  if (pa.pre !== pb.pre) return pa.pre < pb.pre ? -1 : 1;
  return 0;
}

/** Validates a SemVer against the repository's release stage ("pre-1.0" or "stable"). */
export function validateStageSemver(version, stage) {
  const p = parseSemver(version);
  if (!p) {
    return (
      `"${version}" is not a valid TotalDrop SemVer 2.0 string ` +
      "(expected MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-{alpha,beta,rc}.N with N >= 1)"
    );
  }
  if (stage === "pre-1.0") {
    if (p.major !== 0 || p.channel !== "beta") {
      return `"${version}" must have the form 0.MINOR.PATCH-beta.N while stage is "pre-1.0"`;
    }
  } else if (stage === "stable") {
    if (p.major === 0 && p.channel !== "beta") {
      return `"${version}" has major 0 and must have the form 0.MINOR.PATCH-beta.N`;
    }
  } else {
    return `unknown stage "${stage}" (expected "pre-1.0" or "stable")`;
  }
  return null;
}

/** Projects (version, build) into every native package/store version representation. */
export function projectVersion(version, build = null) {
  const p = parseSemver(version);
  if (!p) throw new Error(`invalid SemVer "${version}"`);
  const base = `${p.major}.${p.minor}.${p.patch}`;
  const preSuffix = p.channel ? `${p.channel}.${p.pre}` : null;
  const tilde = preSuffix ? `${base}~${preSuffix}` : base;
  const arch = preSuffix ? `${base}_${preSuffix}` : base;
  const out = {
    semver: p.raw,
    base,
    deb: tilde,
    debChangelog: `${tilde}-1`,
    rpm: tilde,
    arch,
    appstream: tilde,
    appstreamType: preSuffix ? "development" : "stable",
  };
  if (build !== null && build !== undefined) {
    if (!Number.isSafeInteger(build) || build < 1) {
      throw new Error(`build number must be a positive integer, got ${JSON.stringify(build)}`);
    }
    out.build = build;
    out.android = `${p.raw}+${build}`;
    out.androidVersionName = p.raw;
    out.androidVersionCode = build;
    if (p.major <= MAX_MSIX_PART && p.minor <= MAX_MSIX_PART && p.patch <= MAX_MSIX_PART && build <= MAX_MSIX_PART) {
      out.msix = `${p.major}.${p.minor}.${p.patch}.${build}`;
    } else {
      out.msix = null;
    }
    out.apple = {
      shortVersion: base,
      bundleVersion: String(build),
      displayVersion: p.raw,
    };
  }
  return out;
}

function validateManifestData(data) {
  const errors = [];
  if (data === null || typeof data !== "object" || Array.isArray(data)) {
    throw new Error("versions.json: root must be a JSON object");
  }
  if (data.schema !== 1) errors.push(`versions.json: unsupported schema ${JSON.stringify(data.schema)} (expected 1)`);
  if (data.stage !== "pre-1.0" && data.stage !== "stable") {
    errors.push(`versions.json: stage must be "pre-1.0" or "stable", got ${JSON.stringify(data.stage)}`);
  }
  if (!data.components || typeof data.components !== "object" || Array.isArray(data.components)) {
    errors.push("versions.json: components must be an object");
  } else {
    const names = Object.keys(data.components);
    if (names.length === 0) errors.push("versions.json: components cannot be empty");
    for (const name of names) {
      if (!KNOWN_COMPONENTS.includes(name)) {
        errors.push(`versions.json: unknown component "${name}"`);
        continue;
      }
      const entry = data.components[name];
      if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
        errors.push(`versions.json: components.${name} must be an object`);
        continue;
      }
      const stageErr = validateStageSemver(entry.version, data.stage);
      if (stageErr) errors.push(`versions.json (components.${name}.version): ${stageErr}`);
      if (BUILD_REQUIRED.has(name)) {
        if (!Number.isSafeInteger(entry.build) || entry.build < 1) {
          errors.push(`versions.json (components.${name}.build): must be an integer >= 1`);
        } else if (name === "android" && entry.build > MAX_PLAY_VERSION_CODE) {
          errors.push(
            `versions.json (components.android.build): ${entry.build} exceeds Google Play limit ${MAX_PLAY_VERSION_CODE}`,
          );
        } else if (name === "windows") {
          const p = parseSemver(entry.version);
          if (
            entry.build > MAX_MSIX_PART ||
            (p && (p.major > MAX_MSIX_PART || p.minor > MAX_MSIX_PART || p.patch > MAX_MSIX_PART))
          ) {
            errors.push(`versions.json (components.windows): MSIX version parts must each be <= ${MAX_MSIX_PART}`);
          }
        }
      } else if (entry.build !== undefined) {
        errors.push(`versions.json (components.${name}.build): "${name}" does not use an integer build number`);
      }
    }
  }
  return errors;
}

/** Reads and validates versions.json at `root`. */
export function readManifest(root) {
  const file = join(root, "versions.json");
  if (!existsSync(file)) throw new Error("versions.json is missing");
  let data;
  try {
    data = JSON.parse(readFileSync(file, "utf8"));
  } catch (e) {
    throw new Error(`versions.json: invalid JSON (${e.message})`);
  }
  return { data, errors: validateManifestData(data) };
}

function readSectionVersion(tomlText, sectionHeader) {
  const lines = tomlText.split("\n");
  let inSection = false;
  for (const raw of lines) {
    const line = raw.trim();
    if (line.startsWith("[")) {
      inSection = line === sectionHeader;
      continue;
    }
    if (!inSection) continue;
    if (/^version\.workspace\s*=\s*true\b/.test(line)) return "workspace";
    const m = /^version\s*=\s*"([^"]*)"/.exec(line);
    if (m) return m[1];
  }
  return null;
}

function readDepVersions(tomlText, crateNames) {
  const found = [];
  const lines = tomlText.split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^\s*#/.test(line)) continue;
    const m = /^\s*([A-Za-z0-9_-]+)\s*=\s*\{([^}]*)\}/.exec(line);
    if (!m || !crateNames.includes(m[1])) continue;
    const vm = /\bversion\s*=\s*"([^"]*)"/.exec(m[2]);
    if (vm) found.push({ crate: m[1], version: vm[1], line: i + 1 });
  }
  return found;
}

function readCargoLockVersions(lockText) {
  const map = new Map();
  let name = null;
  for (const raw of lockText.split("\n")) {
    const line = raw.trim();
    if (line === "[[package]]") {
      name = null;
      continue;
    }
    const nm = /^name\s*=\s*"([^"]*)"$/.exec(line);
    if (nm) {
      name = nm[1];
      continue;
    }
    if (name !== null) {
      const vm = /^version\s*=\s*"([^"]*)"$/.exec(line);
      if (vm) {
        if (!map.has(name)) map.set(name, []);
        map.get(name).push(vm[1]);
        name = null;
      }
    }
  }
  return map;
}

function listCargoTomls(root) {
  const out = [];
  const walk = (rel) => {
    const abs = rel ? join(root, rel) : root;
    for (const ent of readdirSync(abs, { withFileTypes: true })) {
      if ([".git", "target", "build", ".dart_tool", "node_modules"].includes(ent.name)) continue;
      const next = rel ? `${rel}/${ent.name}` : ent.name;
      if (ent.isDirectory()) walk(next);
      else if (ent.isFile() && ent.name === "Cargo.toml") out.push(next);
    }
  };
  walk("");
  return out.sort();
}

function readMainDartVersion(dartText) {
  const m =
    /String\.fromEnvironment\(\s*['"]TOTALDROP_VERSION['"]\s*,\s*defaultValue:\s*['"]([^'"]*)['"]\s*,?\s*\)/s.exec(
      dartText,
    );
  return m ? m[1] : null;
}

function readJsonTopVersion(jsonText) {
  const data = JSON.parse(jsonText);
  return typeof data?.version === "string" ? data.version : null;
}

function readPackageLockVersions(lockText) {
  const data = JSON.parse(lockText);
  return {
    top: typeof data?.version === "string" ? data.version : null,
    rootPkg: typeof data?.packages?.[""]?.version === "string" ? data.packages[""].version : null,
  };
}

/** Checks every manifest, lockfile, internal dependency, and native recipe in `root` against `versions.json`. */
export function checkRepo(root) {
  const { data, errors } = readManifest(root);
  if (errors.length) return errors;

  const same = (where, got, want) => {
    if (got !== want) errors.push(`mismatch: ${where} says ${JSON.stringify(got)}, expected ${JSON.stringify(want)}`);
  };
  const cargoTomls = listCargoTomls(root);
  const cargoLockPath = join(root, "Cargo.lock");
  const cargoLock = existsSync(cargoLockPath) ? readCargoLockVersions(readFileSync(cargoLockPath, "utf8")) : null;

  const checkLockCrate = (lockMap, label, crate, want) => {
    if (!lockMap) return;
    const versions = lockMap.get(crate);
    if (!versions || versions.length === 0) {
      errors.push(`${label}: missing [[package]] entry for ${crate}`);
    } else if (versions.length > 1) {
      errors.push(`${label}: multiple [[package]] entries for ${crate} (${versions.join(", ")})`);
    } else {
      same(`${label} (${crate})`, versions[0], want);
    }
  };

  const comps = data.components;

  if (comps.core) {
    const want = comps.core.version;
    const rootCargo = join(root, "Cargo.toml");
    if (!existsSync(rootCargo)) {
      errors.push("Cargo.toml: missing (required by components.core)");
    } else {
      same("Cargo.toml ([workspace.package])", readSectionVersion(readFileSync(rootCargo, "utf8"), "[workspace.package]"), want);
    }
    for (const rel of cargoTomls) {
      const text = readFileSync(join(root, rel), "utf8");
      for (const dep of readDepVersions(text, CORE_CRATES)) {
        same(`${rel}:${dep.line} (${dep.crate})`, dep.version, want);
      }
    }
    for (const crate of CORE_LOCK_CRATES) {
      const crateToml = join(root, "crates", crate, "Cargo.toml");
      if (existsSync(crateToml)) {
        same(`crates/${crate}/Cargo.toml`, readSectionVersion(readFileSync(crateToml, "utf8"), "[package]"), "workspace");
        checkLockCrate(cargoLock, "Cargo.lock", crate, want);
      }
    }
    const fuzzLockPath = join(root, "fuzz", "Cargo.lock");
    if (existsSync(fuzzLockPath)) {
      const fuzzLock = readCargoLockVersions(readFileSync(fuzzLockPath, "utf8"));
      for (const crate of CORE_CRATES) {
        if (fuzzLock.has(crate)) checkLockCrate(fuzzLock, "fuzz/Cargo.lock", crate, want);
      }
    }
  }

  if (comps.android) {
    const { version: want, build: wantBuild } = comps.android;
    const proj = projectVersion(want, wantBuild);
    const pubspecPath = join(root, "app", "pubspec.yaml");
    if (!existsSync(pubspecPath)) {
      errors.push("app/pubspec.yaml: missing (required by components.android)");
    } else {
      const pubText = readFileSync(pubspecPath, "utf8");
      const m = /^version:\s*(\S+)\s*$/m.exec(pubText);
      same("app/pubspec.yaml", m ? m[1] : null, proj.android);
    }
    const appRustPath = join(root, "app", "rust", "Cargo.toml");
    if (!existsSync(appRustPath)) {
      errors.push("app/rust/Cargo.toml: missing (required by components.android)");
    } else {
      const v = readSectionVersion(readFileSync(appRustPath, "utf8"), "[package]");
      if (v === "workspace") {
        errors.push("app/rust/Cargo.toml must name its own version, not the workspace's");
      } else {
        same("app/rust/Cargo.toml", v, want);
      }
      checkLockCrate(cargoLock, "Cargo.lock", "rust_lib_totaldrop", want);
    }
    const mainDartPath = join(root, "app", "lib", "main.dart");
    if (!existsSync(mainDartPath)) {
      errors.push("app/lib/main.dart: missing (required by components.android)");
    } else {
      same("app/lib/main.dart (TOTALDROP_VERSION)", readMainDartVersion(readFileSync(mainDartPath, "utf8")), want);
    }
  }

  if (comps.linux) {
    const want = comps.linux.version;
    const proj = projectVersion(want);
    const linuxCargoPath = join(root, "linux", "Cargo.toml");
    if (!existsSync(linuxCargoPath)) {
      errors.push("linux/Cargo.toml: missing (required by components.linux)");
    } else {
      const v = readSectionVersion(readFileSync(linuxCargoPath, "utf8"), "[package]");
      if (v === "workspace") {
        errors.push("linux/Cargo.toml must name its own version, not the workspace's");
      } else {
        same("linux/Cargo.toml", v, want);
      }
      checkLockCrate(cargoLock, "Cargo.lock", "totaldrop-linux", want);
    }
    const pkgbuild = join(root, "packaging", "arch", "PKGBUILD");
    if (existsSync(pkgbuild)) {
      const m = /^_ver=(\S+)\s*$/m.exec(readFileSync(pkgbuild, "utf8"));
      same("packaging/arch/PKGBUILD (_ver)", m ? m[1] : null, want);
    }
    const spec = join(root, "packaging", "rpm", "totaldrop.spec");
    if (existsSync(spec)) {
      const specText = readFileSync(spec, "utf8");
      const up = /^%global\s+upstream_version\s+(\S+)\s*$/m.exec(specText);
      const ver = /^Version:\s*(\S+)\s*$/m.exec(specText);
      same("packaging/rpm/totaldrop.spec (upstream_version)", up ? up[1] : null, want);
      same("packaging/rpm/totaldrop.spec (Version)", ver ? ver[1] : null, proj.rpm);
    }
    const debChangelog = join(root, "packaging", "debian", "changelog");
    if (existsSync(debChangelog)) {
      const first = readFileSync(debChangelog, "utf8").split("\n")[0] ?? "";
      const m = /^totaldrop\s+\(([^)]+)\)/.exec(first);
      const debVer = m ? m[1].replace(/-[0-9]+$/, "") : null;
      same("packaging/debian/changelog", debVer, proj.deb);
    }
    const metainfo = join(root, "packaging", "linux", "app.totaldrop.TotalDrop.metainfo.xml");
    if (existsSync(metainfo)) {
      const xml = readFileSync(metainfo, "utf8");
      const m = /<release[\t\n\r ]([^>]*)>/.exec(xml);
      const attrs = m ? m[1] : "";
      const ver = /\bversion="([^"]*)"/.exec(attrs);
      const type = /\btype="([^"]*)"/.exec(attrs);
      same("packaging/linux/app.totaldrop.TotalDrop.metainfo.xml (version)", ver ? ver[1] : null, proj.appstream);
      same("packaging/linux/app.totaldrop.TotalDrop.metainfo.xml (type)", type ? type[1] : null, proj.appstreamType);
    }
  }

  if (comps.server) {
    const want = comps.server.version;
    for (const crate of SERVER_CRATES) {
      const rel = `server/${crate}/Cargo.toml`;
      const abs = join(root, rel);
      if (!existsSync(abs)) {
        errors.push(`${rel}: missing (required by components.server)`);
      } else {
        const v = readSectionVersion(readFileSync(abs, "utf8"), "[package]");
        if (v === "workspace") {
          errors.push(`${rel} must name its own version, not the workspace's`);
        } else {
          same(rel, v, want);
        }
        checkLockCrate(cargoLock, "Cargo.lock", crate, want);
      }
    }
    for (const worker of ["rendezvous-worker", "relay-worker"]) {
      const pkgRel = `server/${worker}/package.json`;
      const lockRel = `server/${worker}/package-lock.json`;
      if (!existsSync(join(root, pkgRel))) {
        errors.push(`${pkgRel}: missing (required by components.server)`);
      } else {
        same(pkgRel, readJsonTopVersion(readFileSync(join(root, pkgRel), "utf8")), want);
      }
      if (!existsSync(join(root, lockRel))) {
        errors.push(`${lockRel}: missing (required by components.server)`);
      } else {
        const lock = readPackageLockVersions(readFileSync(join(root, lockRel), "utf8"));
        same(lockRel, lock.top, want);
        same(`${lockRel} (root package)`, lock.rootPkg, want);
      }
    }
    for (const rel of cargoTomls) {
      const text = readFileSync(join(root, rel), "utf8");
      for (const dep of readDepVersions(text, SERVER_CRATES)) {
        same(`${rel}:${dep.line} (${dep.crate})`, dep.version, want);
      }
    }
  }

  // Keep docs/ARCHITECTURE.refs.json dependency line snapshots aligned with Cargo.toml.
  const archRefsPath = join(root, "docs", "ARCHITECTURE.refs.json");
  if (existsSync(archRefsPath) && (comps.core || comps.server)) {
    const refs = JSON.parse(readFileSync(archRefsPath, "utf8"));
    for (const [key, lines] of Object.entries(refs)) {
      if (!/^(?:.*\/)?Cargo\.toml:\d+(?:-\d+)?$/.test(key) || !Array.isArray(lines)) continue;
      const joined = lines.join("\n");
      if (comps.core) {
        for (const dep of readDepVersions(joined, CORE_CRATES)) {
          same(`docs/ARCHITECTURE.refs.json (${key}, ${dep.crate})`, dep.version, comps.core.version);
        }
      }
      if (comps.server) {
        for (const dep of readDepVersions(joined, SERVER_CRATES)) {
          same(`docs/ARCHITECTURE.refs.json (${key}, ${dep.crate})`, dep.version, comps.server.version);
        }
      }
    }
  }

  // Forward-compatible with PR #417 (server/mail-worker carries its own version).
  const mailPkg = join(root, "server", "mail-worker", "package.json");
  if (comps.mail || existsSync(mailPkg)) {
    if (!existsSync(mailPkg)) {
      errors.push("server/mail-worker/package.json: missing (required by components.mail)");
    } else {
      const pkgVer = readJsonTopVersion(readFileSync(mailPkg, "utf8"));
      const mailVer = comps.mail ? comps.mail.version : pkgVer;
      if (!comps.mail) {
        const stageErr = validateStageSemver(mailVer, data.stage);
        if (stageErr) errors.push(`server/mail-worker/package.json: ${stageErr}`);
      }
      same("server/mail-worker/package.json", pkgVer, mailVer);
      const mailLock = join(root, "server", "mail-worker", "package-lock.json");
      if (!existsSync(mailLock)) {
        errors.push("server/mail-worker/package-lock.json: missing");
      } else {
        const lock = readPackageLockVersions(readFileSync(mailLock, "utf8"));
        same("server/mail-worker/package-lock.json", lock.top, mailVer);
        same("server/mail-worker/package-lock.json (root package)", lock.rootPkg, mailVer);
      }
    }
  }

  return errors;
}

function git(root, args, { allowFail = false } = {}) {
  try {
    return execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  } catch (e) {
    if (allowFail) return null;
    throw new Error(`git ${args.join(" ")} failed: ${(e.stderr || e.message).trim()}`);
  }
}

function tagComponent(prefix, components) {
  if (prefix === "cli" && !components.cli && components.core) return "core";
  return components[prefix] ? prefix : null;
}

function readHistoricalComponentAtTag(root, tag, component) {
  const vJson = git(root, ["show", `${tag}:versions.json`], { allowFail: true });
  if (vJson !== null) {
    try {
      const parsed = JSON.parse(vJson);
      const entry = parsed?.components?.[component];
      if (entry?.version) return { version: entry.version, build: entry.build ?? null };
    } catch {
      // Fall back to component manifests below.
    }
  }
  if (component === "android") {
    const pub = git(root, ["show", `${tag}:app/pubspec.yaml`], { allowFail: true });
    if (pub !== null) {
      const m = /^version:\s*([^+\s]+)(?:\+([0-9]+))?\s*$/m.exec(pub);
      if (m) return { version: m[1], build: m[2] !== undefined ? Number(m[2]) : null };
    }
  } else if (component === "linux") {
    const toml = git(root, ["show", `${tag}:linux/Cargo.toml`], { allowFail: true });
    if (toml !== null) return { version: readSectionVersion(toml, "[package]"), build: null };
  } else if (component === "core" || component === "cli") {
    const toml = git(root, ["show", `${tag}:Cargo.toml`], { allowFail: true });
    if (toml !== null) return { version: readSectionVersion(toml, "[workspace.package]"), build: null };
  } else if (component === "server") {
    const toml = git(root, ["show", `${tag}:server/totaldrop-rendezvous/Cargo.toml`], { allowFail: true });
    if (toml !== null) return { version: readSectionVersion(toml, "[package]"), build: null };
  }
  const m = TAG_RE.exec(tag);
  return { version: m ? m[2] : null, build: null };
}

/** Verifies a `<prefix>-v<semver>` release tag against `versions.json`, `origin/main`, and tag history. */
export function checkTag(root, tagName, { rev = "HEAD", mainRef = "origin/main" } = {}) {
  const errors = checkRepo(root);
  if (errors.length) return errors;

  const { data } = readManifest(root);
  const m = TAG_RE.exec(tagName);
  if (!m) return [`tag "${tagName}" is not of the form <platform>-v<semver>`];
  const [, prefix, tagVer] = m;
  const comp = tagComponent(prefix, data.components);
  if (!comp) return [`tag "${tagName}": component "${prefix}" is not declared in versions.json`];

  if (git(root, ["rev-parse", "--is-shallow-repository"]) !== "false") {
    return ["shallow clone; fetch the full history and tags (fetch-depth: 0)"];
  }
  const revCommit = git(root, ["rev-parse", "--verify", "-q", `${rev}^{commit}`], { allowFail: true });
  if (revCommit === null) {
    return [`no commit "${rev}"`];
  }

  const existingTagCommit = git(root, ["rev-parse", "--verify", "-q", `refs/tags/${tagName}^{commit}`], {
    allowFail: true,
  });
  if (existingTagCommit !== null && existingTagCommit !== revCommit) {
    errors.push(`tag "${tagName}" points to ${existingTagCommit}, not ${revCommit}`);
  }

  // Fail closed: the main branch reference must exist and contain the tagged commit.
  if (!mainRef || git(root, ["rev-parse", "--verify", "-q", `${mainRef}^{commit}`], { allowFail: true }) === null) {
    errors.push(`main reference "${mainRef}" not found; fetch "${mainRef}" or pass --main-ref <ref>`);
  } else {
    try {
      execFileSync("git", ["merge-base", "--is-ancestor", revCommit, mainRef], { cwd: root, stdio: "ignore" });
    } catch {
      errors.push(`commit ${rev} (${tagName}) is not an ancestor of ${mainRef}`);
    }
  }

  const current = data.components[comp];
  if (tagVer !== current.version) {
    errors.push(`tag "${tagName}" names version "${tagVer}", but versions.json (${comp}) has "${current.version}"`);
    return errors;
  }

  // Inspect every tag reachable from revCommit across ALL merge parents (excluding tagName itself).
  const patterns = comp === "core" ? ["core-v*", "cli-v*"] : [`${prefix}-v*`];
  const mergedRaw = git(root, ["tag", "--merged", revCommit, "-l", ...patterns]);
  const priorTags = mergedRaw ? mergedRaw.split("\n").filter((t) => Boolean(t) && t !== tagName) : [];

  let maxVer = null;
  let maxVerTag = null;
  let maxBuild = null;
  let maxBuildTag = null;

  for (const t of priorTags) {
    const hist = readHistoricalComponentAtTag(root, t, comp);
    if (!hist.version || !parseSemver(hist.version)) {
      errors.push(`prior tag ${t} has invalid SemVer "${hist.version ?? ""}"`);
      continue;
    }
    if (maxVer === null || compareSemver(hist.version, maxVer) > 0) {
      maxVer = hist.version;
      maxVerTag = t;
    }
    if (hist.build !== null && (maxBuild === null || hist.build > maxBuild)) {
      maxBuild = hist.build;
      maxBuildTag = t;
    }
  }

  if (maxVer !== null && compareSemver(current.version, maxVer) <= 0) {
    errors.push(
      `${comp} version ${current.version} (${tagName}) is not higher than ${maxVer} (released in ${maxVerTag})`,
    );
  }
  if (BUILD_REQUIRED.has(comp) && maxBuild !== null && current.build <= maxBuild) {
    errors.push(
      `${comp} build number ${current.build} (${tagName}) is not above ${maxBuild} (released in ${maxBuildTag})`,
    );
  }

  if (prefix === "linux" || prefix === "cli" || prefix === "core") {
    const cliCheck = join(root, "packaging", "check-cli-version.sh");
    if (existsSync(cliCheck)) {
      const cliTagsRaw = git(root, ["tag", "--merged", revCommit, "-l", "linux-v*", "cli-v*", "core-v*"]);
      const hasAncestorCliTag = cliTagsRaw
        ? cliTagsRaw
            .split("\n")
            .filter(Boolean)
            .some((t) => git(root, ["rev-parse", "--verify", "-q", `refs/tags/${t}^{commit}`], { allowFail: true }) !== revCommit)
        : false;
      // On linux-v* always invoke check-cli-version.sh (as check-versions.sh does);
      // on cli-v* / core-v* invoke whenever at least one prior release tag exists.
      if (prefix === "linux" || hasAncestorCliTag) {
        try {
          execFileSync("sh", [cliCheck, revCommit], { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
        } catch (e) {
          errors.push((e.stderr || e.stdout || e.message).trim());
        }
      }
    }
  }

  return errors;
}

function areaMatchesComponent(area, component) {
  const sharedClientAndCore = /\b(all platforms|transfers?|core|crypto|protocol|l10n|translations?)\b/i.test(area);
  if (component === "android") return /\bandroid\b/i.test(area) || sharedClientAndCore;
  if (component === "linux") return /\b(linux|cli|command line)\b/i.test(area) || sharedClientAndCore;
  if (component === "core" || component === "cli") return /\b(cli|command line)\b/i.test(area) || sharedClientAndCore;
  if (component === "server") return /\b(servers?|rendezvous|relay)\b/i.test(area);
  if (component === "mail") return /\bmail\b/i.test(area);
  if (component === "windows") return /\bwindows\b/i.test(area) || sharedClientAndCore;
  if (component === "macos") return /\bmacos\b/i.test(area) || sharedClientAndCore;
  if (component === "ios") return /\bios\b/i.test(area) || sharedClientAndCore;
  return false;
}

function collectPendingFragments(root, component) {
  const dir = join(root, "changelog.d");
  if (!existsSync(dir)) return { sections: new Set(), files: [] };
  const sections = new Set();
  const files = [];
  for (const name of readdirSync(dir).sort()) {
    if (name === "README.md") continue;
    const parsed = parseName(name);
    if (parsed.error) continue;
    if (parsed.section === "none") continue;
    const text = readFileSync(join(dir, name), "utf8");
    let relevant = false;
    for (const line of text.split("\n")) {
      if (!line.startsWith("- ")) continue;
      const m = /^- \*\*([^*]+):\*\*/.exec(line);
      if (!m) continue;
      if (splitAreas(m[1]).some((a) => areaMatchesComponent(a, component))) {
        relevant = true;
        break;
      }
    }
    if (relevant) {
      sections.add(parsed.section);
      files.push({ file: name, section: parsed.section });
    }
  }
  return { sections, files };
}

/** Computes the next SemVer for `current` given `stage` and `level`. */
export function nextSemver(current, stage, level) {
  const p = parseSemver(current);
  if (!p) throw new Error(`invalid current SemVer "${current}"`);

  if (stage === "pre-1.0") {
    if (level === "minor") return formatSemver({ major: 0, minor: p.minor + 1, patch: 0, channel: "beta", pre: 1 });
    if (level === "patch") return formatSemver({ major: 0, minor: p.minor, patch: p.patch + 1, channel: "beta", pre: 1 });
    if (level === "prerelease" || level === "beta") {
      return formatSemver({ major: 0, minor: p.minor, patch: p.patch, channel: "beta", pre: (p.pre ?? 0) + 1 });
    }
    throw new Error(`unsupported bump level "${level}" while stage is "pre-1.0" (use minor, patch, prerelease, or --stage stable)`);
  }

  if (stage !== "stable") throw new Error(`unknown stage "${stage}"`);

  if (p.major === 0) {
    // Under stage "stable", 0.x components can either promote to 1.0.0(-chan.1)
    // or continue advancing on 0.MINOR.PATCH-beta.N until promoted.
    if (CHANNELS.includes(level)) return formatSemver({ major: 1, minor: 0, patch: 0, channel: level, pre: 1 });
    if (level === "release" || level === "major") return "1.0.0";
    if (level === "minor") return formatSemver({ major: 0, minor: p.minor + 1, patch: 0, channel: "beta", pre: 1 });
    if (level === "patch") return formatSemver({ major: 0, minor: p.minor, patch: p.patch + 1, channel: "beta", pre: 1 });
    if (level === "prerelease") {
      return formatSemver({ major: 0, minor: p.minor, patch: p.patch, channel: "beta", pre: (p.pre ?? 0) + 1 });
    }
    throw new Error(`unknown bump level "${level}" for "${current}"`);
  }

  if (level === "major") {
    if (p.channel !== null && p.minor === 0 && p.patch === 0) {
      return formatSemver({ major: p.major, minor: 0, patch: 0 });
    }
    return formatSemver({ major: p.major + 1, minor: 0, patch: 0 });
  }
  if (level === "minor") {
    if (p.channel !== null && p.patch === 0) {
      return formatSemver({ major: p.major, minor: p.minor, patch: 0 });
    }
    return formatSemver({ major: p.major, minor: p.minor + 1, patch: 0 });
  }
  if (level === "patch") {
    if (p.channel !== null) return formatSemver({ major: p.major, minor: p.minor, patch: p.patch });
    return formatSemver({ major: p.major, minor: p.minor, patch: p.patch + 1 });
  }
  if (level === "release") {
    if (p.channel === null) throw new Error(`"${current}" is already a final release`);
    return formatSemver({ major: p.major, minor: p.minor, patch: p.patch });
  }
  if (level === "prerelease") {
    if (p.channel === null) {
      return formatSemver({ major: p.major, minor: p.minor, patch: p.patch + 1, channel: "rc", pre: 1 });
    }
    return formatSemver({ major: p.major, minor: p.minor, patch: p.patch, channel: p.channel, pre: p.pre + 1 });
  }
  if (CHANNELS.includes(level)) {
    if (p.channel === null) {
      return formatSemver({ major: p.major, minor: p.minor + 1, patch: 0, channel: level, pre: 1 });
    }
    const curIdx = CHANNELS.indexOf(p.channel);
    const nextIdx = CHANNELS.indexOf(level);
    if (nextIdx < curIdx) throw new Error(`cannot move backwards from "${p.channel}" to "${level}" on "${current}"`);
    if (nextIdx === curIdx) {
      return formatSemver({ major: p.major, minor: p.minor, patch: p.patch, channel: level, pre: p.pre + 1 });
    }
    return formatSemver({ major: p.major, minor: p.minor, patch: p.patch, channel: level, pre: 1 });
  }
  throw new Error(`unknown bump level "${level}"`);
}

function highestReleasedTag(root, component) {
  if (!existsSync(join(root, ".git"))) return { best: null, shallowClone: false };
  const shallowClone = git(root, ["rev-parse", "--is-shallow-repository"], { allowFail: true }) === "true";
  if (git(root, ["rev-parse", "--verify", "-q", "HEAD"], { allowFail: true }) === null) {
    return { best: null, shallowClone };
  }
  const patterns = component === "core" ? ["core-v*", "cli-v*", "linux-v*"] : [`${component}-v*`];
  const raw = git(root, ["tag", "--merged", "HEAD", "-l", ...patterns], { allowFail: true });
  if (!raw) return { best: null, shallowClone };
  let best = null;
  for (const t of raw.split("\n").filter(Boolean)) {
    const hist = readHistoricalComponentAtTag(root, t, component);
    if (!hist.version || !parseSemver(hist.version)) continue;
    if (best === null || compareSemver(hist.version, best.version) > 0) {
      best = { tag: t, version: hist.version, build: hist.build };
    }
  }
  return { best, shallowClone };
}

/** Plans the next SemVer and build number for one component from `changelog.d/` and git tag history. */
export function planComponent(root, component) {
  const { data, errors } = readManifest(root);
  if (errors.length) throw new Error(errors.join("\n"));
  const entry = data.components[component];
  if (!entry) throw new Error(`unknown component "${component}"`);

  const { sections, files } = collectPendingFragments(root, component);
  const { best: released, shallowClone } = highestReleasedTag(root, component);
  const alreadyAhead = released !== null && compareSemver(entry.version, released.version) > 0;
  const parsedCurrent = parseSemver(entry.version);

  let recommendedLevel;
  if (parsedCurrent && parsedCurrent.major >= 1) {
    if (parsedCurrent.channel !== null) {
      recommendedLevel = "prerelease";
    } else if (sections.has("removed")) {
      recommendedLevel = "major";
    } else if (sections.has("added") || sections.has("changed") || sections.has("deprecated")) {
      recommendedLevel = "minor";
    } else {
      recommendedLevel = "patch";
    }
  } else if (sections.has("added") || sections.has("changed") || sections.has("deprecated") || sections.has("removed")) {
    recommendedLevel = "minor";
  } else if (sections.has("fixed") || sections.has("security")) {
    recommendedLevel = "patch";
  } else {
    recommendedLevel = "prerelease";
  }

  const proposedVersion = alreadyAhead ? entry.version : nextSemver(entry.version, data.stage, recommendedLevel);
  const sameSeriesVersion =
    parsedCurrent?.major === 0 ? nextSemver(entry.version, data.stage, "prerelease") : null;
  const proposedBuild = BUILD_REQUIRED.has(component)
    ? alreadyAhead
      ? entry.build
      : entry.build + 1
    : null;

  return {
    component,
    stage: data.stage,
    currentVersion: entry.version,
    currentBuild: entry.build ?? null,
    lastReleaseTag: released?.tag ?? null,
    lastReleaseVersion: released?.version ?? null,
    shallowClone,
    alreadyAhead,
    recommendedLevel: alreadyAhead ? "none" : recommendedLevel,
    proposedVersion,
    proposedBuild,
    sameSeriesBeta: sameSeriesVersion,
    sameSeriesBuild: BUILD_REQUIRED.has(component) ? entry.build + 1 : null,
    fragmentCount: files.length,
    fragments: files,
  };
}

function updateSectionVersion(tomlText, sectionHeader, newVersion) {
  const lines = tomlText.split("\n");
  let inSection = false;
  let replaced = false;
  for (let i = 0; i < lines.length; i++) {
    const trimmed = lines[i].trim();
    if (trimmed.startsWith("[")) {
      inSection = trimmed === sectionHeader;
      continue;
    }
    if (inSection && !replaced && /^version\s*=\s*"[^"]*"/.test(trimmed)) {
      lines[i] = lines[i].replace(/^(\s*version\s*=\s*")[^"]*(")/, `$1${newVersion}$2`);
      replaced = true;
    }
  }
  if (!replaced) throw new Error(`could not find version under ${sectionHeader}`);
  return lines.join("\n");
}

function updateDepVersions(tomlText, crateNames, newVersion) {
  const lines = tomlText.split("\n");
  let changed = false;
  for (let i = 0; i < lines.length; i++) {
    if (/^\s*#/.test(lines[i])) continue;
    const m = /^(\s*([A-Za-z0-9_-]+)\s*=\s*\{)([^}]*)(\}.*)$/.exec(lines[i]);
    if (!m || !crateNames.includes(m[2])) continue;
    if (/\bversion\s*=\s*"[^"]*"/.test(m[3])) {
      const updatedInner = m[3].replace(/(\bversion\s*=\s*")[^"]*(")/, `$1${newVersion}$2`);
      if (updatedInner !== m[3]) {
        lines[i] = `${m[1]}${updatedInner}${m[4]}`;
        changed = true;
      }
    }
  }
  return { text: lines.join("\n"), changed };
}

function updateCargoLock(lockText, crateNames, newVersion) {
  const targets = new Set(crateNames);
  const lines = lockText.split("\n");
  let active = null;
  for (let i = 0; i < lines.length; i++) {
    const trimmed = lines[i].trim();
    if (trimmed === "[[package]]") {
      active = null;
      continue;
    }
    const nm = /^name\s*=\s*"([^"]*)"$/.exec(trimmed);
    if (nm) {
      active = targets.has(nm[1]) ? nm[1] : null;
      continue;
    }
    if (active !== null && /^version\s*=\s*"[^"]*"$/.test(trimmed)) {
      lines[i] = lines[i].replace(/^(\s*version\s*=\s*")[^"]*(")/, `$1${newVersion}$2`);
      active = null;
    }
  }
  return lines.join("\n");
}

function updateWorkerJsonText(raw, newVersion, isLock = false) {
  JSON.parse(raw);
  let updated = raw.replace(/^(\s{2}"version":\s*")[^"]*(")/m, `$1${newVersion}$2`);
  if (isLock) {
    updated = updated.replace(
      /("packages":\s*\{\s*""\s*:\s*\{(?:(?!\n\s{4}\})[\s\S])*?\n\s{6}"version":\s*")[^"]*(")/,
      `$1${newVersion}$2`,
    );
    const check = readPackageLockVersions(updated);
    if (check.top !== newVersion || check.rootPkg !== newVersion) {
      const parsed = JSON.parse(raw);
      parsed.version = newVersion;
      if (parsed.packages?.[""] && typeof parsed.packages[""] === "object") {
        parsed.packages[""].version = newVersion;
      }
      updated = `${JSON.stringify(parsed, null, 2)}\n`;
    }
  } else if (readJsonTopVersion(updated) !== newVersion) {
    const parsed = JSON.parse(raw);
    parsed.version = newVersion;
    updated = `${JSON.stringify(parsed, null, 2)}\n`;
  }
  return updated;
}

function updateArchitectureRefsText(rawJson, crateNames, newVersion) {
  const refs = JSON.parse(rawJson);
  let anyChanged = false;
  for (const [key, lines] of Object.entries(refs)) {
    if (!/^(?:.*\/)?Cargo\.toml:\d+(?:-\d+)?$/.test(key) || !Array.isArray(lines)) continue;
    for (let i = 0; i < lines.length; i++) {
      if (typeof lines[i] !== "string") continue;
      const { text, changed } = updateDepVersions(lines[i], crateNames, newVersion);
      if (changed) {
        lines[i] = text;
        anyChanged = true;
      }
    }
  }
  return { text: `${JSON.stringify(refs, null, 2)}\n`, changed: anyChanged };
}

function todayUtc(now = new Date()) {
  return now.toISOString().slice(0, 10);
}

function parseStrictIsoDate(isoDate) {
  if (typeof isoDate !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(isoDate)) {
    throw new Error(`invalid ISO date "${isoDate}" (expected YYYY-MM-DD)`);
  }
  const d = new Date(`${isoDate}T12:00:00Z`);
  if (Number.isNaN(d.getTime()) || d.toISOString().slice(0, 10) !== isoDate) {
    throw new Error(`invalid calendar date "${isoDate}"`);
  }
  return d;
}

function formatRfc2822Date(isoDate) {
  return parseStrictIsoDate(isoDate).toUTCString().replace(/GMT$/, "+0000");
}

const RPM_DAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const RPM_MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

function formatRpmDate(isoDate) {
  const d = parseStrictIsoDate(isoDate);
  const day = String(d.getUTCDate()).padStart(2, "0");
  return `${RPM_DAYS[d.getUTCDay()]} ${RPM_MONTHS[d.getUTCMonth()]} ${day} ${d.getUTCFullYear()}`;
}

/** Atomically bumps `component` across `versions.json` and every manifest/lockfile/recipe in `root`. */
export function bumpComponent(
  root,
  component,
  { level = null, setVersion = null, build = null, stage = null, date = todayUtc() } = {},
) {
  if (level !== null && setVersion !== null) {
    throw new Error("cannot pass both --level and --set");
  }
  parseStrictIsoDate(date);

  const { data, errors } = readManifest(root);
  if (errors.length) throw new Error(errors.join("\n"));
  const entry = data.components[component];
  if (!entry) throw new Error(`unknown component "${component}"`);

  const nextStage = stage ?? data.stage;
  if (nextStage !== "pre-1.0" && nextStage !== "stable") {
    throw new Error(`invalid stage "${nextStage}"`);
  }

  let targetVersion;
  if (setVersion !== null) {
    const stageErr = validateStageSemver(setVersion, nextStage);
    if (stageErr) throw new Error(stageErr);
    targetVersion = setVersion;
  } else {
    const chosenLevel = level ?? planComponent(root, component).recommendedLevel;
    if (chosenLevel === "none") {
      throw new Error(
        `${component} (${entry.version}) is already ahead of its last release tag; pass --level or --set to bump again`,
      );
    }
    targetVersion = nextSemver(entry.version, nextStage, chosenLevel);
    const stageErr = validateStageSemver(targetVersion, nextStage);
    if (stageErr) throw new Error(stageErr);
  }

  if (compareSemver(targetVersion, entry.version) <= 0) {
    throw new Error(`new version "${targetVersion}" must be higher than current "${entry.version}"`);
  }

  let targetBuild = null;
  if (BUILD_REQUIRED.has(component)) {
    targetBuild = build !== null ? build : entry.build + 1;
    if (!Number.isSafeInteger(targetBuild) || targetBuild <= entry.build) {
      throw new Error(`new build number (${targetBuild}) must be an integer above current (${entry.build})`);
    }
  } else if (build !== null) {
    throw new Error(`component "${component}" does not carry an integer build number`);
  }

  const candidateData = structuredClone(data);
  candidateData.stage = nextStage;
  candidateData.components[component] =
    targetBuild !== null ? { version: targetVersion, build: targetBuild } : { version: targetVersion };
  const candidateErrors = validateManifestData(candidateData);
  if (candidateErrors.length) {
    throw new Error(candidateErrors.join("\n"));
  }

  const proj = projectVersion(targetVersion, targetBuild);
  const cargoTomls = listCargoTomls(root);
  const cargoLockPath = join(root, "Cargo.lock");
  const archRefsPath = join(root, "docs", "ARCHITECTURE.refs.json");

  // Stage every file mutation in memory first so invalid inputs or missing sections touch zero files on disk.
  const staged = new Map();
  const readStaged = (abs) => (staged.has(abs) ? staged.get(abs) : readFileSync(abs, "utf8"));
  staged.set(join(root, "versions.json"), `${JSON.stringify(candidateData, null, 2)}\n`);

  if (component === "core") {
    const rootCargo = join(root, "Cargo.toml");
    staged.set(rootCargo, updateSectionVersion(readStaged(rootCargo), "[workspace.package]", targetVersion));
    for (const rel of cargoTomls) {
      const abs = join(root, rel);
      const { text, changed } = updateDepVersions(readStaged(abs), CORE_CRATES, targetVersion);
      if (changed) staged.set(abs, text);
    }
    if (existsSync(cargoLockPath)) {
      staged.set(cargoLockPath, updateCargoLock(readStaged(cargoLockPath), CORE_LOCK_CRATES, targetVersion));
    }
    const fuzzLockPath = join(root, "fuzz", "Cargo.lock");
    if (existsSync(fuzzLockPath)) {
      staged.set(fuzzLockPath, updateCargoLock(readStaged(fuzzLockPath), CORE_CRATES, targetVersion));
    }
    if (existsSync(archRefsPath)) {
      const { text, changed } = updateArchitectureRefsText(readStaged(archRefsPath), CORE_CRATES, targetVersion);
      if (changed) staged.set(archRefsPath, text);
    }
  } else if (component === "android") {
    const pubspecPath = join(root, "app", "pubspec.yaml");
    staged.set(pubspecPath, readStaged(pubspecPath).replace(/^version:\s*\S+/m, `version: ${proj.android}`));

    const appRustPath = join(root, "app", "rust", "Cargo.toml");
    staged.set(appRustPath, updateSectionVersion(readStaged(appRustPath), "[package]", targetVersion));

    const mainDartPath = join(root, "app", "lib", "main.dart");
    staged.set(
      mainDartPath,
      readStaged(mainDartPath).replace(
        /(String\.fromEnvironment\(\s*['"]TOTALDROP_VERSION['"]\s*,\s*defaultValue:\s*['"])[^'"]*(['"]\s*,?\s*\))/s,
        `$1${targetVersion}$2`,
      ),
    );

    if (existsSync(cargoLockPath)) {
      staged.set(cargoLockPath, updateCargoLock(readStaged(cargoLockPath), ["rust_lib_totaldrop"], targetVersion));
    }
  } else if (component === "linux") {
    const linuxCargo = join(root, "linux", "Cargo.toml");
    staged.set(linuxCargo, updateSectionVersion(readStaged(linuxCargo), "[package]", targetVersion));

    const pkgbuild = join(root, "packaging", "arch", "PKGBUILD");
    if (existsSync(pkgbuild)) {
      const updated = readStaged(pkgbuild)
        .replace(/^_ver=\S+/m, `_ver=${targetVersion}`)
        .replace(/^pkgrel=[0-9]+/m, "pkgrel=1");
      staged.set(pkgbuild, updated);
    }

    const spec = join(root, "packaging", "rpm", "totaldrop.spec");
    if (existsSync(spec)) {
      let updated = readStaged(spec)
        .replace(/^(%global\s+upstream_version\s+)\S+/m, `$1${targetVersion}`)
        .replace(/^(Version:\s*)\S+/m, `$1${proj.rpm}`)
        .replace(/^(Release:\s*)[0-9]+(%\{\?dist\})/m, "$11$2");
      if (/^%changelog$/m.test(updated) && !updated.includes(` - ${proj.rpm}-`)) {
        const rpmEntry = `* ${formatRpmDate(date)} TotalDrop <hello@totaldrop.app> - ${proj.rpm}-1\n- Release ${targetVersion}.\n\n`;
        updated = updated.replace(/^%changelog\n/m, `%changelog\n${rpmEntry}`);
      }
      staged.set(spec, updated);
    }

    const debChangelog = join(root, "packaging", "debian", "changelog");
    if (existsSync(debChangelog)) {
      const cur = readStaged(debChangelog);
      if (!cur.startsWith(`totaldrop (${proj.deb}-`)) {
        const stanza =
          `totaldrop (${proj.debChangelog}) unstable; urgency=medium\n\n` +
          `  * Release ${targetVersion}.\n\n` +
          ` -- TotalDrop <hello@totaldrop.app>  ${formatRfc2822Date(date)}\n\n`;
        staged.set(debChangelog, stanza + cur);
      }
    }

    const metainfo = join(root, "packaging", "linux", "app.totaldrop.TotalDrop.metainfo.xml");
    if (existsSync(metainfo)) {
      const xml = readStaged(metainfo);
      let hasVersion = false;
      for (const m of xml.matchAll(/<release[\t\n\r ]([^>]*)>/g)) {
        const ver = /\bversion="([^"]*)"/.exec(m[1]);
        if (ver && ver[1] === proj.appstream) {
          hasVersion = true;
          break;
        }
      }
      if (!hasVersion) {
        const entryXml =
          `    <release version="${proj.appstream}" date="${date}" type="${proj.appstreamType}">\n` +
          "      <description>\n" +
          `        <p>TotalDrop ${targetVersion}.</p>\n` +
          "      </description>\n" +
          "    </release>\n";
        staged.set(metainfo, xml.replace("<releases>\n", () => `<releases>\n${entryXml}`));
      }
    }

    if (existsSync(cargoLockPath)) {
      staged.set(cargoLockPath, updateCargoLock(readStaged(cargoLockPath), ["totaldrop-linux"], targetVersion));
    }
  } else if (component === "server") {
    for (const crate of SERVER_CRATES) {
      const abs = join(root, "server", crate, "Cargo.toml");
      staged.set(abs, updateSectionVersion(readStaged(abs), "[package]", targetVersion));
    }
    for (const worker of ["rendezvous-worker", "relay-worker"]) {
      const pkgAbs = join(root, "server", worker, "package.json");
      const lockAbs = join(root, "server", worker, "package-lock.json");
      staged.set(pkgAbs, updateWorkerJsonText(readStaged(pkgAbs), targetVersion, false));
      staged.set(lockAbs, updateWorkerJsonText(readStaged(lockAbs), targetVersion, true));
    }
    for (const rel of cargoTomls) {
      const abs = join(root, rel);
      const { text, changed } = updateDepVersions(readStaged(abs), SERVER_CRATES, targetVersion);
      if (changed) staged.set(abs, text);
    }
    if (existsSync(cargoLockPath)) {
      staged.set(cargoLockPath, updateCargoLock(readStaged(cargoLockPath), SERVER_CRATES, targetVersion));
    }
    if (existsSync(archRefsPath)) {
      const { text, changed } = updateArchitectureRefsText(readStaged(archRefsPath), SERVER_CRATES, targetVersion);
      if (changed) staged.set(archRefsPath, text);
    }
  } else if (component === "mail") {
    const pkgAbs = join(root, "server", "mail-worker", "package.json");
    const lockAbs = join(root, "server", "mail-worker", "package-lock.json");
    staged.set(pkgAbs, updateWorkerJsonText(readStaged(pkgAbs), targetVersion, false));
    staged.set(lockAbs, updateWorkerJsonText(readStaged(lockAbs), targetVersion, true));
  }

  for (const [abs, content] of staged) {
    writeFileSync(abs, content);
  }

  const postErrors = checkRepo(root);
  if (postErrors.length) {
    throw new Error(`post-bump verification failed:\n${postErrors.join("\n")}`);
  }

  return { component, version: targetVersion, build: targetBuild, projections: proj };
}

function parseFlags(args) {
  const pos = [];
  const flags = {};
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a === "--json") flags.json = true;
    else if (["--tag", "--rev", "--main-ref", "--level", "--set", "--build", "--stage", "--date", "--format"].includes(a)) {
      if (i + 1 >= args.length) throw new Error(`missing value for ${a}`);
      flags[a.slice(2)] = args[++i];
    } else if (a.startsWith("-")) {
      throw new Error(`unknown flag "${a}"`);
    } else {
      pos.push(a);
    }
  }
  return { pos, flags };
}

function ensureAllowedFlags(cmd, flags, allowed) {
  const allowedSet = new Set(allowed);
  for (const k of Object.keys(flags)) {
    if (!allowedSet.has(k)) {
      throw new Error(`unknown flag "--${k}" for subcommand "${cmd}"`);
    }
  }
}

export function main(argv, root = fileURLToPath(new URL("..", import.meta.url))) {
  const [cmd, ...rest] = argv;
  const { pos, flags } = parseFlags(rest);

  if (cmd === "check" && pos.length === 0) {
    ensureAllowedFlags("check", flags, ["tag", "rev", "main-ref"]);
    const errors = flags.tag
      ? checkTag(root, flags.tag, { rev: flags.rev ?? "HEAD", mainRef: flags["main-ref"] ?? "origin/main" })
      : checkRepo(root);
    if (errors.length) {
      for (const e of errors) console.error(e);
      return 1;
    }
    const { data } = readManifest(root);
    const summary = Object.entries(data.components)
      .map(([k, v]) => (v.build !== undefined ? `${k} ${v.version}+${v.build}` : `${k} ${v.version}`))
      .join(", ");
    console.log(flags.tag ? `version check ok (${flags.tag}): ${summary}` : `version check ok: ${summary}`);
    return 0;
  }

  if (cmd === "plan" && pos.length <= 1) {
    ensureAllowedFlags("plan", flags, ["json"]);
    const { data, errors } = readManifest(root);
    if (errors.length) throw new Error(errors.join("\n"));
    const targets = pos.length === 1 ? [pos[0]] : Object.keys(data.components);
    const plans = targets.map((c) => planComponent(root, c));
    if (flags.json) {
      console.log(JSON.stringify(plans, null, 2));
    } else {
      for (const p of plans) {
        const cur = p.currentBuild !== null ? `${p.currentVersion}+${p.currentBuild}` : p.currentVersion;
        const next = p.proposedBuild !== null ? `${p.proposedVersion}+${p.proposedBuild}` : p.proposedVersion;
        const alt =
          p.sameSeriesBeta && p.sameSeriesBeta !== p.proposedVersion
            ? ` (or ${p.sameSeriesBuild !== null ? `${p.sameSeriesBeta}+${p.sameSeriesBuild}` : p.sameSeriesBeta} with --level prerelease)`
            : "";
        const shallowWarn = p.shallowClone ? " [shallow clone: fetch tags for full history]" : "";
        console.log(
          `${p.component}: ${cur} -> ${next} [level=${p.recommendedLevel}, fragments=${p.fragmentCount}, lastTag=${p.lastReleaseTag ?? "none"}]${alt}${shallowWarn}`,
        );
      }
    }
    return 0;
  }

  if (cmd === "bump" && pos.length === 1) {
    ensureAllowedFlags("bump", flags, ["level", "set", "build", "stage", "date"]);
    let buildNum = null;
    if (flags.build !== undefined) {
      if (!/^[1-9][0-9]*$/.test(flags.build)) {
        throw new Error(`--build must be a positive integer, got "${flags.build}"`);
      }
      buildNum = Number(flags.build);
    }
    const res = bumpComponent(root, pos[0], {
      level: flags.level ?? null,
      setVersion: flags.set ?? null,
      build: buildNum,
      stage: flags.stage ?? null,
      date: flags.date ?? todayUtc(),
    });
    const label = res.build !== null ? `${res.version}+${res.build}` : res.version;
    console.log(`bumped ${res.component} to ${label}`);
    return 0;
  }

  if (cmd === "project" && pos.length === 1) {
    ensureAllowedFlags("project", flags, ["format"]);
    const { data, errors } = readManifest(root);
    if (errors.length) throw new Error(errors.join("\n"));
    const entry = data.components[pos[0]];
    if (!entry) throw new Error(`unknown component "${pos[0]}"`);
    const proj = projectVersion(entry.version, entry.build ?? null);
    const fmt = flags.format ?? "all";
    if (fmt === "all") {
      console.log(JSON.stringify(proj, null, 2));
    } else if (fmt === "apple") {
      if (!proj.apple) throw new Error(`component "${pos[0]}" has no build number for apple projection`);
      console.log(JSON.stringify(proj.apple, null, 2));
    } else if (Object.hasOwn(proj, fmt) && proj[fmt] !== null) {
      console.log(String(proj[fmt]));
    } else {
      throw new Error(`unknown or unavailable projection format "${fmt}" for component "${pos[0]}"`);
    }
    return 0;
  }

  console.error(
    "usage:\n" +
      "  version.mjs check [--tag <tag>] [--rev <rev>] [--main-ref <ref>]\n" +
      "  version.mjs plan [<component>] [--json]\n" +
      "  version.mjs bump <component> [--level <lvl> | --set <ver>] [--build <n>] [--stage pre-1.0|stable] [--date YYYY-MM-DD]\n" +
      "  version.mjs project <component> [--format all|semver|android|deb|rpm|arch|appstream|msix|apple]",
  );
  return 2;
}

const isScript =
  import.meta.main ??
  (process.argv[1] !== undefined && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url)));
if (isScript) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (e) {
    console.error(e.message);
    process.exitCode = 1;
  }
}
