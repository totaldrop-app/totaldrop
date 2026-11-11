# Çalışma süreci

Bu belge bir değişikliğin fikirden yayına nasıl gittiğini anlatır. Depo
düzeni ve branch'ler için [gelistirme.md](gelistirme.md), sürüm numaraları
için [versioning.md](versioning.md) bağlayıcıdır.

## Akış

1. **İş kaydı:** Her iş bir GitHub issue'su ile başlar. Issue neyin neden
   yapılacağını ve ne zaman bitmiş sayılacağını söyler. Yol haritasındaki
   her madde bir issue'dur; aşamalar milestone'dur (Şimdi, Sırada, Sonra,
   1.0).
2. **Branch:** İş, adı platformu söyleyen kısa ömürlü bir branch'te yapılır
   (`feat/linux-…`, `fix/core-…`).
3. **Pull request:** PR açıklaması issue'ya bağlanır (`Closes #12`) ve PR
   şablonundaki "bitti" listesi doldurulur.
4. **CI:** Bütün kontroller yeşil olmadan birleştirme yapılmaz. Kırmızı bir
   kontrol "sonra bakarız" diye geçilmez; ya düzeltilir ya da nedeni PR'da
   yazılır ve ayrı bir issue açılır.
5. **Bağımsız inceleme:** Her PR'ı, kodu yazmamış biri okur: bir bakımcı
   ya da değişikliği yazan oturumdan ayrı çalışan bir inceleme ajanı. Ajan
   yalnızca diff'i ve ilgili kodu görür, yazarın gerekçesini görmez.
   Bakımcının kendi PR'ını öteki bakımcı ya da inceleme ajanı okur. Bulunan
   her doğruluk ya da güvenlik sorunu birleştirmeden önce düzeltilir;
   düzeltilmeyenler için gerekçe PR'a yazılır.
6. **Birleştirme:** `main`'e yalnızca PR ile, rebase ile birleştirilir;
   geçmiş doğrusal kalır. Doğrudan push ve force push kapalıdır. `CI`
   kontrolü yeşil olmadan kural (`main`, kimse atlayamaz) birleştirmeye
   izin vermez. Her dosyanın kod sahibi `@totaldrop-app/maintainers`
   takımıdır (`.github/CODEOWNERS`); takım dışından gelen bir PR, bir
   bakımcı onaylamadan birleştirilmez (kural `main review`). Bakımcılar
   başkasının PR'ını onay vererek birleştirir. Kendi PR'larını onaysız
   birleştirebilirler; bağımsız inceleme onlar için de şarttır, ama bunu
   GitHub zorlamaz: inceleme sonucu PR'a yazılmadan birleştirmemek
   bakımcıların aralarındaki anlaşmadır.
   Roller: [GOVERNANCE.md](../GOVERNANCE.md).
7. **Yayın:** [Sürüm listesi](#sürüm-listesi) adım adım izlenir.

## Bitti tanımı

Bir değişiklik ancak şunların hepsi doğruysa bitmiştir:

- Davranışı sınayan testler var ve geçiyor. Hata düzeltmesinde, önce hatayı
  gösteren bir test yazılır.
- `cargo fmt`, `cargo clippy -- -D warnings` ve platformun kendi denetimi
  (`flutter analyze`, `dart format`, `tsc`) temiz.
- Dışarıdan gelen veriyi okuyan her yeni ayrıştırıcının bir fuzz hedefi var.
- Protokolü etkileyen her karar `docs/protocol-v0.1-notes.md`'ye, mimariyi
  etkileyen her karar `docs/kararlar/` altına yazıldı.
- Türkçe belgeler güncel; kullanıcıya görünen her metin İngilizce ve
  Türkçe var.
- Arayüz değiştiyse: açık ve koyu temada; Linux'ta KDE Plasma, Hyprland ve
  Omarchy'de; Android'de ekran görüntüsü PR'da.
- Sunucuların gördüğü bilgi değiştiyse gizlilik politikası, özellik
  yayınlanmadan önce güncellendi.
- `changelog.d/` altında kendi girdi dosyası var
  ([changelog.d/README.md](../changelog.d/README.md)); kullanıcıya bir şey
  değiştirmeyen bir iş için `.none.md` dosyası nedenini söyler.
  `CHANGELOG.md`'ye yalnızca sürüm sırasında yazılır. CI ikisini de denetler.
- [İncelemeden çıkan kurallara](#incelemeden-çıkan-kurallar) uyuldu.

## İncelemeden çıkan kurallar

Bağımsız incelemelerde aynı türden hatalar birden çok kez bulundu (#398).
Bu kurallar bitti tanımının parçasıdır. Her türün altında onu yakalayan
otomatik denetim yazılıdır; yazılı olmayan her şeye yazar ve inceleyen
elle bakar.

### Bitti tanımı atlanıyor (#304, #318, #368)

- Elle deneme gereken bir PR, deneme yapılana kadar taslak kalır.
- Zorunlu olmayanlar dahil her kırmızı kontrol için [Akış](#akış)'ın 4.
  adımı geçerlidir: ya düzeltilir ya da nedeni PR'a yazılır ve ayrı bir
  issue açılır. Koddan bağımsız bir altyapı hatası (ör. indirme zaman
  aşımı) yeniden çalıştırılarak gösterilir.
- Bakımcı atlama izniyle birleştirilmez.
- Denetim: elle.

### Zayıf test (#316, #395)

- Hata bekleyen bir test hatanın türünü (ya da kodunu) denetler; "bir hata
  oldu" yetmez.
- Ekran testleri her durumu gerçekten çizer: dolu liste, uzun ad, en
  büyük yazı boyutu.
- Zamanlama testleri sabit bir `sleep` yerine bir son süreyle koşulu
  bekler.
- Testin yorumu, testin gerçekte yakaladığını söyler.
- Denetim: elle.

### Örtük durum makinesi (#342, #343, #345)

- Durum bir metin ya da bool çifti değil, adlandırılmış bir türdür (enum).
- `await`'ten dönen her işlem, neslin ve aşamanın hâlâ aynı olduğunu
  denetler.
- Bir işlem yalnızca geçerli olduğu aşamada çalışır.
- Denetim: Linux'ta QML'in karşılaştırdığı durum adlarını sınayan test
  (`linux/src/bridge/stage.rs`); geri kalanı elle.

### Söz ile kod uyuşmuyor (#368, #393, #394)

- Gizlilik politikası, tehdit modeli, site ya da README bir süre, sınır ya
  da maliyet söylüyorsa, PR o sözü sağlayan kodu ve testi gösterir.
- Canlıdaki bir sözü bozacak parça ayrı bir PR'a bırakılmaz.
- Yeni bir hata gerekçesi, uygulamalardaki iletisiyle birlikte gelir ya da
  birleşmeden önce iletinin issue'su açılır.
- Bir metin değişince bütün dil dosyalarında aranır.
- Denetim: `site/tools/privacy.mjs`, `site/tools/check-i18n.mjs` ve
  `site/tools/check-readmes.mjs` çevirilerin yapısını ve sayılarını
  denetler; sözün kodla uyumu elle.

### Güvenlik (#304, #317, #329, #378)

- Dışarıdan gelen metin terminale ya da arayüze yazılmadan önce kontrol
  karakterleri görünür hâle getirilir.
- Hata sessizce yutulmaz; günlüğe ya da stderr'e yazılır.
- Kaçış çözme ve temizleme tek geçişte yapılır.
- Gizli bilgi taraması CI'da yapılır ve atlanamaz ([Gizli
  bilgiler](#gizli-bilgiler)).
- Denetim: CodeQL (temizleme hataları), `Secrets` işi (gizli bilgiler),
  `check-i18n.mjs` ve `check-readmes.mjs` (çevirilerde kontrol
  karakterleri). Terminale yazılan dış metin elle.

### Sürüm ve taban sürüklenmesi (#369, #390, #392)

- Her araç ve taban sürümünün tek kaynağı vardır; başka yerde geçen her
  kopyayı bir denetim karşılaştırır.
- Derleme ayarı yalnızca betikte değil, paket tarifinin kendisinde durur.
- Dependabot'un açmadığı ana sürüm geçişleri ve taban imajların destek
  bitiş tarihi bir issue'da izlenir.
- Denetim: `packaging/check-versions.sh` (Node dahil, #369),
  `tools/check-component-versions.sh`; destek bitiş tarihleri elle.

## Sürüm listesi

1. Platformun sürümünü artır ve `changelog.d/` girdilerini sürüm başlığına
   taşı: `node tools/changelog.mjs release "[linux-0.4.0-beta.1]"` (ortak
   yayında `"[android-…] and [linux-…]"`). Komut girdileri Keep a
   Changelog sırasıyla yeni bir bölüme yazar, dosyaları siler ve kurallara
   uymayan bir dosya varsa hiçbir şeyi değiştirmeden durur. Tek platformun
   yayını (`"[linux-…]"` ya da `"[android-…]"`) yalnızca o platformu ya da
   hiçbir platformu anmayan girdileri alır (Çekirdek, Site, CI gibi); CLI
   Linux'la gider. İki platformu birden anan girdi, öteki platformun alanı
   çıkarılarak yayına girer (platformsuz alanlar, ör. site, de onunla
   gider); dosyada yalnızca öteki platformun alanıyla onun yayınına kalır
   (#152). Sürüm şu dosyalarda artırılır. Linux: `linux/Cargo.toml`,
   `packaging/arch/PKGBUILD`, `packaging/debian/changelog`,
   `packaging/rpm/totaldrop.spec` ve
   `packaging/linux/app.totaldrop.TotalDrop.metainfo.xml`. Android:
   `app/pubspec.yaml` içinde hem sürüm hem derleme numarası (`+N`); numara
   her yüklemede artmalıdır, CI bunu denetler. Aynı sürüm, derleme
   numarası olmadan `app/rust/Cargo.toml`'a ve `Cargo.lock`'a da yazılır.
   Sunucular: `server/totaldrop-{rendezvous,relay}/Cargo.toml`, iki
   Worker'ın `package.json` ve `package-lock.json` dosyaları, CLI'ın bu
   iki sunucuya olan geliştirme bağımlılığı sürümleri ve `Cargo.lock`
   (`tools/check-component-versions.sh` hepsini denetler). CLI ve çekirdek: kök
   `Cargo.toml`'daki `[workspace.package]` sürümü, iç bağımlılıkların
   (`totaldrop-core = { version = … }`) sürümleri, `Cargo.lock` ve çalışma alanı dışındaki
   `fuzz/Cargo.lock` (denetimler onu görmez). CLI,
   Linux paketleriyle yayınlanır; CLI kodu son Linux yayınından bu yana
   değiştiyse ve sürüm henüz artırılmadıysa Linux yayını bu sürümü de
   artırır.
2. Sürüm PR'ı: kontroller ve bağımsız inceleme, sonra birleştir.
3. `main`'deki sürüm commit'ine açıklamalı etiket at (`linux-v…`,
   `android-v…`) ve gönder. Linux'ta CI, etiketin `main` üzerinde olduğunu
   ve beş dosyadaki sürümün etiketle aynı olduğunu denetler
   (`packaging/check-versions.sh`; aynı denetim her PR'da da çalışır).
   Linux etiketinde ayrıca CLI'nin sürümüne bakılır
   (`packaging/check-cli-version.sh`). Etiketlenen commit'ten önceki
   `linux-v…` ve `cli-v…` etiketlerinin her biri, o etiketteki
   `Cargo.toml` sürümünün yayınıdır. Bunların en yükseğinden daha yüksek
   bir sürüm geçer, daha düşük bir sürüm yayını durdurur. Sürüm aynıysa,
   o sürümü yayınlamış etiketlerin herhangi birinden bu yana `crates/`
   (`tests/` ve `benches/` klasörleri hariç), `l10n/app/`, `l10n/cli/` ya
   da `l10n/status.json` değiştiyse yayın durur. Bütün sürümler
   `0.MINOR.PATCH-beta.N` biçiminde olmalıdır. `Cargo.lock` ve kök
   `Cargo.toml` sayılmaz; CLI'yi etkileyen bir bağımlılık güncellemesinde
   sürümü artırmak yayınlayanın kararıdır.
   Sürüm etiketlerini yalnızca depo yöneticileri oluşturabilir (kural
   "release tags"); etiketi proje lideri atar. Yayın işleri `linux-release`
   ve `play-release` ortamlarında, proje liderinin (ortamların tek
   onaylayıcısı) onayıyla çalışır.
4. CI paketleri etiketten derler. Linux: her pakete derleme kaynağını
   gösteren imzalı bir kanıt (build provenance) ekler, aynı kanıtı Sigstore
   paketi olarak `totaldrop-linux-<sürüm>.sigstore.json` adıyla yayın
   dosyalarına koyar ve taslak bir GitHub yayını açar. Android: `play-release` ortamında yükleme anahtarıyla
   imzalı `.aab` üretir ve iş çıktısı (artifact) olarak bırakır; GitHub
   yayını açılmaz. Kişisel bir makinede derlenen hiçbir şey yayınlanmaz.
   Linux yayınında Flatpak paketi (`totaldrop-<sürüm>-<mimari>.flatpak`,
   x86_64 ve aarch64) de diğer paketlerle aynı kanıtı alır; çalışma
   zamanını Flathub'dan indirir. Tek istisna Flathub'dır
   ([0014](kararlar/0014-steam-deck.md)): Flathub, Flatpak'i kendi
   altyapısında `packaging/flatpak` tarifinden ve etiketli kaynaktan
   derler ve kendi anahtarıyla imzalar. Aynı tarif her PR'da bizim CI'ımızda
   derlenip korumalı alanda çalıştırılır. Flatpak tarifi Rust'ı
   `static.rust-lang.org`'un sürüm arşivinden sha256 ile alır; Rust
   yükseltilirken iki arşivin adresi ve sha256 değeri birlikte değişir
   (`check-versions.sh` sürümü denetler). KDE çalışma zamanı ve derleme
   imajı (`linux.yml`, özetle sabit) birlikte yükseltilir. Crate listesi
   (`cargo-sources.json`) depoda tutulmaz; her derlemeden önce
   `node tools/flatpak-cargo-sources.mjs write` ile `Cargo.lock`'tan
   yazılır.
   Derleyiciler sabittir (Rust `rust-toolchain.toml`'daki sürüm, Flutter
   iş akışındaki sürüm; `check-versions.sh` her yerde aynı olduklarını
   denetler), taban imajlar özetle (digest) sabittir ve kaynak arşivi son
   commit'in zamanıyla (`SOURCE_DATE_EPOCH`) üretilir. Taban imajın sabit
   olması paketleri sabitlemez: kapta kurulan dağıtım paketleri o günün
   sürümleridir. Dependabot Arch imajının özetini yalnız
   `arch.Dockerfile`'da günceller; aynı PR'da `linux.yml`'deki `container:`
   satırı da elle aynı özete getirilir, yoksa sürüm denetimi PR'ı durdurur.
   deb ve rpm imajları rustup-init'i sabit bir rustup sürümünden indirir ve
   sha256 ile doğrular. Sürüm yükseltilirken iki Dockerfile'daki
   `RUSTUP_VERSION` ile iki `RUSTUP_SHA256_*` birlikte değiştirilir;
   değerler `https://static.rust-lang.org/rustup/archive/<sürüm>/<hedef>/rustup-init.sha256`
   dosyalarından alınır (`check-versions.sh` iki Dockerfile'ın aynı
   değerleri taşıdığını denetler).
5. İndirilen paketler sağlama toplamıyla, Linux'ta ayrıca
   `gh attestation verify <paket> --repo totaldrop-app/totaldrop --bundle
   totaldrop-linux-<sürüm>.sigstore.json` ile ve temiz kaplarda kurulumla
   (`packaging/test-install.sh`) denenir. Flatpak paketi
   `flatpak install --user totaldrop-<sürüm>-<mimari>.flatpak` ile kurulup
   açılır.
6. Yayın notları 35 dilde yazılır, taslak yayımlanır.
7. Site güncellenir ve indirme bağlantılarının her biri denenir.
8. Android'de CI'ın imzaladığı paket Play Console'a yüklenir. Bundan önce
   `https://totaldrop.app/.well-known/assetlinks.json` yayında olmalıdır
   (sitenin dağıtım sonrası denetimi `server/tools/probe.mjs` bunu sınar):
   Android, `https://totaldrop.app/r` bağlantılarını uygulamaya ancak bu
   dosyayı kurulumda okuyabilirse verir (#445). İmza anahtarı ya da paket
   adı değişirse dosya da aynı PR'da değişir
   (`site/tools/test-assetlinks.mjs`).

## Node

CI'da Node çalıştıran her iş (Worker denetimleri, birim ve uçtan uca
testler, site betikleri, değişiklik günlüğü aracı, Linux betik testleri,
Wrangler ile dağıtım) aynı sabit Node ana sürümüyle (major) çalışır:
`.github/workflows/` altındaki her `actions/setup-node` adımı aynı
`node-version` değerini taşır. `lts/*` ya da `24.x` gibi kendiliğinden
değişen bir değer kullanılmaz; yeni bir LTS çıktığında CI'ın Node'u
habersiz değişmez. 24.x içinde hangi sürümün kurulacağını `setup-node`
belirler; önce runner imajındaki önbelleğe bakar. Buluşma Worker'ının
birim testleri `@types/node` ile tür denetiminden geçer; onun ana sürümü
de aynıdır, yoksa tür denetimi testlerin çalıştığı Node'da olmayan
API'leri kabul eder. Node çalıştıran yeni bir iş de aynı `setup-node`
adımını alır, runner imajındaki Node'a güvenmez.

`packaging/check-versions.sh` her PR'da (`CI` iş akışı, "Component
versions" işi) şunları denetler:

- her `setup-node` adımının kendi `node-version` satırı var;
- her değer sabit bir ana sürüm (`24`) ya da tam sürüm (`24.11.1`) ve
  hepsi aynı ana sürüm;
- `node`, `npm`, `npx` ya da `node_modules/.bin/` altından bir araç
  çalıştıran her işte bir `setup-node` adımı var (aşağıdaki Flatpak
  istisnası dışında);
- git'in izlediği her `package.json`'da (`.git` yoksa bulunan her
  `package.json`'da) `@types/node` tek bir ana sürüme bağlı
  (`24.19.2`, `^24.19.2`; `latest` ya da `>=24` değil), yanındaki
  `package-lock.json` da aynı ana sürümü kuruyor ve bu, Node'un ana
  sürümüyle aynı.

`packaging/test-check-versions.sh` denetimin kendisini aynı işte sınar.

**Yükseltme:** Node ana sürüm yükseltmesi tek bir PR'dır. Bütün
`node-version` değerleri ve `@types/node` aynı PR'da birlikte değişir.
Yalnızca LTS olan (çift numaralı) bir ana sürüme geçilir. PR'dan önce
şunlara bakılır:

- Wrangler yeni sürümü destekliyor mu: iki Worker'ın `package-lock.json`
  dosyasında `node_modules/wrangler` ve `node_modules/miniflare`
  altındaki `engines.node` aralığı yeni ana sürümü kapsamalıdır.
- Workers ne istiyor: Worker'lar Node'da değil, Cloudflare'in workerd
  çalışma zamanında çalışır; Node'un sürümü onları yalnızca Wrangler,
  yerel `wrangler dev` ve testler üzerinden etkiler. Node API'leri
  Worker'a yalnızca `nodejs_compat` bayrağıyla gelir; bugün iki Worker'da
  ve sitede de kapalıdır. Bayrak açılırsa, yükseltmede Cloudflare'in
  "Node.js compatibility" belgesindeki desteklenen API'lere karşı da
  bakılır ve bu madde güncellenir.
- PR'da Server, Site, Linux ve değişiklik günlüğü denetimleri yeni sürümle
  yeşil olmalıdır (`ci.yml` değiştiği için bütün alanlar çalışır).

Dependabot `actions/setup-node`'un SHA'sını günceller ama `node-version`'a
dokunmaz. `@types/node`'un ana sürüm güncellemelerini de önermez
(`.github/dependabot.yml`): tek başına gelen bir ana sürüm (#200, 24 → 26)
CI'dan geçer ama tür denetimine Node 24'te olmayan API'leri kabul ettirir.
Aynı ana sürüm içindeki `@types/node` güncellemeleri Dependabot'tan gelir.

**İstisna:** Flatpak işi (`linux.yml`, `flatpak`) Node'u Flathub'ın
özetle (digest) sabitlenmiş derleme imajından alır. O Node yalnızca imaj
yükseltilince değişir ve bu denetimin dışındadır; yalnızca
`tools/flatpak-cargo-sources.mjs`'i çalıştırır.

## Dağıtım

Site (totaldrop.app) ile buluşma ve aktarma sunucuları (rendezvous ve
relay Worker'ları) Cloudflare'e yalnızca CI'dan, `main`'den dağıtılır
(`.github/workflows/deploy.yml`). Böylece üretimde çalışan şey her zaman
incelenmiş bir commit'tir.

- **Ne zaman:** `main`'e gelen bir commit `site/public/`,
  `site/functions/`, `site/wrangler.toml` ya da `server/*-worker/`
  altında bir dosyayı değiştirdiyse yalnızca değişen hedef dağıtılır;
  `server/tools/probe.mjs` değişirse üçü de. Elle dağıtmak için Actions →
  Deploy → "Run workflow" ile hedef seçilir; her zaman `main`'in son hâli
  dağıtılır.
- **Önce denetim:** dağıtılacak commit'in site ya da sunucu denetimleri
  (`site.yml`, `server.yml`) aynı çalıştırmada yeniden koşar; geçmezlerse
  dağıtım başlamaz.
- **Onay:** her dağıtım işi `cloudflare` ortamında bekler ve depo
  yöneticisi onaylamadan çalışmaz. Ortam yalnızca `main` dalına açıktır.
  Tek yönetici olduğu için ortamda "Prevent self-review" kapalı kalmalıdır.
- **Sağlık denetimi:** dağıtımdan sonra `server/tools/probe.mjs` çalışır.
  Buluşma sunucusunda hiç ileti göndermeden bir WebSocket açıp kapatır
  (numara ayırmaz) ve hiç verilmeyen bir numarayı sorup dizinin
  (Durable Object) "unknown nameplate" yanıtını bekler; aktarma
  sunucusunda rastgele bir çift için WebSocket açıp kapatır; sitede ana
  sayfayı ve indirme sayfasını açar. Her deneme en çok 10 saniye sürer, 12
  deneme yapılır; en geç yaklaşık üç dakikada sonuç verir.
- **Geri alma:** en kolay yol, Actions'ta son sağlam Deploy çalıştırmasını
  açıp aynı işi yeniden çalıştırmaktır ("Re-run jobs"): yeniden çalıştırma
  o eski commit'i dağıtır ve yine onay ister. Bu, yerelde oturum açmayı
  gerektirmez. Acil durumda Worker için `wrangler login` ile geçici oturum
  açıp `server/<worker>` dizininde `npx wrangler deployments list` ve
  `npx wrangler rollback <sürüm-kimliği>` kullanılabilir; işten sonra
  `wrangler logout`. `wrangler rollback`, arada bir Durable Object geçişi
  (`migrations`) varsa geri almayı reddeder; hiçbir geri alma Durable
  Object'lerdeki veriyi geri almaz. Site için Cloudflare panelinde
  Workers & Pages → totaldrop → Deployments altında önceki dağıtımda
  "Rollback to this deployment" seçilir. Geri almadan sonra hatayı düzelten
  commit normal yoldan gelir.

**Kurulum (bir kez, depo yöneticisi yapar):**

1. Cloudflare'de bir API belirteci oluşturun: Account → Workers Scripts:
   Edit ve Cloudflare Pages: Edit; Zone (yalnız totaldrop.app) → Workers
   Routes: Edit. Cloudflare belirteci tek tek Worker'lara kısıtlayamaz;
   en dar kapsam bu hesap ve bu alan adıdır. İstemci IP'si kısıtlaması
   eklemeyin; GitHub'ın çalıştırıcıları değişken adres kullanır.
2. GitHub'da Settings → Environments → `cloudflare`: gerekli onaylayıcı
   olarak kendinizi ekleyin, dağıtım dalını `main` ile sınırlayın. Ortam,
   iş akışı ilk kez çalışmadan önce var olmalıdır: olmayan bir ortamı
   GitHub korumasız olarak kendisi açar.
3. Aynı ortama gizli değer `CLOUDFLARE_API_TOKEN` (belirteç) ve değişken
   `CLOUDFLARE_ACCOUNT_ID` (hesap kimliği) ekleyin. Belirteç başka hiçbir
   yerde durmaz.
4. Eski, geniş yetkili belirteçleri ya da yerelde kalan oturumları
   (`wrangler logout`) kaldırın; elle dağıtım artık gerekmez. Acil geri
   alma için açılan geçici oturum da iş bitince kapatılır.

## Gizli bilgiler

İmza anahtarları, parolalar ve API anahtarları yalnızca GitHub Actions
gizli değerlerinde ve sahibinin bilgisayarında durur. Hiçbir oturum ya da
araç bunları okumaz, yazdırmaz ya da depoya eklemez.

CI'daki `Secrets` işi her PR'da ve `main`'e gelen her push'ta çalışır
(#378). İş geçmeden `CI` kontrolü yeşil olmaz. Taradığı commit'ler o
çalıştırmanın getirdikleridir: PR'da PR'ın commit'leri, `main`'de push'un
getirdikleri, elle çalıştırmada bütün geçmiş. İki denetim yapar:

- **Dosya adları:** ağaçta ya da taranan commit'lerden birinde
  `key.properties`, `.jks`, `.keystore`, `.p12`, `.pfx`, `.pepk` ya da
  `.gitleaksignore` dosyası varsa başarısız olur (büyük küçük harf fark
  etmez). PR'lar rebase ile birleştirildiği için her commit `main`'e
  girer; bir PR'da eklenip sonraki commit'te silinen anahtar da yakalanır.
- **gitleaks:** taranan commit'lerin değişikliklerinde gizli bilgi arar.
  Taranan değişiklik taramayı susturamaz: ayarlar taban commit'teki
  `.gitleaks.toml`'dan (yoksa gitleaks'in varsayılanından) okunur,
  `gitleaks:allow` yorumları ve `.gitleaksignore` dikkate alınmaz,
  `.gitattributes` ya da bir NUL baytı metni ikili dosya gibi
  gösteremez. Bulgular maskelenerek yazılır. İmaj sürüm ve özetle
  sabittir ve pre-commit hook'unun sürümüyle (v8.30.0) birlikte
  yükseltilir.

Bu işin bulduğu bir gizli bilgi açığa çıkmış sayılır: değeri hemen
değiştirilir, sonra commit PR branch'inin geçmişinden çıkarılır. Yanlış
pozitif ancak kök dizindeki `.gitleaks.toml` dosyasına, her girdinin
yanına nedeni yazılarak eklenir. Bu girdi ayrı bir PR'la önce `main`'e
girer; bir PR kendi değişikliğini kendi ayarıyla taratamaz.

Bilinen boşluk: gitleaks'in varsayılan ayarları bazı yolları hiç taramaz,
ör. `.pdf`, `.bin`, `.exe`, `.doc(x)`, `.xls(x)` uzantılı dosyalar ve
adında `gitleaks.toml` geçen yollar. Bu dosyalara gizli bilgi
yazılmamalıdır; CI onları yakalamaz.
