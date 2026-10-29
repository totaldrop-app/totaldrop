<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### File apa pun, untuk siapa pun, di mana pun.

Gratis dan open source. Terenkripsi end-to-end. Tanpa email, tanpa nomor telepon, tanpa akun.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Situs web](https://totaldrop.app)** · **[Unduh](https://totaldrop.app/download/)** · [Peta jalan](https://totaldrop.app/roadmap/) · [Donasi](https://totaldrop.app/donate/) · [Rilis](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · **Bahasa Indonesia** · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop di Android: layar beranda dalam mode terang dan gelap, serta transfer yang sudah tiba dan terverifikasi" width="820">

</div>

> [!NOTE]
> Terjemahan ini masih draf dan sedang ditinjau. Versi [English](../../README.md) adalah acuannya.

## Mengapa TotalDrop

- **Cukup dengan kode singkat.** Pilih file dan dapatkan kode seperti `7-harbor-rocket`. Bacakan kodenya atau tunjukkan kode QR; orang lain mengetiknya dan file pun tiba.
- **Terenkripsi end-to-end, dan Anda bisa memeriksanya.** Kedua layar menampilkan empat kata yang sama sebelum apa pun dikirim. Jika berbeda, hentikan.
- **Tidak perlu mendaftar apa pun.** Tanpa akun, email, atau nomor telepon. Akun opsional nantinya hanya berupa angka, ala Mullvad ([desain](../../docs/hesap.md)).
- **Langsung dari perangkat ke perangkat, di mana pun.** Server pertemuan hanya memperkenalkan kedua perangkat dan tidak menyimpan log. Perangkat terhubung langsung melalui internet di mana pun memungkinkan. Jika router atau firewall menghalangi, server relai meneruskan lalu lintas yang tidak dapat dibacanya.
- **Menyatu dengan setiap sistem.** Setiap aplikasi mengikuti tema, font, warna aksen, bahasa, dan setelan animasi sistemnya. Aplikasi Linux adalah Qt 6 native di Wayland dan mengikuti tema KDE Plasma, Hyprland, dan Omarchy secara langsung.
- **Gratis, selamanya.** Open source dan dibiayai oleh donasi. Tanpa iklan, tanpa pelacakan, tanpa paket berbayar untuk fitur dasar.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Aplikasi Linux di Omarchy, dengan warna, font, dan ikon desktop" width="560">
  <br><sub>Aplikasi Linux di Omarchy: warna, sudut, font, dan ikon berasal dari desktop.</sub>
</p>

## Unduh

| Platform | Dapatkan | Catatan |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 atau yang lebih baru |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 atau yang lebih baru |
| **Android** | [Ikut beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 atau yang lebih baru, Google Play dengan undangan |
| **Baris perintah** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Segera hadir | Penandatanganan kode untuk platform ini adalah hal pertama yang dibiayai donasi |

Setiap paket dibuat oleh CI dari commit bertag, tidak pernah di mesin pengembang. Untuk memeriksa paket sebelum memasangnya, bandingkan dengan `SHA256SUMS` milik rilis tersebut dan verifikasi di mana paket itu dibuat:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Kode ini belum menjalani tinjauan keamanan independen, jadi mohon jangan mengandalkannya untuk file sensitif.

## Cara kerja

1. Pengirim meminta kode singkat dari server pertemuan. Kode ini adalah satu-satunya rahasia yang dimiliki bersama oleh kedua orang.
2. Kedua perangkat mengubah kode menjadi kunci yang kuat dengan **CPace**. Penyerang hanya mendapat satu tebakan per transfer dan tidak dapat menguji kode secara offline.
3. Perangkat terhubung langsung melalui **QUIC** (TLS 1.3 dengan kunci publik mentah yang di-pin) dan menjalankan **Noise XXpsk3 dengan ML-KEM-768**, yang tetap aman terhadap komputer kuantum di masa depan.
4. File dienkripsi per potongan (STREAM), sehingga setiap bagian diautentikasi begitu tiba.

Protokolnya dijelaskan dalam *TotalDrop Protocol v0*. Keputusan yang diambil saat mengimplementasikannya ada di [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), dan apa yang dapat dan tidak dapat dilihat setiap server ada di [model ancaman](../../docs/tehdit-modeli.md) dan [kebijakan privasi](https://totaldrop.app/privacy).

## Dalam bahasa Anda

TotalDrop tersedia dalam 35 bahasa dan mengikuti format angka wilayah Anda. Bahasa Inggris dan Turki sudah ditinjau; 33 lainnya masih draf yang sedang diperiksa penutur asli, sehingga aplikasi mempertahankan teks keamanan (verifikasi, kode salah, terima dan tolak) dalam bahasa Inggris sampai bahasa tersebut selesai ditinjau. Situs sudah memiliki semuanya hari ini; aplikasi mendapatkannya pada rilis berikutnya.

Menemukan kalimat yang janggal? [Bantu terjemahkan](../../docs/ceviri.md).

## Dukung proyek ini

TotalDrop tetap gratis karena orang-orang ikut menyumbang. Donasi membiayai server relai, sertifikat penandatanganan kode yang diwajibkan Windows dan macOS, serta audit keamanan independen sebelum versi 1.0. Setiap biaya, baris demi baris, dan dana yang terkumpul sejauh ini: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Sponsori di GitHub"></a>

Anda juga dapat membantu dengan [melaporkan bug](https://github.com/totaldrop-app/totaldrop/issues/new/choose), menerjemahkan, atau menguji di desktop Anda.

## Untuk pengembang

### Struktur repositori

| Path | Isinya | Lisensi |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, derivasi kunci, enkripsi kotak surat dan sesi, kunci perangkat, enkripsi potongan STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Kode transfer, batasan, keamanan path, setiap pesan jaringan (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Mesin klien: server pertemuan, QUIC dengan kunci publik mentah, Noise XXpsk3 + ML-KEM-768, transfer | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Teks antarmuka untuk program Rust: negosiasi bahasa, pesan ICU, bentuk jamak, angka regional | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Klien baris perintah `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Setiap string antarmuka, dipakai bersama oleh Android, Linux, dan CLI ([menerjemahkan](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Aplikasi desktop Linux: Qt 6 dan QML di atas `totaldrop-core`, Wayland native | GPL-3.0-or-later |
| `packaging/` | Paket Linux, setiap format tersendiri: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Aplikasi Android (Flutter); iOS menyusul | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app, dan halaman yang dibuka oleh tautan QR | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Target fuzz untuk setiap parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Server pertemuan dalam Rust, untuk di-host sendiri dan pengujian | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Server yang sama di Cloudflare Workers, untuk `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Pustaka-pustakanya berlisensi permisif agar siapa pun dapat membangun di atas protokol ini; aplikasinya GPL dan servernya AGPL. Lihat [LICENSING.md](../../LICENSING.md).

Setiap platform memiliki versi, tag (`linux-v…`, `android-v…`), dan alur kerja CI sendiri, sehingga satu platform dapat dirilis tanpa menyentuh yang lain. Cara kerja branch dan rilis: [docs/gelistirme.md](../../docs/gelistirme.md) dan [docs/versioning.md](../../docs/versioning.md). Aturan desain: [docs/tasarim.md](../../docs/tasarim.md). Keputusan arsitektur: [docs/kararlar](../../docs/kararlar/README.md).

### Build

```bash
cargo test
cargo build --release
```

Coba transfer di satu mesin, dengan tiga terminal:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Kedua sisi menampilkan empat kata verifikasi yang sama. `ws://` tanpa enkripsi hanya diterima untuk server di mesin yang sama; selain itu harus `wss://`.

Proyek ini selalu menargetkan Rust stabil terbaru (di-pin di `rust-toolchain.toml`); rustup memasangnya saat build pertama.

**Aplikasi Linux** (memerlukan Qt 6.8 atau yang lebih baru: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Kontribusi dan keamanan

Baca [CONTRIBUTING.md](../../CONTRIBUTING.md) sebelum membuka pull request, serta [kode etik](../../CODE_OF_CONDUCT.md) kami. Pertanyaan diajukan melalui [SUPPORT.md](../../SUPPORT.md). Mohon laporkan kerentanan secara pribadi seperti dijelaskan di [SECURITY.md](../../SECURITY.md), jangan pernah di issue publik.
