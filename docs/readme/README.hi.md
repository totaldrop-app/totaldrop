<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### कोई भी फ़ाइल, किसी को भी, कहीं भी।

मुफ़्त और ओपन सोर्स। एंड-टू-एंड एन्क्रिप्टेड। न ईमेल, न फ़ोन नंबर, न खाता।

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[वेबसाइट](https://totaldrop.app)** · **[डाउनलोड](https://totaldrop.app/download/)** · [रोडमैप](https://totaldrop.app/roadmap/) · [दान करें](https://totaldrop.app/donate/) · [रिलीज़](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · **हिन्दी** · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android पर TotalDrop: लाइट और डार्क में होम स्क्रीन, और एक ट्रांसफ़र जो पहुँच गया और सत्यापित हुआ" width="820">

</div>

> [!NOTE]
> यह अनुवाद एक मसौदा है और इसकी समीक्षा की जा रही है। [English](../../README.md) संस्करण ही संदर्भ है।

## TotalDrop क्यों

- **बस एक छोटा कोड काफ़ी है।** फ़ाइलें चुनें और `7-harbor-rocket` जैसा कोड पाएँ। उसे बोलकर बताएँ या QR कोड दिखाएँ; दूसरा व्यक्ति उसे टाइप करता है और फ़ाइलें पहुँच जाती हैं।
- **एंड-टू-एंड एन्क्रिप्टेड, और आप इसे जाँच सकते हैं।** कुछ भी भेजे जाने से पहले दोनों स्क्रीन पर वही चार शब्द दिखते हैं। अगर वे अलग हों, तो रुक जाएँ।
- **किसी चीज़ के लिए साइन अप नहीं करना।** न खाता, न ईमेल, न फ़ोन नंबर। वैकल्पिक खाते सिर्फ़ नंबर होंगे, Mullvad की तरह ([डिज़ाइन](../../docs/hesap.md))।
- **सीधे एक डिवाइस से दूसरे डिवाइस तक, कहीं भी।** संपर्क सर्वर सिर्फ़ दोनों डिवाइस का परिचय कराता है और कोई लॉग नहीं रखता। जहाँ भी संभव हो, डिवाइस इंटरनेट पर सीधे कनेक्ट होते हैं। जब कोई राउटर या फ़ायरवॉल बीच में आता है, तो एक रिले सर्वर ट्रैफ़िक को आगे पहुँचाता है, जिसे वह पढ़ नहीं सकता।
- **हर सिस्टम पर घर जैसा।** हर ऐप अपने सिस्टम की थीम, फ़ॉन्ट, एक्सेंट रंग, भाषा और मोशन सेटिंग का पालन करता है। Linux ऐप Wayland पर नेटिव Qt 6 है और KDE Plasma, Hyprland और Omarchy की थीम को लाइव फ़ॉलो करता है।
- **मुफ़्त, हमेशा के लिए।** ओपन सोर्स और दान से चलता है। न विज्ञापन, न ट्रैकिंग, बुनियादी सुविधाओं के लिए कोई पेड प्लान नहीं।

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy पर Linux ऐप, डेस्कटॉप के रंगों, फ़ॉन्ट और आइकन में" width="560">
  <br><sub>Omarchy पर Linux ऐप: रंग, कोने, फ़ॉन्ट और आइकन डेस्कटॉप से आते हैं।</sub>
</p>

## डाउनलोड

| प्लैटफ़ॉर्म | कहाँ से पाएँ | नोट |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 या नया |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 या नया |
| **Android** | [बीटा में शामिल हों](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 या नया, Google Play पर आमंत्रण से |
| **कमांड लाइन** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | जल्द आ रहा है | दान से सबसे पहले इन्हीं के लिए कोड साइनिंग का खर्च उठाया जाएगा |

हर पैकेज CI द्वारा एक टैग किए गए कमिट से बनाया जाता है, कभी किसी डेवलपर की मशीन पर नहीं। इंस्टॉल करने से पहले किसी पैकेज को जाँचने के लिए, उसे रिलीज़ की `SHA256SUMS` से मिलाएँ और सत्यापित करें कि वह कहाँ बना था:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **बीटा।** कोड की अभी तक कोई स्वतंत्र सुरक्षा समीक्षा नहीं हुई है, इसलिए कृपया संवेदनशील फ़ाइलों के लिए इस पर भरोसा न करें।

## यह कैसे काम करता है

1. भेजने वाला संपर्क सर्वर से एक छोटा कोड माँगता है। यह कोड ही वह अकेला रहस्य है जो दोनों लोग साझा करते हैं।
2. दोनों डिवाइस **CPace** से कोड को एक मज़बूत कुंजी में बदलते हैं। हमलावर को हर ट्रांसफ़र में सिर्फ़ एक अनुमान मिलता है और वह कोड को ऑफ़लाइन नहीं परख सकता।
3. वे **QUIC** (पिन की गई raw सार्वजनिक कुंजियों के साथ TLS 1.3) पर सीधे कनेक्ट होते हैं और **ML-KEM-768 के साथ Noise XXpsk3** चलाते हैं, जो भविष्य के क्वांटम कंप्यूटरों के सामने भी सुरक्षित रहता है।
4. फ़ाइलें टुकड़ों में एन्क्रिप्ट होती हैं (STREAM), इसलिए हर टुकड़ा पहुँचते ही प्रमाणित होता है।

प्रोटोकॉल का विवरण *TotalDrop Protocol v0* में है। इसे लागू करते समय लिए गए फ़ैसले [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) में हैं, और हर सर्वर क्या देख सकता है और क्या नहीं, यह [ख़तरा मॉडल](../../docs/tehdit-modeli.md) और [निजता नीति](https://totaldrop.app/privacy) में है।

## आपकी भाषा में

TotalDrop 35 भाषाएँ बोलता है और आपके क्षेत्र के संख्या प्रारूपों का पालन करता है। अंग्रेज़ी और तुर्की की समीक्षा हो चुकी है; बाकी 33 मसौदे हैं जिन्हें मूल भाषी जाँच रहे हैं, इसलिए जब तक किसी भाषा की समीक्षा नहीं हो जाती, ऐप सुरक्षा से जुड़े पाठ (सत्यापन, गलत कोड, स्वीकार और अस्वीकार) अंग्रेज़ी में रखते हैं। साइट पर आज सभी भाषाएँ हैं; ऐप में वे अगली रिलीज़ के साथ आएँगी।

कोई अटपटा वाक्य दिखा? [अनुवाद में मदद करें](../../docs/ceviri.md)।

## प्रोजेक्ट का समर्थन करें

TotalDrop मुफ़्त है क्योंकि लोग योगदान देते हैं। दान से रिले सर्वर, Windows और macOS के लिए ज़रूरी कोड-साइनिंग प्रमाणपत्र, और 1.0 से पहले एक स्वतंत्र सुरक्षा ऑडिट का खर्च चलता है। हर खर्च, एक-एक पंक्ति में, और अब तक कितना जुटा है: [totaldrop.app/donate](https://totaldrop.app/donate/)।

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="GitHub पर स्पॉन्सर करें"></a>

आप [बग रिपोर्ट करके](https://github.com/totaldrop-app/totaldrop/issues/new/choose), अनुवाद करके, या अपने डेस्कटॉप पर टेस्ट करके भी मदद कर सकते हैं।

## डेवलपर्स के लिए

### रिपॉज़िटरी की संरचना

| पाथ | यह क्या है | लाइसेंस |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, कुंजी व्युत्पत्ति, मेलबॉक्स और सत्र एन्क्रिप्शन, डिवाइस कुंजियाँ, STREAM टुकड़ा एन्क्रिप्शन | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | ट्रांसफ़र कोड, सीमाएँ, पाथ सुरक्षा, हर वायर संदेश (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | क्लाइंट इंजन: संपर्क सर्वर, raw सार्वजनिक कुंजियों के साथ QUIC, Noise XXpsk3 + ML-KEM-768, ट्रांसफ़र | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust प्रोग्रामों के लिए इंटरफ़ेस पाठ: भाषा चयन, ICU संदेश, बहुवचन, क्षेत्रीय संख्याएँ | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` कमांड-लाइन क्लाइंट | GPL-3.0-or-later |
| `l10n/` | हर इंटरफ़ेस स्ट्रिंग, जिसे Android, Linux और CLI साझा करते हैं ([अनुवाद](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux डेस्कटॉप ऐप: `totaldrop-core` के ऊपर Qt 6 और QML, नेटिव Wayland | GPL-3.0-or-later |
| `packaging/` | Linux पैकेज, हर फ़ॉर्मैट अलग: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android ऐप (Flutter); iOS बाद में | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app और वह पेज जिसे QR लिंक खोलते हैं | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | हर पार्सर के लिए फ़ज़ टारगेट | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Rust में संपर्क सर्वर, सेल्फ़-होस्टिंग और टेस्ट के लिए | AGPL-3.0-or-later |
| `server/rendezvous-worker` | वही सर्वर Cloudflare Workers पर, `rendezvous.totaldrop.app` के लिए | AGPL-3.0-or-later |

लाइब्रेरी उदार (permissive) लाइसेंस में हैं ताकि कोई भी प्रोटोकॉल पर आगे निर्माण कर सके; ऐप GPL में हैं और सर्वर AGPL में। देखें [LICENSING.md](../../LICENSING.md)।

हर प्लैटफ़ॉर्म का अपना वर्शन, टैग (`linux-v…`, `android-v…`) और CI वर्कफ़्लो है, ताकि कोई एक बाकियों को छुए बिना रिलीज़ हो सके। ब्रांच और रिलीज़ कैसे काम करते हैं: [docs/gelistirme.md](../../docs/gelistirme.md) और [docs/versioning.md](../../docs/versioning.md)। डिज़ाइन के नियम: [docs/tasarim.md](../../docs/tasarim.md)। आर्किटेक्चर के फ़ैसले: [docs/kararlar](../../docs/kararlar/README.md)।

### बिल्ड

```bash
cargo test
cargo build --release
```

एक ही मशीन पर, तीन टर्मिनल में, एक ट्रांसफ़र आज़माएँ:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

दोनों तरफ़ वही चार सत्यापन शब्द दिखते हैं। बिना एन्क्रिप्शन वाला `ws://` सिर्फ़ उसी मशीन पर चल रहे सर्वर के लिए स्वीकार किया जाता है; बाकी सब कुछ `wss://` होना चाहिए।

प्रोजेक्ट हमेशा नवीनतम स्थिर Rust को लक्ष्य करता है (`rust-toolchain.toml` में पिन किया गया); पहले बिल्ड पर rustup उसे इंस्टॉल कर देता है।

**Linux ऐप** (Qt 6.8 या नया चाहिए: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### योगदान और सुरक्षा

पुल रिक्वेस्ट खोलने से पहले [CONTRIBUTING.md](../../CONTRIBUTING.md) और हमारी [आचार संहिता](../../CODE_OF_CONDUCT.md) पढ़ें। सवालों के लिए [SUPPORT.md](../../SUPPORT.md) देखें। कृपया कमज़ोरियों की जानकारी [SECURITY.md](../../SECURITY.md) में बताए तरीके से निजी तौर पर दें, कभी किसी सार्वजनिक issue में नहीं।
