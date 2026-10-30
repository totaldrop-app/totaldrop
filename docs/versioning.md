# Sürümler ve yayınlar

## Sürüm numaraları

TotalDrop [Semantic Versioning 2.0](https://semver.org/) kullanır.

- **Her platformun kendi sürümü vardır.** Linux uygulaması, Android
  uygulaması, CLI ve sunucu ayrı ayrı yayınlanır. Bir platformun
  güncellenmesi diğerlerini beklemez ve bozmaz.
- Sürümün yazıldığı yer:

  | Bileşen | Dosya |
  | --- | --- |
  | Çekirdek kütüphaneler ve CLI | `Cargo.toml`, `[workspace.package]` |
  | Linux uygulaması | `linux/Cargo.toml` |
  | Android uygulaması | `app/pubspec.yaml` |
  | Rendezvous sunucusu | `server/*/Cargo.toml`, `server/rendezvous-worker/package.json` |

- **CLI, Linux paketleriyle yayınlanır.** Her `linux-v…` etiketi aynı
  zamanda o commit'teki CLI sürümünün yayınıdır; `cli-v…` etiketi isteğe
  bağlıdır. Linux yayınında CLI sürümü şimdiye kadar yayınlananların en
  yükseğinden düşük olamaz, CLI kodu değiştiyse de aynı kalamaz
  (`packaging/check-cli-version.sh`). Bu denetim yalnızca
  `0.MINOR.PATCH-beta.N` biçimini tanır; 1.0'da değiştirilmelidir.

- **1.0'a kadar beta.** Beş platform (Linux, Windows, macOS, Android, iOS)
  yayınlanana ve bağımsız güvenlik incelemesi bitene kadar her sürüm bir ön
  sürümdür: `0.MINOR.PATCH-beta.N`. Uygulamalar sürüm numarasının yanında
  "beta" gösterir.
- Beta süresince yeni özellik ya da uyumsuz değişiklik `MINOR`'u, düzeltme
  `PATCH`'i, aynı sürümün yeni bir derlemesi `beta.N`'i artırır.
- **Protokolün** ayrı bir sürümü vardır (`v0`, sonra `v0.1` taslakları),
  çünkü cihazlar bunu birbiriyle pazarlık eder (şartname bölüm 11). Her
  uygulama sürüm notunda hangi protokol sürümlerini konuştuğunu yazar.
  Farklı platformların birlikte çalışmasını bu sağlar, uygulama sürümlerinin
  aynı olması gerekmez.

## Paket sürümleri

Ön sürüm her paket biçiminde son sürümden önce sıralanacak şekilde yazılır:

| Biçim | `0.1.0-beta.2` şöyle yazılır |
| --- | --- |
| Arch (`PKGBUILD`) | `pkgver=0.1.0_beta.2` |
| Debian (`debian/changelog`) | `0.1.0~beta.2-1` |
| RPM (`totaldrop.spec`) | `Version: 0.1.0~beta.2` |
| AppStream | `0.1.0~beta.2` |

## Derleme numaraları

- Android `versionCode` ve iOS/macOS derleme numarası, yayınlanan her
  derlemede bir artan tek bir tam sayıdır. `app/pubspec.yaml`'da `+`'dan
  sonra durur (örneğin `0.1.0-beta.2+2`).
- Bir derleme numarası asla yeniden kullanılmaz; eski bir paket yenisinin
  üstüne kurulamaz.

## Git

- Commit mesajları [Conventional Commits](https://www.conventionalcommits.org/)
  biçimindedir ve kapsam platformu söyler: `feat(linux):`, `fix(android):`,
  `security(core):`, `docs:`, `ci(linux):`, `build(packaging):`. Uyumsuz
  değişiklik `!` (`feat(core)!:`) ya da `BREAKING CHANGE:` notu alır.
- Her yayın `main` üzerinde imzalı, açıklamalı bir etikettir ve platform
  adıyla başlar:

  | Platform | Etiket |
  | --- | --- |
  | Linux | `linux-v0.1.0-beta.2` |
  | Android | `android-v0.1.0-beta.2` |
  | CLI (isteğe bağlı, yukarıya bakın) | `cli-v0.1.0-beta.2` |
  | Rendezvous sunucusu | `server-v0.1.0-beta.2` |

- [`CHANGELOG.md`](../CHANGELOG.md) [Keep a Changelog](https://keepachangelog.com/)
  biçimindedir. Her girdi platformunu söyler; güvenlik düzeltmeleri
  "Security" başlığı altında durur. Yayınlanmamış girdiler her değişiklik
  için ayrı bir dosyada, [`changelog.d/`](../changelog.d/README.md)
  altında bekler; sürümde CHANGELOG.md'ye taşınır.

## Yayın yapmak

Adım adım sürüm listesi [surec.md](surec.md#sürüm-listesi) içindedir. Kısaca:
sürüm ve paket tarifleri artırılır, `changelog.d/` girdileri
`tools/changelog.mjs release` ile sürüm başlığına taşınır, sürüm PR'ı
incelenip birleştirilir, `main`'deki commit'e etiket atılır. CI paketleri etiketten derler, derleme kaynağı kanıtı ekler ve taslak
bir yayın açar. Kişisel bir makinede derlenen hiçbir şey yayınlanmaz.

Android'de imzalı Play paketi `android-v*` etiketinden, yalnızca
`play-release` ortamının gizli değerlerindeki yükleme anahtarıyla derlenir;
paket GitHub Actions çıktısı olarak indirilir ve Play Console'a yüklenir.
