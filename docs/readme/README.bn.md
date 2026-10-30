<!-- Translated from README.md 529368877e8d. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### যেকোনো ফাইল, যে কাউকে, যেকোনো জায়গায়।

বিনামূল্যে ও ওপেন সোর্স। এন্ড-টু-এন্ড এনক্রিপ্টেড। কোনো ইমেল, ফোন নম্বর বা অ্যাকাউন্ট লাগে না।

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[ওয়েবসাইট](https://totaldrop.app)** · **[ডাউনলোড](https://totaldrop.app/download/)** · [রোডম্যাপ](https://totaldrop.app/roadmap/) · [অনুদান দিন](https://totaldrop.app/donate/) · [রিলিজ](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · **বাংলা** · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android-এ TotalDrop: লাইট ও ডার্ক মোডে হোম স্ক্রিন, এবং একটি ট্রান্সফার যা পৌঁছেছে ও যাচাই হয়েছে" width="820">

</div>

> [!NOTE]
> এই অনুবাদটি একটি খসড়া এবং পর্যালোচনা করা হচ্ছে। [ইংরেজি](../../README.md) সংস্করণটিই মূল সূত্র।

## কেন TotalDrop

- **একটি ছোট কোডই যথেষ্ট।** ফাইল বেছে নিন এবং `7-harbor-rocket`-এর মতো একটি কোড পান। কোডটি পড়ে শোনান বা QR কোড দেখান; অন্যজন সেটি টাইপ করলেই ফাইল পৌঁছে যায়।
- **এন্ড-টু-এন্ড এনক্রিপ্টেড, আর আপনি নিজেই যাচাই করতে পারেন।** কিছু পাঠানোর আগেই দুটি স্ক্রিনে একই চারটি শব্দ দেখায়। শব্দগুলো আলাদা হলে থেমে যান।
- **কোথাও সাইন আপ করতে হয় না।** কোনো অ্যাকাউন্ট, ইমেল বা ফোন নম্বর নেই। ঐচ্ছিক অ্যাকাউন্ট হবে শুধু নম্বর, Mullvad-এর মতো ([নকশা](../../docs/hesap.md))।
- **সরাসরি এক ডিভাইস থেকে আরেক ডিভাইসে, যেকোনো জায়গায়।** সংযোগকারী সার্ভার শুধু দুটি ডিভাইসের পরিচয় করিয়ে দেয় এবং কোনো লগ রাখে না; যেকোনো সার্ভারের মতোই এটি ডিভাইস দুটির IP ঠিকানা দেখতে পায়। যেখানে সম্ভব, ডিভাইসগুলো ইন্টারনেটে সরাসরি সংযুক্ত হয়। কোনো রাউটার বা ফায়ারওয়াল বাধা হলে একটি রিলে সার্ভার ট্র্যাফিক বহন করে, যা সে পড়তে পারে না।
- **আপনার সিস্টেমে নিজের মতো।** অ্যাপগুলো সিস্টেমের হালকা বা গাঢ় থিম, অ্যাকসেন্ট রং, ভাষা ও কম মোশনের সেটিং অনুসরণ করে। Linux অ্যাপটি Wayland-এ নেটিভ Qt 6 এবং KDE Plasma, Hyprland ও Omarchy-এর থিম থেকে ফন্ট, কোণ ও আইকনও সরাসরি নেয়।
- **বিনামূল্যে, চিরকাল।** ওপেন সোর্স এবং অনুদানে চলে। কোনো বিজ্ঞাপন নেই, ট্র্যাকিং নেই, মৌলিক সুবিধার জন্য কোনো পেইড স্তর নেই।

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy-তে Linux অ্যাপ, ডেস্কটপের রং, ফন্ট ও আইকনে" width="560">
  <br><sub>Omarchy-তে Linux অ্যাপ: রং, কোণ, ফন্ট ও আইকন আসে ডেস্কটপ থেকে।</sub>
</p>

## ডাউনলোড

| প্ল্যাটফর্ম | কোথায় পাবেন | নোট |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1, Qt 6.8 বা নতুনতর |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1, Qt 6.8 বা নতুনতর |
| **Android** | [বিটাতে যোগ দিন](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1, Android 15 বা নতুনতর, Google Play-তে আমন্ত্রণের মাধ্যমে |
| **কমান্ড লাইন** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | আসছে | অনুদানের টাকা প্রথমে এগুলোর কোড সাইনিংয়েই খরচ হবে |

প্রতিটি প্যাকেজ CI একটি ট্যাগ করা কমিট থেকে তৈরি করে, কখনও কোনো ডেভেলপারের মেশিনে নয়। ইনস্টল করার আগে কোনো প্যাকেজ যাচাই করতে, সেটি রিলিজের `SHA256SUMS`-এর সঙ্গে মিলিয়ে দেখুন এবং কোথায় তৈরি হয়েছে তা যাচাই করুন:

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **বিটা।** কোডটির এখনও কোনো স্বাধীন নিরাপত্তা পর্যালোচনা হয়নি, তাই অনুগ্রহ করে সংবেদনশীল ফাইলের জন্য এর ওপর নির্ভর করবেন না।

## কীভাবে কাজ করে

1. প্রেরক সংযোগকারী সার্ভারের কাছে একটি ছোট কোড চান। এই কোডই দুজনের মধ্যে ভাগ করা একমাত্র গোপন তথ্য।
2. দুটি ডিভাইসই **CPace** দিয়ে কোডটিকে একটি শক্তিশালী কী-তে রূপান্তর করে। আক্রমণকারী প্রতি ট্রান্সফারে মাত্র একবার অনুমান করতে পারে এবং অফলাইনে কোড পরীক্ষা করতে পারে না।
3. এরা **QUIC**-এর মাধ্যমে সরাসরি সংযুক্ত হয় (পিন করা raw পাবলিক কী সহ TLS 1.3) এবং **ML-KEM-768 সহ Noise XXpsk3** চালায়, যা ভবিষ্যতের কোয়ান্টাম কম্পিউটারের বিরুদ্ধেও টিকে থাকার জন্য নকশা করা।
4. ফাইল খণ্ডে খণ্ডে এনক্রিপ্ট হয় (STREAM), তাই প্রতিটি খণ্ড পৌঁছানোর সঙ্গে সঙ্গে প্রমাণীকৃত হয়।

প্রোটোকলটির বিবরণ আছে *TotalDrop Protocol v0*-এ। এটি বাস্তবায়নের সময় নেওয়া সিদ্ধান্তগুলো আছে [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md)-এ, আর প্রতিটি সার্ভার কী দেখতে পারে ও কী পারে না, তা আছে [হুমকি মডেল](../../docs/tehdit-modeli.md) (তুর্কি ভাষায়) ও [গোপনীয়তা নীতি](https://totaldrop.app/privacy)-তে।

## আপনার ভাষায়

TotalDrop 35টি ভাষায় কথা বলে এবং আপনার অঞ্চলের সংখ্যার বিন্যাস অনুসরণ করে। ইংরেজি ও তুর্কি পর্যালোচিত; বাকি 33টি খসড়া, যা মাতৃভাষীরা যাচাই করছেন, তাই কোনো ভাষা পর্যালোচিত না হওয়া পর্যন্ত অ্যাপগুলো নিরাপত্তা-সংক্রান্ত লেখা (যাচাই, ভুল কোড, গ্রহণ ও প্রত্যাখ্যান) ইংরেজিতে রাখে। সাইট ও অ্যাপ, দুটোতেই সব ভাষা আছে, Android ও Linux 0.3.0-beta.1 থেকে।

কোনো আড়ষ্ট বাক্য চোখে পড়েছে? [অনুবাদে সাহায্য করুন](../../docs/ceviri.md) (নির্দেশিকাটি তুর্কি ভাষায়)।

## প্রকল্পটিকে সমর্থন করুন

মানুষ অবদান রাখেন বলেই TotalDrop বিনামূল্যে থাকে। অনুদান দিয়ে রিলে সার্ভার, Windows ও macOS-এর প্রয়োজনীয় কোড-সাইনিং সার্টিফিকেট, এবং 1.0-এর আগে একটি স্বাধীন নিরাপত্তা অডিটের খরচ মেটানো হয়। প্রতিটি খরচ, লাইন ধরে ধরে, এবং এ পর্যন্ত কত উঠেছে: [totaldrop.app/donate](https://totaldrop.app/donate/)।

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="GitHub-এ স্পনসর করুন"></a>

আপনি [বাগ রিপোর্ট করে](https://github.com/totaldrop-app/totaldrop/issues/new/choose), অনুবাদ করে, বা নিজের ডেস্কটপে পরীক্ষা করেও সাহায্য করতে পারেন।

## ডেভেলপারদের জন্য

### রিপোজিটরির গঠন

| পাথ | এটি কী | লাইসেন্স |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, কী ডেরিভেশন, মেইলবক্স ও সেশন এনক্রিপশন, ডিভাইস কী, STREAM খণ্ড এনক্রিপশন | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | ট্রান্সফার কোড, সীমা, পাথ নিরাপত্তা, প্রতিটি ওয়্যার বার্তা (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | ক্লায়েন্ট ইঞ্জিন: সংযোগকারী সার্ভার, raw পাবলিক কী সহ QUIC, Noise XXpsk3 + ML-KEM-768, ট্রান্সফার | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust প্রোগ্রামের ইন্টারফেস লেখা: ভাষা নির্বাচন, ICU বার্তা, বহুবচন, আঞ্চলিক সংখ্যা | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` কমান্ড-লাইন ক্লায়েন্ট | GPL-3.0-or-later |
| `l10n/` | ইন্টারফেসের প্রতিটি স্ট্রিং, যা Android, Linux ও CLI ভাগ করে ব্যবহার করে ([অনুবাদ](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux ডেস্কটপ অ্যাপ: `totaldrop-core`-এর ওপর Qt 6 ও QML, নেটিভ Wayland | GPL-3.0-or-later |
| `packaging/` | Linux প্যাকেজ, প্রতিটি ফরম্যাট আলাদাভাবে: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android অ্যাপ (Flutter); iOS পরে | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app এবং QR লিংক যে পেজটি খোলে | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | প্রতিটি পার্সারের জন্য ফাজ টার্গেট | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Rust-এ লেখা সংযোগকারী সার্ভার, সেলফ-হোস্টিং ও পরীক্ষার জন্য | AGPL-3.0-or-later |
| `server/rendezvous-worker` | একই সার্ভার Cloudflare Workers-এ, `rendezvous.totaldrop.app`-এর জন্য | AGPL-3.0-or-later |

লাইব্রেরিগুলো উদার (permissive) লাইসেন্সে, যাতে যে কেউ প্রোটোকলের ওপর ভিত্তি করে কিছু তৈরি করতে পারেন; অ্যাপগুলো GPL এবং সার্ভারগুলো AGPL। দেখুন [LICENSING.md](../../LICENSING.md)।

প্রতিটি প্ল্যাটফর্মের নিজস্ব সংস্করণ, ট্যাগ (`linux-v…`, `android-v…`) ও CI ওয়ার্কফ্লো আছে, তাই একটি অন্যগুলোকে না ছুঁয়েই প্রকাশ হতে পারে। ব্রাঞ্চ ও রিলিজ কীভাবে কাজ করে: [docs/gelistirme.md](../../docs/gelistirme.md) (তুর্কি ভাষায়) ও [docs/versioning.md](../../docs/versioning.md)। নকশার নিয়ম: [docs/tasarim.md](../../docs/tasarim.md) ও আর্কিটেকচার-সংক্রান্ত সিদ্ধান্ত: [docs/kararlar](../../docs/kararlar/README.md), দুটিই তুর্কি ভাষায়।

### বিল্ড

```bash
cargo test
cargo build --release
```

একটি মেশিনেই, তিনটি টার্মিনালে, একটি ট্রান্সফার চেষ্টা করুন:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

দুই পক্ষেই একই চারটি যাচাই-শব্দ দেখায়। এনক্রিপশনহীন `ws://` শুধু একই মেশিনে চলা সার্ভারের জন্য গ্রহণযোগ্য; বাকি সবকিছুকে `wss://` হতে হবে।

প্রকল্পটি সবসময় সর্বশেষ স্থিতিশীল Rust লক্ষ্য করে (`rust-toolchain.toml`-এ পিন করা); প্রথম বিল্ডে rustup সেটি ইনস্টল করে নেয়।

**Linux অ্যাপ** (Qt 6.8 বা নতুনতর দরকার: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### অবদান ও নিরাপত্তা

পুল রিকোয়েস্ট খোলার আগে [CONTRIBUTING.md](../../CONTRIBUTING.md) এবং আমাদের [আচরণবিধি](../../CODE_OF_CONDUCT.md) পড়ুন। প্রশ্ন থাকলে [SUPPORT.md](../../SUPPORT.md) দেখুন। দুর্বলতার খবর অনুগ্রহ করে [SECURITY.md](../../SECURITY.md)-এ বর্ণিত উপায়ে গোপনে জানান, কখনও কোনো পাবলিক issue-তে নয়।
