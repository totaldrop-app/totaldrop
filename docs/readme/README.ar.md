<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### أي ملف، لأي شخص، في أي مكان.

مجاني ومفتوح المصدر. مشفّر بين الطرفين. بلا بريد إلكتروني، ولا رقم هاتف، ولا حساب.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![الترخيص](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![الرعاية](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[الموقع](https://totaldrop.app)** · **[التنزيل](https://totaldrop.app/download/)** · [خارطة الطريق](https://totaldrop.app/roadmap/) · [التبرّع](https://totaldrop.app/donate/) · [الإصدارات](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>**العربية** · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop على Android: الشاشة الرئيسية بالوضعين الفاتح والداكن، وعملية نقل وصلت وتم التحقق منها" width="820">

</div>

> [!NOTE]
> هذه الترجمة مسودة وقيد المراجعة. النسخة [الإنجليزية](../../README.md) هي المرجع.

## لماذا TotalDrop

- **يكفي رمز قصير.** اختر الملفات واحصل على رمز مثل `7-harbor-rocket`. اقرأه بصوت عالٍ أو اعرض رمز QR؛ يكتبه الطرف الآخر فتصل الملفات.
- **مشفّر بين الطرفين، ويمكنك التحقق من ذلك.** تعرض الشاشتان الكلمات الأربع نفسها قبل أن ينتقل أي شيء. إذا اختلفت، فتوقّف.
- **لا شيء للتسجيل فيه.** بلا حساب أو بريد إلكتروني أو رقم هاتف. ستكون الحسابات الاختيارية أرقامًا فقط، على طريقة Mullvad ([التصميم](../../docs/hesap.md)).
- **مباشرة من جهاز إلى جهاز، في أي مكان.** لا يفعل خادم الالتقاء سوى تعريف الجهازين أحدهما بالآخر، ولا يحتفظ بأي سجلات؛ وهو، مثل أي خادم، يرى عناوين IP الخاصة بهما. تتصل الأجهزة مباشرة عبر الإنترنت حيثما أمكن. وعندما يعترض الطريق موجّه أو جدار حماية، ينقل خادم الترحيل حركة لا يستطيع قراءتها.
- **في بيته على نظامك.** تتبع التطبيقات سمة النظام الفاتحة أو الداكنة، ولون التمييز، واللغة، وإعداد تقليل الحركة. تطبيق Linux مبني أصليًا بـ Qt 6 على Wayland، ويأخذ أيضًا خطوطه وزواياه وأيقوناته من سمات KDE Plasma وHyprland وOmarchy لحظيًا.
- **مجاني، إلى الأبد.** مفتوح المصدر ويموّله المتبرعون. بلا إعلانات، ولا تتبّع، ولا فئة مدفوعة للأساسيات.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="تطبيق Linux على Omarchy، بألوان سطح المكتب وخطوطه وأيقوناته" width="560">
  <br><sub>تطبيق Linux على Omarchy: الألوان والزوايا والخطوط والأيقونات مأخوذة من سطح المكتب.</sub>
</p>

## التنزيل

| المنصة | احصل عليه | ملاحظات |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1، Qt 6.8 أو أحدث |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1، Qt 6.8 أو أحدث |
| **Android** | [انضم إلى النسخة التجريبية](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2، Android 15 أو أحدث، Google Play بدعوة |
| **سطر الأوامر** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux، macOS، Windows |
| **Windows، macOS، iOS** | قريبًا | توقيع الشيفرة لهذه المنصات هو أول ما تدفع التبرعات تكلفته |

تُبنى كل حزمة بواسطة CI من إيداع موسوم، ولا تُبنى أبدًا على جهاز أحد المطورين. للتحقق من حزمة قبل تثبيتها، قارنها بملف `SHA256SUMS` الخاص بالإصدار وتحقق من مكان بنائها:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **تجريبي.** لم تخضع الشيفرة بعد لمراجعة أمنية مستقلة، لذا يُرجى عدم الاعتماد عليها في الملفات الحساسة.

## كيف يعمل

1. يطلب المرسل من خادم الالتقاء رمزًا قصيرًا. الرمز هو السر الوحيد الذي يتشاركه الشخصان.
2. يحوّل الجهازان الرمز إلى مفتاح قوي باستخدام **CPace**. يحصل المهاجم على تخمين واحد لكل عملية نقل، ولا يستطيع اختبار الرموز دون اتصال.
3. يتصل الجهازان مباشرة عبر **QUIC** (TLS 1.3 مع مفاتيح عامة خام مثبّتة) ويشغّلان **Noise XXpsk3 مع ML-KEM-768**، المصمَّم ليصمد أمام الحواسيب الكمومية المستقبلية.
4. تُشفَّر الملفات على شكل أجزاء (STREAM)، فيُصادَق على كل جزء لحظة وصوله.

البروتوكول موصوف في *TotalDrop Protocol v0*. والقرارات المتخذة أثناء تنفيذه موجودة في [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md)، أما ما يستطيع كل خادم رؤيته وما لا يستطيع فموجود في [نموذج التهديدات](../../docs/tehdit-modeli.md) (بالتركية) و[سياسة الخصوصية](https://totaldrop.app/privacy).

## بلغتك

يتحدث TotalDrop 35 لغة ويتبع تنسيقات الأرقام في منطقتك. الإنجليزية والتركية مراجَعتان؛ أما اللغات الـ33 الأخرى فمسودات يراجعها متحدثون أصليون، لذلك تُبقي التطبيقات نصوص الأمان (التحقق، والرمز الخاطئ، والقبول والرفض) بالإنجليزية إلى أن تُراجَع اللغة. الموقع يضمها جميعًا اليوم؛ وتحصل عليها التطبيقات مع إصداراتها القادمة.

هل لاحظت جملة ركيكة؟ [ساعد في الترجمة](../../docs/ceviri.md) (الدليل بالتركية).

## ادعم المشروع

يبقى TotalDrop مجانيًا لأن الناس يساهمون. تغطي التبرعات تكاليف خادم الترحيل، وشهادات توقيع الشيفرة التي يشترطها Windows وmacOS، وتدقيقًا أمنيًا مستقلًا قبل الإصدار 1.0. كل تكلفة، بندًا بندًا، وما جُمع حتى الآن: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="ادعم عبر GitHub"></a>

يمكنك أيضًا المساعدة [بالإبلاغ عن الأخطاء](https://github.com/totaldrop-app/totaldrop/issues/new/choose)، أو بالترجمة، أو بالاختبار على حاسوبك المكتبي.

## للمطورين

### بنية المستودع

| المسار | ما هو | الترخيص |
| --- | --- | --- |
| `crates/totaldrop-crypto` | المجموعة TD0: CPace، واشتقاق المفاتيح، وتشفير صندوق البريد والجلسة، ومفاتيح الأجهزة، وتشفير الأجزاء بـ STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | رموز النقل، والحدود، وأمان المسارات، وكل رسائل الشبكة (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | محرك العميل: الالتقاء، وQUIC بمفاتيح عامة خام، وNoise XXpsk3 + ML-KEM-768، والنقل | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | نصوص الواجهة لبرامج Rust: التفاوض على اللغة، ورسائل ICU، وصيغ الجمع، والأرقام الإقليمية | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | عميل سطر الأوامر `totaldrop` | GPL-3.0-or-later |
| `l10n/` | كل نصوص الواجهة، مشتركة بين Android وLinux وسطر الأوامر ([الترجمة](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | تطبيق Linux لسطح المكتب: Qt 6 وQML فوق `totaldrop-core`، مع Wayland أصلي | GPL-3.0-or-later |
| `packaging/` | حزم Linux، كل تنسيق على حدة: Arch (`PKGBUILD`)، وDebian (`debian/`)، وRPM (`.spec`) | GPL-3.0-or-later |
| `app/` | تطبيق Android (Flutter)؛ وiOS لاحقًا | GPL-3.0-or-later |
| `site/` | totaldrop.app وdownload.totaldrop.app والصفحة التي تفتحها روابط QR | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | أهداف fuzz لكل محلّل | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | خادم الالتقاء مكتوبًا بـ Rust، للاستضافة الذاتية والاختبارات | AGPL-3.0-or-later |
| `server/rendezvous-worker` | الخادم نفسه على Cloudflare Workers، لـ `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

المكتبات بتراخيص متساهلة كي يتمكن أي شخص من البناء على البروتوكول؛ أما التطبيقات فبترخيص GPL والخوادم بترخيص AGPL. راجع [LICENSING.md](../../LICENSING.md).

لكل منصة إصدارها ووسمها (`linux-v…`، `android-v…`) وسير عمل CI خاص بها، فيمكن إصدار إحداها دون المساس بالأخرى. كيف تعمل الفروع والإصدارات: [docs/gelistirme.md](../../docs/gelistirme.md) (بالتركية) و[docs/versioning.md](../../docs/versioning.md). قواعد التصميم: [docs/tasarim.md](../../docs/tasarim.md) وقرارات البنية المعمارية: [docs/kararlar](../../docs/kararlar/README.md)، وكلاهما بالتركية.

### البناء

```bash
cargo test
cargo build --release
```

جرّب عملية نقل على جهاز واحد، في ثلاث نوافذ طرفية:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

يعرض الطرفان كلمات التحقق الأربع نفسها. لا يُقبل `ws://` غير المشفّر إلا لخادم على الجهاز نفسه؛ وأي شيء آخر يجب أن يكون `wss://`.

يستهدف المشروع دائمًا أحدث إصدار مستقر من Rust (مثبّت في `rust-toolchain.toml`)؛ ويثبّته rustup عند أول عملية بناء.

**تطبيق Linux** (يحتاج إلى Qt 6.8 أو أحدث: `qt6-base`، `qt6-declarative`، `qt6-wayland`، `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### المساهمة والأمان

اقرأ [CONTRIBUTING.md](../../CONTRIBUTING.md) قبل فتح طلب سحب، واقرأ [مدونة السلوك](../../CODE_OF_CONDUCT.md) الخاصة بنا. توجَّه الأسئلة إلى [SUPPORT.md](../../SUPPORT.md). يُرجى الإبلاغ عن الثغرات الأمنية بشكل خاص كما هو موضح في [SECURITY.md](../../SECURITY.md)، وليس أبدًا في بلاغ علني.
