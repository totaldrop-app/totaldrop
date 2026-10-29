<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Qualsiasi file, a chiunque, ovunque.

Gratuito e open source. Crittografato end-to-end. Niente email, niente numero di telefono, niente account.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Sito web](https://totaldrop.app)** · **[Scarica](https://totaldrop.app/download/)** · [Roadmap](https://totaldrop.app/roadmap/) · [Dona](https://totaldrop.app/donate/) · [Release](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · **Italiano** · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop su Android: la schermata principale in chiaro e in scuro, e un trasferimento arrivato e verificato" width="820">

</div>

> [!NOTE]
> Questa traduzione è una bozza ed è in fase di revisione. Il testo di riferimento è la [versione inglese](../../README.md).

## Perché TotalDrop

- **Basta un codice breve.** Scegli i file e ottieni un codice come `7-harbor-rocket`. Leggilo ad alta voce o mostra il codice QR; l'altra persona lo inserisce e i file arrivano.
- **Crittografato end-to-end, e puoi verificarlo.** Entrambi gli schermi mostrano le stesse quattro parole prima che parta qualsiasi cosa. Se sono diverse, fermati.
- **Nessuna registrazione.** Niente account, email o numero di telefono. Gli account facoltativi saranno solo un numero, in stile Mullvad ([progetto](../../docs/hesap.md)).
- **Direttamente da dispositivo a dispositivo, ovunque.** Il server di incontro si limita a presentare i due dispositivi e non conserva log. I dispositivi si connettono direttamente via internet ovunque possibile. Quando un router o un firewall si mette in mezzo, un server di inoltro trasporta il traffico senza poterlo leggere.
- **A casa su ogni sistema.** Ogni app segue il tema, i caratteri, il colore di accento, la lingua e le impostazioni di animazione del proprio sistema. L'app Linux è Qt 6 nativa su Wayland e segue in tempo reale i temi di KDE Plasma, Hyprland e Omarchy.
- **Gratuito, per sempre.** Open source e sostenuto dalle donazioni. Niente pubblicità, niente tracciamento, nessun piano a pagamento per le funzioni di base.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="L'app Linux su Omarchy, con i colori, i caratteri e le icone del desktop" width="560">
  <br><sub>L'app Linux su Omarchy: colori, angoli, caratteri e icone vengono dal desktop.</sub>
</p>

## Scarica

| Piattaforma | Dove trovarlo | Note |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 o successivo |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 o successivo |
| **Android** | [Partecipa alla beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 o successivo, Google Play su invito |
| **Riga di comando** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | In arrivo | La firma del codice per queste piattaforme è la prima cosa che le donazioni pagano |

Ogni pacchetto viene compilato dalla CI a partire da un commit con tag, mai sul computer di uno sviluppatore. Per verificarne uno prima di installarlo, confrontalo con il file `SHA256SUMS` della release e verifica dove è stato compilato:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Il codice non ha ancora avuto una revisione di sicurezza indipendente, quindi non affidarti a esso per file sensibili.

## Come funziona

1. Il mittente chiede un codice breve al server di incontro. Il codice è l'unico segreto che le due persone condividono.
2. Entrambi i dispositivi trasformano il codice in una chiave robusta con **CPace**. Un attaccante ha un solo tentativo per trasferimento e non può provare codici offline.
3. Si connettono direttamente tramite **QUIC** (TLS 1.3 con chiavi pubbliche grezze fissate) ed eseguono **Noise XXpsk3 con ML-KEM-768**, che resta sicuro anche contro i futuri computer quantistici.
4. I file vengono crittografati a blocchi (STREAM), così ogni parte viene autenticata appena arriva.

Il protocollo è descritto in *TotalDrop Protocol v0*. Le decisioni prese durante l'implementazione sono in [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), e ciò che ogni server può e non può vedere è nel [modello delle minacce](../../docs/tehdit-modeli.md) e nell'[informativa sulla privacy](https://totaldrop.app/privacy).

## Nella tua lingua

TotalDrop parla 35 lingue e segue i formati numerici della tua regione. L'inglese e il turco sono revisionati; le altre 33 sono bozze che madrelingua stanno controllando, quindi le app mantengono in inglese i testi di sicurezza (verifica, codice errato, accetta e rifiuta) finché una lingua non viene revisionata. Il sito le ha già tutte; le app le riceveranno con le prossime versioni.

Hai notato una frase goffa? [Aiuta a tradurre](../../docs/ceviri.md).

## Sostieni il progetto

TotalDrop resta gratuito perché le persone contribuiscono. Le donazioni pagano il server di inoltro, i certificati di firma del codice richiesti da Windows e macOS e un audit di sicurezza indipendente prima della 1.0. Ogni costo, voce per voce, e quanto è stato raccolto finora: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Sostieni su GitHub"></a>

Puoi aiutare anche [segnalando bug](https://github.com/totaldrop-app/totaldrop/issues/new/choose), traducendo o facendo test sul tuo desktop.

## Per gli sviluppatori

### Struttura del repository

| Percorso | Cos'è | Licenza |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, derivazione delle chiavi, crittografia della mailbox e della sessione, chiavi del dispositivo, crittografia a blocchi STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Codici di trasferimento, limiti, sicurezza dei percorsi, tutti i messaggi di rete (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Motore del client: incontro, QUIC con chiavi pubbliche grezze, Noise XXpsk3 + ML-KEM-768, trasferimento | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Testi dell'interfaccia per i programmi Rust: negoziazione della lingua, messaggi ICU, plurali, numeri regionali | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Il client a riga di comando `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Tutte le stringhe dell'interfaccia, condivise da Android, Linux e dalla CLI ([traduzione](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | App desktop per Linux: Qt 6 e QML sopra `totaldrop-core`, Wayland nativo | GPL-3.0-or-later |
| `packaging/` | Pacchetti Linux, ogni formato a sé: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | App Android (Flutter); iOS in seguito | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app e la pagina aperta dai link QR | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Target di fuzzing per ogni parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Server di incontro in Rust, per il self-hosting e i test | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Lo stesso server su Cloudflare Workers, per `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Le librerie hanno licenze permissive, così chiunque può costruire sul protocollo; le app sono GPL e i server AGPL. Vedi [LICENSING.md](../../LICENSING.md).

Ogni piattaforma ha la propria versione, il proprio tag (`linux-v…`, `android-v…`) e il proprio workflow di CI, così una può essere rilasciata senza toccare le altre. Come funzionano branch e release: [docs/gelistirme.md](../../docs/gelistirme.md) e [docs/versioning.md](../../docs/versioning.md). Regole di design: [docs/tasarim.md](../../docs/tasarim.md). Decisioni di architettura: [docs/kararlar](../../docs/kararlar/README.md).

### Compilazione

```bash
cargo test
cargo build --release
```

Prova un trasferimento su una sola macchina, in tre terminali:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Entrambi i lati mostrano le stesse quattro parole di verifica. `ws://` non crittografato è accettato solo per un server sulla stessa macchina; tutto il resto deve usare `wss://`.

Il progetto usa sempre la versione stabile più recente di Rust (fissata in `rust-toolchain.toml`); rustup la installa alla prima compilazione.

**App Linux** (richiede Qt 6.8 o successivo: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Contribuire e sicurezza

Leggi [CONTRIBUTING.md](../../CONTRIBUTING.md) prima di aprire una pull request, e il nostro [codice di condotta](../../CODE_OF_CONDUCT.md). Le domande vanno in [SUPPORT.md](../../SUPPORT.md). Segnala le vulnerabilità in privato come descritto in [SECURITY.md](../../SECURITY.md), mai in una issue pubblica.
