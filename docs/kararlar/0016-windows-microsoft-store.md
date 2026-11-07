# 0016: Windows uygulaması WinUI 3 ile yazılır, Microsoft Store'dan dağıtılır

- **Durum:** Önerilen, 7 Kasım 2026 (#33)
- **Bağlam:** Windows uygulaması (#33) bugüne kadar iki şeyi
  bekliyordu: arayüz teknolojisinin seçilmesi ve imza sertifikasının
  bağışlarla karşılanması (README, tehdit modelinde T12). İmzasız bir
  `.exe` ya da `.msi` Windows'ta SmartScreen ve Defender uyarısıyla açılır;
  şifrelemeye dayanan bir uygulama için bu güveni baştan zedeler. Ücretli
  imza hizmetleri bağış ilkesi gereği seçenek değildir
  ([0005](0005-bagis.md)).

  İki şey değişti:
  - Microsoft Store'a geliştirici kaydı artık ücretsiz: bireysel hesapta
    Eylül 2025'ten, şirket hesabında 2026'dan beri. Kayıt
    `storedeveloper.microsoft.com` üzerinden yapılır; eski yollar eski,
    ücretli akışı gösterir.
  - Store'a MSIX paketi gönderilirse Microsoft paketi sertifikasyondan
    sonra kendi sertifikasıyla yeniden imzalar; bizim sertifika almamız
    gerekmez. Bu yalnızca MSIX için geçerlidir: Store'a `.exe` ya da `.msi`
    gönderilirse imzayı biz atmak zorundayız.

  Böylece imza sorunu parasız çözülebiliyor. Paket yöneticileri kararı
  önerisi (#344, PR #347) Microsoft Store kanalını ve Store'un imza
  durumunu bu karara bırakır.

  Kaynaklar (7 Kasım 2026'da okundu):
  [hesap açma](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account),
  [şirket hesabı](https://learn.microsoft.com/en-us/windows/apps/publish/whats-new-company-developer),
  [imza seçenekleri](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options),
  [Store SSS](https://learn.microsoft.com/en-us/windows/apps/publish/faq/get-started-with-the-microsoft-store).

## Arayüz

- **Seçenekler:**
  1. **Qt 6 ve QML, Linux kodunu paylaşarak.** Qt 6.8'den beri Windows 11
     görünümünü taklit eden bir "FluentWinUI3" stili var. En az iş bu;
     ama Mica arka planı, başlık çubuğu tümleşmesi, sistem vurgu rengi ve
     yüksek karşıtlık gibi ayrıntılar taklit düzeyinde kalır. Tasarım kuralı
     ([tasarim.md](../tasarim.md)) sistem temasına yüzde yüz uyum ister.
  2. **Flutter, Android uygulamasının Windows hedefi.** Kod paylaşılır ama
     Material görünümü Windows'a yabancıdır; Fluent görünüm üçüncü taraf
     paketlere kalır.
  3. **WinUI 3 (Windows App SDK) ve C# (.NET, Native AOT), çekirdeğe C
     ABI üzerinden bağlı.** Windows 11'in kendi arayüz takımıdır: Mica,
     başlık çubuğu, vurgu rengi, açık/koyu/yüksek karşıtlık temaları,
     Segoe UI Variable ve Fluent simgeleri kendiliğinden gelir ve tema
     değişikliği canlı uygulanır. Native AOT ile .NET çalışma zamanı ayrıca
     kurulmaz, açılış hızlıdır.
- **Önerilen:** Seçenek 3. Linux'ta Qt seçilirken ölçüt neyse
  ([0006](0006-linux-qt.md)) burada da odur: platformun kendi araç takımı
  ve sisteme tam uyum.

### Kurallar

- Çekirdek değişmez: protokol, kriptografi ve aktarım yalnızca Rust'ta
  (`crates/`) durur; C# tarafı kriptografi yapmaz ([mimari.md](../mimari.md)).
  Çekirdek için ince bir `windows/core-ffi` crate'i C ABI açar. Bağlama
  aracı (uniffi ile C# üreticisi ya da csbindgen) adım 1'de seçilir;
  ölçüt, bağımlılığın bakımı ve iptal/ilerleme olaylarının iş parçacığı
  güvenliğidir. Arayüz iş parçacığı çekirdeği hiç beklemez.
- Hedefler ilk günden x64 ve ARM64'tür (#86); ikisi tek `.msixbundle`'da.
  En düşük sürüm Windows 11 23H2 ile Windows 10 22H2 arasında adım 1'de
  seçilir. Windows 10 Ekim 2025'te genel destekten çıktı; Windows App SDK
  hâlâ destekliyor.
- Diller: 35 dil, tek kaynaktan ([0008](0008-diller-ve-bolgeler.md)).
  ARB kaynağı derlemede `.resw`'ye çevrilir; çoğul ve sayılar ICU ile
  (Windows ICU'yu kendisi taşır). Sağdan sola diller `FlowDirection` ile;
  kod ve doğrulama kelimeleri soldan sağa kalır.
- İşaret aktarım sırasında web sitesindeki animasyonun aynısını oynatır
  ([tasarim.md](../tasarim.md), "Hareket"); "Animasyonları kapat"
  ayarında durur.
- Erişilebilirlik: her denetimin UI Automation adı; Narrator ve NVDA ile,
  yalnızca klavyeyle ve yüksek karşıtlıkta denenir; metin boyutu %225'e
  kadar taşmaz.

### Sistem tümleşmesi

- Gezgin'de sağ tık "TotalDrop ile gönder": Windows 11'in
  `IExplorerCommand` arayüzü, MSIX paketinin kimliğiyle. Eski menü yalnızca
  yedek.
- Windows paylaşım penceresinde hedef olmak (Share Target) ve uygulamadan
  paylaşmak (`DataTransferManager`).
- Pencereye ve görev çubuğu simgesine sürükle-bırak, tek pencere
  (açıkken ikinci açılış dosyaları açık pencereye verir), işlemli bildirim
  (toast).
- Aktarım sürerken bilgisayarın uyumaması (#340) ve pencere kapatıldığında
  aktarımın ne olacağı Linux'taki kurallarla aynıdır.
- **Doğrulanacak:** Tam yetkili (`runFullTrust`) masaüstü uygulamasında
  QUIC'in UDP üzerinden dinlemeye başlaması Windows Güvenlik Duvarı
  izin penceresini açar mı; açarsa ilk açılışta ne söylenir.

## Dağıtım

- **Seçenekler:**
  1. İmzasız `.exe`/`.msi` sürüm sayfasında. Ücretsiz ama her kurulumda
     uyarı; güven için kabul edilemez.
  2. Ücretsiz açık kaynak imza programı (SignPath Foundation) ile imzalı
     MSIX ya da kurulum paketi, sürüm sayfasında ve winget'te. Yayıncı
     olarak "SignPath Foundation" görünür, TotalDrop değil (PR #347).
  3. **Microsoft Store, MSIX paketiyle; Microsoft imzalar.** winget
     uygulamayı `msstore` kaynağından aynı paketle kurar.
- **Önerilen:** Seçenek 3. Mağaza dışında indirilecek bir Windows paketi
  olmaz; seçenek 2 ayrıca gerekirse ayrı karar olur.

### Kurallar

- Paket yalnızca CI'da, etiketli commit'ten, GitHub'ın standart Windows
  koşucularında (`windows-2025`, `windows-11-arm`) derlenir
  ([0007](0007-surec.md), [0009](0009-ios-macos.md) CI kuralları). Her
  PR'da paket derlenir ve testler çalışır.
- Store paketi yeniden derlemez, yalnızca yeniden imzalar. Bu yüzden Store
  bir istisna değildir. Ama kullanıcıya giden dosyanın imzası bizim yapı
  kaynağı kanıtımızla (attestation) aynı olmaz; kanıt, CI'ın ürettiği
  `.msixupload` ve içindeki ikililer için verilir.
- Sürüm iş akışı (`windows-release.yml`) Android'deki gibi yalnızca
  artifact üretir; Store'a gönderimi Ozan Partner Center'dan yapar
  ("Play'e YÜKLEMEZ" kuralının aynısı). **Doğrulanacak:** Store gönderim
  API'si ya da `msstore` CLI bireysel hesapta kullanılabiliyor mu; bu,
  Entra ID kiracısı ve uygulama kaydı ister. Kullanılabiliyorsa sonra ayrı
  bir adımda `windows-release` ortamına, onaylı olarak eklenir.
- Paket kimliği (`Identity Name`, `Publisher`) Partner Center'daki ad
  ayırmadan gelir ve depoya yazılır; gizli bilgi değildir.
- Yetenekler (capabilities) en azdır: `internetClient`,
  `internetClientServer` ve `privateNetworkClientServer` (yerel ağda
  doğrudan QUIC), masaüstü uygulaması için `runFullTrust`. `runFullTrust`
  kısıtlı bir yetenektir; gönderimde gerekçesi yazılır.
- Güncellemeyi Store yapar; uygulamanın kendi güncelleyicisi olmaz.

### Hesap

- **Bireysel hesap:** Ücretsiz. Kişisel Microsoft hesabıyla açılır;
  kimlik belgesi ve özçekimle doğrulanır. Microsoft'a göre ticari olmayan,
  "kendi adıyla" yayımlayan geliştiriciler içindir. Mağazada müşteriye
  görünen ad, 50 karakteri geçmeyen ve benzersiz olan "yayıncı görünen
  adı"dır. Bu adı kullanma hakkının yayıncıda olması gerekir. Kullanıcı
  eklenemez.
- **Şirket hesabı:** Ücretsiz. Kayıtlı bir tüzel kişilik ister (D-U-N-S
  numarası ya da resmî kuruluş belgesi) ve alan adıyla eşleşen bir iş
  e-postası. Ekip üyeleri rol alabilir. TotalDrop'un bugün tüzel kişiliği
  yok; Open Collective'deki emanetçinin (Open Source Collective) tüzel
  kişiliği TotalDrop'un değildir.
- Bireysel hesap sonradan şirkete çevrilemez; şirket için yeni hesap
  açılır ve uygulama ona taşınır.
- **Önerilen:** Bireysel hesap, `hello@totaldrop.app` ile açılmış bir
  Microsoft hesabıyla, yayıncı görünen adı "TotalDrop". Hesabı Ozan açar,
  kimlik doğrulamasını Ozan yapar; parola ve kimlik bilgileri depoya ve
  Claude'a verilmez. Ekip tüzel kişilik kurarsa şirket hesabına geçilir.
- **Doğrulanacak:**
  - Bireysel hesapta yayıncı görünen adı olarak "TotalDrop" kabul ediliyor
    mu, yoksa kişinin adı mı isteniyor. Marka kuralı Store'da yayıncının
    TotalDrop görünmesini ister ([0012](0012-ad-ve-isaret.md)).
  - Avrupa Birliği Dijital Hizmetler Yasası gereği Partner Center'ın
    sorduğu "tacir" (trader) beyanı: tacir beyan edilirse ad, adres,
    e-posta ve telefon AB'de mağazada herkese açık görünür. Ücretsiz,
    bağışla yürüyen bir uygulama için "tacir değil" beklenir.
  - Ad ayırmanın gönderim yapılmazsa ne kadar sürede düştüğü.

### Mağaza sayfası

- 35 dilde. Ad, kısa açıklama ve açıklama, AppStream dosyasını üreten aynı
  kaynaktan (`tools/desktop-entries.mjs`) üretilir; elle kopyalanmaz.
  Windows'a özgü cümleler (tema, Gezgin menüsü) önce `l10n/`'a eklenir.
  **Doğrulanacak:** Store'un 35 dilin hepsini (özellikle `sw`, `ur`,
  `bn`, `fa`, `ms` ve `zh-HK`/`zh-MO`/`zh-TW`) kabul edip etmediği ve
  hangi kodlarla.
- Gizlilik politikası adresi: `https://totaldrop.app/privacy/` (35 dil).
  Destek: `hello@totaldrop.app`. Yaş derecelendirmesi IARC anketiyle.
- Ekran görüntüleri açık ve koyu temada, en az 1366×768, birkaç dilde
  (biri sağdan sola). Platform ya da üretici logosu kullanılmaz
  ([tasarim.md](../tasarim.md)).

## Sıra

1. Bu kararın kabulü. Ozan Partner Center hesabını açar ve "TotalDrop"
   adını ayırır (ad ayırma süresi doğrulanınca, uygulama hazır olmadan
   düşmeyecek bir zamanda).
2. `windows/` klasörü: boş WinUI 3 penceresi, çekirdek bağlaması, CI'da
   x64 ve ARM64 derlemesi, imzasız test MSIX'i (yalnızca geliştirici
   modunda kurulur), birim testleri.
3. Gönderme ve alma ekranları, tema, 35 dil, erişilebilirlik; UI
   Automation testleri CI'da.
4. Gezgin menüsü, paylaşım hedefi, bildirimler, tek pencere.
5. Sürüm iş akışı, mağaza sayfası metinleri, Store'a ilk gönderim (Ozan),
   kapalı test için Store'un paket uçuşları (package flights).
6. İndirme sayfası, README (35 dil), tehdit modeli T12 ve bağış sayfasındaki
   "Windows imzası bağış bekliyor" ifadesi güncellenir.

## Riskler

- Store sertifikasyonu reddedebilir ya da gecikebilir; her gönderim birkaç
  gün sürebilir. Acil güvenlik düzeltmesi de bu sürede bekler.
- Store dışından kurmak isteyen (Store'u kapalı kurumsal bilgisayarlar)
  bu kararla kuramaz; talep olursa seçenek 2 ayrı karar olur.
- Windows makinesi yok: denemeler CI koşucularında ve ekran görüntüleriyle
  yapılır; elde bir Windows bilgisayarı olunca o da kullanılır.
- Bireysel hesapta "TotalDrop" yayıncı adı kabul edilmezse marka kuralı
  Store'da karşılanmaz; bu durumda yayın öncesi Ozan yeniden karar verir.

## Sonuçlar

- Windows için imza sertifikası parası gerekmez; bağış sayfasındaki bu
  kalem Windows için düşer (macOS için kalır).
- `windows/` klasörü ve `windows.yml`, `windows-release.yml` iş akışları
  açılır; CLAUDE.md'deki alan tablosunda `windows/` için
  `windows-app-engineering` zaten yazılı.
