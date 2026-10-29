<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Qualquer arquivo, para qualquer pessoa, em qualquer lugar.

Gratuito e de código aberto. Criptografia de ponta a ponta. Sem e-mail, sem número de telefone, sem conta.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Site](https://totaldrop.app)** · **[Baixar](https://totaldrop.app/download/)** · [Planejamento](https://totaldrop.app/roadmap/) · [Doar](https://totaldrop.app/donate/) · [Versões](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · **Português (Brasil)** · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop no Android: a tela inicial nos modos claro e escuro, e uma transferência que chegou e foi verificada" width="820">

</div>

> [!NOTE]
> Esta tradução é um rascunho e está em revisão. A referência é a [versão em inglês](../../README.md).

## Por que o TotalDrop

- **Basta um código curto.** Escolha os arquivos e receba um código como `7-harbor-rocket`. Leia em voz alta ou mostre o QR code; a outra pessoa digita o código e os arquivos chegam.
- **Criptografia de ponta a ponta, e você pode conferir.** As duas telas mostram as mesmas quatro palavras antes de qualquer coisa ser enviada. Se forem diferentes, pare.
- **Nada para cadastrar.** Sem conta, e-mail ou número de telefone. As contas opcionais serão apenas um número, no estilo Mullvad ([projeto](../../docs/hesap.md)).
- **Direto de dispositivo para dispositivo, em qualquer lugar.** O servidor de encontro só apresenta os dois dispositivos e não guarda registros. Os dispositivos se conectam diretamente pela internet sempre que possível. Quando um roteador ou firewall atrapalha, um servidor de retransmissão transporta o tráfego sem conseguir lê-lo.
- **Em casa em qualquer sistema.** Cada app segue o tema, as fontes, a cor de destaque, o idioma e as configurações de movimento do seu sistema. O app para Linux é Qt 6 nativo no Wayland e acompanha ao vivo os temas do KDE Plasma, Hyprland e Omarchy.
- **Gratuito, para sempre.** Código aberto e mantido por doações. Sem anúncios, sem rastreamento, sem plano pago para o básico.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="O app para Linux no Omarchy, com as cores, as fontes e os ícones do desktop" width="560">
  <br><sub>O app para Linux no Omarchy: cores, cantos, fontes e ícones vêm do desktop.</sub>
</p>

## Baixar

| Plataforma | Onde baixar | Observações |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 ou mais recente |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 ou mais recente |
| **Android** | [Entre no beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 ou mais recente, Google Play por convite |
| **Linha de comando** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | A caminho | A assinatura de código para essas plataformas é a primeira coisa que as doações vão pagar |

Cada pacote é compilado pela CI a partir de um commit com tag, nunca na máquina de um desenvolvedor. Para conferir um pacote antes de instalá-lo, compare-o com o `SHA256SUMS` da versão e verifique onde ele foi compilado:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** O código ainda não passou por uma revisão de segurança independente, então não confie nele para arquivos sensíveis.

## Como funciona

1. O remetente pede um código curto ao servidor de encontro. O código é o único segredo que as duas pessoas compartilham.
2. Os dois dispositivos transformam o código em uma chave forte com **CPace**. Um atacante tem uma única tentativa por transferência e não consegue testar códigos offline.
3. Eles se conectam diretamente via **QUIC** (TLS 1.3 com chaves públicas brutas fixadas) e executam **Noise XXpsk3 com ML-KEM-768**, que continua seguro contra futuros computadores quânticos.
4. Os arquivos são criptografados em blocos (STREAM), então cada parte é autenticada assim que chega.

O protocolo está descrito em *TotalDrop Protocol v0*. As decisões tomadas durante a implementação estão em [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), e o que cada servidor pode e não pode ver está no [modelo de ameaças](../../docs/tehdit-modeli.md) e na [política de privacidade](https://totaldrop.app/privacy).

## No seu idioma

O TotalDrop fala 35 idiomas e segue os formatos numéricos da sua região. Inglês e turco estão revisados; os outros 33 são rascunhos que falantes nativos estão conferindo, por isso os apps mantêm os textos de segurança (verificação, código errado, aceitar e recusar) em inglês até que o idioma seja revisado. O site já tem todos hoje; os apps vão recebê-los nas próximas versões.

Encontrou uma frase estranha? [Ajude a traduzir](../../docs/ceviri.md).

## Apoie o projeto

O TotalDrop continua gratuito porque as pessoas contribuem. As doações pagam o servidor de retransmissão, os certificados de assinatura de código que o Windows e o macOS exigem e uma auditoria de segurança independente antes da 1.0. Cada custo, item por item, e quanto já foi arrecadado: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Patrocinar no GitHub"></a>

Você também pode ajudar [relatando bugs](https://github.com/totaldrop-app/totaldrop/issues/new/choose), traduzindo ou testando no seu desktop.

## Para desenvolvedores

### Estrutura do repositório

| Caminho | O que é | Licença |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, derivação de chaves, criptografia da caixa postal e da sessão, chaves do dispositivo, criptografia em blocos STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Códigos de transferência, limites, segurança de caminhos, todas as mensagens de rede (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Motor do cliente: encontro, QUIC com chaves públicas brutas, Noise XXpsk3 + ML-KEM-768, transferência | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Textos da interface para os programas em Rust: negociação de idioma, mensagens ICU, plurais, números regionais | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | O cliente de linha de comando `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Todos os textos da interface, compartilhados por Android, Linux e a CLI ([tradução](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | App de desktop para Linux: Qt 6 e QML sobre o `totaldrop-core`, Wayland nativo | GPL-3.0-or-later |
| `packaging/` | Pacotes para Linux, cada formato separado: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | App para Android (Flutter); iOS depois | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app e a página que os links QR abrem | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Alvos de fuzzing para cada parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Servidor de encontro em Rust, para auto-hospedagem e testes | AGPL-3.0-or-later |
| `server/rendezvous-worker` | O mesmo servidor no Cloudflare Workers, para `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

As bibliotecas têm licenças permissivas para que qualquer pessoa possa construir sobre o protocolo; os apps são GPL e os servidores, AGPL. Veja [LICENSING.md](../../LICENSING.md).

Cada plataforma tem sua própria versão, tag (`linux-v…`, `android-v…`) e workflow de CI, então uma pode ser lançada sem mexer nas outras. Como funcionam branches e versões: [docs/gelistirme.md](../../docs/gelistirme.md) e [docs/versioning.md](../../docs/versioning.md). Regras de design: [docs/tasarim.md](../../docs/tasarim.md). Decisões de arquitetura: [docs/kararlar](../../docs/kararlar/README.md).

### Compilação

```bash
cargo test
cargo build --release
```

Teste uma transferência em uma única máquina, com três terminais:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Os dois lados mostram as mesmas quatro palavras de verificação. O `ws://` sem criptografia só é aceito para um servidor na mesma máquina; todo o resto precisa ser `wss://`.

O projeto sempre usa a versão estável mais recente do Rust (fixada em `rust-toolchain.toml`); o rustup a instala na primeira compilação.

**App para Linux** (requer Qt 6.8 ou mais recente: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Contribuição e segurança

Leia o [CONTRIBUTING.md](../../CONTRIBUTING.md) antes de abrir um pull request, e também nosso [código de conduta](../../CODE_OF_CONDUCT.md). Dúvidas vão para o [SUPPORT.md](../../SUPPORT.md). Informe vulnerabilidades de forma privada, como descrito no [SECURITY.md](../../SECURITY.md), nunca em uma issue pública.
