# Geliştirme düzeni

Bu belge deponun nasıl düzenlendiğini, hangi branch'te ne yapıldığını ve
platformların birbirini bozmadan nasıl güncellendiğini anlatır. Bir işin
issue'dan yayına giden yolu, bitti tanımı ve inceleme kuralı
[surec.md](surec.md) içindedir.

## İlke: tek depo, bağımsız platformlar

Bütün platformlar aynı Rust çekirdeğini ve aynı protokolü kullanır. Bu yüzden
kod tek depoda, tek `main` branch'inde durur. Platformlar birbirinden şu
yollarla ayrılır:

- **Klasör:** her platformun kendi klasörü vardır.
- **Sürüm:** her platformun kendi sürüm numarası ve etiketi vardır
  ([versioning.md](versioning.md)).
- **Derleme hattı:** her platformun kendi CI iş akışı vardır ve yalnızca
  ilgili klasörler değişince çalışır.
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
| `docs/` | Tasarım kuralları, sürümler, protokol notları, bu belge | |

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
  platformu bozan çekirdek değişikliği `main`'e giremez.

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

## Arayüz değişiklikleri

[tasarim.md](tasarim.md) bağlayıcıdır. Arayüzü değiştiren her değişiklikten
sonra uygulama açık ve koyu temada şu ortamlarda denenir ve PR'a ekran
görüntüsü eklenir: Omarchy, Hyprland (Omarchy olmadan), KDE Plasma.
