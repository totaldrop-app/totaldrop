<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### 어떤 파일이든, 누구에게든, 어디로든.

무료 오픈 소스. 종단간 암호화. 이메일도, 전화번호도, 계정도 필요 없습니다.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[웹사이트](https://totaldrop.app)** · **[다운로드](https://totaldrop.app/download/)** · [로드맵](https://totaldrop.app/roadmap/) · [후원](https://totaldrop.app/donate/) · [릴리스](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · **한국어** · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android의 TotalDrop: 라이트 모드와 다크 모드의 홈 화면, 그리고 도착해서 검증된 전송" width="820">

</div>

> [!NOTE]
> 이 번역은 초안이며 검토 중입니다. 기준이 되는 원문: [영어판](../../README.md)

## TotalDrop을 쓰는 이유

- **짧은 코드 하나면 됩니다.** 파일을 고르면 `7-harbor-rocket` 같은 코드가 나옵니다. 코드를 읽어 주거나 QR 코드를 보여 주면, 상대가 그 코드를 입력하고 파일이 도착합니다.
- **종단간 암호화되며, 직접 확인할 수 있습니다.** 무엇이든 전송되기 전에 두 화면에 같은 네 단어가 표시됩니다. 다르면 중단하세요.
- **가입할 것이 없습니다.** 계정도, 이메일도, 전화번호도 필요 없습니다. 선택형 계정이 생기더라도 Mullvad처럼 번호만 사용합니다 ([설계](../../docs/hesap.md)).
- **어디서든 기기에서 기기로 바로.** 중개 서버는 두 기기를 서로 소개할 뿐이며 로그를 남기지 않습니다. 다른 모든 서버와 마찬가지로 두 기기의 IP 주소는 볼 수 있습니다. 기기들은 가능한 한 인터넷을 통해 직접 연결됩니다. 공유기나 방화벽이 가로막으면, 내용을 읽을 수 없는 릴레이가 트래픽을 전달합니다.
- **사용 중인 시스템에서 자연스럽게.** 앱은 시스템의 라이트 또는 다크 테마, 강조 색상, 언어, 동작 줄이기 설정을 따릅니다. Linux 앱은 Wayland에서 동작하는 네이티브 Qt 6 앱이며, KDE Plasma, Hyprland, Omarchy 테마에서 글꼴, 모서리, 아이콘도 실시간으로 가져옵니다.
- **영원히 무료.** 오픈 소스이며 후원으로 운영됩니다. 광고도, 추적도, 기본 기능을 위한 유료 등급도 없습니다.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy에서 데스크톱의 색상, 글꼴, 아이콘으로 표시된 Linux 앱" width="560">
  <br><sub>Omarchy의 Linux 앱: 색상, 모서리, 글꼴, 아이콘을 데스크톱에서 가져옵니다.</sub>
</p>

## 다운로드

| 플랫폼 | 받기 | 참고 |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 이상 |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 이상 |
| **Android** | [베타 참여하기](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 이상, Google Play 초대제 |
| **명령줄** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | 준비 중 | 후원금은 가장 먼저 이 플랫폼들의 코드 서명에 쓰입니다 |

모든 패키지는 개발자의 컴퓨터가 아니라, 태그가 붙은 커밋에서 CI가 빌드합니다. 설치하기 전에 확인하려면 릴리스의 `SHA256SUMS`와 비교하고, 어디서 빌드되었는지 검증하세요.

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **베타입니다.** 코드는 아직 독립적인 보안 검토를 받지 않았으므로, 민감한 파일에는 의존하지 마세요.

## 작동 방식

1. 보내는 사람이 중개 서버에 짧은 코드를 요청합니다. 이 코드가 두 사람이 공유하는 유일한 비밀입니다.
2. 두 기기는 **CPace**로 코드를 강력한 키로 바꿉니다. 공격자는 전송 한 번에 한 번만 추측할 수 있고, 오프라인에서 코드를 시험해 볼 수 없습니다.
3. 두 기기는 **QUIC**(고정된 원시 공개 키를 사용하는 TLS 1.3)으로 직접 연결하고, 미래의 양자 컴퓨터에도 견디도록 설계된 **ML-KEM-768을 더한 Noise XXpsk3**를 실행합니다.
4. 파일은 청크 단위로 암호화되므로(STREAM), 각 조각은 도착하는 즉시 인증됩니다.

프로토콜은 *TotalDrop Protocol v0* 문서에 설명되어 있습니다. 구현하면서 내린 결정은 [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) 문서에 있고, 각 서버가 볼 수 있는 것과 볼 수 없는 것은 [위협 모델](../../docs/tehdit-modeli.md) (터키어) 및 [개인정보 처리방침](https://totaldrop.app/privacy) 문서에 나와 있습니다.

## 사용자의 언어로

TotalDrop은 35개 언어를 지원하며 사용자 지역의 숫자 형식을 따릅니다. 영어와 터키어는 검토를 마쳤고, 나머지 33개 언어는 원어민이 확인 중인 초안입니다. 그래서 앱은 해당 언어의 검토가 끝날 때까지 보안 관련 문구(검증, 잘못된 코드, 수락과 거절)를 영어로 유지합니다. 사이트에는 지금 모든 언어가 있으며, 앱에는 다음 릴리스부터 들어갑니다.

어색한 문장을 발견하셨나요? [번역 돕기](../../docs/ceviri.md) (안내서는 터키어로 되어 있습니다)

## 프로젝트 후원하기

TotalDrop이 무료로 유지되는 것은 사람들이 힘을 보태 주기 때문입니다. 후원금은 릴레이, Windows와 macOS에 필요한 코드 서명 인증서, 그리고 1.0 이전의 독립 보안 감사에 쓰입니다. 모든 비용의 항목별 내역과 지금까지 모인 금액: [totaldrop.app/donate](https://totaldrop.app/donate/)

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="GitHub에서 후원하기"></a>

[버그 신고](https://github.com/totaldrop-app/totaldrop/issues/new/choose), 번역, 데스크톱에서의 테스트로도 도울 수 있습니다.

## 개발자를 위한 정보

### 저장소 구성

| 경로 | 설명 | 라이선스 |
| --- | --- | --- |
| `crates/totaldrop-crypto` | 스위트 TD0: CPace, 키 유도, 메일박스 및 세션 암호화, 기기 키, STREAM 청크 암호화 | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | 전송 코드, 제한, 경로 안전성, 모든 통신 메시지(CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | 클라이언트 엔진: 중개, 원시 공개 키를 사용하는 QUIC, Noise XXpsk3 + ML-KEM-768, 전송 | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust 프로그램의 인터페이스 텍스트: 언어 협상, ICU 메시지, 복수형, 지역별 숫자 | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` 명령줄 클라이언트 | GPL-3.0-or-later |
| `l10n/` | Android, Linux, CLI가 공유하는 모든 인터페이스 문자열 ([번역하기](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux 데스크톱 앱: `totaldrop-core` 위의 Qt 6와 QML, 네이티브 Wayland | GPL-3.0-or-later |
| `packaging/` | Linux 패키지, 형식마다 따로: Arch(`PKGBUILD`), Debian(`debian/`), RPM(`.spec`) | GPL-3.0-or-later |
| `app/` | Android 앱(Flutter), iOS는 나중에 | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app, 그리고 QR 링크가 여는 페이지 | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | 모든 파서를 위한 퍼즈 타깃 | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Rust로 만든 중개 서버, 자체 호스팅과 테스트용 | AGPL-3.0-or-later |
| `server/rendezvous-worker` | 같은 서버를 Cloudflare Workers에서 실행한 것, `rendezvous.totaldrop.app`용 | AGPL-3.0-or-later |

라이브러리는 누구나 프로토콜 위에 무언가를 만들 수 있도록 관대한 라이선스를 쓰고, 앱은 GPL, 서버는 AGPL입니다. 자세한 내용: [LICENSING.md](../../LICENSING.md)

각 플랫폼은 자체 버전, 태그(`linux-v…`, `android-v…`), CI 워크플로를 가지므로, 하나를 출시할 때 다른 플랫폼을 건드리지 않아도 됩니다. 브랜치와 릴리스 운영 방식: [docs/gelistirme.md](../../docs/gelistirme.md) (터키어), [docs/versioning.md](../../docs/versioning.md). 디자인 규칙: [docs/tasarim.md](../../docs/tasarim.md), 아키텍처 결정: [docs/kararlar](../../docs/kararlar/README.md) (둘 다 터키어).

### 빌드

```bash
cargo test
cargo build --release
```

한 대의 컴퓨터에서 터미널 세 개로 전송을 시험해 보세요.

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

양쪽에 같은 네 개의 검증 단어가 표시됩니다. 암호화되지 않은 `ws://`는 같은 컴퓨터의 서버에 대해서만 허용되며, 그 밖에는 모두 `wss://`여야 합니다.

프로젝트는 항상 최신 안정 버전의 Rust를 대상으로 하며(`rust-toolchain.toml`에 고정), 첫 빌드 때 rustup이 설치합니다.

**Linux 앱**(Qt 6.8 이상 필요: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### 기여와 보안

풀 리퀘스트를 열기 전에 [CONTRIBUTING.md](../../CONTRIBUTING.md) 및 [행동 강령](../../CODE_OF_CONDUCT.md) 문서를 읽어 주세요. 질문은 [SUPPORT.md](../../SUPPORT.md) 문서를 참고하세요. 취약점은 공개 이슈로 올리지 말고, [SECURITY.md](../../SECURITY.md) 문서에 설명된 대로 비공개로 신고해 주세요.
