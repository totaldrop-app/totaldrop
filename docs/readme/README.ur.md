<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### کوئی بھی فائل، کسی کو بھی، کہیں بھی۔

مفت اور اوپن سورس۔ اینڈ ٹو اینڈ انکرپٹڈ۔ نہ ای میل، نہ فون نمبر، نہ اکاؤنٹ۔

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![لائسنس](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![اسپانسر](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[ویب سائٹ](https://totaldrop.app)** · **[ڈاؤن لوڈ](https://totaldrop.app/download/)** · [روڈ میپ](https://totaldrop.app/roadmap/) · [عطیہ دیں](https://totaldrop.app/donate/) · [ریلیزز](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · **اردو** · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android پر TotalDrop: لائٹ اور ڈارک موڈ میں ہوم اسکرین، اور ایک منتقلی جو پہنچی اور جس کی تصدیق ہو گئی" width="820">

</div>

> [!NOTE]
> یہ ترجمہ مسودہ ہے اور اس پر نظرثانی ہو رہی ہے۔ [انگریزی](../../README.md) ورژن ہی معیاری حوالہ ہے۔

## TotalDrop کیوں

- **بس ایک مختصر کوڈ کافی ہے۔** فائلیں منتخب کریں اور `7-harbor-rocket` جیسا کوڈ حاصل کریں۔ اسے پڑھ کر سنائیں یا QR کوڈ دکھائیں؛ دوسرا شخص اسے ٹائپ کرتا ہے اور فائلیں پہنچ جاتی ہیں۔
- **اینڈ ٹو اینڈ انکرپٹڈ، اور آپ اسے جانچ سکتے ہیں۔** کچھ بھی منتقل ہونے سے پہلے دونوں اسکرینیں یہی چار الفاظ دکھاتی ہیں۔ اگر یہ مختلف ہوں تو رک جائیں۔
- **کسی چیز کے لیے سائن اپ نہیں۔** نہ اکاؤنٹ، نہ ای میل، نہ فون نمبر۔ اختیاری اکاؤنٹس صرف نمبر ہوں گے، Mullvad کی طرز پر ([ڈیزائن](../../docs/hesap.md))۔
- **براہ راست ایک آلے سے دوسرے آلے تک، کہیں بھی۔** رابطہ سرور صرف دونوں آلات کا تعارف کراتا ہے اور کوئی لاگ نہیں رکھتا؛ کسی بھی سرور کی طرح، یہ ان کے IP پتے دیکھتا ہے۔ آلات جہاں ممکن ہو انٹرنیٹ کے پار براہ راست جڑتے ہیں۔ جب کوئی راؤٹر یا فائر وال راستے میں ہو تو ریلے وہ ٹریفک آگے پہنچاتا ہے جسے وہ پڑھ نہیں سکتا۔
- **آپ کے سسٹم پر اپنے گھر جیسا۔** ایپس سسٹم کی لائٹ یا ڈارک تھیم، ایکسنٹ رنگ، زبان اور کم حرکت کی ترتیب کی پیروی کرتی ہیں۔ Linux ایپ Wayland پر مقامی Qt 6 ایپ ہے اور KDE Plasma، Hyprland اور Omarchy کی تھیمز سے فونٹس، کونے اور آئیکنز بھی فوری طور پر لیتی ہے۔
- **مفت، ہمیشہ کے لیے۔** اوپن سورس اور عطیات سے چلنے والا۔ نہ اشتہارات، نہ ٹریکنگ، نہ بنیادی چیزوں کے لیے کوئی ادا شدہ درجہ۔

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy پر Linux ایپ، ڈیسک ٹاپ کے رنگوں، فونٹس اور آئیکنز کے ساتھ" width="560">
  <br><sub>Omarchy پر Linux ایپ: رنگ، کونے، فونٹس اور آئیکنز ڈیسک ٹاپ سے آتے ہیں۔</sub>
</p>

## ڈاؤن لوڈ

| پلیٹ فارم | حاصل کریں | نوٹس |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1، Qt 6.8 یا نیا |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1، Qt 6.8 یا نیا |
| **Android** | [بیٹا میں شامل ہوں](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2، Android 15 یا نیا، Google Play پر دعوت کے ذریعے |
| **کمانڈ لائن** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux، macOS، Windows |
| **Windows، macOS، iOS** | جلد آ رہا ہے | ان کے لیے کوڈ سائننگ وہ پہلی چیز ہے جس کا خرچ عطیات اٹھاتے ہیں |

ہر پیکیج CI ایک ٹیگ شدہ کمٹ سے بناتا ہے، کبھی کسی ڈویلپر کی مشین پر نہیں۔ انسٹال کرنے سے پہلے کسی پیکیج کو جانچنے کے لیے، اس کا موازنہ ریلیز کی `SHA256SUMS` سے کریں اور تصدیق کریں کہ یہ کہاں بنایا گیا:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **بیٹا۔** کوڈ کا ابھی تک کوئی آزاد سیکیورٹی جائزہ نہیں ہوا، اس لیے براہ کرم حساس فائلوں کے لیے اس پر انحصار نہ کریں۔

## یہ کیسے کام کرتا ہے

1. بھیجنے والا رابطہ سرور سے ایک مختصر کوڈ مانگتا ہے۔ یہ کوڈ وہ واحد راز ہے جو دونوں افراد کے درمیان مشترک ہوتا ہے۔
2. دونوں آلات **CPace** کے ذریعے کوڈ کو ایک مضبوط کلید میں بدلتے ہیں۔ حملہ آور کو ہر منتقلی میں صرف ایک اندازے کا موقع ملتا ہے اور وہ کوڈز کو آف لائن نہیں جانچ سکتا۔
3. یہ **QUIC** (پن کی گئی خام پبلک کلیدوں کے ساتھ TLS 1.3) پر براہ راست جڑتے ہیں اور **ML-KEM-768 کے ساتھ Noise XXpsk3** چلاتے ہیں، جسے مستقبل کے کوانٹم کمپیوٹرز کے مقابلے میں بھی ٹکنے کے لیے ڈیزائن کیا گیا ہے۔
4. فائلیں ٹکڑوں میں انکرپٹ کی جاتی ہیں (STREAM)، اس لیے ہر ٹکڑے کی تصدیق اس کے پہنچتے ہی ہو جاتی ہے۔

پروٹوکول *TotalDrop Protocol v0* میں بیان کیا گیا ہے۔ اسے نافذ کرتے وقت کیے گئے فیصلے [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) میں ہیں، اور ہر سرور کیا دیکھ سکتا ہے اور کیا نہیں، یہ [خطرے کے ماڈل](../../docs/tehdit-modeli.md) (ترکی میں) اور [رازداری کی پالیسی](https://totaldrop.app/privacy) میں ہے۔

## آپ کی زبان میں

TotalDrop 35 زبانیں بولتا ہے اور آپ کے علاقے کے نمبر فارمیٹس کی پیروی کرتا ہے۔ انگریزی اور ترکی پر نظرثانی ہو چکی ہے؛ باقی 33 مسودے ہیں جنہیں مقامی بولنے والے جانچ رہے ہیں، اس لیے جب تک کسی زبان پر نظرثانی نہ ہو جائے، ایپس سیکیورٹی کے متن (تصدیق، غلط کوڈ، قبول اور مسترد کرنا) انگریزی میں رکھتی ہیں۔ سائٹ پر یہ سب آج موجود ہیں؛ ایپس کو یہ اپنی اگلی ریلیزز کے ساتھ ملیں گی۔

کوئی بے ڈھنگا جملہ نظر آیا؟ [ترجمے میں مدد کریں](../../docs/ceviri.md) (رہنما ترکی میں ہے)۔

## پروجیکٹ کی مدد کریں

TotalDrop اس لیے مفت ہے کہ لوگ اپنا حصہ ڈالتے ہیں۔ عطیات ریلے، Windows اور macOS کے لیے لازمی کوڈ سائننگ سرٹیفکیٹس، اور 1.0 سے پہلے ایک آزاد سیکیورٹی آڈٹ کا خرچ اٹھاتے ہیں۔ ہر خرچ، سطر بہ سطر، اور اب تک کتنا جمع ہوا: [totaldrop.app/donate](https://totaldrop.app/donate/)۔

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="GitHub پر اسپانسر کریں"></a>

آپ [بگز رپورٹ کر کے](https://github.com/totaldrop-app/totaldrop/issues/new/choose)، ترجمہ کر کے، یا اپنے ڈیسک ٹاپ پر جانچ کر کے بھی مدد کر سکتے ہیں۔

## ڈویلپرز کے لیے

### ریپوزٹری کی ترتیب

| پاتھ | یہ کیا ہے | لائسنس |
| --- | --- | --- |
| `crates/totaldrop-crypto` | سوئٹ TD0: CPace، کلید اخذ کرنا، میل باکس اور سیشن انکرپشن، آلے کی کلیدیں، STREAM کے ذریعے ٹکڑوں کی انکرپشن | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | منتقلی کوڈ، حدود، پاتھ کی حفاظت، ہر وائر پیغام (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | کلائنٹ انجن: رابطہ، خام پبلک کلیدوں کے ساتھ QUIC، Noise XXpsk3 + ML-KEM-768، منتقلی | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust پروگراموں کے لیے انٹرفیس متن: زبان کا انتخاب، ICU پیغامات، جمع کی صورتیں، علاقائی نمبر | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` کمانڈ لائن کلائنٹ | GPL-3.0-or-later |
| `l10n/` | انٹرفیس کی ہر سٹرنگ، جو Android، Linux اور کمانڈ لائن میں مشترک ہے ([ترجمہ](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux ڈیسک ٹاپ ایپ: `totaldrop-core` کے اوپر Qt 6 اور QML، مقامی Wayland | GPL-3.0-or-later |
| `packaging/` | Linux پیکیجز، ہر فارمیٹ الگ: Arch (`PKGBUILD`)، Debian (`debian/`)، RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android ایپ (Flutter)؛ iOS بعد میں | GPL-3.0-or-later |
| `site/` | totaldrop.app، download.totaldrop.app اور وہ صفحہ جو QR لنکس کھولتے ہیں | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | ہر پارسر کے لیے fuzz اہداف | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Rust میں رابطہ سرور، سیلف ہوسٹنگ اور ٹیسٹس کے لیے | AGPL-3.0-or-later |
| `server/rendezvous-worker` | یہی سرور Cloudflare Workers پر، `rendezvous.totaldrop.app` کے لیے | AGPL-3.0-or-later |

لائبریریاں نرم اجازتی لائسنسوں کے تحت ہیں تاکہ کوئی بھی پروٹوکول پر تعمیر کر سکے؛ ایپس GPL اور سرورز AGPL کے تحت ہیں۔ دیکھیں [LICENSING.md](../../LICENSING.md)۔

ہر پلیٹ فارم کا اپنا ورژن، ٹیگ (`linux-v…`، `android-v…`) اور CI ورک فلو ہے، تاکہ ایک کو دوسروں کو چھیڑے بغیر جاری کیا جا سکے۔ برانچز اور ریلیزز کیسے کام کرتی ہیں: [docs/gelistirme.md](../../docs/gelistirme.md) (ترکی میں) اور [docs/versioning.md](../../docs/versioning.md)۔ ڈیزائن کے اصول: [docs/tasarim.md](../../docs/tasarim.md) اور آرکیٹیکچر کے فیصلے: [docs/kararlar](../../docs/kararlar/README.md)، دونوں ترکی میں۔

### بلڈ

```bash
cargo test
cargo build --release
```

ایک ہی مشین پر، تین ٹرمینلز میں، ایک منتقلی آزمائیں:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

دونوں طرف یہی چار تصدیقی الفاظ دکھائی دیتے ہیں۔ غیر انکرپٹڈ `ws://` صرف اسی مشین پر موجود سرور کے لیے قبول کیا جاتا ہے؛ باقی سب کچھ `wss://` ہونا چاہیے۔

پروجیکٹ ہمیشہ Rust کے تازہ ترین مستحکم ورژن کو ہدف بناتا ہے (`rust-toolchain.toml` میں پن کیا گیا)؛ rustup اسے پہلے بلڈ پر انسٹال کر دیتا ہے۔

**Linux ایپ** (Qt 6.8 یا نیا درکار ہے: `qt6-base`، `qt6-declarative`، `qt6-wayland`، `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### تعاون اور سیکیورٹی

پل ریکویسٹ کھولنے سے پہلے [CONTRIBUTING.md](../../CONTRIBUTING.md) اور ہمارا [ضابطۂ اخلاق](../../CODE_OF_CONDUCT.md) پڑھیں۔ سوالات کے لیے [SUPPORT.md](../../SUPPORT.md) دیکھیں۔ براہ کرم سیکیورٹی کی کمزوریوں کی اطلاع [SECURITY.md](../../SECURITY.md) میں بیان کردہ طریقے سے نجی طور پر دیں، کبھی کسی عوامی issue میں نہیں۔
