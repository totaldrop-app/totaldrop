<!-- Translated from README.md 529368877e8d. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Mọi tệp, cho bất kỳ ai, ở bất cứ đâu.

Miễn phí và mã nguồn mở. Mã hóa đầu cuối. Không cần email, số điện thoại hay tài khoản.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Trang web](https://totaldrop.app)** · **[Tải xuống](https://totaldrop.app/download/)** · [Lộ trình](https://totaldrop.app/roadmap/) · [Quyên góp](https://totaldrop.app/donate/) · [Các bản phát hành](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · **Tiếng Việt** · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop trên Android: màn hình chính ở giao diện sáng và tối, và một lần truyền đã đến nơi và đã được xác minh" width="820">

</div>

> [!NOTE]
> Bản dịch này là bản nháp và đang được xem lại. Bản [tiếng Anh](../../README.md) là bản tham chiếu.

## Vì sao chọn TotalDrop

- **Chỉ cần một mã ngắn.** Chọn tệp và nhận một mã như `7-harbor-rocket`. Đọc mã cho người kia nghe hoặc cho xem mã QR; người kia nhập mã và tệp sẽ đến.
- **Mã hóa đầu cuối, và bạn tự kiểm tra được.** Cả hai màn hình hiện cùng bốn từ trước khi bất cứ thứ gì được truyền đi. Nếu chúng khác nhau, hãy dừng lại.
- **Không cần đăng ký gì cả.** Không tài khoản, email hay số điện thoại. Tài khoản tùy chọn sẽ chỉ là một dãy số, theo kiểu Mullvad ([thiết kế](../../docs/hesap.md)).
- **Thẳng từ thiết bị đến thiết bị, ở bất cứ đâu.** Máy chủ điểm hẹn chỉ giới thiệu hai thiết bị với nhau và không lưu nhật ký; như mọi máy chủ khác, nó thấy địa chỉ IP của hai thiết bị. Thiết bị kết nối trực tiếp qua internet ở bất cứ nơi nào có thể. Khi bộ định tuyến hoặc tường lửa cản trở, máy chủ chuyển tiếp sẽ chuyển lưu lượng mà nó không thể đọc được.
- **Hòa hợp với hệ thống của bạn.** Các ứng dụng theo chủ đề sáng hoặc tối, màu nhấn, ngôn ngữ và cài đặt giảm chuyển động của hệ thống. Ứng dụng Linux là Qt 6 gốc trên Wayland và còn lấy phông chữ, góc bo và biểu tượng từ chủ đề của KDE Plasma, Hyprland và Omarchy ngay lập tức.
- **Miễn phí, mãi mãi.** Mã nguồn mở và được duy trì bằng quyên góp. Không quảng cáo, không theo dõi, không có gói trả phí cho các tính năng cơ bản.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Ứng dụng Linux trên Omarchy, theo màu sắc, phông chữ và biểu tượng của màn hình nền" width="560">
  <br><sub>Ứng dụng Linux trên Omarchy: màu sắc, góc bo, phông chữ và biểu tượng đều lấy từ màn hình nền.</sub>
</p>

## Tải xuống

| Nền tảng | Tải về | Ghi chú |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1, Qt 6.8 trở lên |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1, Qt 6.8 trở lên |
| **Android** | [Tham gia bản beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1, Android 15 trở lên, Google Play theo lời mời |
| **Dòng lệnh** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Sắp có | Ký mã cho các nền tảng này là khoản đầu tiên tiền quyên góp sẽ chi trả |

Mọi gói đều được CI xây dựng từ một commit có gắn thẻ, không bao giờ trên máy của nhà phát triển. Để kiểm tra một gói trước khi cài đặt, hãy so sánh nó với `SHA256SUMS` của bản phát hành và xác minh nơi nó được xây dựng:

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Mã nguồn chưa qua đợt kiểm định bảo mật độc lập nào, vì vậy xin đừng dựa vào nó cho các tệp nhạy cảm.

## Cách hoạt động

1. Người gửi xin máy chủ điểm hẹn một mã ngắn. Mã này là bí mật duy nhất mà hai người cùng biết.
2. Cả hai thiết bị biến mã thành một khóa mạnh bằng **CPace**. Kẻ tấn công chỉ được đoán một lần cho mỗi lần truyền và không thể thử mã ngoại tuyến.
3. Hai thiết bị kết nối trực tiếp qua **QUIC** (TLS 1.3 với khóa công khai thô được ghim) và chạy **Noise XXpsk3 với ML-KEM-768**, được thiết kế để chống chịu trước các máy tính lượng tử trong tương lai.
4. Tệp được mã hóa theo từng khối (STREAM), nên mỗi phần đều được xác thực ngay khi đến nơi.

Giao thức được mô tả trong *TotalDrop Protocol v0*. Các quyết định đưa ra khi triển khai nằm trong [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), còn những gì mỗi máy chủ có thể và không thể thấy nằm trong [mô hình mối đe dọa](../../docs/tehdit-modeli.md) (tiếng Thổ Nhĩ Kỳ) và [chính sách quyền riêng tư](https://totaldrop.app/privacy).

## Bằng ngôn ngữ của bạn

TotalDrop có 35 ngôn ngữ và theo định dạng số của khu vực bạn. Tiếng Anh và tiếng Thổ Nhĩ Kỳ đã được xem lại; 33 ngôn ngữ còn lại là bản nháp đang được người bản ngữ kiểm tra, vì vậy các ứng dụng giữ các văn bản bảo mật (xác minh, mã sai, chấp nhận và từ chối) bằng tiếng Anh cho đến khi ngôn ngữ đó được xem lại. Trang web và các ứng dụng đã có đủ tất cả, từ Android và Linux 0.3.0-beta.1.

Thấy một câu vụng về? [Giúp dịch](../../docs/ceviri.md) (hướng dẫn bằng tiếng Thổ Nhĩ Kỳ).

## Ủng hộ dự án

TotalDrop vẫn miễn phí nhờ mọi người cùng góp sức. Tiền quyên góp chi trả cho máy chủ chuyển tiếp, các chứng chỉ ký mã mà Windows và macOS yêu cầu, và một đợt kiểm định bảo mật độc lập trước phiên bản 1.0. Mọi khoản chi, từng dòng một, và số tiền quyên góp được đến nay: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Tài trợ trên GitHub"></a>

Bạn cũng có thể giúp bằng cách [báo lỗi](https://github.com/totaldrop-app/totaldrop/issues/new/choose), dịch thuật, hoặc thử nghiệm trên máy tính của mình.

## Dành cho nhà phát triển

### Cấu trúc kho mã

| Đường dẫn | Là gì | Giấy phép |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, dẫn xuất khóa, mã hóa hộp thư và phiên, khóa thiết bị, mã hóa theo khối STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Mã truyền, giới hạn, an toàn đường dẫn, mọi thông điệp truyền trên mạng (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Bộ máy phía máy khách: điểm hẹn, QUIC với khóa công khai thô, Noise XXpsk3 + ML-KEM-768, truyền tệp | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Văn bản giao diện cho các chương trình Rust: chọn ngôn ngữ, thông điệp ICU, số nhiều, định dạng số theo khu vực | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Máy khách dòng lệnh `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Mọi chuỗi giao diện, dùng chung cho Android, Linux và CLI ([dịch](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Ứng dụng máy tính Linux: Qt 6 và QML trên nền `totaldrop-core`, Wayland gốc | GPL-3.0-or-later |
| `packaging/` | Các gói Linux, mỗi định dạng riêng: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Ứng dụng Android (Flutter); iOS sau | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app và trang mà liên kết QR mở ra | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Mục tiêu fuzz cho mọi trình phân tích cú pháp | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Máy chủ điểm hẹn viết bằng Rust, để tự lưu trữ và kiểm thử | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Cùng máy chủ đó trên Cloudflare Workers, cho `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Các thư viện dùng giấy phép dễ dãi để bất kỳ ai cũng có thể xây dựng dựa trên giao thức; các ứng dụng dùng GPL và các máy chủ dùng AGPL. Xem [LICENSING.md](../../LICENSING.md).

Mỗi nền tảng có phiên bản, thẻ (`linux-v…`, `android-v…`) và quy trình CI riêng, nên một nền tảng có thể phát hành mà không ảnh hưởng đến các nền tảng khác. Cách hoạt động của nhánh và bản phát hành: [docs/gelistirme.md](../../docs/gelistirme.md) (tiếng Thổ Nhĩ Kỳ) và [docs/versioning.md](../../docs/versioning.md). Quy tắc thiết kế: [docs/tasarim.md](../../docs/tasarim.md) và các quyết định kiến trúc: [docs/kararlar](../../docs/kararlar/README.md), cả hai đều bằng tiếng Thổ Nhĩ Kỳ.

### Xây dựng

```bash
cargo test
cargo build --release
```

Thử một lần truyền trên cùng một máy, trong ba cửa sổ dòng lệnh:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Cả hai bên hiện cùng bốn từ xác minh. `ws://` không mã hóa chỉ được chấp nhận cho máy chủ trên cùng một máy; mọi trường hợp khác phải là `wss://`.

Dự án luôn nhắm tới phiên bản Rust ổn định mới nhất (được ghim trong `rust-toolchain.toml`); rustup sẽ cài nó ở lần xây dựng đầu tiên.

**Ứng dụng Linux** (cần Qt 6.8 trở lên: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Đóng góp và bảo mật

Hãy đọc [CONTRIBUTING.md](../../CONTRIBUTING.md) trước khi mở pull request, cùng [quy tắc ứng xử](../../CODE_OF_CONDUCT.md) của chúng tôi. Câu hỏi xin gửi theo [SUPPORT.md](../../SUPPORT.md). Vui lòng báo cáo lỗ hổng một cách riêng tư như mô tả trong [SECURITY.md](../../SECURITY.md), không bao giờ trong một issue công khai.
