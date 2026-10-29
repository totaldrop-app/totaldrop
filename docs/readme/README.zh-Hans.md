<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### 任何文件，发给任何人，送到任何地方。

免费开源。端到端加密。无需邮箱、手机号码或账号。

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[网站](https://totaldrop.app)** · **[下载](https://totaldrop.app/download/)** · [路线图](https://totaldrop.app/roadmap/) · [捐赠](https://totaldrop.app/donate/) · [发行版](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · **简体中文** · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android 上的 TotalDrop：浅色和深色模式下的主屏幕，以及一次已送达并通过验证的传输" width="820">

</div>

> [!NOTE]
> 此翻译为草稿，正在审校。请以 [英文版](../../README.md) 为准。

## 为什么选择 TotalDrop

- **只需一个简短的代码。** 选择文件后，你会得到一个类似 `7-harbor-rocket` 的代码。把它念给对方，或出示二维码；对方输入代码后，文件就会送达。
- **端到端加密，而且你可以亲自核对。** 在传输任何内容之前，两块屏幕会显示同样的四个单词。如果不一致，请停止。
- **无需注册任何东西。** 没有账号、邮箱或手机号码。日后的可选账号将像 Mullvad 一样只使用编号（[设计](../../docs/hesap.md)）。
- **设备直连，身在何处都可以。** 中介服务器只负责为两台设备牵线，不保留日志。设备会尽可能通过互联网直接连接。如果路由器或防火墙阻挡，流量会由中继服务器转发，而中继无法读取其内容。
- **在每个系统上都自然融入。** 每个应用都遵循所在系统的主题、字体、强调色、语言和动画设置。Linux 应用是运行在 Wayland 上的原生 Qt 6 应用，并实时跟随 KDE Plasma、Hyprland 和 Omarchy 的主题。
- **永久免费。** 开源，由捐赠支持。没有广告，没有追踪，基本功能也没有付费版。

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy 上的 Linux 应用，使用桌面的配色、字体和图标" width="560">
  <br><sub>Omarchy 上的 Linux 应用：配色、圆角、字体和图标都来自桌面。</sub>
</p>

## 下载

| 平台 | 获取 | 说明 |
| --- | --- | --- |
| **Linux**（x86-64） | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1，Qt 6.8 或更高版本 |
| **Linux**（ARM64） | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1，Qt 6.8 或更高版本 |
| **Android** | [加入测试版](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2，Android 15 或更高版本，通过邀请在 Google Play 上提供 |
| **命令行** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux、macOS、Windows |
| **Windows、macOS、iOS** | 即将推出 | 捐款首先用于这些平台的代码签名 |

每个软件包都由 CI 从带标签的提交构建，绝不在开发者的机器上构建。如需在安装前检查，请将其与发行版的 `SHA256SUMS` 比对，并验证它的构建来源：

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **测试版。** 代码尚未经过独立的安全审查，因此请不要依赖它传输敏感文件。

## 工作原理

1. 发送方向中介服务器请求一个简短的代码。这个代码是双方共享的唯一秘密。
2. 两台设备用 **CPace** 将代码转换为强密钥。攻击者每次传输只有一次猜测机会，也无法离线测试代码。
3. 两台设备通过 **QUIC**（使用固定原始公钥的 TLS 1.3）直接连接，并运行 **结合 ML-KEM-768 的 Noise XXpsk3**，即使面对未来的量子计算机也依然安全。
4. 文件按块加密（STREAM），因此每一块在到达时都会经过认证。

协议记载于 *TotalDrop Protocol v0*。实现过程中做出的决定见 [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md)，每台服务器能看到什么、不能看到什么，见 [威胁模型](../../docs/tehdit-modeli.md) 和 [隐私政策](https://totaldrop.app/privacy)。

## 使用你的语言

TotalDrop 支持 35 种语言，并遵循你所在地区的数字格式。英语和土耳其语已经过审校；其余 33 种语言是母语者正在检查的草稿，因此在某种语言审校完成之前，应用会将安全相关的文字（验证、代码错误、接受和拒绝）保留为英文。网站现已提供全部语言；应用将在下一个版本中加入。

发现了别扭的句子？[帮助翻译](../../docs/ceviri.md)。

## 支持本项目

TotalDrop 能保持免费，是因为大家出手相助。捐款用于支付中继服务器、Windows 和 macOS 要求的代码签名证书，以及 1.0 之前的独立安全审计。每一项费用的明细，以及目前已筹得的金额：[totaldrop.app/donate](https://totaldrop.app/donate/)。

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="在 GitHub 上赞助"></a>

你也可以通过 [报告错误](https://github.com/totaldrop-app/totaldrop/issues/new/choose)、翻译或在桌面上测试来提供帮助。

## 面向开发者

### 仓库结构

| 路径 | 说明 | 许可证 |
| --- | --- | --- |
| `crates/totaldrop-crypto` | 套件 TD0：CPace、密钥派生、信箱与会话加密、设备密钥、STREAM 分块加密 | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | 传输代码、限制、路径安全、所有通信消息（CBOR） | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | 客户端引擎：中介连接、使用原始公钥的 QUIC、Noise XXpsk3 + ML-KEM-768、传输 | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust 程序的界面文字：语言协商、ICU 消息、复数形式、地区数字格式 | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` 命令行客户端 | GPL-3.0-or-later |
| `l10n/` | 所有界面字符串，由 Android、Linux 和 CLI 共享（[翻译说明](../../docs/ceviri.md)） | GPL-3.0-or-later |
| `linux/` | Linux 桌面应用：基于 `totaldrop-core` 的 Qt 6 和 QML，原生 Wayland | GPL-3.0-or-later |
| `packaging/` | Linux 软件包，每种格式各自独立：Arch（`PKGBUILD`）、Debian（`debian/`）、RPM（`.spec`） | GPL-3.0-or-later |
| `app/` | Android 应用（Flutter）；iOS 以后推出 | GPL-3.0-or-later |
| `site/` | totaldrop.app、download.totaldrop.app，以及二维码链接打开的页面 | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | 针对每个解析器的模糊测试目标 | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | 用 Rust 编写的中介服务器，用于自托管和测试 | AGPL-3.0-or-later |
| `server/rendezvous-worker` | 运行在 Cloudflare Workers 上的同一服务器，用于 `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

这些库采用宽松许可证，让任何人都能基于该协议进行开发；应用采用 GPL，服务器采用 AGPL。详见 [LICENSING.md](../../LICENSING.md)。

每个平台都有自己的版本、标签（`linux-v…`、`android-v…`）和 CI 工作流，因此一个平台可以单独发布，而不影响其他平台。分支和发布的流程：[docs/gelistirme.md](../../docs/gelistirme.md) 和 [docs/versioning.md](../../docs/versioning.md)。设计规则：[docs/tasarim.md](../../docs/tasarim.md)。架构决策：[docs/kararlar](../../docs/kararlar/README.md)。

### 构建

```bash
cargo test
cargo build --release
```

在一台机器上用三个终端试一次传输：

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

双方会显示同样的四个验证单词。未加密的 `ws://` 仅允许用于同一台机器上的服务器；其他情况必须使用 `wss://`。

本项目始终以最新的稳定版 Rust 为目标（固定在 `rust-toolchain.toml` 中）；首次构建时 rustup 会安装它。

**Linux 应用**（需要 Qt 6.8 或更高版本：`qt6-base`、`qt6-declarative`、`qt6-wayland`、`qt6-svg`）：

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### 贡献与安全

提交拉取请求之前，请阅读 [CONTRIBUTING.md](../../CONTRIBUTING.md) 和我们的 [行为准则](../../CODE_OF_CONDUCT.md)。有问题请参阅 [SUPPORT.md](../../SUPPORT.md)。请按照 [SECURITY.md](../../SECURITY.md) 中的说明私下报告漏洞，切勿在公开 issue 中报告。
