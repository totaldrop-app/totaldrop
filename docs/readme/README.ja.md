<!-- Translated from README.md 529368877e8d. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### どんなファイルでも、誰にでも、どこへでも。

無料のオープンソース。エンドツーエンドで暗号化。メールアドレスも電話番号もアカウントも不要です。

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[ウェブサイト](https://totaldrop.app)** · **[ダウンロード](https://totaldrop.app/download/)** · [ロードマップ](https://totaldrop.app/roadmap/) · [寄付](https://totaldrop.app/donate/) · [リリース](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · **日本語** · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android 版 TotalDrop：ライトとダークのホーム画面、そして受信して検証された転送" width="820">

</div>

> [!NOTE]
> この翻訳は下書きで、現在確認中です。基準となるのは [英語版](../../README.md) です。

## TotalDrop を選ぶ理由

- **短いコードがあれば十分です。** ファイルを選ぶと、`7-harbor-rocket` のようなコードが表示されます。読み上げるか QR コードを見せれば、相手がそれを入力してファイルが届きます。
- **エンドツーエンドで暗号化され、それを自分で確かめられます。** 何かが送られる前に、両方の画面に同じ4つの単語が表示されます。違っていたら、中止してください。
- **登録するものは何もありません。** アカウントもメールアドレスも電話番号も不要です。任意のアカウントは、Mullvad と同じように番号だけになる予定です（[設計](../../docs/hesap.md)）。
- **どこにいても、デバイスからデバイスへ直接。** 仲介サーバーは2台のデバイスを引き合わせるだけで、ログを残しません。どのサーバーでも同じですが、2台の IP アドレスは見えます。デバイスはインターネット越しに、可能な限り直接つながります。ルーターやファイアウォールが妨げになる場合は、中身を読めないリレーが通信を運びます。
- **お使いのシステムになじみます。** アプリは、システムのライト／ダークのテーマ、アクセントカラー、言語、動きを減らす設定に従います。Linux アプリは Wayland 上で動くネイティブの Qt 6 アプリで、KDE Plasma、Hyprland、Omarchy のテーマからフォント、角の形、アイコンもリアルタイムで取り込みます。
- **ずっと無料です。** オープンソースで、寄付によって運営されています。広告なし、追跡なし、基本機能の有料プランもありません。

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy 上の Linux アプリ。デスクトップの配色、フォント、アイコンで表示されている" width="560">
  <br><sub>Omarchy 上の Linux アプリ：配色、角の丸み、フォント、アイコンはデスクトップから取り込まれます。</sub>
</p>

## ダウンロード

| プラットフォーム | 入手 | 備考 |
| --- | --- | --- |
| **Linux**（x86-64） | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1、Qt 6.8 以降 |
| **Linux**（ARM64） | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1、Qt 6.8 以降 |
| **Android** | [ベータに参加する](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1、Android 15 以降、Google Play で招待制 |
| **コマンドライン** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux、macOS、Windows |
| **Windows、macOS、iOS** | 準備中 | 寄付でまず賄うのは、これらのコード署名です |

すべてのパッケージは、タグ付きのコミットから CI でビルドされます。開発者のマシンでビルドされることはありません。インストールする前に確認するには、リリースの `SHA256SUMS` と照合し、どこでビルドされたかを検証してください：

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **ベータ版です。** コードはまだ独立したセキュリティレビューを受けていません。機密性の高いファイルには頼らないでください。

## 仕組み

1. 送信者が仲介サーバーに短いコードを求めます。このコードが、2人が共有する唯一の秘密です。
2. 両方のデバイスが、**CPace** によってコードを強力な鍵に変えます。攻撃者が推測できるのは転送1回につき1回だけで、オフラインでコードを試すことはできません。
3. 2台は **QUIC**（ピン留めした生の公開鍵を使う TLS 1.3）で直接つながり、**ML-KEM-768 を組み合わせた Noise XXpsk3** を実行します。これは将来の量子コンピューターにも耐えられるよう設計されています。
4. ファイルはチャンク単位で暗号化される（STREAM）ため、各部分は届いた時点で認証されます。

プロトコルは *TotalDrop Protocol v0* に記述されています。実装中に下した判断は [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) に、各サーバーが何を見られて何を見られないかは [脅威モデル](../../docs/tehdit-modeli.md)（トルコ語）と [プライバシーポリシー](https://totaldrop.app/privacy) にまとめています。

## あなたの言語で

TotalDrop は35の言語に対応し、地域の数値形式に従います。英語とトルコ語は確認済みです。残りの33言語はネイティブスピーカーが確認中の下書きのため、アプリでは言語の確認が済むまで、セキュリティに関わる文言（検証、コードの誤り、承認と拒否）を英語のまま表示します。サイトとアプリは、Android と Linux の 0.3.0-beta.1 からすべての言語に対応しています。

ぎこちない文を見つけましたか？[翻訳に協力する](../../docs/ceviri.md)（ガイドはトルコ語です）

## プロジェクトを支援する

TotalDrop が無料であり続けられるのは、支えてくれる人がいるからです。寄付は、リレー、Windows と macOS が求めるコード署名証明書、そして 1.0 の前に行う独立したセキュリティ監査に使われます。すべての費用の明細と、これまでに集まった額はこちら：[totaldrop.app/donate](https://totaldrop.app/donate/)

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="GitHub でスポンサーになる"></a>

[バグの報告](https://github.com/totaldrop-app/totaldrop/issues/new/choose)、翻訳、デスクトップでのテストでも協力できます。

## 開発者向け

### リポジトリの構成

| パス | 内容 | ライセンス |
| --- | --- | --- |
| `crates/totaldrop-crypto` | スイート TD0：CPace、鍵導出、メールボックスとセッションの暗号化、デバイス鍵、STREAM によるチャンク暗号化 | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | 転送コード、制限、パスの安全性、すべての通信メッセージ（CBOR） | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | クライアントエンジン：仲介、生の公開鍵を使う QUIC、Noise XXpsk3 + ML-KEM-768、転送 | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust プログラムのインターフェース文言：言語ネゴシエーション、ICU メッセージ、複数形、地域ごとの数値 | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` コマンドラインクライアント | GPL-3.0-or-later |
| `l10n/` | Android、Linux、CLI が共有するすべてのインターフェース文字列（[翻訳について](../../docs/ceviri.md)） | GPL-3.0-or-later |
| `linux/` | Linux デスクトップアプリ：`totaldrop-core` の上に Qt 6 と QML、ネイティブ Wayland | GPL-3.0-or-later |
| `packaging/` | Linux パッケージ、形式ごとに独立：Arch（`PKGBUILD`）、Debian（`debian/`）、RPM（`.spec`） | GPL-3.0-or-later |
| `app/` | Android アプリ（Flutter）。iOS は今後 | GPL-3.0-or-later |
| `site/` | totaldrop.app、download.totaldrop.app、QR リンクが開くページ | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | すべてのパーサーのファズターゲット | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Rust で書かれた仲介サーバー。セルフホストとテスト用 | AGPL-3.0-or-later |
| `server/rendezvous-worker` | 同じサーバーを Cloudflare Workers で動かしたもの。`rendezvous.totaldrop.app` 用 | AGPL-3.0-or-later |

ライブラリは、誰でもプロトコルの上に構築できるよう寛容なライセンスにしています。アプリは GPL、サーバーは AGPL です。詳しくは [LICENSING.md](../../LICENSING.md) を参照してください。

各プラットフォームには独自のバージョン、タグ（`linux-v…`、`android-v…`）、CI ワークフローがあるため、ほかに手を加えずに1つだけをリリースできます。ブランチとリリースの運用：[docs/gelistirme.md](../../docs/gelistirme.md)（トルコ語）と [docs/versioning.md](../../docs/versioning.md)。デザインのルール：[docs/tasarim.md](../../docs/tasarim.md)、アーキテクチャの決定：[docs/kararlar](../../docs/kararlar/README.md)（いずれもトルコ語）。

### ビルド

```bash
cargo test
cargo build --release
```

1台のマシンで、3つのターミナルを使って転送を試せます：

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

両側に、同じ4つの検証用の単語が表示されます。暗号化されていない `ws://` を受け付けるのは、同じマシン上のサーバーに対してだけです。それ以外はすべて `wss://` でなければなりません。

プロジェクトは常に最新の安定版 Rust を対象にしています（`rust-toolchain.toml` で固定）。初回のビルド時に rustup がインストールします。

**Linux アプリ**（Qt 6.8 以降が必要：`qt6-base`、`qt6-declarative`、`qt6-wayland`、`qt6-svg`）：

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### コントリビューションとセキュリティ

プルリクエストを開く前に、[CONTRIBUTING.md](../../CONTRIBUTING.md) と [行動規範](../../CODE_OF_CONDUCT.md) を読んでください。質問は [SUPPORT.md](../../SUPPORT.md) へ。脆弱性は公開の issue ではなく、[SECURITY.md](../../SECURITY.md) に記載の方法で非公開で報告してください。
