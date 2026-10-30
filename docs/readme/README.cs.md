<!-- Translated from README.md 529368877e8d. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Jakýkoli soubor, komukoli, kamkoli.

Zdarma a s otevřeným kódem. Koncové šifrování. Žádný e-mail, žádné telefonní číslo, žádný účet.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Web](https://totaldrop.app)** · **[Stáhnout](https://totaldrop.app/download/)** · [Plán vývoje](https://totaldrop.app/roadmap/) · [Přispět](https://totaldrop.app/donate/) · [Vydání](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · **Čeština** · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop na Androidu: domovská obrazovka ve světlém a tmavém režimu a přenos, který dorazil a byl ověřen" width="820">

</div>

> [!NOTE]
> Tento překlad je koncept a prochází kontrolou. Závazná je [anglická verze](../../README.md).

## Proč TotalDrop

- **Stačí krátký kód.** Vyberte soubory a získáte kód jako `7-harbor-rocket`. Přečtěte ho nahlas nebo ukažte QR kód; druhá strana ho zadá a soubory dorazí.
- **Koncově šifrované, a můžete si to ověřit.** Než se cokoli přenese, obě obrazovky ukážou stejná čtyři slova. Pokud se liší, přestaňte.
- **Nikde se neregistrujete.** Žádný účet, e-mail ani telefonní číslo. Volitelné účty budou jen čísla, ve stylu Mullvad ([návrh](../../docs/hesap.md)).
- **Přímo ze zařízení do zařízení, kdekoli.** Server pro navázání spojení obě zařízení jen seznámí a nevede žádné záznamy; jako každý server vidí jejich IP adresy. Zařízení se přes internet spojí přímo všude, kde to jde. Když překáží router nebo firewall, provoz přenáší zprostředkující server, který ho nemůže přečíst.
- **Jako doma ve vašem systému.** Aplikace se řídí světlým nebo tmavým motivem systému, barvou zvýraznění, jazykem a nastavením omezení pohybu. Linuxová aplikace je nativní Qt 6 na Waylandu a navíc živě přebírá písma, rohy a ikony z motivů KDE Plasma, Hyprland a Omarchy.
- **Zdarma, natrvalo.** Otevřený kód, financovaný z darů. Žádné reklamy, žádné sledování, žádná placená verze pro základní funkce.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Linuxová aplikace v Omarchy, v barvách, písmech a ikonách prostředí" width="560">
  <br><sub>Linuxová aplikace v Omarchy: barvy, rohy, písma a ikony přebírá z prostředí.</sub>
</p>

## Stáhnout

| Platforma | Kde získat | Poznámky |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1, Qt 6.8 nebo novější |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1, Qt 6.8 nebo novější |
| **Android** | [Připojit se k betě](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1, Android 15 nebo novější, Google Play na pozvání |
| **Příkazový řádek** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Připravujeme | Podepisování kódu pro tyto systémy je první, co dary zaplatí |

Každý balíček sestavuje CI z otagovaného commitu, nikdy na počítači vývojáře. Chcete-li balíček před instalací zkontrolovat, porovnejte ho se `SHA256SUMS` daného vydání a ověřte, kde byl sestaven:

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Kód zatím neprošel nezávislou bezpečnostní kontrolou, proto se na něj prosím u citlivých souborů nespoléhejte.

## Jak to funguje

1. Odesílatel si vyžádá krátký kód od serveru pro navázání spojení. Kód je jediné tajemství, které obě osoby sdílejí.
2. Obě zařízení promění kód v silný klíč pomocí **CPace**. Útočník má na každý přenos jeden pokus a nemůže kódy zkoušet offline.
3. Zařízení se spojí přímo přes **QUIC** (TLS 1.3 s připnutými surovými veřejnými klíči) a provedou **Noise XXpsk3 s ML-KEM-768**, který je navržen tak, aby odolal i budoucím kvantovým počítačům.
4. Soubory se šifrují po částech (STREAM), takže každá část je ověřena hned, jak dorazí.

Protokol popisuje dokument *TotalDrop Protocol v0*. Rozhodnutí učiněná při jeho implementaci jsou v [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) a co který server může a nemůže vidět, popisují [model hrozeb](../../docs/tehdit-modeli.md) (v turečtině) a [zásady ochrany soukromí](https://totaldrop.app/privacy).

## Ve vašem jazyce

TotalDrop mluví 35 jazyky a řídí se formáty čísel vašeho regionu. Angličtina a turečtina jsou zkontrolované; zbývajících 33 jsou koncepty, které kontrolují rodilí mluvčí, a proto aplikace ponechávají bezpečnostní texty (ověření, špatný kód, přijmout a odmítnout) v angličtině, dokud jazyk není zkontrolován. Web i aplikace je mají všechny, od verze 0.3.0-beta.1 pro Android a Linux.

Narazili jste na neobratnou větu? [Pomozte s překladem](../../docs/ceviri.md) (průvodce je v turečtině).

## Podpořte projekt

TotalDrop zůstává zdarma, protože lidé přispívají. Dary platí zprostředkující server, certifikáty pro podepisování kódu, které vyžadují Windows a macOS, a nezávislý bezpečnostní audit před verzí 1.0. Všechny náklady položku po položce a kolik se zatím vybralo: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Podpořit na GitHubu"></a>

Pomoci můžete také [hlášením chyb](https://github.com/totaldrop-app/totaldrop/issues/new/choose), překládáním nebo testováním na svém počítači.

## Pro vývojáře

### Struktura repozitáře

| Cesta | Co to je | Licence |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, odvozování klíčů, šifrování schránky a relace, klíče zařízení, šifrování částí STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Kódy přenosu, limity, bezpečnost cest, všechny síťové zprávy (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Klientské jádro: navázání spojení, QUIC se surovými veřejnými klíči, Noise XXpsk3 + ML-KEM-768, přenos | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Texty rozhraní pro programy v Rustu: výběr jazyka, zprávy ICU, množná čísla, regionální formáty čísel | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Klient příkazového řádku `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Všechny texty rozhraní, společné pro Android, Linux a CLI ([překlad](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linuxová desktopová aplikace: Qt 6 a QML nad `totaldrop-core`, nativní Wayland | GPL-3.0-or-later |
| `packaging/` | Balíčky pro Linux, každý formát zvlášť: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Aplikace pro Android (Flutter); iOS později | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app a stránka, kterou otevírají odkazy z QR kódů | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz cíle pro každý parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Server pro navázání spojení v Rustu, pro vlastní hosting a testy | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Tentýž server na Cloudflare Workers, pro `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Knihovny mají permisivní licence, aby na protokolu mohl stavět kdokoli; aplikace jsou pod GPL a servery pod AGPL. Viz [LICENSING.md](../../LICENSING.md).

Každá platforma má vlastní verzi, tag (`linux-v…`, `android-v…`) a CI workflow, takže jednu lze vydat bez zásahu do ostatních. Jak fungují větve a vydání: [docs/gelistirme.md](../../docs/gelistirme.md) (v turečtině) a [docs/versioning.md](../../docs/versioning.md). Pravidla návrhu: [docs/tasarim.md](../../docs/tasarim.md) a architektonická rozhodnutí: [docs/kararlar](../../docs/kararlar/README.md), obojí v turečtině.

### Sestavení

```bash
cargo test
cargo build --release
```

Vyzkoušejte přenos na jednom počítači ve třech terminálech:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Obě strany ukážou stejná čtyři ověřovací slova. Nešifrované `ws://` se přijímá jen pro server na stejném počítači; vše ostatní musí být `wss://`.

Projekt vždy cílí na nejnovější stabilní Rust (připnutý v `rust-toolchain.toml`); rustup ho nainstaluje při prvním sestavení.

**Linuxová aplikace** (vyžaduje Qt 6.8 nebo novější: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Přispívání a bezpečnost

Než otevřete pull request, přečtěte si [CONTRIBUTING.md](../../CONTRIBUTING.md) a náš [kodex chování](../../CODE_OF_CONDUCT.md). Otázky patří do [SUPPORT.md](../../SUPPORT.md). Zranitelnosti prosím hlaste soukromě, jak popisuje [SECURITY.md](../../SECURITY.md), nikdy ve veřejném issue.
