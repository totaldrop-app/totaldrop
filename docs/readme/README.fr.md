<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### N'importe quel fichier, à n'importe qui, n'importe où.

Gratuit et open source. Chiffré de bout en bout. Pas d'e-mail, pas de numéro de téléphone, pas de compte.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Site web](https://totaldrop.app)** · **[Télécharger](https://totaldrop.app/download/)** · [Feuille de route](https://totaldrop.app/roadmap/) · [Faire un don](https://totaldrop.app/donate/) · [Versions](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · **Français** · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop sur Android : l'écran d'accueil en clair et en sombre, et un transfert arrivé et vérifié" width="820">

</div>

> [!NOTE]
> Cette traduction est une ébauche en cours de relecture. La [version anglaise](../../README.md) fait référence.

## Pourquoi TotalDrop

- **Un code court suffit.** Choisissez des fichiers et obtenez un code comme `7-harbor-rocket`. Lisez-le à voix haute ou montrez le code QR ; l'autre personne le saisit et les fichiers arrivent.
- **Chiffré de bout en bout, et vous pouvez le vérifier.** Les deux écrans affichent les quatre mêmes mots avant que quoi que ce soit ne parte. S'ils diffèrent, arrêtez.
- **Aucune inscription.** Pas de compte, d'e-mail ni de numéro de téléphone. Les comptes facultatifs ne seront qu'un numéro, à la manière de Mullvad ([conception](../../docs/hesap.md)).
- **Directement d'appareil à appareil, partout.** Le serveur de rendez-vous ne fait que présenter les deux appareils l'un à l'autre et ne conserve aucun journal. Les appareils se connectent directement via Internet chaque fois que c'est possible. Quand un routeur ou un pare-feu fait obstacle, un serveur relais transporte le trafic sans pouvoir le lire.
- **Chez soi sur chaque système.** Chaque application suit le thème, les polices, la couleur d'accentuation, la langue et les réglages d'animation de son système. L'application Linux est en Qt 6 natif sous Wayland et suit en direct les thèmes de KDE Plasma, Hyprland et Omarchy.
- **Gratuit, pour de bon.** Open source et financé par les dons. Pas de publicité, pas de pistage, pas d'offre payante pour l'essentiel.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="L'application Linux sur Omarchy, aux couleurs, polices et icônes du bureau" width="560">
  <br><sub>L'application Linux sur Omarchy : les couleurs, les angles, les polices et les icônes viennent du bureau.</sub>
</p>

## Télécharger

| Plateforme | Obtenir | Remarques |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 ou plus récent |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 ou plus récent |
| **Android** | [Rejoindre la bêta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 ou plus récent, Google Play sur invitation |
| **Ligne de commande** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | En préparation | La signature de code de ces plateformes est la première chose que financent les dons |

Chaque paquet est compilé par la CI à partir d'un commit étiqueté, jamais sur la machine d'un développeur. Pour en vérifier un avant de l'installer, comparez-le au fichier `SHA256SUMS` de la version et vérifiez où il a été compilé :

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Bêta.** Le code n'a pas encore fait l'objet d'un audit de sécurité indépendant ; ne vous y fiez donc pas pour des fichiers sensibles.

## Fonctionnement

1. L'expéditeur demande un code court au serveur de rendez-vous. Ce code est le seul secret que les deux personnes partagent.
2. Les deux appareils transforment le code en une clé robuste avec **CPace**. Un attaquant n'a droit qu'à une tentative par transfert et ne peut pas tester de codes hors ligne.
3. Ils se connectent directement via **QUIC** (TLS 1.3 avec des clés publiques brutes épinglées) et exécutent **Noise XXpsk3 avec ML-KEM-768**, qui reste sûr face aux futurs ordinateurs quantiques.
4. Les fichiers sont chiffrés par blocs (STREAM), si bien que chaque morceau est authentifié à son arrivée.

Le protocole est décrit dans *TotalDrop Protocol v0*. Les décisions prises lors de son implémentation figurent dans [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), et ce que chaque serveur peut voir ou non est décrit dans le [modèle de menace](../../docs/tehdit-modeli.md) et la [politique de confidentialité](https://totaldrop.app/privacy).

## Dans votre langue

TotalDrop parle 35 langues et suit les formats de nombres de votre région. L'anglais et le turc sont relus ; les 33 autres sont des ébauches que des locuteurs natifs vérifient. Les applications gardent donc les textes de sécurité (vérification, code erroné, accepter et refuser) en anglais tant qu'une langue n'a pas été relue. Le site les propose toutes dès aujourd'hui ; les applications les recevront avec leurs prochaines versions.

Vous avez repéré une phrase maladroite ? [Aidez à traduire](../../docs/ceviri.md).

## Soutenir le projet

TotalDrop reste gratuit parce que des gens y contribuent. Les dons paient le serveur relais, les certificats de signature de code qu'exigent Windows et macOS, et un audit de sécurité indépendant avant la 1.0. Chaque coût, poste par poste, et ce qui a été collecté jusqu'ici : [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Soutenir sur GitHub"></a>

Vous pouvez aussi aider en [signalant des bugs](https://github.com/totaldrop-app/totaldrop/issues/new/choose), en traduisant ou en testant sur votre ordinateur.

## Pour les développeurs

### Organisation du dépôt

| Chemin | Contenu | Licence |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0 : CPace, dérivation de clés, chiffrement de la boîte aux lettres et des sessions, clés d'appareil, chiffrement par blocs STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Codes de transfert, limites, sécurité des chemins, tous les messages réseau (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Moteur client : rendez-vous, QUIC avec clés publiques brutes, Noise XXpsk3 + ML-KEM-768, transfert | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Textes d'interface des programmes Rust : négociation de la langue, messages ICU, pluriels, nombres régionaux | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Le client en ligne de commande `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Toutes les chaînes d'interface, partagées par Android, Linux et la CLI ([traduire](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Application de bureau Linux : Qt 6 et QML au-dessus de `totaldrop-core`, Wayland natif | GPL-3.0-or-later |
| `packaging/` | Paquets Linux, chaque format séparément : Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Application Android (Flutter) ; iOS plus tard | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app et la page qu'ouvrent les liens QR | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Cibles de fuzzing pour chaque analyseur | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Serveur de rendez-vous en Rust, pour l'auto-hébergement et les tests | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Le même serveur sur Cloudflare Workers, pour `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Les bibliothèques sont sous licences permissives pour que chacun puisse bâtir sur le protocole ; les applications sont sous GPL et les serveurs sous AGPL. Voir [LICENSING.md](../../LICENSING.md).

Chaque plateforme a sa propre version, son propre tag (`linux-v…`, `android-v…`) et son propre workflow de CI, de sorte que l'une peut être publiée sans toucher aux autres. Fonctionnement des branches et des versions : [docs/gelistirme.md](../../docs/gelistirme.md) et [docs/versioning.md](../../docs/versioning.md). Règles de design : [docs/tasarim.md](../../docs/tasarim.md). Décisions d'architecture : [docs/kararlar](../../docs/kararlar/README.md).

### Compilation

```bash
cargo test
cargo build --release
```

Essayez un transfert sur une seule machine, dans trois terminaux :

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Les deux côtés affichent les quatre mêmes mots de vérification. Le `ws://` non chiffré n'est accepté que pour un serveur sur la même machine ; tout le reste doit passer par `wss://`.

Le projet cible toujours la dernière version stable de Rust (fixée dans `rust-toolchain.toml`) ; rustup l'installe lors de la première compilation.

**Application Linux** (nécessite Qt 6.8 ou plus récent : `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`) :

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Contribution et sécurité

Lisez [CONTRIBUTING.md](../../CONTRIBUTING.md) avant d'ouvrir une pull request, ainsi que notre [code de conduite](../../CODE_OF_CONDUCT.md). Les questions vont dans [SUPPORT.md](../../SUPPORT.md). Signalez les vulnérabilités en privé comme décrit dans [SECURITY.md](../../SECURITY.md), jamais dans une issue publique.
