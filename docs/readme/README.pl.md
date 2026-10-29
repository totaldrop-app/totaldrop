<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Każdy plik, do każdego, wszędzie.

Za darmo i na otwartych źródłach. Szyfrowanie end-to-end. Bez e-maila, bez numeru telefonu, bez konta.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Strona](https://totaldrop.app)** · **[Pobierz](https://totaldrop.app/download/)** · [Plan rozwoju](https://totaldrop.app/roadmap/) · [Wesprzyj](https://totaldrop.app/donate/) · [Wydania](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · **Polski** · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop na Androidzie: ekran główny w jasnym i ciemnym motywie oraz transfer, który dotarł i został zweryfikowany" width="820">

</div>

> [!NOTE]
> To tłumaczenie jest wersją roboczą i jest sprawdzane. Wersją wzorcową jest [wersja angielska](../../README.md).

## Dlaczego TotalDrop

- **Wystarczy krótki kod.** Wybierz pliki i otrzymaj kod w rodzaju `7-harbor-rocket`. Przeczytaj go na głos albo pokaż kod QR; druga osoba go wpisuje i pliki docierają.
- **Szyfrowanie end-to-end, które możesz sprawdzić.** Zanim cokolwiek zostanie przesłane, oba ekrany pokazują te same cztery słowa. Jeśli się różnią, przerwij.
- **Nie trzeba się nigdzie rejestrować.** Bez konta, e-maila i numeru telefonu. Opcjonalne konta będą tylko numerami, w stylu Mullvad ([projekt](../../docs/hesap.md)).
- **Prosto z urządzenia na urządzenie, wszędzie.** Serwer spotkań tylko przedstawia sobie dwa urządzenia i nie prowadzi logów; jak każdy serwer widzi ich adresy IP. Urządzenia łączą się bezpośrednio przez internet wszędzie, gdzie to możliwe. Gdy przeszkadza router lub zapora, ruch przenosi serwer pośredniczący, który nie może go odczytać.
- **Jak u siebie w twoim systemie.** Aplikacje podążają za jasnym lub ciemnym motywem systemu, kolorem akcentu, językiem i ustawieniem ograniczenia ruchu. Aplikacja na Linuksa to natywne Qt 6 na Waylandzie, które dodatkowo na bieżąco pobiera czcionki, narożniki i ikony z motywów KDE Plasma, Hyprland i Omarchy.
- **Za darmo, na zawsze.** Otwarte źródła, finansowane z darowizn. Bez reklam, bez śledzenia, bez płatnej wersji dla podstawowych funkcji.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Aplikacja na Linuksa w Omarchy, w kolorach, czcionkach i ikonach pulpitu" width="560">
  <br><sub>Aplikacja na Linuksa w Omarchy: kolory, narożniki, czcionki i ikony pochodzą z pulpitu.</sub>
</p>

## Pobierz

| Platforma | Gdzie pobrać | Uwagi |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 lub nowszy |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 lub nowszy |
| **Android** | [Dołącz do bety](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 lub nowszy, Google Play na zaproszenie |
| **Wiersz poleceń** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | W drodze | Podpisywanie kodu dla tych systemów to pierwsze, na co pójdą darowizny |

Każdy pakiet jest budowany przez CI z oznaczonego tagiem commita, nigdy na komputerze dewelopera. Aby sprawdzić pakiet przed instalacją, porównaj go z plikiem `SHA256SUMS` wydania i zweryfikuj, gdzie został zbudowany:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Kod nie przeszedł jeszcze niezależnego przeglądu bezpieczeństwa, więc prosimy nie polegać na nim przy poufnych plikach.

## Jak to działa

1. Nadawca prosi serwer spotkań o krótki kod. Kod jest jedynym sekretem, który dzielą obie osoby.
2. Oba urządzenia zamieniają kod w silny klucz za pomocą **CPace**. Atakujący ma jedną próbę na transfer i nie może sprawdzać kodów offline.
3. Urządzenia łączą się bezpośrednio przez **QUIC** (TLS 1.3 z przypiętymi surowymi kluczami publicznymi) i wykonują **Noise XXpsk3 z ML-KEM-768**, który zaprojektowano tak, by wytrzymał także przyszłe komputery kwantowe.
4. Pliki są szyfrowane fragmentami (STREAM), więc każdy fragment jest uwierzytelniany w chwili, gdy dociera.

Protokół jest opisany w dokumencie *TotalDrop Protocol v0*. Decyzje podjęte podczas jego implementacji znajdują się w [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), a to, co każdy serwer może, a czego nie może zobaczyć, opisują [model zagrożeń](../../docs/tehdit-modeli.md) (po turecku) i [polityka prywatności](https://totaldrop.app/privacy).

## W twoim języku

TotalDrop mówi w 35 językach i stosuje formaty liczb twojego regionu. Angielski i turecki są sprawdzone; pozostałe 33 to wersje robocze, które sprawdzają rodzimi użytkownicy, dlatego aplikacje zostawiają teksty dotyczące bezpieczeństwa (weryfikacja, błędny kod, akceptacja i odrzucenie) po angielsku, dopóki dany język nie zostanie sprawdzony. Strona ma je wszystkie już dziś; aplikacje dostaną je w kolejnych wydaniach.

Zauważyłeś niezgrabne zdanie? [Pomóż w tłumaczeniu](../../docs/ceviri.md) (przewodnik jest po turecku).

## Wesprzyj projekt

TotalDrop pozostaje darmowy, bo ludzie się dokładają. Darowizny opłacają serwer pośredniczący, certyfikaty podpisywania kodu wymagane przez Windows i macOS oraz niezależny audyt bezpieczeństwa przed wersją 1.0. Każdy koszt, pozycja po pozycji, i to, ile udało się dotąd zebrać: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Wesprzyj na GitHubie"></a>

Możesz też pomóc, [zgłaszając błędy](https://github.com/totaldrop-app/totaldrop/issues/new/choose), tłumacząc lub testując na swoim komputerze.

## Dla deweloperów

### Struktura repozytorium

| Ścieżka | Co to jest | Licencja |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, wyprowadzanie kluczy, szyfrowanie skrzynki i sesji, klucze urządzeń, szyfrowanie fragmentów STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Kody transferu, limity, bezpieczeństwo ścieżek, każdy komunikat sieciowy (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Silnik klienta: serwer spotkań, QUIC z surowymi kluczami publicznymi, Noise XXpsk3 + ML-KEM-768, transfer | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Teksty interfejsu dla programów w Ruście: wybór języka, komunikaty ICU, liczba mnoga, regionalne formaty liczb | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Klient wiersza poleceń `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Wszystkie teksty interfejsu, wspólne dla Androida, Linuksa i CLI ([tłumaczenie](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Aplikacja desktopowa na Linuksa: Qt 6 i QML nad `totaldrop-core`, natywny Wayland | GPL-3.0-or-later |
| `packaging/` | Pakiety dla Linuksa, każdy format osobno: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Aplikacja na Androida (Flutter); iOS później | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app i strona, którą otwierają linki z kodów QR | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Cele fuzzingu dla każdego parsera | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Serwer spotkań w Ruście, do samodzielnego hostowania i testów | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Ten sam serwer na Cloudflare Workers, dla `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Biblioteki mają licencje permisywne, aby każdy mógł budować na protokole; aplikacje są na GPL, a serwery na AGPL. Zobacz [LICENSING.md](../../LICENSING.md).

Każda platforma ma własną wersję, tag (`linux-v…`, `android-v…`) i proces CI, więc jedną można wydać, nie ruszając pozostałych. Jak działają gałęzie i wydania: [docs/gelistirme.md](../../docs/gelistirme.md) (po turecku) i [docs/versioning.md](../../docs/versioning.md). Zasady projektowe: [docs/tasarim.md](../../docs/tasarim.md) i decyzje architektoniczne: [docs/kararlar](../../docs/kararlar/README.md), jedno i drugie po turecku.

### Budowanie

```bash
cargo test
cargo build --release
```

Wypróbuj transfer na jednym komputerze, w trzech terminalach:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Obie strony pokazują te same cztery słowa weryfikacyjne. Nieszyfrowany `ws://` jest akceptowany tylko dla serwera na tym samym komputerze; w każdym innym przypadku musi to być `wss://`.

Projekt zawsze celuje w najnowszą stabilną wersję Rusta (przypiętą w `rust-toolchain.toml`); rustup zainstaluje ją przy pierwszym budowaniu.

**Aplikacja na Linuksa** (wymaga Qt 6.8 lub nowszego: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Współtworzenie i bezpieczeństwo

Przed otwarciem pull requesta przeczytaj [CONTRIBUTING.md](../../CONTRIBUTING.md) oraz nasz [kodeks postępowania](../../CODE_OF_CONDUCT.md). Pytania kieruj zgodnie z [SUPPORT.md](../../SUPPORT.md). Luki w zabezpieczeniach zgłaszaj prywatnie, jak opisano w [SECURITY.md](../../SECURITY.md), nigdy w publicznym zgłoszeniu.
