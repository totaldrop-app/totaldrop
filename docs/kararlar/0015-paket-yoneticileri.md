# 0015: Paket yöneticileri önce CI'da derlenen CLI ikililerine dayanır

- **Durum:** Önerilen, 7 Kasım 2026 (#344)
- **Bağlam:** TotalDrop bugün iki yoldan kurulur: sürüm sayfasındaki
  Linux paketleri (Arch, Debian, Fedora; Flathub yolda,
  [0014](0014-steam-deck.md)) ve `cargo install --git`. CLI'ın hazır
  derlenmiş ikilisi yoktur; macOS ve Windows kullanıcısı CLI için Rust araç
  zincirini kurmak zorundadır. Homebrew, winget, Scoop, AUR ve crates.io'da
  yer almıyoruz. 7 Kasım 2026'da bu yerlerdeki adlar boştu: crates.io'da
  `totaldrop`, `totaldrop-cli`, `-core`, `-crypto`, `-proto`, `-l10n`;
  AUR'da `totaldrop`; winget'te `TotalDrop` yayıncı klasörü. Başkası önce
  alırsa, kullanıcı TotalDrop'un yayımlamadığı bir paketi TotalDrop sanabilir
  ([0012](0012-ad-ve-isaret.md)).

  Proje bağışla yürür; ücretli hizmet kullanılmaz ([0005](0005-bagis.md),
  [0009](0009-ios-macos.md)). Paketler yalnızca CI'da, etiketli bir
  commit'ten derlenir ([0007](0007-surec.md)). Tek istisna Flathub'dır:
  Flathub paketi kendi altyapısında derler, ama aynı tarif her PR'da bizim
  CI'ımızda da derlenir (0014).

- **Seçenekler:**
  1. Paket yöneticilerini topluluğa bırakmak. Marka kuralı Linux
     dağıtımlarına ve AUR'a izin verir, ama yayıncının geliştirici olarak
     göründüğü yerlerde (winget, Store) kimliği başkası alır. Paketin hangi
     ikiliyi kurduğunu denetleyemeyiz.
  2. Her kanalın kendi altyapısında kaynaktan derlemesi (Homebrew bottle,
     COPR, OBS). Kullanıcıya giden ikili bizim CI'ımızdan çıkmaz. Yapı
     kaynağı kanıtı (attestation) ve yeniden üretilebilirlik denetimi bu
     ikilileri kapsamaz.
  3. **CI tek kaynaktır: sürüm iş akışı ikilileri, özetleri ve yapı kaynağı
     kanıtını üretir; kanallar yalnızca bu dosyalara işaret eden bir tarif
     tutar. Kendi altyapısında derleyen kanallar istisnadır ve burada adıyla
     yazılır.**
- **Karar:** Seçenek 3.

## Ortak temel: CLI ikilileri

Bütün kanallar aynı arşivlere dayanır. Bunları yeni bir sürüm iş akışı
üretir:

| Hedef | Koşucu | Arşiv |
| --- | --- | --- |
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` | `.tar.gz` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` | `.tar.gz` |
| `aarch64-apple-darwin` ve `x86_64-apple-darwin`, `lipo` ile tek dosya | `macos-15` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-2025` | `.zip` |
| `aarch64-pc-windows-msvc` | `windows-11-arm` | `.zip` |

- Linux ikilileri musl ile statik bağlanır, dağıtımdan bağımsız çalışır.
  Kriptografi `ring` üzerindendir; `ring` musl'ı destekler, ama C
  derleyicisi ister.
- Arşiv adı `totaldrop-cli-<sürüm>-<hedef>`'tir. İçinde `totaldrop`
  ikilisi, lisans dosyaları ve README vardır.
- Sürüm, Linux paketlerindeki gibi `SHA256SUMS`, yapı kaynağı kanıtı ve
  `.sigstore.json` dosyasıyla çıkar.
- Etiket `cli-v<sürüm>`'dür (bugün `0.2.0-beta.5`). CLI'ın sürümü Linux
  uygulamasınınkinden ayrıdır ve paket yöneticileri CLI'ın sürümünü
  görmelidir. `cli-v…` etiketi [versioning.md](../versioning.md)'de bugün
  isteğe bağlıdır; CLI ikilileri için zorunlu olur. CLI yine Linux
  yayınıyla birlikte yayımlanır: CLI'ın sürümü artırıldıysa aynı commit'e
  `cli-v…` etiketi de konur. `surec.md`'deki sürüm listesi ve
  `versioning.md` bu adımla, uygulamayı getiren PR'da güncellenir.
- İş akışı `linux-release.yml` gibi kurulur: etiket `main`'de mi, sürümler
  etiketle uyuşuyor mu (`packaging/check-cli-version.sh`), sonra onaylı bir
  ortamda (environment) taslak sürüm. ARM64 hedefleri #86'yı da karşılar.

## Kanallar

| Kanal | Ne | Ne zaman |
| --- | --- | --- |
| crates.io | `totaldrop-cli` ve bağımlı olduğu dört crate | Engeller kalkınca (aşağıda) |
| `cargo binstall` | crates.io kaydı + CLI ikilileri | crates.io ile |
| Homebrew tap | `totaldrop-app/homebrew-tap`, `totaldrop` formülü (CLI) | CLI ikilileriyle |
| Scoop | `totaldrop-app/scoop-bucket`, `totaldrop` (CLI) | CLI ikilileriyle |
| AUR | `totaldrop` (kaynaktan) ve `totaldrop-bin` | Şimdi; Linux yayınıyla |
| winget | `TotalDrop.CLI`; uygulama gelince `TotalDrop.TotalDrop` | CLI ikililerinden sonra |
| Flathub | `app.totaldrop.TotalDrop` | 0014 |
| Microsoft Store | Windows uygulaması | Windows uygulamasıyla (#33) |
| Homebrew cask | macOS uygulaması | Apple hesabı ve noter onayıyla (0009, adım 5) |
| homebrew-core | `totaldrop` formülü | 1.0'dan sonra |
| APT ve RPM depoları | Linux uygulaması | Ayrı karar |

### crates.io ve cargo binstall

- `cargo install totaldrop-cli` için `totaldrop-cli` ile bağımlı olduğu
  `totaldrop-core`, `-crypto`, `-proto` ve `-l10n` crate'leri yayımlanır.
  Yayın CI'dan, crates.io'nun Trusted Publishing yoluyla (GitHub OIDC)
  yapılır. Uzun ömürlü bir crates.io token'ı tutulmaz.
- **Engel 1:** `totaldrop-l10n/build.rs` çeviri dosyalarını crate'in
  dışından okur (`../../l10n`). `cargo package` yalnızca crate klasörünü
  pakete koyar; bu haliyle yayımlanan crate derlenmez. Çözüm, ayrı bir
  işte seçilir. Seçenekler: çeviri dosyalarını paketleme sırasında crate'e
  kopyalamak ya da üretilen Rust kodunu crate'e koymak.
- **Engel 2:** CLI'ın geliştirme bağımlılıkları (`totaldrop-relay`,
  `totaldrop-rendezvous`) hem yol hem sürümle yazılı. Sürümlü bir
  geliştirme bağımlılığı crates.io'da bulunmalıdır; sunucular ise
  yayımlanmaz. Ya sürüm alanı kaldırılır ve
  `tools/check-component-versions.sh` buna göre değişir, ya da yayın
  bunları `cargo publish` öncesinde çıkarır.
- Adlar boş dururken yer tutucu yayımlanmaz: crates.io'nun kuralları ad
  ayırmak için boş crate yayımlamayı istemez. Engeller kalkınca gerçek
  crate'ler yayımlanır. Bu süreçte adların başkasınca alınması riski kabul
  edilir.
- `totaldrop-cli`'a `[package.metadata.binstall]` eklenir.
  `cargo binstall totaldrop-cli` derlemeden, bizim CI ikilimizi indirir.

### Homebrew

- **Tap:** `totaldrop-app/homebrew-tap` deposunda bir formül.
  `brew install totaldrop-app/tap/totaldrop` macOS (Apple Silicon, Intel)
  ve Linux'ta bizim CI arşivimizi indirir, özeti denetler ve `totaldrop`
  ikilisini kurar. Bottle yoktur, Homebrew bir şey derlemez.
- macOS'ta CLI ikilisi yalnızca ad-hoc imzalıdır (0009). Homebrew'un
  formül indirmelerine karantina işareti koymadığını ve ikilinin
  Gatekeeper'a takılmadan çalıştığını bekliyoruz. **Doğrulanacak:** ilk
  sürümde macOS 15 ve 26'da denenir. 0009'daki "Homebrew yol sayılmaz"
  hükmü cask'ler, yani uygulama paketleri içindir; formüllere değil.
- **Cask:** Homebrew 5.0'dan beri Gatekeeper denetiminden geçmeyen
  cask'leri kabul etmez ve Eylül 2026'dan beri bunları kapatır. macOS
  uygulamasının cask'i bu yüzden Developer ID imzası ve noter onayından
  sonra gelir (0009, adım 5).
- **homebrew-core:** Homebrew kararlı sürüm ister; beta sürümler
  `homebrew/core`'a giremez. 1.0'dan sonra başvurulur. homebrew-core kendi
  altyapısında kaynaktan derler; bu, aşağıdaki istisnalardan biridir.
  Başvuru zamanında Homebrew'un bilinirlik ölçütleri yeniden okunur.

### Scoop

- `totaldrop-app/scoop-bucket` deposunda bir JSON bildirimi. Bizim
  `.zip` arşivimize ve özetine işaret eder. Microsoft incelemesi yoktur;
  bildirimi sürüm iş akışı günceller.

### winget

- Bildirimler `microsoft/winget-pkgs` deposuna PR ile girer; Microsoft'un
  doğrulama hattı ve inceleyicileri birleştirir.
- **CLI:** `TotalDrop.CLI` kimliği. Kurulum türü `zip`, içindeki kurulum
  türü `portable`. winget ikiliyi kendi klasörüne açar ve `totaldrop`
  komutunu PATH'e bağlar.
- **Uygulama:** `TotalDrop.TotalDrop` kimliği, Windows uygulaması (#33)
  geldiğinde. Store'da yayımlanırsa winget onu `msstore` kaynağından da
  kurar.
- Kimlikleri başkası almadan proje alır. İlk bildirim elle, `komac` ya da
  `wingetcreate` ile gönderilir. Sonraki sürümleri iş akışı gönderir.
- İmza: winget imzasız ikili kabul eder, ama Windows SmartScreen ve
  Defender uyarı verebilir. Ücretsiz yol SignPath Foundation'ın açık kaynak
  imza programıdır. Koşulları: OSI lisansı, açık depo, kaynağı doğrulanan
  CI derlemesi, ekipte iki aşamalı doğrulama. Sertifikada yayıncı
  "SignPath Foundation" görünür, TotalDrop değil. Bunun marka kuralıyla
  (0012) uyumu ayrı değerlendirilir. Ücretli imza hizmetleri 0005 gereği
  seçenek değildir.
- **Doğrulanacak:** winget'in `0.2.0-beta.5` gibi bir sürüm dizesini
  sıralarken betayı kararlı sürümden önce koyup koymadığı.

### Microsoft Store

- Microsoft, Eylül 2025'ten beri bireysel geliştirici kaydını ücretsiz
  yaptı. Store MSIX paketini kendisi imzalar. Bu, Windows uygulamasının
  imza sorununu parasız çözebilir.
- **Doğrulanacak:** bireysel hesapta Store'da yayıncı olarak kişinin adı
  mı görünür. Marka kuralı Store'da yayıncının TotalDrop görünmesini ister
  (0012). Görünmüyorsa şirket hesabı gerekir; şirket hesabının tek
  seferlik bir ücreti vardır.
- Karar #33'ün karar kaydında verilir: Store, MSIX ya da imzalı kurulum
  paketi.

### AUR

- İki paket: `totaldrop` bizim `packaging/arch/PKGBUILD`'imizden
  kaynaktan derlenir; `totaldrop-bin` sürümdeki `.pkg.tar.zst`'yi ya da
  ikiliyi kurar ve özetini denetler. Omarchy ve Arch kullanıcıları AUR
  yardımcılarıyla (`yay`, `paru`) kurar.
- `totaldrop` paketini kullanıcı kendi makinesinde derler. Bu, bizim
  yayımladığımız bir ikili değildir; kaynak dağıtımıdır. Kurala aykırı
  değildir.
- AUR deposu Linux sürüm iş akışından güncellenir.

### APT ve RPM depoları, diğerleri

- `apt install` ve `dnf install` için imzalı bir depo gerekir. Seçenekler:
  Cloudflare R2'de kendi depomuz ([0003](0003-cloudflare-altyapi.md)),
  Fedora COPR, openSUSE OBS. Depo imza anahtarı ve güncelleme yolu ayrı bir
  karardır.
- Topluluğa bırakılanlar: nixpkgs, Chocolatey, MacPorts ve Linux
  dağıtımlarının kendi depoları. Marka kuralı bunlara izin verir (0012).
- Snap Store'da TotalDrop'u yalnızca TotalDrop yayımlar (0012). Snap şimdilik
  planlanmıyor.
- F-Droid Android'in işidir; ayrı bir issue'da ele alınır.

## "Yalnızca CI'da derlenir" kuralının istisnaları

- Bizim CI çıktımızı kuranlar istisna değildir: Homebrew tap, Scoop,
  winget, AUR `totaldrop-bin`, `cargo binstall`.
- Kaynak dağıtanlar istisna değildir: crates.io ve AUR `totaldrop`.
  Kullanıcı kendi makinesinde derler.
- Kendi altyapısında derleyip ikili dağıtanlar istisnadır: Flathub (0014),
  ileride homebrew-core, seçilirse COPR ve OBS. Her biri için koşul
  Flathub'daki ile aynıdır: yalnızca etiketli kaynaktan derlenir ve aynı
  tarif her PR'da bizim CI'ımızda da derlenir.

## Gizli bilgiler

- Yayın işleri yalnızca sürüm etiketlerinde ve onay isteyen bir ortamda
  (environment) çalışır. CLI için `cli-release` ortamı açılır; AUR için
  `linux-release` kullanılır. Çatallardan gelen PR'lar gizli değişkenleri
  görmez.
- crates.io: Trusted Publishing; token yok.
- Homebrew tap ve Scoop bucket: her depoda yalnızca o depoya yazabilen bir
  deploy anahtarı.
- AUR: yalnızca AUR'a kayıtlı, başka bir yerde kullanılmayan bir SSH
  anahtarı.
- winget: PR açmak için `microsoft/winget-pkgs`'i çatallayabilen bir
  GitHub token'ı gerekir. Bu token için ayrı bir bot hesabı açılır. Hesabın
  TotalDrop depolarında hiçbir yetkisi yoktur; token yalnızca
  `public_repo` kapsamındadır.
- Hiçbiri depoya ya da geliştirme makinesine yazılmaz.

## Sıra

1. CLI ikilileri, `cli-v…` etiketi ve sürüm iş akışı. #86'nın CLI
   maddesini de kapatır.
2. Homebrew tap ve Scoop bucket.
3. AUR: `totaldrop` ve `totaldrop-bin`.
4. winget: `TotalDrop.CLI`.
5. crates.io ve `cargo binstall`, iki engel kalkınca.
6. Windows uygulaması: Store, winget `TotalDrop.TotalDrop` (#33).
   macOS uygulaması: cask (0009). 1.0'dan sonra homebrew-core.

Her adım kendi issue'su ve PR'ıyla gelir. README'deki ve sitedeki indirme
tablosu aynı PR'da güncellenir.

## Riskler

- winget PR'ları Microsoft incelemesine bağlıdır; bir sürümün winget'e
  gelmesi günler sürebilir.
- İmzasız Windows ikilileri SmartScreen uyarısı alır. Bu, gizlilik odaklı
  bir uygulamaya güveni azaltır.
- Her kanal sürüm iş akışına bir adım ve bir gizli bilgi ekler. Bir kanalın
  hatası yayını durdurmamalıdır: kanal adımları taslak sürüm çıktıktan
  sonra ve birbirinden bağımsız çalışır.
- Adlar şimdi boş, ama alınmış değil. crates.io adları engeller kalkana
  kadar açıkta kalır.

## Sonuçlar

- `surec.md`'nin sürüm listesine CLI etiketi ve kanal güncellemeleri
  eklenir.
- `docs/gelistirme.md`'ye kanalların tarifleri ve nasıl denendikleri
  eklenir.
- **Yeniden bakılacak zaman:** Homebrew formüller için de imza ya da noter
  onayı isterse; winget imzasız ikilileri reddetmeye başlarsa; SignPath
  programı kapanır ya da koşulları değişirse; Apple hesabı alınınca.

## Kaynaklar

- Homebrew, kabul edilen formüller: [docs.brew.sh](https://docs.brew.sh/Acceptable-Formulae)
- Homebrew, kabul edilen cask'ler: [docs.brew.sh](https://docs.brew.sh/Acceptable-Casks)
- Homebrew 5.0 ve Gatekeeper: [Workbrew](https://workbrew.com/blog/what-homebrew-5-0-0-means-for-your-mac-fleet)
- Microsoft Store'da ücretsiz bireysel kayıt: [Windows Developer Blog](https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/)
- SignPath Foundation koşulları: [signpath.org](https://signpath.org/terms)
- winget bildirim türleri: [winget-types](https://docs.rs/winget-types/latest/winget_types/installer/struct.InstallerManifest.html)
