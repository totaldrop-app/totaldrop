<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Qualquer ficheiro, para qualquer pessoa, em qualquer lugar.

Gratuito e de código aberto. Encriptação ponto a ponto. Sem e-mail, sem número de telefone, sem conta.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Site](https://totaldrop.app)** · **[Transferir](https://totaldrop.app/download/)** · [Planeamento](https://totaldrop.app/roadmap/) · [Fazer um donativo](https://totaldrop.app/donate/) · [Versões](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · **Português (Portugal)** · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop no Android: o ecrã inicial nos modos claro e escuro, e uma transferência que chegou e foi verificada" width="820">

</div>

> [!NOTE]
> Esta tradução é um rascunho e está em revisão. A referência é a [versão em inglês](../../README.md).

## Porquê o TotalDrop

- **Basta um código curto.** Escolha os ficheiros e obtenha um código como `7-harbor-rocket`. Leia-o em voz alta ou mostre o código QR; a outra pessoa introduz o código e os ficheiros chegam.
- **Encriptação ponto a ponto, e pode confirmá-la.** Os dois ecrãs mostram as mesmas quatro palavras antes de qualquer coisa ser enviada. Se forem diferentes, pare.
- **Sem qualquer registo.** Sem conta, e-mail ou número de telefone. As contas opcionais serão apenas um número, ao estilo da Mullvad ([conceção](../../docs/hesap.md)).
- **Diretamente de dispositivo para dispositivo, em qualquer lugar.** O servidor de encontro apenas apresenta os dois dispositivos e não guarda registos; como qualquer servidor, vê os respetivos endereços IP. Os dispositivos ligam-se diretamente através da Internet sempre que possível. Quando um router ou uma firewall o impede, um servidor de retransmissão transporta o tráfego sem o conseguir ler.
- **Em casa no seu sistema.** As aplicações seguem o tema claro ou escuro do sistema, a cor de destaque, o idioma e a definição de movimento reduzido. A aplicação para Linux é Qt 6 nativo no Wayland e também usa em tempo real os tipos de letra, os cantos e os ícones dos temas do KDE Plasma, Hyprland e Omarchy.
- **Gratuito, para sempre.** Código aberto e financiado por donativos. Sem anúncios, sem rastreio, sem plano pago para o essencial.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="A aplicação para Linux no Omarchy, com as cores, os tipos de letra e os ícones do ambiente de trabalho" width="560">
  <br><sub>A aplicação para Linux no Omarchy: cores, cantos, tipos de letra e ícones vêm do ambiente de trabalho.</sub>
</p>

## Transferir

| Plataforma | Onde obter | Notas |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 ou mais recente |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 ou mais recente |
| **Android** | [Aderir à beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 ou mais recente, Google Play por convite |
| **Linha de comandos** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | A caminho | A assinatura de código para estas plataformas é a primeira coisa que os donativos pagam |

Cada pacote é compilado pela CI a partir de um commit com etiqueta, nunca no computador de um programador. Para confirmar um pacote antes de o instalar, compare-o com o `SHA256SUMS` da versão e verifique onde foi compilado:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** O código ainda não foi alvo de uma revisão de segurança independente, pelo que não deve confiar nele para ficheiros sensíveis.

## Como funciona

1. O remetente pede um código curto ao servidor de encontro. O código é o único segredo que as duas pessoas partilham.
2. Os dois dispositivos transformam o código numa chave forte com **CPace**. Um atacante tem uma única tentativa por transferência e não consegue testar códigos offline.
3. Ligam-se diretamente através de **QUIC** (TLS 1.3 com chaves públicas em bruto fixadas) e executam **Noise XXpsk3 com ML-KEM-768**, que foi concebido para resistir a futuros computadores quânticos.
4. Os ficheiros são encriptados em blocos (STREAM), pelo que cada parte é autenticada à chegada.

O protocolo está descrito em *TotalDrop Protocol v0*. As decisões tomadas durante a implementação estão em [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), e o que cada servidor pode e não pode ver está no [modelo de ameaças](../../docs/tehdit-modeli.md) (em turco) e na [política de privacidade](https://totaldrop.app/privacy).

## No seu idioma

O TotalDrop fala 35 idiomas e segue os formatos numéricos da sua região. O inglês e o turco estão revistos; os outros 33 são rascunhos que falantes nativos estão a verificar, pelo que as aplicações mantêm os textos de segurança (verificação, código errado, aceitar e recusar) em inglês até que o idioma seja revisto. O site já os tem todos; as aplicações vão recebê-los nas próximas versões.

Encontrou uma frase pouco natural? [Ajude a traduzir](../../docs/ceviri.md) (o guia está em turco).

## Apoie o projeto

O TotalDrop continua gratuito porque as pessoas contribuem. Os donativos pagam o servidor de retransmissão, os certificados de assinatura de código que o Windows e o macOS exigem e uma auditoria de segurança independente antes da 1.0. Cada custo, rubrica a rubrica, e o que já foi angariado: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Patrocinar no GitHub"></a>

Também pode ajudar [comunicando erros](https://github.com/totaldrop-app/totaldrop/issues/new/choose), traduzindo ou testando no seu computador.

## Para programadores

### Estrutura do repositório

| Caminho | O que é | Licença |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, derivação de chaves, encriptação da caixa de correio e da sessão, chaves do dispositivo, encriptação em blocos STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Códigos de transferência, limites, segurança de caminhos, todas as mensagens de rede (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Motor do cliente: encontro, QUIC com chaves públicas em bruto, Noise XXpsk3 + ML-KEM-768, transferência | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Textos da interface para os programas em Rust: negociação de idioma, mensagens ICU, plurais, números regionais | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | O cliente de linha de comandos `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Todos os textos da interface, partilhados pelo Android, pelo Linux e pela CLI ([tradução](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Aplicação de ambiente de trabalho para Linux: Qt 6 e QML sobre o `totaldrop-core`, Wayland nativo | GPL-3.0-or-later |
| `packaging/` | Pacotes para Linux, cada formato à parte: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Aplicação para Android (Flutter); iOS mais tarde | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app e a página que as ligações QR abrem | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Alvos de fuzzing para cada parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Servidor de encontro em Rust, para alojamento próprio e testes | AGPL-3.0-or-later |
| `server/rendezvous-worker` | O mesmo servidor no Cloudflare Workers, para `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

As bibliotecas têm licenças permissivas para que qualquer pessoa possa construir sobre o protocolo; as aplicações são GPL e os servidores, AGPL. Consulte [LICENSING.md](../../LICENSING.md).

Cada plataforma tem a sua própria versão, etiqueta (`linux-v…`, `android-v…`) e workflow de CI, pelo que uma pode ser lançada sem mexer nas outras. Como funcionam os branches e as versões: [docs/gelistirme.md](../../docs/gelistirme.md) (em turco) e [docs/versioning.md](../../docs/versioning.md). Regras de design: [docs/tasarim.md](../../docs/tasarim.md) e decisões de arquitetura: [docs/kararlar](../../docs/kararlar/README.md), ambas em turco.

### Compilação

```bash
cargo test
cargo build --release
```

Experimente uma transferência num único computador, em três terminais:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Os dois lados mostram as mesmas quatro palavras de verificação. O `ws://` sem encriptação só é aceite para um servidor no mesmo computador; tudo o resto tem de ser `wss://`.

O projeto usa sempre a versão estável mais recente do Rust (fixada em `rust-toolchain.toml`); o rustup instala-a na primeira compilação.

**Aplicação para Linux** (requer Qt 6.8 ou mais recente: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Contribuições e segurança

Leia o [CONTRIBUTING.md](../../CONTRIBUTING.md) antes de abrir um pull request, bem como o nosso [código de conduta](../../CODE_OF_CONDUCT.md). As dúvidas devem ser colocadas no [SUPPORT.md](../../SUPPORT.md). Comunique vulnerabilidades em privado, como descrito no [SECURITY.md](../../SECURITY.md), e nunca num issue público.
