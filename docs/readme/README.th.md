<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### ไฟล์อะไรก็ได้ ถึงใครก็ได้ ที่ไหนก็ได้

ฟรีและเป็นโอเพนซอร์ส เข้ารหัสตั้งแต่ต้นทางถึงปลายทาง ไม่ต้องใช้อีเมล ไม่ต้องใช้เบอร์โทรศัพท์ ไม่ต้องมีบัญชี

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[เว็บไซต์](https://totaldrop.app)** · **[ดาวน์โหลด](https://totaldrop.app/download/)** · [แผนงาน](https://totaldrop.app/roadmap/) · [บริจาค](https://totaldrop.app/donate/) · [รุ่นที่เผยแพร่](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · **ไทย** · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop บน Android: หน้าจอหลักในโหมดสว่างและโหมดมืด และการโอนที่มาถึงและผ่านการยืนยันแล้ว" width="820">

</div>

> [!NOTE]
> คำแปลนี้เป็นฉบับร่างและอยู่ระหว่างการตรวจทาน ฉบับ[ภาษาอังกฤษ](../../README.md) คือฉบับอ้างอิง

## ทำไมต้อง TotalDrop

- **ใช้แค่รหัสสั้นๆ** เลือกไฟล์แล้วรับรหัสอย่าง `7-harbor-rocket` อ่านให้ฟังหรือแสดง QR โค้ด อีกฝ่ายพิมพ์รหัสนั้น แล้วไฟล์ก็จะมาถึง
- **เข้ารหัสตั้งแต่ต้นทางถึงปลายทาง และคุณตรวจสอบได้เอง** หน้าจอทั้งสองแสดงสี่คำเดียวกันก่อนที่จะมีการส่งข้อมูลใดๆ ถ้าคำไม่ตรงกัน ให้หยุด
- **ไม่ต้องสมัครอะไรเลย** ไม่ต้องมีบัญชี อีเมล หรือเบอร์โทรศัพท์ บัญชีแบบไม่บังคับจะเป็นเพียงตัวเลขเท่านั้น แบบเดียวกับ Mullvad ([การออกแบบ](../../docs/hesap.md))
- **ตรงจากอุปกรณ์ถึงอุปกรณ์ ที่ไหนก็ได้** เซิร์ฟเวอร์นัดพบทำหน้าที่แนะนำอุปกรณ์ทั้งสองให้รู้จักกันเท่านั้น และไม่เก็บบันทึก ทั้งนี้ เช่นเดียวกับเซิร์ฟเวอร์ทุกตัว เซิร์ฟเวอร์นี้เห็นที่อยู่ IP ของอุปกรณ์ทั้งสอง อุปกรณ์จะเชื่อมต่อกันโดยตรงผ่านอินเทอร์เน็ตทุกครั้งที่ทำได้ เมื่อมีเราเตอร์หรือไฟร์วอลล์ขวางอยู่ รีเลย์จะส่งต่อทราฟฟิกที่ตัวมันเองอ่านไม่ได้
- **กลมกลืนกับระบบของคุณ** แอปใช้ธีมสว่างหรือมืด สีเน้น ภาษา และการตั้งค่าลดการเคลื่อนไหวตามระบบ แอป Linux เป็น Qt 6 แบบเนทีฟบน Wayland และยังรับฟอนต์ มุม และไอคอนจากธีมของ KDE Plasma, Hyprland และ Omarchy แบบทันที
- **ฟรี ตลอดไป** เป็นโอเพนซอร์สและดำเนินงานด้วยเงินบริจาค ไม่มีโฆษณา ไม่มีการติดตาม ไม่มีแพ็กเกจแบบเสียเงินสำหรับฟังก์ชันพื้นฐาน

<p align="center">
  <img src="../../docs/assets/linux.png" alt="แอป Linux บน Omarchy ในสี ฟอนต์ และไอคอนของเดสก์ท็อป" width="560">
  <br><sub>แอป Linux บน Omarchy: สี มุมโค้ง ฟอนต์ และไอคอน มาจากเดสก์ท็อป</sub>
</p>

## ดาวน์โหลด

| แพลตฟอร์ม | ดาวน์โหลดได้ที่ | หมายเหตุ |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 ขึ้นไป |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 ขึ้นไป |
| **Android** | [เข้าร่วมเบต้า](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 ขึ้นไป, Google Play ตามคำเชิญ |
| **บรรทัดคำสั่ง** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | กำลังมา | การเซ็นโค้ดสำหรับแพลตฟอร์มเหล่านี้คือสิ่งแรกที่เงินบริจาคจะนำไปจ่าย |

ทุกแพ็กเกจสร้างโดย CI จากคอมมิตที่ติดแท็ก ไม่เคยสร้างบนเครื่องของนักพัฒนา หากต้องการตรวจสอบแพ็กเกจก่อนติดตั้ง ให้เทียบกับ `SHA256SUMS` ของรุ่นนั้น และยืนยันว่าแพ็กเกจสร้างขึ้นที่ใด:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **เบต้า** โค้ดยังไม่ผ่านการตรวจสอบความปลอดภัยโดยอิสระ จึงไม่ควรใช้กับไฟล์ที่มีความอ่อนไหว

## วิธีการทำงาน

1. ผู้ส่งขอรหัสสั้นๆ จากเซิร์ฟเวอร์นัดพบ รหัสนี้เป็นความลับเพียงอย่างเดียวที่ทั้งสองคนมีร่วมกัน
2. อุปกรณ์ทั้งสองแปลงรหัสเป็นคีย์ที่แข็งแรงด้วย **CPace** ผู้โจมตีเดาได้เพียงครั้งเดียวต่อการโอนหนึ่งครั้ง และไม่สามารถทดสอบรหัสแบบออฟไลน์ได้
3. อุปกรณ์เชื่อมต่อกันโดยตรงผ่าน **QUIC** (TLS 1.3 กับคีย์สาธารณะแบบ raw ที่ปักหมุดไว้) และใช้ **Noise XXpsk3 กับ ML-KEM-768** ซึ่งออกแบบมาให้ทนทานต่อคอมพิวเตอร์ควอนตัมในอนาคต
4. ไฟล์ถูกเข้ารหัสเป็นส่วนๆ (STREAM) ทุกส่วนจึงได้รับการยืนยันความถูกต้องทันทีที่มาถึง

โปรโตคอลอธิบายไว้ใน *TotalDrop Protocol v0* การตัดสินใจระหว่างการพัฒนาอยู่ใน [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) และสิ่งที่เซิร์ฟเวอร์แต่ละตัวเห็นได้และเห็นไม่ได้อยู่ใน[แบบจำลองภัยคุกคาม](../../docs/tehdit-modeli.md) (ภาษาตุรกี) และ[นโยบายความเป็นส่วนตัว](https://totaldrop.app/privacy)

## ในภาษาของคุณ

TotalDrop รองรับ 35 ภาษา และใช้รูปแบบตัวเลขตามภูมิภาคของคุณ ภาษาอังกฤษและภาษาตุรกีผ่านการตรวจทานแล้ว อีก 33 ภาษาเป็นฉบับร่างที่เจ้าของภาษากำลังตรวจทาน แอปจึงคงข้อความด้านความปลอดภัย (การยืนยัน รหัสผิด การยอมรับและการปฏิเสธ) ไว้เป็นภาษาอังกฤษจนกว่าภาษานั้นจะผ่านการตรวจทาน วันนี้เว็บไซต์มีครบทุกภาษาแล้ว ส่วนแอปจะได้รับในรุ่นถัดไป

เจอประโยคที่ฟังไม่เป็นธรรมชาติใช่ไหม [ช่วยแปล](../../docs/ceviri.md) (คู่มือเป็นภาษาตุรกี)

## สนับสนุนโครงการ

TotalDrop ฟรีได้เพราะมีผู้คนช่วยกันสนับสนุน เงินบริจาคใช้จ่ายค่ารีเลย์ ค่าใบรับรองการเซ็นโค้ดที่ Windows และ macOS กำหนด และค่าการตรวจสอบความปลอดภัยโดยอิสระก่อนเวอร์ชัน 1.0 ดูค่าใช้จ่ายทุกรายการ ทีละบรรทัด และยอดที่ระดมได้จนถึงตอนนี้ได้ที่ [totaldrop.app/donate](https://totaldrop.app/donate/)

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="สนับสนุนบน GitHub"></a>

คุณยังช่วยได้ด้วยการ[รายงานบั๊ก](https://github.com/totaldrop-app/totaldrop/issues/new/choose) แปลภาษา หรือทดสอบบนเดสก์ท็อปของคุณ

## สำหรับนักพัฒนา

### โครงสร้างรีโพซิทอรี

| พาธ | คืออะไร | สัญญาอนุญาต |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace การสร้างคีย์ การเข้ารหัสกล่องจดหมายและเซสชัน คีย์ของอุปกรณ์ การเข้ารหัสแบบแบ่งส่วนด้วย STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | รหัสการโอน ขีดจำกัด ความปลอดภัยของพาธ ข้อความทุกชนิดที่ส่งผ่านเครือข่าย (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | เอนจินฝั่งไคลเอนต์: เซิร์ฟเวอร์นัดพบ, QUIC กับคีย์สาธารณะแบบ raw, Noise XXpsk3 + ML-KEM-768, การโอน | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | ข้อความอินเทอร์เฟซของโปรแกรม Rust: การเลือกภาษา ข้อความ ICU รูปพหูพจน์ ตัวเลขตามภูมิภาค | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | ไคลเอนต์บรรทัดคำสั่ง `totaldrop` | GPL-3.0-or-later |
| `l10n/` | ข้อความอินเทอร์เฟซทุกข้อความ ใช้ร่วมกันโดย Android, Linux และ CLI ([การแปล](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | แอปเดสก์ท็อป Linux: Qt 6 และ QML บน `totaldrop-core`, Wayland แบบเนทีฟ | GPL-3.0-or-later |
| `packaging/` | แพ็กเกจ Linux แยกตามรูปแบบ: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | แอป Android (Flutter); iOS ภายหลัง | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app และหน้าที่ลิงก์ QR เปิด | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | เป้าหมาย fuzz สำหรับตัวแยกวิเคราะห์ทุกตัว | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | เซิร์ฟเวอร์นัดพบที่เขียนด้วย Rust สำหรับโฮสต์เองและการทดสอบ | AGPL-3.0-or-later |
| `server/rendezvous-worker` | เซิร์ฟเวอร์เดียวกันบน Cloudflare Workers สำหรับ `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

ไลบรารีใช้สัญญาอนุญาตแบบผ่อนปรน เพื่อให้ใครก็สร้างงานต่อบนโปรโตคอลได้ แอปใช้ GPL และเซิร์ฟเวอร์ใช้ AGPL ดู [LICENSING.md](../../LICENSING.md)

แต่ละแพลตฟอร์มมีเวอร์ชัน แท็ก (`linux-v…`, `android-v…`) และเวิร์กโฟลว์ CI ของตัวเอง จึงออกรุ่นได้โดยไม่กระทบแพลตฟอร์มอื่น วิธีการทำงานของสาขาและการออกรุ่น: [docs/gelistirme.md](../../docs/gelistirme.md) (ภาษาตุรกี) และ [docs/versioning.md](../../docs/versioning.md) กฎการออกแบบ: [docs/tasarim.md](../../docs/tasarim.md) และการตัดสินใจด้านสถาปัตยกรรม: [docs/kararlar](../../docs/kararlar/README.md) ทั้งสองเป็นภาษาตุรกี

### การบิลด์

```bash
cargo test
cargo build --release
```

ลองโอนไฟล์บนเครื่องเดียวด้วยเทอร์มินัลสามหน้าต่าง:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

ทั้งสองฝั่งแสดงคำยืนยันสี่คำเดียวกัน `ws://` ที่ไม่เข้ารหัสยอมรับเฉพาะเซิร์ฟเวอร์บนเครื่องเดียวกันเท่านั้น กรณีอื่นทั้งหมดต้องเป็น `wss://`

โครงการใช้ Rust รุ่นเสถียรล่าสุดเสมอ (ปักหมุดไว้ใน `rust-toolchain.toml`) rustup จะติดตั้งให้ในการบิลด์ครั้งแรก

**แอป Linux** (ต้องใช้ Qt 6.8 ขึ้นไป: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### การมีส่วนร่วมและความปลอดภัย

โปรดอ่าน [CONTRIBUTING.md](../../CONTRIBUTING.md) และ[แนวปฏิบัติของชุมชน](../../CODE_OF_CONDUCT.md)ของเราก่อนเปิด pull request คำถามต่างๆ ให้ดูที่ [SUPPORT.md](../../SUPPORT.md) โปรดรายงานช่องโหว่แบบส่วนตัวตามที่อธิบายไว้ใน [SECURITY.md](../../SECURITY.md) และอย่ารายงานใน issue สาธารณะ
