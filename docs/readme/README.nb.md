<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Hvilken som helst fil, til hvem som helst, hvor som helst.

Gratis og åpen kildekode. Ende-til-ende-kryptert. Ingen e-post, intet telefonnummer, ingen konto.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Nettsted](https://totaldrop.app)** · **[Last ned](https://totaldrop.app/download/)** · [Veikart](https://totaldrop.app/roadmap/) · [Doner](https://totaldrop.app/donate/) · [Utgivelser](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · **Norsk bokmål** · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop på Android: startskjermen i lyst og mørkt tema, og en overføring som har kommet fram og er bekreftet" width="820">

</div>

> [!NOTE]
> Denne oversettelsen er et utkast og blir gjennomgått. [English](../../README.md) er referanseversjonen.

## Hvorfor TotalDrop

- **En kort kode er alt som trengs.** Velg filer og få en kode som `7-harbor-rocket`. Les den høyt eller vis QR-koden; den andre personen skriver den inn, og filene kommer fram.
- **Kryptert ende-til-ende, og du kan sjekke det.** Begge skjermene viser de samme fire ordene før noe overføres. Hvis de er ulike, stopp.
- **Ingenting å registrere seg for.** Ingen konto, e-post eller telefonnummer. Valgfrie kontoer vil bare være numre, slik som hos Mullvad ([utforming](../../docs/hesap.md)).
- **Rett fra enhet til enhet, hvor som helst.** Møteserveren bare introduserer de to enhetene for hverandre og lagrer ingen logger. Enhetene kobler direkte over internett der det er mulig. Når en ruter eller brannmur står i veien, bærer en reléserver trafikken uten å kunne lese den.
- **Hjemme på alle systemer.** Hver app følger systemets tema, skrifttyper, aksentfarge, språk og innstillinger for bevegelse. Linux-appen er native Qt 6 på Wayland og følger temaer i KDE Plasma, Hyprland og Omarchy fortløpende.
- **Gratis, for godt.** Åpen kildekode, finansiert av donasjoner. Ingen reklame, ingen sporing, ikke noe betalt nivå for det grunnleggende.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Linux-appen på Omarchy, i skrivebordets farger, skrifttyper og ikoner" width="560">
  <br><sub>Linux-appen på Omarchy: farger, hjørner, skrifttyper og ikoner kommer fra skrivebordet.</sub>
</p>

## Last ned

| Plattform | Hent den | Merknader |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 eller nyere |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 eller nyere |
| **Android** | [Bli med i betaen](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 eller nyere, Google Play etter invitasjon |
| **Kommandolinje** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Kommer | Kodesignering for disse er det donasjonene betaler for først |

Hver pakke bygges av CI fra en tagget commit, aldri på en utviklers maskin. Hvis du vil sjekke en pakke før du installerer den, sammenlign den med utgivelsens `SHA256SUMS` og bekreft hvor den ble bygget:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Koden har ennå ikke vært gjennom en uavhengig sikkerhetsrevisjon, så ikke stol på den for sensitive filer.

## Slik fungerer det

1. Avsenderen ber møteserveren om en kort kode. Koden er den eneste hemmeligheten de to personene deler.
2. Begge enhetene gjør koden om til en sterk nøkkel med **CPace**. En angriper får én gjetning per overføring og kan ikke teste koder offline.
3. De kobler direkte over **QUIC** (TLS 1.3 med festede rå offentlige nøkler) og kjører **Noise XXpsk3 med ML-KEM-768**, som forblir sikkert mot fremtidige kvantedatamaskiner.
4. Filene krypteres i blokker (STREAM), så hver del autentiseres idet den kommer fram.

Protokollen er beskrevet i *TotalDrop Protocol v0*. Beslutninger som ble tatt under implementeringen, står i [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), og hva hver server kan og ikke kan se, står i [trusselmodellen](../../docs/tehdit-modeli.md) og [personvernerklæringen](https://totaldrop.app/privacy).

## På ditt språk

TotalDrop snakker 35 språk og følger tallformatene i regionen din. Engelsk og tyrkisk er gjennomgått; de andre 33 er utkast som morsmålsbrukere sjekker, så appene beholder sikkerhetstekstene (bekreftelse, feil kode, godta og avslå) på engelsk til et språk er gjennomgått. Nettstedet har alle allerede i dag; appene får dem med de neste utgivelsene sine.

Har du sett en klønete setning? [Hjelp til med å oversette](../../docs/ceviri.md).

## Støtt prosjektet

TotalDrop forblir gratis fordi folk bidrar. Donasjonene betaler for reléserveren, kodesigneringssertifikatene som Windows og macOS krever, og en uavhengig sikkerhetsrevisjon før 1.0. Hver kostnad, linje for linje, og hvor mye som er samlet inn så langt: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Spons på GitHub"></a>

Du kan også hjelpe ved å [rapportere feil](https://github.com/totaldrop-app/totaldrop/issues/new/choose), oversette eller teste på datamaskinen din.

## For utviklere

### Oppbygging av repositoriet

| Sti | Hva det er | Lisens |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, nøkkelavledning, kryptering av postkasse og økt, enhetsnøkler, blokkryptering med STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Overføringskoder, grenser, trygge stier, alle protokollmeldinger (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Klientmotor: kontakt med møteserveren, QUIC med rå offentlige nøkler, Noise XXpsk3 + ML-KEM-768, overføring | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Grensesnittekst for Rust-programmene: språkforhandling, ICU-meldinger, flertallsformer, regionale tallformater | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Kommandolinjeklienten `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Alle grensesnittstrenger, delt av Android, Linux og CLI ([oversettelse](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux-skrivebordsapp: Qt 6 og QML oppå `totaldrop-core`, native Wayland | GPL-3.0-or-later |
| `packaging/` | Linux-pakker, hvert format for seg: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android-app (Flutter); iOS senere | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app og siden som QR-lenker åpner | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz-mål for hver parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Møteserver i Rust, for egen drift og tester | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Den samme serveren på Cloudflare Workers, for `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Bibliotekene har tillatende lisenser slik at alle kan bygge videre på protokollen; appene er GPL og serverne AGPL. Se [LICENSING.md](../../LICENSING.md).

Hver plattform har sin egen versjon, sin egen tag (`linux-v…`, `android-v…`) og sin egen CI-arbeidsflyt, så én kan lanseres uten å røre de andre. Slik fungerer brancher og utgivelser: [docs/gelistirme.md](../../docs/gelistirme.md) og [docs/versioning.md](../../docs/versioning.md). Designregler: [docs/tasarim.md](../../docs/tasarim.md). Arkitekturbeslutninger: [docs/kararlar](../../docs/kararlar/README.md).

### Bygg

```bash
cargo test
cargo build --release
```

Prøv en overføring på én maskin, i tre terminaler:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Begge sider viser de samme fire bekreftelsesordene. Ukryptert `ws://` godtas bare for en server på samme maskin; alt annet må være `wss://`.

Prosjektet sikter alltid mot den nyeste stabile Rust-versjonen (festet i `rust-toolchain.toml`); rustup installerer den ved første bygg.

**Linux-appen** (krever Qt 6.8 eller nyere: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Bidrag og sikkerhet

Les [CONTRIBUTING.md](../../CONTRIBUTING.md) før du åpner en pull request, og våre [atferdsregler](../../CODE_OF_CONDUCT.md). Spørsmål hører hjemme i [SUPPORT.md](../../SUPPORT.md). Rapporter sårbarheter privat slik det er beskrevet i [SECURITY.md](../../SECURITY.md), aldri i en offentlig sak.
