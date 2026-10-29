<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### 任何檔案，傳給任何人，送到任何地方。

免費且開放原始碼。端對端加密。不需要電子郵件、手機號碼或帳號。

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[網站](https://totaldrop.app)** · **[下載](https://totaldrop.app/download/)** · [開發藍圖](https://totaldrop.app/roadmap/) · [捐款](https://totaldrop.app/donate/) · [發行版本](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · **繁體中文**</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android 上的 TotalDrop：淺色與深色模式的主畫面，以及一次已送達並通過驗證的傳輸" width="820">

</div>

> [!NOTE]
> 此翻譯為草稿，正在審校。請以 [英文版](../../README.md) 為準。

## 為什麼選擇 TotalDrop

- **只需要一組簡短的代碼。** 選好檔案後，你會拿到一組類似 `7-harbor-rocket` 的代碼。把它念給對方，或出示 QR 碼；對方輸入代碼後，檔案就會送達。
- **端對端加密，而且你可以親自核對。** 在傳輸任何內容之前，兩個螢幕會顯示同樣的四個單字。如果不一致，請停止。
- **不需要註冊任何東西。** 沒有帳號、電子郵件或手機號碼。日後的選用帳號將像 Mullvad 一樣只使用編號（[設計](../../docs/hesap.md)）。
- **裝置直連，身在何處都可以。** 中介伺服器只負責為兩部裝置牽線，不保留記錄。裝置會盡可能透過網際網路直接連線。如果路由器或防火牆阻擋，流量會由中繼伺服器轉送，而中繼伺服器無法讀取其內容。
- **在每個系統上都自然融入。** 每個應用程式都會遵循所在系統的主題、字型、強調色、語言和動態效果設定。Linux 應用程式是在 Wayland 上執行的原生 Qt 6 應用程式，並即時跟隨 KDE Plasma、Hyprland 和 Omarchy 的主題。
- **永久免費。** 開放原始碼，由捐款支持。沒有廣告，沒有追蹤，基本功能也沒有付費版。

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy 上的 Linux 應用程式，使用桌面的配色、字型和圖示" width="560">
  <br><sub>Omarchy 上的 Linux 應用程式：配色、圓角、字型和圖示都來自桌面。</sub>
</p>

## 下載

| 平台 | 取得 | 說明 |
| --- | --- | --- |
| **Linux**（x86-64） | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1，Qt 6.8 或更新版本 |
| **Linux**（ARM64） | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1，Qt 6.8 或更新版本 |
| **Android** | [加入測試版](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2，Android 15 或更新版本，透過邀請在 Google Play 上提供 |
| **命令列** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux、macOS、Windows |
| **Windows、macOS、iOS** | 即將推出 | 捐款會優先用於這些平台的程式碼簽署 |

每個套件都由 CI 從加上標籤的提交建置，絕不在開發者的電腦上建置。若要在安裝前檢查，請將它與發行版本的 `SHA256SUMS` 比對，並驗證它的建置來源：

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **測試版。** 程式碼尚未經過獨立的安全審查，因此請不要依賴它傳輸敏感檔案。

## 運作方式

1. 傳送方向中介伺服器要求一組簡短的代碼。這組代碼是雙方共享的唯一祕密。
2. 兩部裝置用 **CPace** 將代碼轉換為強金鑰。攻擊者每次傳輸只有一次猜測機會，也無法離線測試代碼。
3. 兩部裝置透過 **QUIC**（使用固定原始公開金鑰的 TLS 1.3）直接連線，並執行 **結合 ML-KEM-768 的 Noise XXpsk3**，即使面對未來的量子電腦也依然安全。
4. 檔案以區塊為單位加密（STREAM），因此每個區塊在抵達時都會經過驗證。

協定記載於 *TotalDrop Protocol v0*。實作過程中做出的決定請見 [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md)，每部伺服器能看到什麼、不能看到什麼，請見 [威脅模型](../../docs/tehdit-modeli.md) 和 [隱私權政策](https://totaldrop.app/privacy)。

## 使用你的語言

TotalDrop 支援 35 種語言，並依照你所在地區的數字格式。英文和土耳其文已經過審校；其餘 33 種語言是母語人士正在檢查的草稿，因此在某個語言審校完成之前，應用程式會將安全相關的文字（驗證、代碼錯誤、接受和拒絕）保留為英文。網站現已提供所有語言；應用程式會在下一個版本中加入。

發現了不通順的句子嗎？[協助翻譯](../../docs/ceviri.md)。

## 支持本專案

TotalDrop 能保持免費，是因為大家出手相助。捐款用於支付中繼伺服器、Windows 和 macOS 要求的程式碼簽署憑證，以及 1.0 之前的獨立安全稽核。每一項費用的明細，以及目前已募得的金額：[totaldrop.app/donate](https://totaldrop.app/donate/)。

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="在 GitHub 上贊助"></a>

你也可以透過 [回報錯誤](https://github.com/totaldrop-app/totaldrop/issues/new/choose)、翻譯，或在桌面上測試來提供協助。

## 開發者資訊

### 儲存庫結構

| 路徑 | 說明 | 授權 |
| --- | --- | --- |
| `crates/totaldrop-crypto` | 套組 TD0：CPace、金鑰衍生、信箱與工作階段加密、裝置金鑰、STREAM 區塊加密 | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | 傳輸代碼、限制、路徑安全、所有通訊訊息（CBOR） | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | 用戶端引擎：中介連線、使用原始公開金鑰的 QUIC、Noise XXpsk3 + ML-KEM-768、傳輸 | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust 程式的介面文字：語言協商、ICU 訊息、複數形式、地區數字格式 | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` 命令列用戶端 | GPL-3.0-or-later |
| `l10n/` | 所有介面字串，由 Android、Linux 和 CLI 共用（[翻譯說明](../../docs/ceviri.md)） | GPL-3.0-or-later |
| `linux/` | Linux 桌面應用程式：建立在 `totaldrop-core` 之上的 Qt 6 和 QML，原生 Wayland | GPL-3.0-or-later |
| `packaging/` | Linux 套件，每種格式各自獨立：Arch（`PKGBUILD`）、Debian（`debian/`）、RPM（`.spec`） | GPL-3.0-or-later |
| `app/` | Android 應用程式（Flutter）；iOS 之後推出 | GPL-3.0-or-later |
| `site/` | totaldrop.app、download.totaldrop.app，以及 QR 碼連結開啟的頁面 | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | 針對每個剖析器的模糊測試目標 | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | 以 Rust 撰寫的中介伺服器，用於自行架設和測試 | AGPL-3.0-or-later |
| `server/rendezvous-worker` | 在 Cloudflare Workers 上執行的同一個伺服器，用於 `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

程式庫採用寬鬆授權，讓任何人都能以這個協定為基礎進行開發；應用程式採用 GPL，伺服器採用 AGPL。詳見 [LICENSING.md](../../LICENSING.md)。

每個平台都有自己的版本、標籤（`linux-v…`、`android-v…`）和 CI 工作流程，因此一個平台可以單獨發布，而不影響其他平台。分支和發布的流程：[docs/gelistirme.md](../../docs/gelistirme.md) 和 [docs/versioning.md](../../docs/versioning.md)。設計規則：[docs/tasarim.md](../../docs/tasarim.md)。架構決策：[docs/kararlar](../../docs/kararlar/README.md)。

### 建置

```bash
cargo test
cargo build --release
```

在一部電腦上用三個終端機試一次傳輸：

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

雙方會顯示同樣的四個驗證單字。未加密的 `ws://` 只允許用於同一部電腦上的伺服器；其他情況都必須使用 `wss://`。

本專案一律以最新的穩定版 Rust 為目標（固定在 `rust-toolchain.toml` 中）；第一次建置時 rustup 會安裝它。

**Linux 應用程式**（需要 Qt 6.8 或更新版本：`qt6-base`、`qt6-declarative`、`qt6-wayland`、`qt6-svg`）：

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### 貢獻與安全

提出拉取請求之前，請閱讀 [CONTRIBUTING.md](../../CONTRIBUTING.md) 和我們的 [行為準則](../../CODE_OF_CONDUCT.md)。有問題請參閱 [SUPPORT.md](../../SUPPORT.md)。請依照 [SECURITY.md](../../SECURITY.md) 中的說明私下回報漏洞，切勿在公開的 issue 中回報。
