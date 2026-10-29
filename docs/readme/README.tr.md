<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Her dosyayı, herkese, her yere.

Ücretsiz ve açık kaynak. Uçtan uca şifreli. E-posta yok, telefon numarası yok, hesap yok.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Web sitesi](https://totaldrop.app)** · **[İndir](https://totaldrop.app/download/)** · [Yol haritası](https://totaldrop.app/roadmap/) · [Bağış](https://totaldrop.app/donate/) · [Sürümler](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · **Türkçe** · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Android'de TotalDrop: açık ve koyu temada ana ekran ile ulaşmış ve doğrulanmış bir aktarım" width="820">

</div>

> [!NOTE]
> Başvuru metni [İngilizce](../../README.md) sürümüdür.

## Neden TotalDrop

- **Kısa bir kod yeter.** Dosyaları seçin ve `7-harbor-rocket` gibi bir kod alın. Kodu sesli okuyun ya da QR kodunu gösterin; karşı taraf kodu girer ve dosyalar gelir.
- **Uçtan uca şifreli ve bunu siz de doğrulayabilirsiniz.** Hiçbir şey aktarılmadan önce iki ekran da aynı dört sözcüğü gösterir. Sözcükler farklıysa durun.
- **Kaydolacak hiçbir şey yok.** Hesap, e-posta ya da telefon numarası yok. İsteğe bağlı hesaplar Mullvad tarzında yalnızca bir numaradan oluşacak ([tasarım](../../docs/hesap.md)).
- **Doğrudan cihazdan cihaza, her yerde.** Buluşma sunucusu yalnızca iki cihazı tanıştırır ve kayıt tutmaz; her sunucu gibi, cihazların IP adreslerini görür. Cihazlar internet üzerinden mümkün olan her yerde doğrudan bağlanır. Arada bir modem ya da güvenlik duvarı varsa trafiği, içeriğini okuyamayan bir aktarma sunucusu taşır.
- **Sisteminizde evindeymiş gibi.** Uygulamalar sistemin açık ya da koyu temasını, vurgu rengini, dilini ve azaltılmış hareket ayarını izler. Linux uygulaması Wayland üzerinde yerel bir Qt 6 uygulamasıdır; yazı tiplerini, köşelerini ve simgelerini de KDE Plasma, Hyprland ve Omarchy temalarından canlı alır.
- **Ücretsiz, kalıcı olarak.** Açık kaynak, bağışlarla ayakta. Reklam yok, takip yok, temel özellikler için ücretli sürüm yok.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Omarchy'de Linux uygulaması, masaüstünün renkleri, yazı tipleri ve simgeleriyle" width="560">
  <br><sub>Omarchy'de Linux uygulaması: renkler, köşeler, yazı tipleri ve simgeler masaüstünden gelir.</sub>
</p>

## İndir

| Platform | İndirme | Notlar |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 veya daha yenisi |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 veya daha yenisi |
| **Android** | [Betaya katılın](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 veya daha yenisi, Google Play'de davetle |
| **Komut satırı** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Yolda | Bağışların ilk karşılayacağı şey bunların kod imzalaması |

Her paket, bir geliştiricinin bilgisayarında değil, etiketli bir commit'ten CI tarafından derlenir. Bir paketi kurmadan önce denetlemek için onu sürümdeki `SHA256SUMS` ile karşılaştırın ve nerede derlendiğini doğrulayın:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Kod henüz bağımsız bir güvenlik incelemesinden geçmedi; lütfen hassas dosyalar için ona güvenmeyin.

## Nasıl çalışır

1. Gönderen, buluşma sunucusundan kısa bir kod ister. Kod, iki kişinin paylaştığı tek sırdır.
2. İki cihaz da kodu **CPace** ile güçlü bir anahtara dönüştürür. Bir saldırganın her aktarımda yalnızca bir tahmin hakkı olur ve kodları çevrimdışı deneyemez.
3. Cihazlar **QUIC** (sabitlenmiş ham açık anahtarlarla TLS 1.3) üzerinden doğrudan bağlanır ve gelecekteki kuantum bilgisayarlara karşı da dayanacak şekilde tasarlanmış **ML-KEM-768 ile Noise XXpsk3** çalıştırır.
4. Dosyalar parçalar halinde şifrelenir (STREAM); böylece her parça ulaştığı anda doğrulanır.

Protokol *TotalDrop Protokolü v0* belgesinde anlatılır. Uygulanırken alınan kararlar [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md) dosyasındadır; her sunucunun neyi görüp neyi göremediği ise [tehdit modeli](../../docs/tehdit-modeli.md) ve [gizlilik politikası](https://totaldrop.app/privacy) içinde yer alır.

## Kendi dilinizde

TotalDrop 35 dil konuşur ve bölgenizin sayı biçimlerini izler. İngilizce ve Türkçe gözden geçirildi; diğer 33 dil, ana dili konuşanların kontrol ettiği taslaklardır. Bu yüzden uygulamalar, bir dil gözden geçirilene kadar güvenlik metinlerini (doğrulama, yanlış kod, kabul ve ret) İngilizce gösterir. Sitede bugün hepsi var; uygulamalar onları sonraki sürümlerinde alacak.

Kulağa tuhaf gelen bir cümle mi gördünüz? [Çeviriye yardım edin](../../docs/ceviri.md).

## Projeye destek olun

TotalDrop, insanlar katkıda bulunduğu için ücretsiz kalıyor. Bağışlar aktarma sunucusunu, Windows ve macOS'un istediği kod imzalama sertifikalarını ve 1.0'dan önce bağımsız bir güvenlik denetimini karşılar. Her maliyet satır satır ve şimdiye kadar toplanan miktar: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="GitHub'da destek olun"></a>

[Hata bildirerek](https://github.com/totaldrop-app/totaldrop/issues/new/choose), çeviri yaparak ya da masaüstünüzde test ederek de yardım edebilirsiniz.

## Geliştiriciler için

### Depo yapısı

| Yol | Nedir | Lisans |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, anahtar türetme, posta kutusu ve oturum şifrelemesi, cihaz anahtarları, STREAM parça şifrelemesi | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Aktarım kodları, sınırlar, yol güvenliği, ağdaki her mesaj (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | İstemci motoru: buluşma sunucusu, ham açık anahtarlarla QUIC, Noise XXpsk3 + ML-KEM-768, aktarım | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Rust programlarının arayüz metinleri: dil eşleştirme, ICU mesajları, çoğul biçimler, bölgesel sayılar | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | `totaldrop` komut satırı istemcisi | GPL-3.0-or-later |
| `l10n/` | Android, Linux ve CLI'ın paylaştığı tüm arayüz metinleri ([çeviri](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux masaüstü uygulaması: `totaldrop-core` üzerinde Qt 6 ve QML, yerel Wayland | GPL-3.0-or-later |
| `packaging/` | Linux paketleri, her biçim kendi başına: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android uygulaması (Flutter); iOS daha sonra | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app ve QR bağlantılarının açtığı sayfa | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Her ayrıştırıcı için fuzz hedefleri | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Rust ile yazılmış buluşma sunucusu, kendi sunucunuzda barındırmak ve testler için | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Aynı sunucu Cloudflare Workers üzerinde, `rendezvous.totaldrop.app` için | AGPL-3.0-or-later |

Kütüphaneler serbest lisanslıdır, böylece herkes protokolün üzerine bir şey kurabilir; uygulamalar GPL, sunucular AGPL lisanslıdır. Bkz. [LICENSING.md](../../LICENSING.md).

Her platformun kendi sürümü, etiketi (`linux-v…`, `android-v…`) ve CI iş akışı vardır; böylece biri diğerlerine dokunmadan yayımlanabilir. Dalların ve sürümlerin nasıl işlediği: [docs/gelistirme.md](../../docs/gelistirme.md) ve [docs/versioning.md](../../docs/versioning.md). Tasarım kuralları: [docs/tasarim.md](../../docs/tasarim.md). Mimari kararlar: [docs/kararlar](../../docs/kararlar/README.md).

### Derleme

```bash
cargo test
cargo build --release
```

Tek bir makinede, üç terminalde bir aktarım deneyin:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

İki taraf da aynı dört doğrulama sözcüğünü gösterir. Şifresiz `ws://` yalnızca aynı makinedeki bir sunucu için kabul edilir; diğer her şey `wss://` olmalıdır.

Proje her zaman en yeni kararlı Rust sürümünü hedefler (`rust-toolchain.toml` içinde sabitlenmiştir); rustup onu ilk derlemede kurar.

**Linux uygulaması** (Qt 6.8 veya daha yenisi gerekir: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Katkı ve güvenlik

Pull request açmadan önce [CONTRIBUTING.md](../../CONTRIBUTING.md) dosyasını ve [davranış kurallarımızı](../../CODE_OF_CONDUCT.md) okuyun. Sorular için: [SUPPORT.md](../../SUPPORT.md). Güvenlik açıklarını lütfen herkese açık bir issue'da değil, [SECURITY.md](../../SECURITY.md) dosyasında anlatıldığı gibi gizli olarak bildirin.
