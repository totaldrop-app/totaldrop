<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Elk bestand, naar iedereen, overal.

Gratis en open source. End-to-end versleuteld. Geen e-mail, geen telefoonnummer, geen account.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Website](https://totaldrop.app)** · **[Downloaden](https://totaldrop.app/download/)** · [Roadmap](https://totaldrop.app/roadmap/) · [Doneren](https://totaldrop.app/donate/) · [Releases](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · **Nederlands** · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop op Android: het startscherm in licht en donker, en een overdracht die is aangekomen en geverifieerd" width="820">

</div>

> [!NOTE]
> Deze vertaling is een concept en wordt nagekeken. De [Engelse versie](../../README.md) is leidend.

## Waarom TotalDrop

- **Een korte code is genoeg.** Kies bestanden en je krijgt een code zoals `7-harbor-rocket`. Lees hem voor of laat de QR-code zien; de ander typt hem in en de bestanden komen aan.
- **End-to-end versleuteld, en je kunt het controleren.** Beide schermen tonen dezelfde vier woorden voordat er iets wordt verstuurd. Verschillen ze, stop dan.
- **Nergens voor aanmelden.** Geen account, e-mail of telefoonnummer. Optionele accounts worden alleen een nummer, in de stijl van Mullvad ([ontwerp](../../docs/hesap.md)).
- **Rechtstreeks van apparaat naar apparaat, overal.** De ontmoetingsserver stelt de twee apparaten alleen aan elkaar voor en houdt geen logboeken bij. Apparaten verbinden waar mogelijk rechtstreeks via het internet. Als een router of firewall in de weg zit, vervoert een relayserver het verkeer zonder het te kunnen lezen.
- **Thuis op elk systeem.** Elke app volgt het thema, de lettertypen, de accentkleur, de taal en de bewegingsinstellingen van zijn systeem. De Linux-app is native Qt 6 op Wayland en volgt live de thema's van KDE Plasma, Hyprland en Omarchy.
- **Gratis, voorgoed.** Open source en betaald met donaties. Geen advertenties, geen tracking, geen betaald abonnement voor de basis.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="De Linux-app op Omarchy, in de kleuren, lettertypen en pictogrammen van het bureaublad" width="560">
  <br><sub>De Linux-app op Omarchy: kleuren, hoeken, lettertypen en pictogrammen komen van het bureaublad.</sub>
</p>

## Downloaden

| Platform | Downloaden | Opmerkingen |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 of nieuwer |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 of nieuwer |
| **Android** | [Doe mee met de bèta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 of nieuwer, Google Play op uitnodiging |
| **Opdrachtregel** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Onderweg | Codeondertekening voor deze platforms is het eerste waar donaties voor betalen |

Elk pakket wordt door de CI gebouwd vanuit een getagde commit, nooit op de computer van een ontwikkelaar. Wil je er een controleren voordat je het installeert, vergelijk het dan met de `SHA256SUMS` van de release en controleer waar het is gebouwd:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Bèta.** De code heeft nog geen onafhankelijke beveiligingscontrole gehad, dus vertrouw er niet op voor gevoelige bestanden.

## Hoe het werkt

1. De verzender vraagt de ontmoetingsserver om een korte code. De code is het enige geheim dat de twee mensen delen.
2. Beide apparaten maken van de code een sterke sleutel met **CPace**. Een aanvaller krijgt één poging per overdracht en kan codes niet offline testen.
3. Ze verbinden rechtstreeks via **QUIC** (TLS 1.3 met vastgepinde ruwe publieke sleutels) en voeren **Noise XXpsk3 met ML-KEM-768** uit, dat ook veilig blijft tegen toekomstige kwantumcomputers.
4. Bestanden worden in blokken versleuteld (STREAM), zodat elk stuk bij aankomst wordt geauthenticeerd.

Het protocol is beschreven in *TotalDrop Protocol v0*. Beslissingen die bij de implementatie zijn genomen staan in [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), en wat elke server wel en niet kan zien staat in het [dreigingsmodel](../../docs/tehdit-modeli.md) en het [privacybeleid](https://totaldrop.app/privacy).

## In jouw taal

TotalDrop spreekt 35 talen en volgt de getalnotatie van jouw regio. Engels en Turks zijn nagekeken; de andere 33 zijn concepten die moedertaalsprekers nu controleren. Daarom houden de apps de beveiligingsteksten (verificatie, verkeerde code, accepteren en weigeren) in het Engels tot een taal is nagekeken. De site heeft ze vandaag al allemaal; de apps krijgen ze bij hun volgende releases.

Een onhandige zin gezien? [Help mee met vertalen](../../docs/ceviri.md).

## Steun het project

TotalDrop blijft gratis omdat mensen bijdragen. Donaties betalen de relayserver, de certificaten voor codeondertekening die Windows en macOS vereisen, en een onafhankelijke beveiligingsaudit vóór 1.0. Alle kosten, post voor post, en wat er tot nu toe is opgehaald: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Sponsor op GitHub"></a>

Je kunt ook helpen door [bugs te melden](https://github.com/totaldrop-app/totaldrop/issues/new/choose), te vertalen of te testen op je desktop.

## Voor ontwikkelaars

### Indeling van de repository

| Pad | Wat het is | Licentie |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, sleutelafleiding, versleuteling van mailbox en sessie, apparaatsleutels, STREAM-blokversleuteling | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Overdrachtscodes, limieten, padveiligheid, elk netwerkbericht (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Client-engine: ontmoeting, QUIC met ruwe publieke sleutels, Noise XXpsk3 + ML-KEM-768, overdracht | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Interfaceteksten voor de Rust-programma's: taalonderhandeling, ICU-berichten, meervouden, regionale getallen | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | De opdrachtregelclient `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Alle interfaceteksten, gedeeld door Android, Linux en de CLI ([vertalen](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux-desktopapp: Qt 6 en QML bovenop `totaldrop-core`, native Wayland | GPL-3.0-or-later |
| `packaging/` | Linux-pakketten, elk formaat apart: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android-app (Flutter); iOS later | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app en de pagina die QR-links openen | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzzdoelen voor elke parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Ontmoetingsserver in Rust, voor zelfhosting en tests | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Dezelfde server op Cloudflare Workers, voor `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

De bibliotheken hebben permissieve licenties, zodat iedereen op het protocol kan voortbouwen; de apps zijn GPL en de servers AGPL. Zie [LICENSING.md](../../LICENSING.md).

Elk platform heeft een eigen versie, tag (`linux-v…`, `android-v…`) en CI-workflow, zodat er één kan uitkomen zonder de andere aan te raken. Hoe branches en releases werken: [docs/gelistirme.md](../../docs/gelistirme.md) en [docs/versioning.md](../../docs/versioning.md). Ontwerpregels: [docs/tasarim.md](../../docs/tasarim.md). Architectuurbeslissingen: [docs/kararlar](../../docs/kararlar/README.md).

### Bouwen

```bash
cargo test
cargo build --release
```

Probeer een overdracht op één computer, in drie terminals:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Beide kanten tonen dezelfde vier verificatiewoorden. Onversleuteld `ws://` wordt alleen geaccepteerd voor een server op dezelfde computer; al het andere moet `wss://` zijn.

Het project gebruikt altijd de nieuwste stabiele Rust (vastgelegd in `rust-toolchain.toml`); rustup installeert die bij de eerste build.

**Linux-app** (vereist Qt 6.8 of nieuwer: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Bijdragen en beveiliging

Lees [CONTRIBUTING.md](../../CONTRIBUTING.md) en onze [gedragscode](../../CODE_OF_CONDUCT.md) voordat je een pull request opent. Vragen horen in [SUPPORT.md](../../SUPPORT.md). Meld kwetsbaarheden vertrouwelijk zoals beschreven in [SECURITY.md](../../SECURITY.md), nooit in een openbare issue.
