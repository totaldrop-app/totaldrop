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
  [0009](0009-ios-macos.md)). Paketler yalnızca CI'da, `main` üzerindeki
  etiketli bir commit'ten derlenir ([surec.md](../surec.md), sürüm listesi;
  [versioning.md](../versioning.md)). Tek istisna Flathub'dır: Flathub
  paketi kendi altyapısında derler, ama aynı tarif her PR'da bizim CI'ımızda
  da derlenir (0014).

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
- **Neden:** Kullanıcının kurduğu dosya, hangi kanaldan gelirse gelsin,
  sürüm sayfasındaki dosyanın aynısıdır. `SHA256SUMS` ve
  `gh attestation verify` her kanalda aynı sonucu verir. Yeniden
  üretilebilirlik denetimi de bu dosyaları kapsar. Kanal tarifleri birkaç
  satırlık adres ve özet listesidir, derleme mantığı taşımaz. Bir kanalın
  ele geçirilmesi ikiliyi değiştiremez, yalnızca başka bir adresi
  gösterebilir; bu da özet ve kanıtla yakalanır. Seçenek 1 kimlikleri
  kaybettirir. Seçenek 2 ise her kanalda ayrı bir derleme zinciri ve ayrı
  bir güven demektir.

## Ortak temel: CLI ikilileri

Bütün kanallar aynı arşivlere dayanır. Bunları yeni bir sürüm iş akışı
üretir:

| Hedef | Koşucu | Arşiv |
| --- | --- | --- |
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` | `.tar.gz` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` | `.tar.gz` |
| `universal-apple-darwin` (`aarch64` ve `x86_64`, `lipo` ile tek dosya) | `macos-15` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-2025` | `.zip` |
| `aarch64-pc-windows-msvc` | `windows-11-arm` | `.zip` |

- Linux ikilileri musl ile statik bağlanır, dağıtımdan bağımsız çalışır.
  Protokolün kriptografisi RustCrypto (`chacha20poly1305`,
  `curve25519-dalek`, `ed25519-dalek`, `ml-kem`), `snow` ve `blake3`
  üzerindendir. `ring` yalnızca TLS ve QUIC (`rustls`, `quinn`) içindir.
  `ring` musl'ı destekler, ama C derleyicisi ister.
- Windows ikilileri C çalışma zamanını statik bağlar
  (`-C target-feature=+crt-static`). Yoksa ikili `vcruntime140.dll` ister
  ve temiz bir makinede winget ya da Scoop'un açtığı zip çalışmaz.
- macOS ikilisi `lipo` ve `strip`'ten sonra yeniden ad-hoc imzalanır. CI
  imzayı `codesign -v` ile denetler: Apple Silicon'da imzasız ya da imzası
  bozuk kod çalışmaz.
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
- İş akışı `linux-release.yml` gibi kurulur. Önce etiketin `main`'de
  olduğu ve sürümlerin etiketle uyuştuğu denetlenir
  (`packaging/check-cli-version.sh`). Sonra onaylı bir ortamda (environment)
  taslak sürüm açılır.
- #86'yı yalnızca kısmen karşılar. #86, `aarch64-unknown-linux-gnu` ve
  `aarch64-pc-windows-msvc` için her PR'da CI'da derleme ve test ister. Bu
  karar ise ARM64 ikililerini musl ile ve yalnızca sürüm etiketinde derler.
  PR'lardaki ARM64 derleme ve testi #86'nın işi olarak kalır.

## Kanallar ne zaman güncellenir

Sürüm iş akışı sürümü taslak olarak açar. Taslağın dosyaları herkese açık
indirilemez ve taslağı henüz kimse gözden geçirmemiştir. Bu yüzden kanal
güncellemeleri sürüm iş akışının parçası değildir. Ayrı bir iş akışı
(`channels.yml`) `release: published` olayıyla, yani bakımcı taslağı
yayımladığında çalışır. Her kanal ayrı bir iştir; birinin hatası
ötekileri durdurmaz. İşler korumalı ortamlarda çalışır (aşağıda, "Gizli
bilgiler"). Bir kanal başarısız olursa iş elle yeniden çalıştırılır.

## Kanallar

| Sıra | Kanal | Ne |
| --- | --- | --- |
| 1 | — | CLI ikilileri ve `cli-v…` etiketi (yukarıda) |
| 2 | Homebrew tap | `totaldrop-app/homebrew-tap`, `totaldrop-cli` formülü |
| 2 | Scoop | `totaldrop-app/scoop-bucket`, `totaldrop-cli` |
| 3 | AUR | `totaldrop` (kaynaktan) ve `totaldrop-bin`; Linux yayınıyla |
| 4 | winget | `TotalDrop.CLI` |
| 5 | crates.io ve `cargo binstall` | `totaldrop-cli` ve bağımlı olduğu dört crate; engeller kalkınca |
| 6 | Microsoft Store | Windows uygulaması: [0016](0016-windows-microsoft-store.md) |
| 6 | Homebrew cask | macOS uygulaması; Apple hesabı ve noter onayıyla (0009, adım 5) |
| 6 | homebrew-core | 1.0'dan sonra |
| — | Flathub | 0014 |
| — | APT ve RPM depoları | Ayrı karar |

Adlar: CLI her kanalda `totaldrop-cli` adını taşır, komutun adı
`totaldrop`'tur. Böylece ileride macOS uygulamasının cask'i `totaldrop`
adını alabilir; `brew install totaldrop` belirsiz kalmaz. AUR'da
`totaldrop` Arch paketiyle aynıdır, yani uygulama ve CLI birlikte.

### crates.io ve cargo binstall

- `cargo install totaldrop-cli` için `totaldrop-cli` ile bağımlı olduğu
  `totaldrop-core`, `-crypto`, `-proto` ve `-l10n` crate'leri yayımlanır.
- **İlk yayın:** crates.io'nun Trusted Publishing yolu (GitHub OIDC) yeni
  crate oluşturamaz. Bu yüzden beş crate'in ilk yayını da CI'dan,
  `cli-release` ortamında yapılır. Bunun için kısa ömürlü bir crates.io
  token'ı kullanılır: kapsamı yalnızca `publish-new`, adları `totaldrop*`
  ile sınırlı. Hemen ardından her crate için Trusted Publishing kurulur ve
  token iptal edilir. Sonraki yayınlarda uzun ömürlü bir token tutulmaz.
- **Engel 1:** `totaldrop-l10n/build.rs` çeviri dosyalarını crate'in
  dışından okur (`../../l10n`). `cargo package` yalnızca crate klasörünü
  pakete koyar; bu haliyle yayımlanan crate derlenmez. Çözüm ayrı bir işte
  seçilir. Seçenekler: çeviri dosyalarını paketleme sırasında crate'e
  kopyalamak ya da üretilen Rust kodunu crate'e koymak.
- **Engel 2:** CLI'ın geliştirme bağımlılıkları (`totaldrop-relay`,
  `totaldrop-rendezvous`) hem yol hem sürümle yazılı. Sürümlü bir
  geliştirme bağımlılığı crates.io'da bulunmalıdır; sunucular ise
  yayımlanmaz. Ya sürüm alanı kaldırılır ve
  `tools/check-component-versions.sh` buna göre değişir, ya da yayın
  bunları `cargo publish` öncesinde çıkarır.
- **Engel 3:** Crate klasörlerinde lisans dosyası yok; metinler kökteki
  `LICENSES/` altında. Yayımlanan her crate kendi lisans metinlerini
  taşımalıdır: `-core`, `-crypto`, `-proto` ve `-l10n` için MIT ve Apache
  2.0, `-cli` için GPL 3.0.
- Adlar boş dururken yer tutucu yayımlanmaz: crates.io'nun kuralları ad
  ayırmak için boş crate yayımlamayı istemez. Engeller kalkınca gerçek
  crate'ler yayımlanır. Bu süreçte adların başkasınca alınması riski kabul
  edilir.
- `totaldrop-cli`'a `[package.metadata.binstall]` eklenir. Etiket öneki
  `cli-v` olduğu için `pkg-url` açıkça yazılır; macOS için hedef adı
  `universal-apple-darwin`'dir. `cargo binstall totaldrop-cli`
  derlemeden bizim CI ikilimizi indirir.

### Homebrew

- **Tap:** `totaldrop-app/homebrew-tap` deposunda `totaldrop-cli`
  formülü. `brew install totaldrop-app/tap/totaldrop-cli` macOS (Apple
  Silicon, Intel) ve Linux'ta bizim CI arşivimizi indirir, özeti denetler
  ve `totaldrop` ikilisini kurar. Bottle yoktur, Homebrew bir şey derlemez.
- macOS'ta CLI ikilisi yalnızca ad-hoc imzalıdır (0009). Homebrew'un
  formül indirmelerine karantina işareti koymadığını ve ikilinin
  Gatekeeper'a takılmadan çalıştığını bekliyoruz. **Doğrulanacak:** ilk
  sürümde macOS 15 ve 26'da denenir.
- 0009 "hesapsız Homebrew kurulumu bir yol sayılmaz" der. Bu karar o
  cümleyi uygulama paketlerine, yani cask'lere daraltır; CLI formülü
  hesapsız da bir yoldur (Sonuçlar).
- **Cask:** Homebrew 5.0'dan beri resmî `homebrew/cask` deposu Gatekeeper
  denetiminden geçmeyen cask'leri kabul etmez; Eylül 2026'dan beri de
  bunları kapatır. Kendi tap'imizdeki bir cask bu kuraldan muaf olsa bile
  macOS onu yine karantinaya alır; kullanıcı aynı zahmetli onay adımlarıyla
  karşılaşır. macOS uygulamasının cask'i bu yüzden Developer ID imzası ve
  noter onayından sonra gelir (0009, adım 5).
- **homebrew-core:** Homebrew kararlı sürüm ister; beta sürümler
  `homebrew/core`'a giremez. 1.0'dan sonra başvurulur. Başvuru zamanında
  Homebrew'un bilinirlik ölçütleri yeniden okunur.
  - homebrew-core kendi altyapısında kaynaktan derler. Bu bir istisnadır
    (aşağıda).
  - Formül Homebrew'un deposundadır; onu Homebrew'un bakımcıları ve
    botları değiştirir. "Aynı tarif her PR'da bizim CI'ımızda da derlenir"
    koşulu bu yüzden Flathub'daki gibi tam karşılanamaz. Bunun yerine her
    PR'da formülün bir aynası `brew install --build-from-source` ile
    derlenir. Kullanıcıya giden bottle'ı ise Homebrew derler ve kendi
    anahtarıyla imzalar; bizim kanıtımızı taşımaz. Kullanıcı Homebrew'a
    güvenir; bu, tehdit modeline Flathub için T15'teki gibi yazılır.

### Scoop

- `totaldrop-app/scoop-bucket` deposunda `totaldrop-cli` için bir JSON
  bildirimi. Bizim `.zip` arşivimize ve özetine işaret eder.
- Dış inceleme yoktur; bucket bizimdir. Bildirimi `channels.yml` günceller.

### winget

- Bildirimler `microsoft/winget-pkgs` deposuna PR ile girer; Microsoft'un
  doğrulama hattı ve inceleyicileri birleştirir.
- **CLI:** `TotalDrop.CLI` kimliği. Kurulum türü `zip`, içindeki kurulum
  türü `portable`. winget ikiliyi kendi klasörüne açar ve `totaldrop`
  komutunu PATH'e bağlar.
- **Uygulama:** Windows uygulaması yalnızca Microsoft Store'dan dağıtılır
  ([0016](0016-windows-microsoft-store.md)). winget onu `msstore`
  kaynağından, Store ürün kimliğiyle kurar. Topluluk deposundaki
  `TotalDrop.TotalDrop` kimliği bir kurulum adresi ister; Store dışında
  paket olmadığı için yayımlanmaz (0016).
- winget'te bildirimin `Publisher` alanı ve kimliğin öneki TotalDrop'u
  yayıncı olarak gösterir. Bu yüzden winget, yayıncının geliştirici
  olarak göründüğü yerler arasına girer. Orada TotalDrop'u yalnızca
  TotalDrop yayımlar; 0012 ve `marka.md`'deki liste buna göre güncellenir
  (Sonuçlar). `TotalDrop.CLI` kimliğini başkası almadan proje alır.
- İlk bildirim elle, `komac` ya da `wingetcreate` ile gönderilir. Sonraki
  sürümleri `channels.yml` gönderir.
- İmza: winget imzasız ikili kabul eder, ama Windows SmartScreen ve
  Defender uyarı verebilir. CLI ikilisi Store'dan geçmediği için Store'un
  imzası (0016) onu kapsamaz. Ücretsiz yol SignPath Foundation'ın açık
  kaynak imza programıdır. Koşulları: OSI lisansı, açık depo, kaynağı
  doğrulanan CI derlemesi, ekipte iki aşamalı doğrulama. Sertifikada
  yayıncı "SignPath Foundation" görünür, TotalDrop değil. Bunun marka
  kuralıyla (0012) uyumu ayrı değerlendirilir. Ücretli imza hizmetleri
  0005 gereği seçenek değildir. CLI ilk sürümlerde imzasız çıkar.
- **Doğrulanacak:** winget'in `1.0.0-beta.1` gibi bir sürüm dizesini
  `1.0.0`'dan önce sıralayıp sıralamadığı. Bugünkü `0.x-beta` sürümlerde
  sıralama sorun çıkarmaz; soru 1.0'ın betalarında önem kazanır.

### Microsoft Store

- Windows uygulamasının dağıtımı, Store hesabı, yayıncı adı ve imzası
  [0016](0016-windows-microsoft-store.md)'nın konusudur. Bu karar orada
  verilenleri tekrarlamaz.
- Bu kararı ilgilendiren tek sonuç: Windows uygulaması için ayrı bir
  winget bildirimi ya da Scoop paketi olmaz. Scoop ve winget'te yalnızca
  CLI vardır.

### AUR

- **`totaldrop`:** bizim `packaging/arch/PKGBUILD`'imizden kaynaktan
  derlenir; uygulama ve CLI birlikte gelir. Depodaki PKGBUILD'de
  `sha256sums=('SKIP')` vardır. AUR `SKIP`'i yalnızca VCS kaynaklarında
  kabul eder. Bu yüzden `channels.yml`, AUR'a gönderirken etiket
  arşivinin gerçek SHA-256 özetini yazar.
- **`totaldrop-bin`:** sürümdeki, bizim CI'ımızın derlediği
  `.pkg.tar.zst`'yi açıp kurar ve özetini denetler. Derlenmiş tek bir CLI
  ikilisi değil, uygulamayla birlikte bütün paket kurulur. Yalnızca
  `x86_64` içindir: Arch paketini yalnızca bu mimari için derliyoruz.
  `provides=(totaldrop)` ve `conflicts=(totaldrop)` taşır.
- Omarchy ve Arch kullanıcıları AUR yardımcılarıyla (`yay`, `paru`)
  kurar.
- `totaldrop` paketini kullanıcı kendi makinesinde derler. Bu, bizim
  yayımladığımız bir ikili değildir; kaynak dağıtımıdır. Kurala aykırı
  değildir.

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
  ileride homebrew-core, seçilirse COPR ve OBS. Her biri yalnızca etiketli
  kaynaktan derler. Tarif bizim depomuzdaysa (Flathub, COPR, OBS) aynı
  tarif her PR'da bizim CI'ımızda da derlenir. Tarif kanalın deposundaysa
  (homebrew-core) her PR'da bir aynası derlenir ve kullanıcının kanala
  güvendiği tehdit modelinde yazılır.
- **Kural değişikliği, Ozan'ın onayını ister.** `CLAUDE.md` ve
  `surec.md` bugün "tek istisna Flathub" der. Bu karar istisnaları bir
  listeye çevirir. İki dosya, böyle bir kanalı ilk getiren PR'da
  güncellenir (homebrew-core ya da COPR/OBS). Sıradaki ilk beş adımın
  hiçbiri istisna değildir; o zamana kadar iki dosya değişmez.

## Gizli bilgiler

- Kanal işleri yalnızca yayımlanmış sürümlerde ve onay isteyen bir ortamda
  (environment) çalışır. CLI kanalları için `cli-release` ortamı açılır;
  AUR için `linux-release` kullanılır. Çatallardan gelen PR'lar gizli
  değişkenleri görmez.
- **crates.io:** ilk yayında kısa ömürlü, `publish-new` kapsamlı token
  (yukarıda), sonra Trusted Publishing.
- **Homebrew tap ve Scoop bucket:** uzun ömürlü deploy anahtarları yerine
  bir GitHub App. App yalnızca bu iki depoya kuruludur ve yalnızca
  içeriğe yazma yetkisi vardır. İş, `actions/create-github-app-token` ile
  bir saatlik bir token alır. App'in özel anahtarı ortamın gizli
  değişkenindedir.
- **AUR:** yalnızca AUR'a kayıtlı, başka bir yerde kullanılmayan bir SSH
  anahtarı. AUR hesabının sahibi proje sahibidir; en az bir ortak bakımcı
  eklenir, böylece paket tek bir kişiye bağlı kalmaz.
- **winget:** PR açmak için `microsoft/winget-pkgs`'i çatallayabilen bir
  GitHub token'ı gerekir. Bu token için ayrı bir bot hesabı açılır.
  - Hesabın TotalDrop depolarında hiçbir yetkisi yoktur ve iki aşamalı
    doğrulama açıktır.
  - Token yalnızca `public_repo` kapsamındadır ve süresi sınırlıdır; süre
    dolmadan yenilenir.
  - winget-pkgs'in katkı sözleşmesini (CLA) bot hesabı bir kez imzalar.
- Hiçbiri depoya ya da geliştirme makinesine yazılmaz.

## Sıra

1. CLI ikilileri, `cli-v…` etiketi ve sürüm iş akışı.
2. `channels.yml`, Homebrew tap ve Scoop bucket.
3. AUR: `totaldrop` ve `totaldrop-bin`.
4. winget: `TotalDrop.CLI`.
5. crates.io ve `cargo binstall`, üç engel kalkınca.
6. Windows uygulaması: Store (0016). macOS uygulaması: cask (0009).
   1.0'dan sonra homebrew-core.

Her adım kendi issue'su ve PR'ıyla gelir. README'deki ve sitedeki indirme
tablosu aynı PR'da güncellenir.

## Riskler

- winget PR'ları Microsoft incelemesine bağlıdır; bir sürümün winget'e
  gelmesi günler sürebilir.
- İmzasız Windows ikilileri SmartScreen uyarısı alır. Bu, gizlilik odaklı
  bir uygulamaya güveni azaltır.
- Her kanal bir gizli bilgi ve bir saldırı yüzeyi ekler. Ele geçirilmiş
  bir tap, bucket ya da AUR paketi kullanıcıyı başka bir ikiliye
  yönlendirebilir. Bunu `SHA256SUMS` ve `gh attestation verify` yakalar,
  ama yalnızca kullanıcı denetlerse. Tehdit modeli bunu yazar (Sonuçlar).
- Adlar şimdi boş, ama alınmış değil. crates.io adları engeller kalkana
  kadar açıkta kalır.

## Sonuçlar

- `surec.md`'nin sürüm listesine ve `versioning.md`'ye CLI etiketi ve
  kanal adımları eklenir. Bu, 1. adımın PR'ında yapılır.
- `docs/gelistirme.md`'ye kanalların tarifleri ve nasıl denendikleri
  eklenir.
- **Tehdit modeli:** 2. adımın PR'ında T12 güncellenir.
  - Resmî kanallar listesine bu karardaki kanallar eklenir.
  - Kanalların anahtarları ve hesapları varlık olarak yazılır: GitHub App
    anahtarı, AUR SSH anahtarı, winget bot hesabı, crates.io sahipleri.
  - Her kanalın kurduğu dosyanın `gh attestation verify` ile nasıl
    denetleneceği yazılır.
  - homebrew-core geldiğinde Homebrew'a güven, T15'teki Flathub gibi
    yazılır.
- **0009:** "Hesapsız Homebrew kurulumu bir yol sayılmaz" cümlesi
  cask'lerle daraltılır. Kabulle birlikte, birleştirmeden önce 0009'a bu
  karara bağlanan bir not eklenir.
- **0012 ve `marka.md`:** yayıncının geliştirici olarak göründüğü yerler
  listesine winget eklenir. Bu, 4. adımın PR'ında yapılır.
- **Kural değişikliği:** `CLAUDE.md` ve `surec.md`'deki "tek istisna
  Flathub" ifadesi istisnalar listesine çevrilir. Bu, Ozan'ın onayını
  ister ve böyle bir kanalı ilk getiren PR'da yapılır (yukarıda).
- **Yeniden bakılacak zaman:** Homebrew formüller için de imza ya da noter
  onayı isterse; winget imzasız ikilileri reddetmeye başlarsa; SignPath
  programı kapanır ya da koşulları değişirse; Apple hesabı alınınca.

## Kaynaklar

- Homebrew, kabul edilen formüller: [docs.brew.sh](https://docs.brew.sh/Acceptable-Formulae)
- Homebrew, kabul edilen cask'ler: [docs.brew.sh](https://docs.brew.sh/Acceptable-Casks)
- Homebrew 5.0 ve Gatekeeper: [Workbrew](https://workbrew.com/blog/what-homebrew-5-0-0-means-for-your-mac-fleet)
- crates.io Trusted Publishing: [crates.io](https://crates.io/docs/trusted-publishing)
- Microsoft Store'da ücretsiz bireysel kayıt: [Windows Developer Blog](https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/)
- SignPath Foundation koşulları: [signpath.org](https://signpath.org/terms)
- winget bildirim türleri: [winget-types](https://docs.rs/winget-types/latest/winget_types/installer/struct.InstallerManifest.html)
