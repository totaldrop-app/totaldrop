<!-- Translated from README.md 529368877e8d. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Bármilyen fájlt, bárkinek, bárhová.

Ingyenes és nyílt forráskódú. Végpontok közötti titkosítás. Nincs e-mail-cím, nincs telefonszám, nincs fiók.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Weboldal](https://totaldrop.app)** · **[Letöltés](https://totaldrop.app/download/)** · [Ütemterv](https://totaldrop.app/roadmap/) · [Adományozás](https://totaldrop.app/donate/) · [Kiadások](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · **Magyar** · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="A TotalDrop Androidon: a kezdőképernyő világos és sötét témában, valamint egy megérkezett és ellenőrzött átvitel" width="820">

</div>

> [!NOTE]
> Ez a fordítás vázlat, ellenőrzése folyamatban van. A mérvadó változat az [angol](../../README.md).

## Miért a TotalDrop

- **Csak egy rövid kód kell.** Válaszd ki a fájlokat, és kapsz egy kódot, például `7-harbor-rocket`. Mondd ki hangosan, vagy mutasd meg a QR-kódot; a másik fél beírja, és a fájlok megérkeznek.
- **Végpontok közötti titkosítás, amelyet ellenőrizni is tudsz.** Mielőtt bármi elindulna, mindkét képernyő ugyanazt a négy szót mutatja. Ha eltérnek, állj meg.
- **Nincs hová regisztrálni.** Nincs fiók, e-mail-cím vagy telefonszám. Az opcionális fiókok csak számok lesznek, Mullvad-stílusban ([terv](../../docs/hesap.md)).
- **Közvetlenül eszközről eszközre, bárhol.** A találkozási szerver csak bemutatja egymásnak a két eszközt, és nem naplóz; mint bármely szerver, látja az IP-címüket. Az eszközök az interneten keresztül mindenhol közvetlenül kapcsolódnak, ahol csak lehet. Ha router vagy tűzfal áll az útban, a forgalmat egy továbbító szerver viszi, amely nem tudja elolvasni.
- **Otthon érzi magát a rendszereden.** Az alkalmazások követik a rendszer világos vagy sötét témáját, kiemelőszínét, nyelvét és a csökkentett mozgás beállítását. A linuxos alkalmazás natív Qt 6 Waylandön, és a betűtípusokat, a sarkokat és az ikonokat is élőben veszi át a KDE Plasma, a Hyprland és az Omarchy témáiból.
- **Ingyenes, végleg.** Nyílt forráskódú, adományokból fenntartva. Nincs reklám, nincs követés, nincs fizetős csomag az alapfunkciókhoz.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="A linuxos alkalmazás Omarchyn, az asztali környezet színeivel, betűtípusaival és ikonjaival" width="560">
  <br><sub>A linuxos alkalmazás Omarchyn: a színek, a sarkok, a betűtípusok és az ikonok az asztali környezetből jönnek.</sub>
</p>

## Letöltés

| Platform | Honnan | Megjegyzések |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1, Qt 6.8 vagy újabb |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1, Qt 6.8 vagy újabb |
| **Android** | [Csatlakozz a bétához](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1, Android 15 vagy újabb, Google Play meghívással |
| **Parancssor** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Úton van | Ezek kódaláírását fedezik elsőként az adományok |

Minden csomagot a CI készít egy címkézett commitból, soha nem egy fejlesztő gépén. Ha telepítés előtt ellenőriznél egy csomagot, hasonlítsd össze a kiadás `SHA256SUMS` fájljával, és ellenőrizd, hol készült:

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Béta.** A kód még nem esett át független biztonsági átvizsgáláson, ezért kérjük, érzékeny fájlokhoz ne hagyatkozz rá.

## Hogyan működik

1. A küldő rövid kódot kér a találkozási szervertől. A kód az egyetlen titok, amelyen a két ember osztozik.
2. Mindkét eszköz erős kulccsá alakítja a kódot a **CPace** segítségével. Egy támadónak átvitelenként egyetlen tippje van, és nem tud kódokat offline kipróbálni.
3. Az eszközök közvetlenül kapcsolódnak **QUIC**-en keresztül (TLS 1.3 rögzített nyers nyilvános kulcsokkal), és **Noise XXpsk3-at futtatnak ML-KEM-768-cal**, amelyet úgy terveztek, hogy a jövő kvantumszámítógépeivel szemben is ellenálljon.
4. A fájlok darabokban titkosítódnak (STREAM), így minden darab hitelesítése megtörténik, amint megérkezik.

A protokoll leírása a *TotalDrop Protocol v0* dokumentumban található. A megvalósítás közben hozott döntések a [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) fájlban vannak, azt pedig, hogy melyik szerver mit lát és mit nem, a [fenyegetésmodell](../../docs/tehdit-modeli.md) (törökül) és az [adatvédelmi irányelvek](https://totaldrop.app/privacy) írják le.

## A te nyelveden

A TotalDrop 35 nyelven beszél, és a régiód számformátumait követi. Az angol és a török ellenőrzött; a többi 33 vázlat, amelyet anyanyelvi beszélők ellenőriznek, ezért az alkalmazások a biztonsági szövegeket (ellenőrzés, hibás kód, elfogadás és elutasítás) angolul hagyják, amíg egy nyelv ellenőrzése meg nem történik. A weboldalon és az alkalmazásokban mind megvan, az Android és Linux 0.3.0-beta.1 verziótól kezdve.

Esetlen mondatot találtál? [Segíts a fordításban](../../docs/ceviri.md) (az útmutató törökül van).

## Támogasd a projektet

A TotalDrop azért marad ingyenes, mert az emberek beszállnak. Az adományok fedezik a továbbító szervert, a Windows és a macOS által megkövetelt kódaláíró tanúsítványokat, valamint egy független biztonsági auditot az 1.0 előtt. Minden költség tételesen, és az, hogy eddig mennyi gyűlt össze: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Támogatás a GitHubon"></a>

Segíthetsz úgy is, hogy [hibákat jelentesz](https://github.com/totaldrop-app/totaldrop/issues/new/choose), fordítasz, vagy tesztelsz a saját asztali gépeden.

## Fejlesztőknek

### A tároló felépítése

| Útvonal | Mi ez | Licenc |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, kulcsszármaztatás, postafiók- és munkamenet-titkosítás, eszközkulcsok, STREAM darabtitkosítás | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Átviteli kódok, korlátok, útvonalbiztonság, minden hálózati üzenet (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Kliensmotor: találkozás a szerveren, QUIC nyers nyilvános kulcsokkal, Noise XXpsk3 + ML-KEM-768, átvitel | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Felületi szövegek a Rust-programokhoz: nyelvválasztás, ICU-üzenetek, többes számú alakok, regionális számformátumok | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | A `totaldrop` parancssori kliens | GPL-3.0-or-later |
| `l10n/` | Minden felületi szöveg, amelyen az Android, a Linux és a CLI osztozik ([fordítás](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linuxos asztali alkalmazás: Qt 6 és QML a `totaldrop-core` fölött, natív Wayland | GPL-3.0-or-later |
| `packaging/` | Linuxos csomagok, minden formátum külön: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android-alkalmazás (Flutter); iOS később | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app és az oldal, amelyet a QR-kódok linkjei megnyitnak | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz-célpontok minden elemzőhöz | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Találkozási szerver Rustban, saját üzemeltetéshez és tesztekhez | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Ugyanez a szerver Cloudflare Workersen, a `rendezvous.totaldrop.app` számára | AGPL-3.0-or-later |

A könyvtárak megengedő licencűek, hogy bárki építhessen a protokollra; az alkalmazások GPL, a szerverek AGPL licencűek. Lásd: [LICENSING.md](../../LICENSING.md).

Minden platformnak saját verziója, címkéje (`linux-v…`, `android-v…`) és CI-munkafolyamata van, így az egyik kiadható a többi érintése nélkül. Hogyan működnek az ágak és a kiadások: [docs/gelistirme.md](../../docs/gelistirme.md) (törökül) és [docs/versioning.md](../../docs/versioning.md). Tervezési szabályok: [docs/tasarim.md](../../docs/tasarim.md) és architekturális döntések: [docs/kararlar](../../docs/kararlar/README.md), mindkettő törökül.

### Összeállítás

```bash
cargo test
cargo build --release
```

Próbálj ki egy átvitelt egyetlen gépen, három terminálban:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Mindkét oldal ugyanazt a négy ellenőrző szót mutatja. A titkosítatlan `ws://` csak ugyanazon a gépen futó szerver esetén elfogadott; minden másnak `wss://`-nek kell lennie.

A projekt mindig a legújabb stabil Rustot célozza (a `rust-toolchain.toml` rögzíti); a rustup az első fordításkor telepíti.

**Linuxos alkalmazás** (Qt 6.8 vagy újabb kell hozzá: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Közreműködés és biztonság

Pull request nyitása előtt olvasd el a [CONTRIBUTING.md](../../CONTRIBUTING.md) fájlt és a [magatartási kódexünket](../../CODE_OF_CONDUCT.md). Kérdésekkel lásd: [SUPPORT.md](../../SUPPORT.md). A sebezhetőségeket kérjük, privát módon jelentsd, a [SECURITY.md](../../SECURITY.md) leírása szerint, soha ne nyilvános issue-ban.
