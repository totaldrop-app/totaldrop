<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Orice fișier, oricui, oriunde.

Gratuit și open source. Criptat end-to-end. Fără e-mail, fără număr de telefon, fără cont.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Site](https://totaldrop.app)** · **[Descărcare](https://totaldrop.app/download/)** · [Foaie de parcurs](https://totaldrop.app/roadmap/) · [Donează](https://totaldrop.app/donate/) · [Versiuni](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · **Română** · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop pe Android: ecranul principal în modul luminos și întunecat și un transfer sosit și verificat" width="820">

</div>

> [!NOTE]
> Această traducere este o ciornă și este în curs de verificare. Versiunea de referință este cea în [engleză](../../README.md).

## De ce TotalDrop

- **Ajunge un cod scurt.** Alegeți fișierele și primiți un cod de tipul `7-harbor-rocket`. Citiți-l cu voce tare sau arătați codul QR; cealaltă persoană îl introduce și fișierele sosesc.
- **Criptat end-to-end, iar dvs. puteți verifica.** Înainte să se transfere ceva, ambele ecrane arată aceleași patru cuvinte. Dacă diferă, opriți-vă.
- **Nu trebuie să vă înregistrați nicăieri.** Fără cont, e-mail sau număr de telefon. Conturile opționale vor fi doar numere, în stilul Mullvad ([design](../../docs/hesap.md)).
- **Direct de pe un dispozitiv pe altul, oriunde.** Serverul de întâlnire doar face prezentările între cele două dispozitive și nu păstrează jurnale; ca orice server, vede adresele lor IP. Dispozitivele se conectează direct prin internet oriunde pot. Când un router sau un firewall stă în cale, traficul este transportat de un server releu care nu îl poate citi.
- **Ca acasă pe sistemul dumneavoastră.** Aplicațiile urmează tema deschisă sau întunecată a sistemului, culoarea de accent, limba și setarea de reducere a mișcării. Aplicația pentru Linux este Qt 6 nativ pe Wayland și preia în plus, în timp real, fonturile, colțurile și pictogramele din temele KDE Plasma, Hyprland și Omarchy.
- **Gratuit, definitiv.** Open source și finanțat din donații. Fără reclame, fără urmărire, fără versiune plătită pentru funcțiile de bază.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Aplicația pentru Linux pe Omarchy, în culorile, fonturile și pictogramele desktopului" width="560">
  <br><sub>Aplicația pentru Linux pe Omarchy: culorile, colțurile, fonturile și pictogramele vin de la desktop.</sub>
</p>

## Descărcare

| Platformă | De unde | Note |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 sau mai nou |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 sau mai nou |
| **Android** | [Intrați în beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 sau mai nou, Google Play pe bază de invitație |
| **Linie de comandă** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | În curând | Semnarea codului pentru acestea este primul lucru pe care îl plătesc donațiile |

Fiecare pachet este compilat de CI dintr-un commit etichetat, niciodată pe calculatorul unui dezvoltator. Ca să verificați un pachet înainte de instalare, comparați-l cu `SHA256SUMS` din versiunea respectivă și verificați unde a fost compilat:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Codul nu a trecut încă printr-o analiză de securitate independentă, așa că vă rugăm să nu vă bazați pe el pentru fișiere sensibile.

## Cum funcționează

1. Expeditorul cere serverului de întâlnire un cod scurt. Codul este singurul secret pe care îl împart cele două persoane.
2. Ambele dispozitive transformă codul într-o cheie puternică folosind **CPace**. Un atacator are o singură încercare pe transfer și nu poate testa coduri offline.
3. Dispozitivele se conectează direct prin **QUIC** (TLS 1.3 cu chei publice brute fixate) și rulează **Noise XXpsk3 cu ML-KEM-768**, care este conceput să reziste și în fața viitoarelor computere cuantice.
4. Fișierele sunt criptate pe bucăți (STREAM), așa că fiecare bucată este autentificată pe măsură ce sosește.

Protocolul este descris în *TotalDrop Protocol v0*. Deciziile luate la implementarea lui se află în [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), iar ce poate și ce nu poate vedea fiecare server este descris în [modelul de amenințări](../../docs/tehdit-modeli.md) (în turcă) și în [politica de confidențialitate](https://totaldrop.app/privacy).

## În limba dvs.

TotalDrop vorbește 35 de limbi și urmează formatele de numere ale regiunii dvs. Engleza și turca sunt verificate; celelalte 33 sunt ciorne pe care le verifică vorbitori nativi, așa că aplicațiile păstrează textele de securitate (verificare, cod greșit, acceptare și refuz) în engleză până când o limbă este verificată. Site-ul le are pe toate de azi; aplicațiile le primesc în versiunile următoare.

Ați găsit o frază stângace? [Ajutați la traducere](../../docs/ceviri.md) (ghidul este în turcă).

## Susțineți proiectul

TotalDrop rămâne gratuit pentru că oamenii contribuie. Donațiile plătesc serverul releu, certificatele de semnare a codului cerute de Windows și macOS și un audit de securitate independent înainte de 1.0. Fiecare cost, linie cu linie, și cât s-a strâns până acum: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Sponsorizați pe GitHub"></a>

Puteți ajuta și [raportând erori](https://github.com/totaldrop-app/totaldrop/issues/new/choose), traducând sau testând pe desktopul dvs.

## Pentru dezvoltatori

### Structura depozitului

| Cale | Ce este | Licență |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, derivarea cheilor, criptarea cutiei poștale și a sesiunii, cheile dispozitivelor, criptarea pe bucăți STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Coduri de transfer, limite, siguranța căilor, fiecare mesaj de rețea (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Motorul clientului: întâlnirea prin server, QUIC cu chei publice brute, Noise XXpsk3 + ML-KEM-768, transfer | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Textele interfeței pentru programele Rust: alegerea limbii, mesaje ICU, forme de plural, numere regionale | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Clientul de linie de comandă `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Toate textele interfeței, comune pentru Android, Linux și CLI ([traducere](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Aplicația desktop pentru Linux: Qt 6 și QML peste `totaldrop-core`, Wayland nativ | GPL-3.0-or-later |
| `packaging/` | Pachete Linux, fiecare format separat: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Aplicația Android (Flutter); iOS mai târziu | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app și pagina deschisă de linkurile din codurile QR | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Ținte de fuzzing pentru fiecare parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Serverul de întâlnire în Rust, pentru găzduire proprie și teste | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Același server pe Cloudflare Workers, pentru `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Bibliotecile au licențe permisive, ca oricine să poată construi pe protocol; aplicațiile sunt GPL, iar serverele AGPL. Vedeți [LICENSING.md](../../LICENSING.md).

Fiecare platformă are propria versiune, etichetă (`linux-v…`, `android-v…`) și flux CI, așa că una poate fi lansată fără să le atingă pe celelalte. Cum funcționează ramurile și versiunile: [docs/gelistirme.md](../../docs/gelistirme.md) (în turcă) și [docs/versioning.md](../../docs/versioning.md). Reguli de design: [docs/tasarim.md](../../docs/tasarim.md) și decizii de arhitectură: [docs/kararlar](../../docs/kararlar/README.md), ambele în turcă.

### Compilare

```bash
cargo test
cargo build --release
```

Încercați un transfer pe un singur calculator, în trei terminale:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Ambele părți arată aceleași patru cuvinte de verificare. `ws://` necriptat este acceptat doar pentru un server de pe același calculator; orice altceva trebuie să fie `wss://`.

Proiectul țintește mereu cea mai nouă versiune stabilă de Rust (fixată în `rust-toolchain.toml`); rustup o instalează la prima compilare.

**Aplicația pentru Linux** (necesită Qt 6.8 sau mai nou: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Contribuții și securitate

Citiți [CONTRIBUTING.md](../../CONTRIBUTING.md) înainte să deschideți un pull request, precum și [codul nostru de conduită](../../CODE_OF_CONDUCT.md). Întrebările merg la [SUPPORT.md](../../SUPPORT.md). Vă rugăm să raportați vulnerabilitățile în privat, așa cum este descris în [SECURITY.md](../../SECURITY.md), niciodată într-un issue public.
