<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Enhver fil, til hvem som helst, hvor som helst.

Gratis og open source. End-to-end-krypteret. Ingen e-mail, intet telefonnummer, ingen konto.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Websted](https://totaldrop.app)** · **[Download](https://totaldrop.app/download/)** · [Køreplan](https://totaldrop.app/roadmap/) · [Donér](https://totaldrop.app/donate/) · [Udgivelser](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · **Dansk** · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop på Android: startskærmen i lyst og mørkt tema, og en overførsel, der er ankommet og bekræftet" width="820">

</div>

> [!NOTE]
> Denne oversættelse er et udkast og er under gennemsyn. [English](../../README.md) er referenceversionen.

## Hvorfor TotalDrop

- **En kort kode er alt, der skal til.** Vælg filer, og få en kode som `7-harbor-rocket`. Læs den højt, eller vis QR-koden; den anden person taster den ind, og filerne ankommer.
- **Krypteret end-to-end, og du kan tjekke det.** Begge skærme viser de samme fire ord, før noget overføres. Hvis de er forskellige, så stop.
- **Intet at tilmelde sig.** Ingen konto, e-mail eller telefonnummer. Valgfri konti vil kun være numre, som hos Mullvad ([design](../../docs/hesap.md)).
- **Direkte fra enhed til enhed, hvor som helst.** Mødeserveren præsenterer kun de to enheder for hinanden og gemmer ingen logs. Enhederne forbinder direkte over internettet, hvor det er muligt. Når en router eller firewall står i vejen, bærer en relæserver trafikken uden at kunne læse den.
- **Hjemme på alle systemer.** Hver app følger systemets tema, skrifttyper, accentfarve, sprog og indstillinger for bevægelse. Linux-appen er native Qt 6 på Wayland og følger temaer i KDE Plasma, Hyprland og Omarchy med det samme.
- **Gratis, for altid.** Open source og finansieret af donationer. Ingen reklamer, ingen sporing, intet betalt niveau for det grundlæggende.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Linux-appen på Omarchy, i skrivebordets farver, skrifttyper og ikoner" width="560">
  <br><sub>Linux-appen på Omarchy: farver, hjørner, skrifttyper og ikoner kommer fra skrivebordet.</sub>
</p>

## Download

| Platform | Hent den | Bemærkninger |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 eller nyere |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 eller nyere |
| **Android** | [Deltag i betaen](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 eller nyere, Google Play efter invitation |
| **Kommandolinje** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | På vej | Kodesignering til disse er det, donationerne betaler for først |

Hver pakke bygges af CI fra et tagget commit, aldrig på en udviklers maskine. Hvis du vil tjekke en pakke, før du installerer den, så sammenlign den med udgivelsens `SHA256SUMS`, og bekræft, hvor den blev bygget:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Koden har endnu ikke været gennem en uafhængig sikkerhedsrevision, så stol ikke på den til følsomme filer.

## Sådan virker det

1. Afsenderen beder mødeserveren om en kort kode. Koden er den eneste hemmelighed, de to personer deler.
2. Begge enheder gør koden til en stærk nøgle med **CPace**. En angriber får ét gæt pr. overførsel og kan ikke teste koder offline.
3. De forbinder direkte over **QUIC** (TLS 1.3 med fastlåste rå offentlige nøgler) og kører **Noise XXpsk3 med ML-KEM-768**, som forbliver sikkert mod fremtidige kvantecomputere.
4. Filerne krypteres i blokke (STREAM), så hver del godkendes, efterhånden som den ankommer.

Protokollen er beskrevet i *TotalDrop Protocol v0*. Beslutninger, der blev truffet under implementeringen, står i [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), og hvad hver server kan og ikke kan se, står i [trusselsmodellen](../../docs/tehdit-modeli.md) og [privatlivspolitikken](https://totaldrop.app/privacy).

## På dit sprog

TotalDrop taler 35 sprog og følger din regions talformater. Engelsk og tyrkisk er gennemgået; de øvrige 33 er udkast, som modersmålstalere er ved at tjekke, så apps beholder sikkerhedsteksterne (bekræftelse, forkert kode, accepter og afvis) på engelsk, indtil et sprog er gennemgået. Webstedet har dem alle allerede i dag; apps får dem med deres næste udgivelser.

Har du set en klodset sætning? [Hjælp med at oversætte](../../docs/ceviri.md).

## Støt projektet

TotalDrop forbliver gratis, fordi folk giver et bidrag. Donationerne betaler for relæserveren, de kodesigneringscertifikater, som Windows og macOS kræver, og en uafhængig sikkerhedsrevision før 1.0. Hver udgift, linje for linje, og hvad der er indsamlet indtil nu: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Sponsorér på GitHub"></a>

Du kan også hjælpe ved at [rapportere fejl](https://github.com/totaldrop-app/totaldrop/issues/new/choose), oversætte eller teste på din computer.

## For udviklere

### Repositoriets opbygning

| Sti | Hvad det er | Licens |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, nøgleafledning, kryptering af postkasse og session, enhedsnøgler, blokkryptering med STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Overførselskoder, grænser, sikre stier, alle protokolbeskeder (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Klientmotor: kontakt med mødeserveren, QUIC med rå offentlige nøgler, Noise XXpsk3 + ML-KEM-768, overførsel | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Brugerfladetekst til Rust-programmerne: sprogforhandling, ICU-beskeder, flertalsformer, regionale talformater | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Kommandolinjeklienten `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Alle brugerfladestrenge, delt af Android, Linux og CLI ([oversættelse](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux-skrivebordsapp: Qt 6 og QML oven på `totaldrop-core`, native Wayland | GPL-3.0-or-later |
| `packaging/` | Linux-pakker, hvert format for sig: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android-app (Flutter); iOS senere | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app og siden, som QR-links åbner | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz-mål for hver parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Mødeserver i Rust, til selvhosting og test | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Den samme server på Cloudflare Workers, til `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Bibliotekerne har lempelige licenser, så alle kan bygge videre på protokollen; apps er GPL og serverne AGPL. Se [LICENSING.md](../../LICENSING.md).

Hver platform har sin egen version, sit eget tag (`linux-v…`, `android-v…`) og sit eget CI-workflow, så én kan udgives uden at røre de andre. Sådan fungerer branches og udgivelser: [docs/gelistirme.md](../../docs/gelistirme.md) og [docs/versioning.md](../../docs/versioning.md). Designregler: [docs/tasarim.md](../../docs/tasarim.md). Arkitekturbeslutninger: [docs/kararlar](../../docs/kararlar/README.md).

### Byg

```bash
cargo test
cargo build --release
```

Prøv en overførsel på én maskine, i tre terminaler:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Begge sider viser de samme fire bekræftelsesord. Ukrypteret `ws://` accepteres kun for en server på samme maskine; alt andet skal være `wss://`.

Projektet sigter altid mod den nyeste stabile Rust (fastlåst i `rust-toolchain.toml`); rustup installerer den ved første build.

**Linux-appen** (kræver Qt 6.8 eller nyere: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Bidrag og sikkerhed

Læs [CONTRIBUTING.md](../../CONTRIBUTING.md), før du åbner en pull request, og vores [adfærdskodeks](../../CODE_OF_CONDUCT.md). Spørgsmål hører til i [SUPPORT.md](../../SUPPORT.md). Rapportér sårbarheder privat som beskrevet i [SECURITY.md](../../SECURITY.md), aldrig i et offentligt issue.
