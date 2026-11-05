# Geliştirme düzeni

Bu belge deponun nasıl düzenlendiğini, hangi branch'te ne yapıldığını ve
platformların birbirini bozmadan nasıl güncellendiğini anlatır. Bir işin
issue'dan yayına giden yolu, bitti tanımı ve inceleme kuralı
[surec.md](surec.md), parçaların üst düzey mimarisi [mimari.md](mimari.md)
içindedir.

## İlke: tek depo, bağımsız platformlar

Bütün platformlar aynı Rust çekirdeğini ve aynı protokolü kullanır. Bu yüzden
kod tek depoda, tek `main` branch'inde durur. Platformlar birbirinden şu
yollarla ayrılır:

- **Klasör:** her platformun kendi klasörü vardır.
- **Sürüm:** her platformun kendi sürüm numarası ve etiketi vardır
  ([versioning.md](versioning.md)).
- **Derleme hattı:** her platformun kendi CI iş akışı vardır. Her PR'da
  `ci.yml` çalışır, hangi klasörlerin değiştiğine bakar ve yalnızca ilgili
  iş akışlarını çağırır. Sondaki `CI` işi `main` için tek zorunlu
  kontroldür. Sürümleri `android-release.yml` ve `linux-release.yml`
  etiketten derler.
- **Paket:** Linux'ta Arch, Debian ve RPM paketleri ayrı tariflerden, ayrı
  kaplarda derlenir.

Bir güvenlik düzeltmesi çekirdeğe bir kez girer ve bütün platformlara aynı
anda ulaşır. Her platform kendi temposunda yayınlanır.

## Klasörler

| Klasör | İçerik | CI |
| --- | --- | --- |
| `crates/` | Protokol çekirdeği (`totaldrop-crypto`, `-proto`, `-core`) ve CLI | `core.yml` |
| `linux/` | Linux masaüstü uygulaması (Qt 6, QML, CXX-Qt) | `linux.yml` |
| `packaging/` | Linux paket tarifleri ve derleme kapları | `linux.yml` |
| `app/` | Android uygulaması (Flutter); ileride iOS | `android.yml` |
| `server/` | Rendezvous sunucusu: Rust sürümü ve Cloudflare Workers sürümü | `core.yml`, `server.yml` |
| `site/` | totaldrop.app, download.totaldrop.app ve QR bağlantı sayfası | `site.yml` |
| `fuzz/` | Ayrıştırıcıların fuzz hedefleri | `fuzz.yml` (haftalık) |
| `formal/` | Hesap tasarımının Tamarin modeli ve mutant sınamaları (#251) | `formal.yml` |
| `docs/` | Tasarım kuralları, sürümler, protokol notları, bu belge | |
| `tools/` | Depo araçları: changelog, bileşen sürümleri, `deny.toml` ile `osv-scanner.toml`'un aynı danışma kayıtlarını yok saydığının denetimi | `ci.yml` |
| `.github/` | İş akışları; ayrıca `main`'de ve haftalık OpenSSF Scorecard, her PR'da ve `main`'de CodeQL kod taraması (GitHub varsayılan kurulumu, depo ayarlarında) | `ci.yml`, `scorecard.yml` |

Kökteki `deny.toml` (cargo-deny) ve `osv-scanner.toml` (OpenSSF Scorecard'ın
kullandığı OSV-Scanner) aynı danışma kayıtlarını, aynı gerekçeyle yok sayar.
`SECURITY-INSIGHTS.yml` güvenlik bilgilerini araçların okuyabileceği
biçimde özetler. Anlattığı bir şeyi değiştiren PR onu da günceller.

Windows ve macOS geldiğinde kendi klasörlerini (`windows/`, `macos/`) ve
kendi iş akışlarını alır.

## Branch'ler

- **`main`:** her zaman derlenen, testleri geçen hâl. Doğrudan push ve
  force push kapalıdır; değişiklik yalnızca PR ile, rebase ederek birleşir.
- **İş branch'leri:** kısa ömürlüdür ve adı platformu söyler:

  | Branch | Ne için |
  | --- | --- |
  | `feat/linux-...` | Linux uygulamasında yeni özellik |
  | `fix/android-...` | Android'de hata düzeltmesi |
  | `feat/core-...` | Çekirdekte değişiklik (bütün platformları etkiler) |
  | `build/packaging-...` | Paket tarifleri |
  | `docs/...` | Belgeler |

- İş bitince branch `main`'e birleşir ve silinir. Aylarca yaşayan platform
  branch'leri açılmaz: ayrılırlarsa çekirdekteki bir düzeltmeyi her birine
  ayrı taşımak gerekir ve biri unutulur.
- Çekirdekte bir değişiklik bütün platform iş akışlarını çalıştırır. Bir
  platformu bozan çekirdek değişikliği `main`'e giremez: `CI` kontrolü
  kırmızıyken birleştirme kapalıdır.

## Linux paketleri

Her biçim ayrı tariften ve ayrı kaptan derlenir:

| Biçim | Tarif | Derleme kabı | Hedef |
| --- | --- | --- | --- |
| Arch | `packaging/arch/PKGBUILD` | `archlinux:latest` | Arch, Omarchy, EndeavourOS, Manjaro; AUR |
| Debian | `packaging/debian/` | `debian:trixie` (Qt 6.8) | Debian 13+, Ubuntu 25.04+ ve türevleri |
| RPM | `packaging/rpm/totaldrop.spec` | `fedora:43` (Qt 6.10) | Fedora 43+ (openSUSE için ayrı bir tarif gerekir) |

Ortak dosyalar (`.desktop`, AppStream, simgeler, kurulum betiği)
`packaging/linux/` altındadır. Üç paket de aynı dosyaları aynı yerlere
kurar.

Yerelde bir paket derlemek için (Docker gerekir):

```bash
packaging/build.sh arch   # ya da deb, rpm
ls dist/packages/
```

Paketler yeniden üretilebilir olmalıdır: aynı kaynaktan ikinci bir
derleme bayt bayt aynı paketi vermelidir. `Reproducible` iş akışı
(`.github/workflows/reproducible.yml`) bunu denetler: her formatı aynı
commit'in iki temiz kopyasından iki kez derleyip karşılaştırır. Kabın
içindeki derleme yolu her zaman aynıdır (`build.sh` çalışma klasörünü
`/w`'ye bağlar), yani denetim yoldan bağımsızlığı değil, zaman damgalarını,
sıralamayı ve derleme ortamını sınar. Haftada bir, elle ya da `packaging/`, `linux/build.rs`, `linux/Cargo.toml` veya `Cargo.lock` değişince
çalışır. Paketler farklıysa ikisini de iş çıktısı olarak bırakır, fark
`diffoscope` ile incelenir. Aynılık aynı derleme imajı için geçerlidir:
kapta kurulan dağıtım paketleri günden güne değişir ([surec.md](surec.md),
sürüm adımı 4). Yerelde denemek için aynı commit'in iki temiz kopyasında
(`build.sh` çalışma ağacını, commit edilmemiş değişikliklerle birlikte
paketler) aynı komut çalıştırılır ve `sha256sum dist/packages/*`
çıktıları karşılaştırılır.

Karşılaştırmayı `packaging/compare-packages.sh` yapar: paketler aynı
değilse içlerindeki dosyaları tek tek karşılaştırır ve her farkta başarısız
olur. Tek istisna bilinen bir farktır: cxx-qt Qt kitaplıklarını rastgele
sırayla bağladığı için `totaldrop-desktop` ikilisi derlemeden derlemeye
değişebilir (#256). Bu ikilide salt okunur veri (Qt kaynakları dahil),
bağlanan kitaplıklar ve kodun boyutu yine aynı olmalıdır; öyleyse fark yalnızca
uyarı olarak yazılır. İkili farklıyken paket üst verisi (ikilinin sağlama
toplamını içerir) karşılaştırılmaz. Denetim yalnızca x86_64 paketlerini
derler; arm64 paketleri aynı betiklerden çıkar.

## Arayüz değişiklikleri

[tasarim.md](tasarim.md) bağlayıcıdır. Arayüzü değiştiren her değişiklikten
sonra uygulama açık ve koyu temada şu ortamlarda denenir ve PR'a ekran
görüntüsü eklenir: Omarchy, Hyprland (Omarchy olmadan), KDE Plasma.
