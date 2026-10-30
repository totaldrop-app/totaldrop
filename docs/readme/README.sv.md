<!-- Translated from README.md 529368877e8d. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Vilken fil som helst, till vem som helst, var som helst.

Gratis och öppen källkod. End-to-end-krypterat. Ingen e-post, inget telefonnummer, inget konto.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Webbplats](https://totaldrop.app)** · **[Ladda ner](https://totaldrop.app/download/)** · [Färdplan](https://totaldrop.app/roadmap/) · [Donera](https://totaldrop.app/donate/) · [Versioner](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · **Svenska** · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop på Android: startskärmen i ljust och mörkt läge, och en överföring som har kommit fram och verifierats" width="820">

</div>

> [!NOTE]
> Den här översättningen är ett utkast och granskas. [Den engelska versionen](../../README.md) är referensen.

## Varför TotalDrop

- **En kort kod är allt som behövs.** Välj filer och få en kod som `7-harbor-rocket`. Läs upp den eller visa QR-koden; den andra personen skriver in den och filerna kommer fram.
- **Krypterat end-to-end, och du kan kontrollera det.** Båda skärmarna visar samma fyra ord innan något överförs. Om de skiljer sig åt, avbryt.
- **Inget att registrera sig för.** Inget konto, ingen e-post, inget telefonnummer. Valfria konton kommer bara att vara nummer, som hos Mullvad ([utformning](../../docs/hesap.md)).
- **Direkt från enhet till enhet, var som helst.** Mötesservern presenterar bara de två enheterna för varandra och sparar inga loggar; som alla servrar ser den deras IP-adresser. Enheterna ansluter direkt över internet där det går. När en router eller brandvägg står i vägen bär en reläserver trafiken, utan att kunna läsa den.
- **Hemma i ditt system.** Apparna följer systemets ljusa eller mörka tema, accentfärg, språk och inställning för reducerad rörelse. Linux-appen är inbyggd Qt 6 på Wayland och hämtar dessutom typsnitt, hörn och ikoner från teman i KDE Plasma, Hyprland och Omarchy direkt.
- **Gratis, för gott.** Öppen källkod, finansierat av donationer. Inga annonser, ingen spårning, ingen betalnivå för grunderna.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Linux-appen på Omarchy, i skrivbordets färger, typsnitt och ikoner" width="560">
  <br><sub>Linux-appen på Omarchy: färger, hörn, typsnitt och ikoner kommer från skrivbordet.</sub>
</p>

## Ladda ner

| Plattform | Hämta | Anmärkningar |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1, Qt 6.8 eller senare |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1, Qt 6.8 eller senare |
| **Android** | [Gå med i betan](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1, Android 15 eller senare, Google Play via inbjudan |
| **Kommandorad** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Kommer | Kodsignering för dessa är det som donationerna betalar för först |

Varje paket byggs av CI från en taggad commit, aldrig på en utvecklares dator. Om du vill kontrollera ett paket innan du installerar det, jämför det med versionens `SHA256SUMS` och verifiera var det byggdes:

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Koden har ännu inte genomgått någon oberoende säkerhetsgranskning, så förlita dig inte på den för känsliga filer.

## Så fungerar det

1. Avsändaren ber mötesservern om en kort kod. Koden är den enda hemlighet som de två personerna delar.
2. Båda enheterna gör om koden till en stark nyckel med **CPace**. En angripare får en gissning per överföring och kan inte testa koder offline.
3. De ansluter direkt över **QUIC** (TLS 1.3 med fästa råa publika nycklar) och kör **Noise XXpsk3 med ML-KEM-768**, som är utformat för att stå emot även framtida kvantdatorer.
4. Filerna krypteras i block (STREAM), så varje del autentiseras när den kommer fram.

Protokollet beskrivs i *TotalDrop Protocol v0*. Beslut som fattats under implementeringen finns i [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), och vad varje server kan och inte kan se står i [hotmodellen](../../docs/tehdit-modeli.md) (på turkiska) och [integritetspolicyn](https://totaldrop.app/privacy).

## På ditt språk

TotalDrop finns på 35 språk och följer din regions talformat. Engelska och turkiska är granskade; de övriga 33 är utkast som modersmålstalare håller på att kontrollera, så apparna behåller säkerhetstexterna (verifiering, fel kod, acceptera och avböj) på engelska tills ett språk har granskats. Webbplatsen och apparna har alla, från Android och Linux 0.3.0-beta.1.

Har du hittat en klumpig mening? [Hjälp till att översätta](../../docs/ceviri.md) (guiden är på turkiska).

## Stöd projektet

TotalDrop förblir gratis för att folk bidrar. Donationerna betalar för reläservern, de kodsigneringscertifikat som Windows och macOS kräver, och en oberoende säkerhetsgranskning före 1.0. Varje kostnad, rad för rad, och hur mycket som har samlats in hittills: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Sponsra på GitHub"></a>

Du kan också hjälpa till genom att [rapportera buggar](https://github.com/totaldrop-app/totaldrop/issues/new/choose), översätta eller testa på din dator.

## För utvecklare

### Repots struktur

| Sökväg | Vad det är | Licens |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Svit TD0: CPace, nyckelhärledning, kryptering av brevlåda och session, enhetsnycklar, blockkryptering med STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Överföringskoder, gränser, säkra sökvägar, alla protokollmeddelanden (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Klientmotor: kontakt med mötesservern, QUIC med råa publika nycklar, Noise XXpsk3 + ML-KEM-768, överföring | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Gränssnittstext för Rust-programmen: språkförhandling, ICU-meddelanden, pluralformer, regionala talformat | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Kommandoradsklienten `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Alla gränssnittssträngar, delade av Android, Linux och CLI ([översättning](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux-skrivbordsapp: Qt 6 och QML ovanpå `totaldrop-core`, inbyggt Wayland | GPL-3.0-or-later |
| `packaging/` | Linux-paket, varje format för sig: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android-app (Flutter); iOS senare | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app och sidan som QR-länkarna öppnar | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz-mål för varje parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Mötesserver i Rust, för egen drift och tester | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Samma server på Cloudflare Workers, för `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Biblioteken har tillåtande licenser så att vem som helst kan bygga vidare på protokollet; apparna är GPL och servrarna AGPL. Se [LICENSING.md](../../LICENSING.md).

Varje plattform har sin egen version, tagg (`linux-v…`, `android-v…`) och sitt eget CI-arbetsflöde, så att en kan släppas utan att de andra berörs. Hur grenar och versioner fungerar: [docs/gelistirme.md](../../docs/gelistirme.md) (på turkiska) och [docs/versioning.md](../../docs/versioning.md). Designregler: [docs/tasarim.md](../../docs/tasarim.md) och arkitekturbeslut: [docs/kararlar](../../docs/kararlar/README.md), båda på turkiska.

### Bygga

```bash
cargo test
cargo build --release
```

Prova en överföring på en dator, i tre terminaler:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Båda sidor visar samma fyra verifieringsord. Okrypterat `ws://` accepteras bara för en server på samma dator; allt annat måste vara `wss://`.

Projektet siktar alltid på den senaste stabila versionen av Rust (fäst i `rust-toolchain.toml`); rustup installerar den vid första bygget.

**Linux-appen** (kräver Qt 6.8 eller senare: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Bidra och säkerhet

Läs [CONTRIBUTING.md](../../CONTRIBUTING.md) innan du öppnar en pull request, och vår [uppförandekod](../../CODE_OF_CONDUCT.md). Frågor hör hemma i [SUPPORT.md](../../SUPPORT.md). Rapportera sårbarheter privat enligt [SECURITY.md](../../SECURITY.md), aldrig i ett offentligt ärende.
