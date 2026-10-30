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
5. **Bağımsız inceleme:** Her PR'ı, kodu yazmamış biri okur. Proje tek kişiyle
   yürüdüğü için bu, değişikliği yazan oturumdan ayrı çalışan bir inceleme
   ajanıdır: yalnızca diff'i ve ilgili kodu görür, yazarın gerekçesini
   görmez. Bulduğu her doğruluk ya da güvenlik sorunu birleştirmeden önce
   düzeltilir; düzeltilmeyenler için gerekçe PR'a yazılır.
6. **Birleştirme:** `main`'e yalnızca PR ile, rebase ile birleştirilir;
   geçmiş doğrusal kalır. Doğrudan push ve force push kapalıdır. `CI`
   kontrolü yeşil olmadan kural birleştirmeye izin vermez.
7. **Yayın:** [Sürüm listesi](#sürüm-listesi) adım adım izlenir.

## Bitti tanımı

Bir değişiklik ancak şunların hepsi doğruysa bitmiştir:

- Davranışı sınayan testler var ve geçiyor. Hata düzeltmesinde, önce hatayı
  gösteren bir test yazılır.
- `cargo fmt`, `cargo clippy -- -D warnings` ve platformun kendi denetimi
  (`flutter analyze`, `tsc`) temiz.
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

## Sürüm listesi

1. Platformun sürümünü artır ve `changelog.d/` girdilerini sürüm başlığına
   taşı: `node tools/changelog.mjs release "[linux-0.4.0-beta.1]"` (ortak
   yayında `"[android-…] and [linux-…]"`). Komut girdileri Keep a
   Changelog sırasıyla yeni bir bölüme yazar, dosyaları siler ve kurallara
   uymayan bir dosya varsa hiçbir şeyi değiştirmeden durur. Komut bütün
   dosyaları taşır; yalnızca bir platform yayınlanıyorsa öteki platformun
   dosyaları önce `changelog.d/` dışına alınır, sonra aynı PR'da geri
   konur (#152). Sürüm şu dosyalarda artırılır. Linux: `linux/Cargo.toml`,
   `packaging/arch/PKGBUILD`, `packaging/debian/changelog`,
   `packaging/rpm/totaldrop.spec` ve
   `packaging/linux/app.totaldrop.TotalDrop.metainfo.xml`. Android:
   `app/pubspec.yaml` içinde hem sürüm hem derleme numarası (`+N`); numara
   her yüklemede artmalıdır, CI bunu denetler. CLI ve çekirdek: kök
   `Cargo.toml`'daki `[workspace.package]` sürümü, iç bağımlılıkların
   (`totaldrop-core = { version = … }`) sürümleri ve `Cargo.lock`. CLI,
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
   Sürüm etiketlerini yalnızca depo yöneticisi oluşturabilir (kural "release
   tags"); yayın işleri `linux-release` ve `play-release` ortamlarında,
   yöneticinin onayıyla çalışır.
4. CI paketleri etiketten derler. Linux: her pakete derleme kaynağını
   gösteren imzalı bir kanıt (build provenance) ekler ve taslak bir GitHub
   yayını açar. Android: `play-release` ortamında yükleme anahtarıyla
   imzalı `.aab` üretir ve iş çıktısı (artifact) olarak bırakır; GitHub
   yayını açılmaz. Kişisel bir makinede derlenen hiçbir şey yayınlanmaz.
   Derleyiciler sabittir (Rust `rust-toolchain.toml`'daki sürüm, Flutter
   iş akışındaki sürüm; `check-versions.sh` her yerde aynı olduklarını
   denetler), taban imajlar özetle (digest) sabittir ve kaynak arşivi son
   commit'in zamanıyla (`SOURCE_DATE_EPOCH`) üretilir. Taban imajın sabit
   olması paketleri sabitlemez: kapta kurulan dağıtım paketleri o günün
   sürümleridir. Dependabot Arch imajının özetini yalnız
   `arch.Dockerfile`'da günceller; aynı PR'da `linux.yml`'deki `container:`
   satırı da elle aynı özete getirilir, yoksa sürüm denetimi PR'ı durdurur.
5. İndirilen paketler sağlama toplamıyla ve temiz kaplarda kurulumla
   (`packaging/test-install.sh`) denenir.
6. Yayın notları İngilizce ve Türkçe yazılır, taslak yayımlanır.
7. Site güncellenir ve indirme bağlantılarının her biri denenir.
8. Android'de CI'ın imzaladığı paket Play Console'a yüklenir.

## Dağıtım

Site (totaldrop.app) ile buluşma ve aktarma sunucuları (rendezvous ve
relay Worker'ları) Cloudflare'e yalnızca CI'dan, `main`'den dağıtılır
(`.github/workflows/deploy.yml`). Böylece üretimde çalışan şey her zaman
incelenmiş bir commit'tir.

- **Ne zaman:** `main`'e gelen bir commit `site/` ya da
  `server/*-worker/` altında bir dosyayı değiştirdiyse yalnızca değişen
  hedef dağıtılır. Elle dağıtmak için Actions → Deploy → "Run workflow"
  ile hedef seçilir; her zaman `main`'in son hâli dağıtılır.
- **Onay:** her dağıtım işi `cloudflare` ortamında bekler ve depo
  yöneticisi onaylamadan çalışmaz. Ortam yalnızca `main` dalına açıktır.
- **Denetim:** dağıtımdan sonra `server/tools/probe.mjs` çalışır. Buluşma
  ve aktarma sunucusunda gerçek bir WebSocket el sıkışması yapar (hiç ileti
  göndermeden kapatır, numara ayırmaz), sitede ana sayfayı ve indirme
  sayfasını açar. Bir dakika içinde yanıt yoksa iş başarısız olur.
- **Geri alma:** Worker için `server/<worker>` dizininde
  `npx wrangler deployments list` ile önceki sürümü bulup
  `npx wrangler rollback <sürüm-kimliği>` çalıştırın. Site için
  Cloudflare panelinde Workers & Pages → totaldrop → Deployments altında
  önceki dağıtımda "Rollback to this deployment" seçilir. Geri almadan
  sonra hatayı düzelten commit normal yoldan gelir; CI'ın bir sonraki
  dağıtımı geri alınmış sürümün yerine geçer.

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
   (`wrangler logout`) kaldırın; elle dağıtım artık gerekmez.

## Gizli bilgiler

İmza anahtarları, parolalar ve API anahtarları yalnızca GitHub Actions
gizli değerlerinde ve sahibinin bilgisayarında durur. Hiçbir oturum ya da
araç bunları okumaz, yazdırmaz ya da depoya eklemez.
