<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Mikä tahansa tiedosto, kenelle tahansa, minne tahansa.

Ilmainen ja avointa lähdekoodia. Päästä päähän salattu. Ei sähköpostia, ei puhelinnumeroa, ei tiliä.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Verkkosivusto](https://totaldrop.app)** · **[Lataa](https://totaldrop.app/download/)** · [Tiekartta](https://totaldrop.app/roadmap/) · [Lahjoita](https://totaldrop.app/donate/) · [Julkaisut](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · **Suomi** · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop Androidilla: aloitusnäkymä vaaleana ja tummana sekä perille tullut ja tarkistettu siirto" width="820">

</div>

> [!NOTE]
> Tämä käännös on luonnos, ja sitä tarkistetaan parhaillaan. Viiteversio on [englanninkielinen](../../README.md).

## Miksi TotalDrop

- **Lyhyt koodi riittää.** Valitse tiedostot, niin saat koodin, kuten `7-harbor-rocket`. Lue se ääneen tai näytä QR-koodi; toinen henkilö kirjoittaa sen, ja tiedostot saapuvat.
- **Päästä päähän salattu, ja voit tarkistaa sen.** Molemmat näytöt näyttävät samat neljä sanaa ennen kuin mitään siirretään. Jos ne eroavat, keskeytä.
- **Ei mitään, mihin rekisteröityä.** Ei tiliä, sähköpostia tai puhelinnumeroa. Valinnaiset tilit ovat pelkkiä numeroita, Mullvadin tapaan ([suunnitelma](../../docs/hesap.md)).
- **Suoraan laitteelta laitteelle, missä tahansa.** Kohtauspalvelin vain esittelee kaksi laitetta toisilleen eikä tallenna lokeja; kuten mikä tahansa palvelin, se näkee niiden IP-osoitteet. Laitteet yhdistävät suoraan internetin yli aina kun voivat. Kun reititin tai palomuuri on tiellä, välityspalvelin kuljettaa liikenteen pystymättä lukemaan sitä.
- **Kotonaan järjestelmässäsi.** Sovellukset noudattavat järjestelmän vaaleaa tai tummaa teemaa, korostusväriä, kieltä ja vähennetyn liikkeen asetusta. Linux-sovellus on natiivi Qt 6 -sovellus Waylandilla ja ottaa lisäksi fontit, kulmat ja kuvakkeet KDE Plasman, Hyprlandin ja Omarchyn teemoista reaaliajassa.
- **Ilmainen, pysyvästi.** Avointa lähdekoodia ja lahjoituksilla rahoitettu. Ei mainoksia, ei seurantaa, ei maksullista tasoa perustoiminnoille.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Linux-sovellus Omarchyssa työpöydän värein, fontein ja kuvakkein" width="560">
  <br><sub>Linux-sovellus Omarchyssa: värit, kulmat, fontit ja kuvakkeet tulevat työpöydältä.</sub>
</p>

## Lataa

| Alusta | Hanki | Huomiot |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 tai uudempi |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 tai uudempi |
| **Android** | [Liity betaan](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 tai uudempi, Google Play kutsusta |
| **Komentorivi** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Tulossa | Näiden koodiallekirjoitus on se, mihin lahjoitukset käytetään ensimmäisenä |

Jokainen paketti rakennetaan CI:ssä tagatusta commitista, ei koskaan kehittäjän koneella. Jos haluat tarkistaa paketin ennen asennusta, vertaa sitä julkaisun `SHA256SUMS`-tiedostoon ja varmista, missä se on rakennettu:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Koodille ei ole vielä tehty riippumatonta tietoturvatarkastusta, joten älä luota siihen arkaluonteisten tiedostojen kanssa.

## Miten se toimii

1. Lähettäjä pyytää kohtauspalvelimelta lyhyen koodin. Koodi on ainoa salaisuus, jonka kaksi henkilöä jakavat.
2. Molemmat laitteet muuttavat koodin vahvaksi avaimeksi **CPacella**. Hyökkääjä saa yhden arvauksen siirtoa kohden eikä voi testata koodeja offline-tilassa.
3. Laitteet yhdistävät suoraan **QUIC**-yhteydellä (TLS 1.3 kiinnitetyillä raaoilla julkisilla avaimilla) ja ajavat **Noise XXpsk3:n ML-KEM-768:lla**, joka on suunniteltu kestämään myös tulevia kvanttitietokoneita.
4. Tiedostot salataan lohkoina (STREAM), joten jokainen osa todennetaan sen saapuessa.

Protokolla on kuvattu dokumentissa *TotalDrop Protocol v0*. Toteutuksen aikana tehdyt päätökset ovat tiedostossa [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), ja se, mitä kukin palvelin näkee ja ei näe, on kerrottu [uhkamallissa](../../docs/tehdit-modeli.md) (turkiksi) ja [tietosuojakäytännössä](https://totaldrop.app/privacy).

## Omalla kielelläsi

TotalDrop puhuu 35:tä kieltä ja noudattaa alueesi numeromuotoja. Englanti ja turkki on tarkistettu; muut 33 ovat luonnoksia, joita äidinkieliset puhujat tarkistavat, joten sovellukset pitävät tietoturvatekstit (tarkistus, väärä koodi, hyväksy ja hylkää) englanniksi, kunnes kieli on tarkistettu. Sivustolla ne kaikki ovat jo nyt; sovellukset saavat ne seuraavissa julkaisuissaan.

Huomasitko kömpelön lauseen? [Auta kääntämisessä](../../docs/ceviri.md) (opas on turkiksi).

## Tue projektia

TotalDrop pysyy ilmaisena, koska ihmiset osallistuvat kuluihin. Lahjoituksilla maksetaan välityspalvelin, Windowsin ja macOS:n vaatimat koodiallekirjoitusvarmenteet sekä riippumaton tietoturvatarkastus ennen versiota 1.0. Jokainen kulu rivi riviltä ja tähän mennessä kerätty summa: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Tue GitHubissa"></a>

Voit auttaa myös [ilmoittamalla virheistä](https://github.com/totaldrop-app/totaldrop/issues/new/choose), kääntämällä tai testaamalla työpöydälläsi.

## Kehittäjille

### Repositorion rakenne

| Polku | Mitä se on | Lisenssi |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, avainten johtaminen, postilaatikon ja istunnon salaus, laiteavaimet, STREAM-lohkosalaus | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Siirtokoodit, rajat, polkujen turvallisuus, kaikki protokollaviestit (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Asiakasmoottori: kohtauspalvelinyhteys, QUIC raaoilla julkisilla avaimilla, Noise XXpsk3 + ML-KEM-768, siirto | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust-ohjelmien käyttöliittymätekstit: kielen neuvottelu, ICU-viestit, monikkomuodot, alueelliset numeromuodot | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop`-komentoriviasiakas | GPL-3.0-or-later |
| `l10n/` | Kaikki käyttöliittymän merkkijonot, yhteiset Androidille, Linuxille ja CLI:lle ([kääntäminen](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linuxin työpöytäsovellus: Qt 6 ja QML `totaldrop-core`-kirjaston päällä, natiivi Wayland | GPL-3.0-or-later |
| `packaging/` | Linux-paketit, kukin muoto erikseen: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android-sovellus (Flutter); iOS myöhemmin | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app ja sivu, jonka QR-linkit avaavat | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz-kohteet jokaiselle jäsentimelle | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Kohtauspalvelin Rustilla, omaan ylläpitoon ja testeihin | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Sama palvelin Cloudflare Workersissa, osoitteelle `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Kirjastoilla on sallivat lisenssit, jotta kuka tahansa voi rakentaa protokollan päälle; sovellukset ovat GPL- ja palvelimet AGPL-lisensoituja. Katso [LICENSING.md](../../LICENSING.md).

Jokaisella alustalla on oma versionsa, tagi (`linux-v…`, `android-v…`) ja CI-työnkulku, joten yhden voi julkaista koskematta muihin. Haarojen ja julkaisujen toiminta: [docs/gelistirme.md](../../docs/gelistirme.md) (turkiksi) ja [docs/versioning.md](../../docs/versioning.md). Suunnittelusäännöt: [docs/tasarim.md](../../docs/tasarim.md) ja arkkitehtuuripäätökset: [docs/kararlar](../../docs/kararlar/README.md), molemmat turkiksi.

### Rakentaminen

```bash
cargo test
cargo build --release
```

Kokeile siirtoa yhdellä koneella kolmessa päätteessä:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Molemmat osapuolet näyttävät samat neljä tarkistussanaa. Salaamaton `ws://` hyväksytään vain samalla koneella olevalle palvelimelle; kaiken muun on oltava `wss://`.

Projekti käyttää aina uusinta vakaata Rustia (kiinnitetty tiedostossa `rust-toolchain.toml`); rustup asentaa sen ensimmäisellä käännöskerralla.

**Linux-sovellus** (vaatii Qt 6.8:n tai uudemman: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Osallistuminen ja tietoturva

Lue [CONTRIBUTING.md](../../CONTRIBUTING.md) ennen kuin avaat pull requestin, sekä [toimintaohjeemme](../../CODE_OF_CONDUCT.md). Kysymyksiä varten katso [SUPPORT.md](../../SUPPORT.md). Ilmoita haavoittuvuuksista yksityisesti [SECURITY.md](../../SECURITY.md):n ohjeiden mukaan, ei koskaan julkisessa issuessa.
