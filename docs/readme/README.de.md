<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Jede Datei, an jeden, überall.

Kostenlos und quelloffen. Ende-zu-Ende-verschlüsselt. Keine E-Mail, keine Telefonnummer, kein Konto.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Website](https://totaldrop.app)** · **[Herunterladen](https://totaldrop.app/download/)** · [Roadmap](https://totaldrop.app/roadmap/) · [Spenden](https://totaldrop.app/donate/) · [Releases](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · **Deutsch** · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop auf Android: der Startbildschirm hell und dunkel sowie eine Übertragung, die angekommen ist und geprüft wurde" width="820">

</div>

> [!NOTE]
> Diese Übersetzung ist ein Entwurf und wird geprüft. Maßgeblich ist die [englische Fassung](../../README.md).

## Warum TotalDrop

- **Ein kurzer Code genügt.** Wählen Sie Dateien aus und Sie erhalten einen Code wie `7-harbor-rocket`. Lesen Sie ihn vor oder zeigen Sie den QR-Code; die andere Person gibt ihn ein, und die Dateien kommen an.
- **Ende-zu-Ende-verschlüsselt, und Sie können es prüfen.** Beide Bildschirme zeigen dieselben vier Wörter, bevor etwas übertragen wird. Stimmen sie nicht überein, brechen Sie ab.
- **Keine Anmeldung.** Kein Konto, keine E-Mail, keine Telefonnummer. Optionale Konten werden nur aus einer Nummer bestehen, wie bei Mullvad ([Konzept](../../docs/hesap.md)).
- **Direkt von Gerät zu Gerät, überall.** Der Vermittlungsserver stellt die beiden Geräte nur einander vor und speichert keine Protokolle; wie jeder Server sieht er ihre IP-Adressen. Die Geräte verbinden sich über das Internet direkt, wo immer es geht. Steht ein Router oder eine Firewall im Weg, transportiert ein Relay-Server den Verkehr, ohne ihn lesen zu können.
- **Auf Ihrem System zu Hause.** Die Apps folgen dem hellen oder dunklen Design des Systems, seiner Akzentfarbe, seiner Sprache und der Einstellung für reduzierte Bewegung. Die Linux-App ist natives Qt 6 unter Wayland und übernimmt außerdem live Schriften, Ecken und Symbole aus den Designs von KDE Plasma, Hyprland und Omarchy.
- **Kostenlos, für immer.** Quelloffen und durch Spenden finanziert. Keine Werbung, kein Tracking, kein kostenpflichtiger Tarif für die Grundfunktionen.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Die Linux-App unter Omarchy, in den Farben, Schriften und Symbolen des Desktops" width="560">
  <br><sub>Die Linux-App unter Omarchy: Farben, Ecken, Schriften und Symbole kommen vom Desktop.</sub>
</p>

## Herunterladen

| Plattform | Bezugsquelle | Hinweise |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 oder neuer |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 oder neuer |
| **Android** | [An der Beta teilnehmen](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 oder neuer, Google Play auf Einladung |
| **Kommandozeile** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | In Arbeit | Die Codesignatur für diese Plattformen bezahlen Spenden als Erstes |

Jedes Paket wird von der CI aus einem getaggten Commit gebaut, nie auf dem Rechner eines Entwicklers. Um ein Paket vor der Installation zu prüfen, vergleichen Sie es mit den `SHA256SUMS` des Releases und prüfen Sie, wo es gebaut wurde:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Der Code wurde noch nicht unabhängig auf Sicherheit geprüft. Verlassen Sie sich daher bei sensiblen Dateien bitte nicht darauf.

## So funktioniert es

1. Der Absender bittet den Vermittlungsserver um einen kurzen Code. Der Code ist das einzige Geheimnis, das die beiden Personen teilen.
2. Beide Geräte machen aus dem Code mit **CPace** einen starken Schlüssel. Ein Angreifer hat pro Übertragung einen Versuch und kann Codes nicht offline testen.
3. Sie verbinden sich direkt über **QUIC** (TLS 1.3 mit fest hinterlegten öffentlichen Rohschlüsseln) und führen **Noise XXpsk3 mit ML-KEM-768** aus, das darauf ausgelegt ist, auch künftigen Quantencomputern standzuhalten.
4. Dateien werden in Blöcken verschlüsselt (STREAM), sodass jedes Stück beim Eintreffen authentifiziert wird.

Das Protokoll ist in *TotalDrop Protocol v0* beschrieben. Entscheidungen, die bei der Implementierung getroffen wurden, stehen in [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md). Was jeder Server sehen kann und was nicht, steht im [Bedrohungsmodell](../../docs/tehdit-modeli.md) (auf Türkisch) und in der [Datenschutzerklärung](https://totaldrop.app/privacy).

## In Ihrer Sprache

TotalDrop spricht 35 Sprachen und folgt den Zahlenformaten Ihrer Region. Englisch und Türkisch sind geprüft; die anderen 33 sind Entwürfe, die Muttersprachler gerade prüfen. Deshalb behalten die Apps die Sicherheitstexte (Prüfung, falscher Code, Annehmen und Ablehnen) auf Englisch, bis eine Sprache geprüft ist. Die Website hat schon heute alle Sprachen; die Apps erhalten sie mit ihren nächsten Versionen.

Einen holprigen Satz entdeckt? [Beim Übersetzen helfen](../../docs/ceviri.md) (die Anleitung ist auf Türkisch).

## Das Projekt unterstützen

TotalDrop bleibt kostenlos, weil Menschen etwas beitragen. Spenden bezahlen den Relay-Server, die Codesignaturzertifikate, die Windows und macOS verlangen, und ein unabhängiges Sicherheitsaudit vor 1.0. Alle Kosten, Posten für Posten, und was bisher gesammelt wurde: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Auf GitHub unterstützen"></a>

Sie können auch helfen, indem Sie [Fehler melden](https://github.com/totaldrop-app/totaldrop/issues/new/choose), übersetzen oder auf Ihrem Desktop testen.

## Für Entwickler

### Aufbau des Repositorys

| Pfad | Was es ist | Lizenz |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, Schlüsselableitung, Mailbox- und Sitzungsverschlüsselung, Geräteschlüssel, STREAM-Blockverschlüsselung | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Übertragungscodes, Limits, Pfadsicherheit, jede Netzwerknachricht (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Client-Engine: Vermittlung, QUIC mit öffentlichen Rohschlüsseln, Noise XXpsk3 + ML-KEM-768, Übertragung | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Oberflächentexte für die Rust-Programme: Sprachaushandlung, ICU-Nachrichten, Pluralformen, regionale Zahlenformate | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Der Kommandozeilen-Client `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Alle Oberflächentexte, gemeinsam genutzt von Android, Linux und der CLI ([Übersetzen](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux-Desktop-App: Qt 6 und QML über `totaldrop-core`, natives Wayland | GPL-3.0-or-later |
| `packaging/` | Linux-Pakete, jedes Format für sich: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android-App (Flutter); iOS später | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app und die Seite, die QR-Links öffnen | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz-Ziele für jeden Parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Vermittlungsserver in Rust, zum Selbsthosten und für Tests | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Derselbe Server auf Cloudflare Workers, für `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Die Bibliotheken stehen unter freizügigen Lizenzen, damit jeder auf dem Protokoll aufbauen kann; die Apps stehen unter der GPL, die Server unter der AGPL. Siehe [LICENSING.md](../../LICENSING.md).

Jede Plattform hat ihre eigene Version, ihren eigenen Tag (`linux-v…`, `android-v…`) und ihren eigenen CI-Workflow, sodass eine Plattform veröffentlicht werden kann, ohne die anderen anzufassen. Wie Branches und Releases funktionieren: [docs/gelistirme.md](../../docs/gelistirme.md) (auf Türkisch) und [docs/versioning.md](../../docs/versioning.md). Designregeln: [docs/tasarim.md](../../docs/tasarim.md) und Architekturentscheidungen: [docs/kararlar](../../docs/kararlar/README.md), beides auf Türkisch.

### Bauen

```bash
cargo test
cargo build --release
```

Probieren Sie eine Übertragung auf einem einzigen Rechner aus, in drei Terminals:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Beide Seiten zeigen dieselben vier Prüfwörter. Unverschlüsseltes `ws://` wird nur für einen Server auf demselben Rechner akzeptiert; alles andere muss `wss://` sein.

Das Projekt zielt immer auf die neueste stabile Rust-Version (festgelegt in `rust-toolchain.toml`); rustup installiert sie beim ersten Build.

**Linux-App** (benötigt Qt 6.8 oder neuer: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Mitwirken und Sicherheit

Lesen Sie [CONTRIBUTING.md](../../CONTRIBUTING.md) und unseren [Verhaltenskodex](../../CODE_OF_CONDUCT.md), bevor Sie einen Pull Request öffnen. Fragen gehören in [SUPPORT.md](../../SUPPORT.md). Bitte melden Sie Sicherheitslücken vertraulich, wie in [SECURITY.md](../../SECURITY.md) beschrieben, und niemals in einem öffentlichen Issue.
