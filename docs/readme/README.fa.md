<!-- Translated from README.md 529368877e8d. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### هر فایلی، برای هر کسی، در هر جایی.

رایگان و متن‌باز. رمزگذاری سرتاسری. بدون ایمیل، بدون شماره تلفن، بدون حساب.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![مجوز](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![حمایت](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[وب‌سایت](https://totaldrop.app)** · **[دانلود](https://totaldrop.app/download/)** · [نقشه راه](https://totaldrop.app/roadmap/) · [کمک مالی](https://totaldrop.app/donate/) · [انتشارها](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · **فارسی** · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop روی Android: صفحه اصلی در حالت روشن و تیره، و انتقالی که رسید و تأیید شد" width="820">

</div>

> [!NOTE]
> این ترجمه پیش‌نویس است و در حال بازبینی است. نسخه [انگلیسی](../../README.md) مرجع است.

## چرا TotalDrop

- **یک کد کوتاه کافی است.** فایل‌ها را انتخاب کنید و کدی مثل `7-harbor-rocket` بگیرید. آن را بلند بخوانید یا کد QR را نشان دهید؛ طرف مقابل آن را وارد می‌کند و فایل‌ها می‌رسند.
- **رمزگذاری سرتاسری، و می‌توانید آن را بررسی کنید.** پیش از آنکه چیزی جابه‌جا شود، هر دو صفحه همان چهار کلمه را نشان می‌دهند. اگر فرق داشتند، متوقف شوید.
- **نیازی به ثبت‌نام نیست.** بدون حساب، ایمیل یا شماره تلفن. حساب‌های اختیاری فقط یک عدد خواهند بود، به سبک Mullvad ([طراحی](../../docs/hesap.md)).
- **مستقیم از دستگاه به دستگاه، در هر جایی.** سرور هماهنگی فقط دو دستگاه را به هم معرفی می‌کند و هیچ گزارشی نگه نمی‌دارد؛ مانند هر سروری، نشانی‌های IP آن‌ها را می‌بیند. دستگاه‌ها هر جا ممکن باشد از طریق اینترنت مستقیم به هم وصل می‌شوند. وقتی روتر یا دیوار آتش سر راه باشد، سرور رله ترافیکی را جابه‌جا می‌کند که نمی‌تواند بخواند.
- **در خانه خود، روی سیستم شما.** برنامه‌ها از پوسته روشن یا تیره سیستم، رنگ تأکیدی، زبان و تنظیم کاهش حرکت پیروی می‌کنند. برنامه Linux به‌صورت بومی با Qt 6 روی Wayland ساخته شده و قلم‌ها، گوشه‌ها و نمادهایش را هم از پوسته‌های KDE Plasma، Hyprland و Omarchy در لحظه می‌گیرد.
- **رایگان، برای همیشه.** متن‌باز و با پشتوانه کمک‌های مالی. بدون تبلیغات، بدون ردیابی، بدون نسخه پولی برای امکانات پایه.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="برنامه Linux روی Omarchy، با رنگ‌ها، قلم‌ها و آیکون‌های میز کار" width="560">
  <br><sub>برنامه Linux روی Omarchy: رنگ‌ها، گوشه‌ها، قلم‌ها و آیکون‌ها از میز کار می‌آیند.</sub>
</p>

## دانلود

| پلتفرم | دریافت | یادداشت‌ها |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1، Qt 6.8 یا جدیدتر |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1، Qt 6.8 یا جدیدتر |
| **Android** | [پیوستن به نسخه بتا](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1، Android 15 یا جدیدتر، Google Play با دعوت |
| **خط فرمان** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux، macOS، Windows |
| **Windows، macOS، iOS** | در راه | امضای کد برای این‌ها نخستین چیزی است که کمک‌های مالی هزینه‌اش را می‌پردازند |

هر بسته را CI از یک کامیت برچسب‌دار می‌سازد، هرگز روی دستگاه یک توسعه‌دهنده. برای بررسی یک بسته پیش از نصب، آن را با `SHA256SUMS` همان انتشار مقایسه کنید و محل ساخت آن را تأیید کنید:

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **بتا.** کد هنوز بازبینی امنیتی مستقلی نداشته است، پس لطفا برای فایل‌های حساس به آن تکیه نکنید.

## چگونه کار می‌کند

1. فرستنده از سرور هماهنگی یک کد کوتاه می‌خواهد. این کد تنها رازی است که دو نفر با هم شریک‌اند.
2. هر دو دستگاه با **CPace** کد را به یک کلید قوی تبدیل می‌کنند. مهاجم در هر انتقال فقط یک بار می‌تواند حدس بزند و نمی‌تواند کدها را به‌صورت آفلاین بیازماید.
3. دو دستگاه از طریق **QUIC** (TLS 1.3 با کلیدهای عمومی خام سنجاق‌شده) مستقیم به هم وصل می‌شوند و **Noise XXpsk3 با ML-KEM-768** را اجرا می‌کنند که طراحی شده تا در برابر رایانه‌های کوانتومی آینده هم دوام بیاورد.
4. فایل‌ها تکه‌تکه رمزگذاری می‌شوند (STREAM)، بنابراین اصالت هر تکه به محض رسیدن تأیید می‌شود.

پروتکل در *TotalDrop Protocol v0* شرح داده شده است. تصمیم‌هایی که هنگام پیاده‌سازی آن گرفته شده در [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) آمده است، و اینکه هر سرور چه چیزی را می‌تواند و نمی‌تواند ببیند در [مدل تهدید](../../docs/tehdit-modeli.md) (به ترکی) و [سیاست حریم خصوصی](https://totaldrop.app/privacy).

## به زبان شما

TotalDrop به 35 زبان صحبت می‌کند و از قالب اعداد منطقه شما پیروی می‌کند. انگلیسی و ترکی بازبینی شده‌اند؛ 33 زبان دیگر پیش‌نویس‌هایی هستند که گویشوران بومی در حال بررسی آن‌ها هستند، بنابراین برنامه‌ها متن‌های امنیتی (تأیید، کد اشتباه، پذیرفتن و رد کردن) را تا زمان بازبینی هر زبان به انگلیسی نگه می‌دارند. سایت و برنامه‌ها همه آن‌ها را دارند، از Android و Linux 0.3.0-beta.1 به بعد.

جمله‌ای ناشیانه دیدید؟ [در ترجمه کمک کنید](../../docs/ceviri.md) (راهنما به ترکی است).

## حمایت از پروژه

TotalDrop رایگان می‌ماند چون مردم کمک می‌کنند. کمک‌های مالی هزینه سرور رله، گواهی‌های امضای کد که Windows و macOS لازم دارند، و یک ممیزی امنیتی مستقل پیش از نسخه 1.0 را می‌پردازند. همه هزینه‌ها، قلم به قلم، و آنچه تاکنون جمع شده است: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="حمایت در GitHub"></a>

همچنین می‌توانید با [گزارش اشکال‌ها](https://github.com/totaldrop-app/totaldrop/issues/new/choose)، ترجمه یا آزمایش روی رایانه رومیزی خود کمک کنید.

## برای توسعه‌دهندگان

### ساختار مخزن

| مسیر | چیست | مجوز |
| --- | --- | --- |
| `crates/totaldrop-crypto` | مجموعه TD0: CPace، استخراج کلید، رمزگذاری صندوق پیام و نشست، کلیدهای دستگاه، رمزگذاری تکه‌ای STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | کدهای انتقال، محدودیت‌ها، ایمنی مسیرها، همه پیام‌های شبکه (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | موتور کلاینت: هماهنگی، QUIC با کلیدهای عمومی خام، Noise XXpsk3 + ML-KEM-768، انتقال | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | متن رابط کاربری برای برنامه‌های Rust: انتخاب زبان، پیام‌های ICU، صورت‌های جمع، اعداد منطقه‌ای | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | کلاینت خط فرمان `totaldrop` | GPL-3.0-or-later |
| `l10n/` | همه رشته‌های رابط کاربری، مشترک میان Android، Linux و خط فرمان ([ترجمه](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | برنامه رومیزی Linux: Qt 6 و QML روی `totaldrop-core`، با Wayland بومی | GPL-3.0-or-later |
| `packaging/` | بسته‌های Linux، هر قالب جداگانه: Arch (`PKGBUILD`)، Debian (`debian/`)، RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | برنامه Android (Flutter)؛ iOS بعدا | GPL-3.0-or-later |
| `site/` | totaldrop.app، download.totaldrop.app و صفحه‌ای که پیوندهای QR باز می‌کنند | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | هدف‌های fuzz برای همه تجزیه‌گرها | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | سرور هماهنگی به زبان Rust، برای میزبانی شخصی و آزمون‌ها | AGPL-3.0-or-later |
| `server/rendezvous-worker` | همین سرور روی Cloudflare Workers، برای `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

کتابخانه‌ها مجوزهای آسان‌گیر دارند تا هر کسی بتواند روی پروتکل بسازد؛ برنامه‌ها GPL و سرورها AGPL هستند. [LICENSING.md](../../LICENSING.md) را ببینید.

هر پلتفرم نسخه، برچسب (`linux-v…`، `android-v…`) و گردش‌کار CI خودش را دارد، تا یکی بتواند بدون دست زدن به بقیه منتشر شود. شاخه‌ها و انتشارها چگونه کار می‌کنند: [docs/gelistirme.md](../../docs/gelistirme.md) (به ترکی) و [docs/versioning.md](../../docs/versioning.md). قواعد طراحی: [docs/tasarim.md](../../docs/tasarim.md) و تصمیم‌های معماری: [docs/kararlar](../../docs/kararlar/README.md)، هر دو به ترکی.

### ساخت

```bash
cargo test
cargo build --release
```

یک انتقال را روی یک دستگاه، در سه ترمینال امتحان کنید:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

هر دو طرف همان چهار کلمه تأیید را نشان می‌دهند. `ws://` رمزگذاری‌نشده فقط برای سروری روی همان دستگاه پذیرفته می‌شود؛ هر چیز دیگری باید `wss://` باشد.

پروژه همیشه جدیدترین نسخه پایدار Rust را هدف می‌گیرد (سنجاق‌شده در `rust-toolchain.toml`)؛ rustup آن را در نخستین ساخت نصب می‌کند.

**برنامه Linux** (به Qt 6.8 یا جدیدتر نیاز دارد: `qt6-base`، `qt6-declarative`، `qt6-wayland`، `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### مشارکت و امنیت

پیش از باز کردن درخواست ادغام (pull request)، [CONTRIBUTING.md](../../CONTRIBUTING.md) و [آیین‌نامه رفتاری](../../CODE_OF_CONDUCT.md) ما را بخوانید. پرسش‌ها را در [SUPPORT.md](../../SUPPORT.md) مطرح کنید. لطفا آسیب‌پذیری‌ها را همان‌طور که در [SECURITY.md](../../SECURITY.md) آمده، به‌طور خصوصی گزارش دهید، هرگز در یک issue عمومی.
