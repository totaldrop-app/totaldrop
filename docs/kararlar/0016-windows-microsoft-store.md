# 0016: Windows uygulaması WinUI 3 ile yazılır, Microsoft Store'dan dağıtılır

- **Durum:** Kabul, 10 Kasım 2026 (Ozan, #33)
- **Bağlam:** Windows uygulaması (#33) bugüne kadar iki şeyi
  bekliyordu: arayüz teknolojisinin seçilmesi ve imza sertifikasının
  bağışlarla alınması. Bağış sayfası bunu "Windows'tan önce" bir gider
  olarak yazar (`don.b4`: Certum'un açık kaynak sertifikası, ilk yıl
  69 EUR, sonra 49 EUR); tehdit modelindeki T12 de "Windows ve macOS
  imzası bağışları bekliyor" der. İmzasız bir `.exe` ya da `.msi`
  Windows'ta SmartScreen ve Defender uyarısıyla açılır; şifrelemeye
  dayanan bir uygulama için bu güveni baştan zedeler.

  İki şey değişti:
  - Microsoft Store'a geliştirici kaydı artık ücretsiz: bireysel hesapta
    Eylül 2025'ten, şirket hesabında 2026'dan beri. Kayıt
    `storedeveloper.microsoft.com` üzerinden yapılır; başka yollardan
    girilirse eski akış açılır.
  - Store'a MSIX paketi gönderilirse Microsoft paketi sertifikasyondan
    sonra kendi sertifikasıyla yeniden imzalar; bizim sertifika almamız
    gerekmez. Bu yalnızca MSIX için geçerlidir: Store'a `.exe` ya da `.msi`
    gönderilirse imzayı biz atmak zorundayız.

  Böylece imza sorunu parasız çözülebiliyor. Paket yöneticileri kararı
  önerisi (#344, PR #347) Microsoft Store kanalını ve Store'un imza
  durumunu bu karara bırakır. O öneri şirket hesabının ücretli olduğunu
  yazar; Microsoft'un Şubat 2026 tarihli sayfasına göre artık değildir.

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
     ABI üzerinden bağlı.** Windows 11'in kendi arayüz takımıdır: Fluent
     denetimleri, açık/koyu/yüksek karşıtlık temaları, sistem vurgu rengi,
     Segoe UI Variable ve Fluent simgeleri doğrudan gelir. Mica
     (`MicaBackdrop`) ve başlık çubuğu tümleşmesi
     (`ExtendsContentIntoTitleBar`, başlık düğmelerinin renkleri) yerel
     olarak vardır ama uygulama kurar; başlık düğmeleri canlı tema
     değişikliğini kendiliğinden izlemeyebilir, bu adım 3'te denenir.
     Native AOT ile .NET çalışma zamanı ayrıca kurulmaz, açılış hızlıdır;
     karşılığında XAML budamaya uygun yazılır (`x:Bind`, yansımalı
     `{Binding}` yok).
- **Önerilen:** Seçenek 3. Linux'ta Qt seçilirken ölçüt neyse
  ([0006](0006-linux-qt.md)) burada da odur: platformun kendi araç takımı
  ve sisteme tam uyum.

### Kurallar

- Çekirdek değişmez: protokol, kriptografi ve aktarım yalnızca Rust'ta
  (`crates/`) durur; C# tarafı kriptografi yapmaz ([mimari.md](../mimari.md)).
  Çekirdek için ince bir `windows/core-ffi` crate'i C ABI açar. Bağlama
  aracı (uniffi ile C# üreticisi ya da csbindgen) adım 2'de seçilir;
  ölçüt, bağımlılığın bakımı ve iptal/ilerleme olaylarının iş parçacığı
  güvenliğidir. Arayüz iş parçacığı çekirdeği hiç beklemez.
- Windows App SDK çalışma zamanı pakete gömülü mü (self-contained), Store
  bağımlılığı olarak mı gelir, adım 2'de seçilir.
- Hedefler ilk günden x64 ve ARM64'tür (#86); ikisi tek `.msixbundle`'da.
- En düşük sürüm adım 2'de seçilir. Windows App SDK Windows 10 1809'dan
  beri destekler; Windows 10 Ekim 2025'te genel destekten çıktı. Windows
  10'da Mica yoktur: Windows 10 desteklenirse orada sistem arka plan rengi
  kullanılır ve bu, tasarim.md'deki Windows satırına yazılır.
- Diller: 35 dil, tek kaynaktan ([0008](0008-diller-ve-bolgeler.md)).
  ARB kaynağı derlemede `.resw`'ye çevrilir; çoğul ve sayılar ICU ile
  (Windows ICU'yu kendisi taşır). Sağdan sola diller `FlowDirection` ile;
  kod ve doğrulama kelimeleri soldan sağa kalır.
- İşaret aktarım sırasında web sitesindeki animasyonun aynısını oynatır
  ([tasarim.md](../tasarim.md), "Hareket"). **Hareketi azalt** kuralı
  Windows'ta "Animasyon efektleri" ayarına (`UISettings.AnimationsEnabled`)
  bağlanır: kapalıysa işaret dinlenme hâlinde durur.
- Her arayüz adımı Windows'ta açık, koyu ve yüksek karşıtlık temalarında
  denenir, ekran görüntüleri PR'a konur (Linux'taki kural gibi, 0006).
- Erişilebilirlik: her denetimin UI Automation adı; Narrator ve NVDA ile,
  yalnızca klavyeyle ve yüksek karşıtlıkta denenir; metin boyutu %225'e
  kadar taşmaz.

### Sistem tümleşmesi

- Gezgin'de sağ tık "TotalDrop ile gönder": Windows 11'in
  `IExplorerCommand` arayüzü, MSIX paketinin kimliğiyle
  (`desktop4`/`desktop5:FileExplorerContextMenus` ve `com:Extension`). Bu,
  `explorer.exe` içinde çalışan küçük bir COM sunucusudur; C# Native AOT
  buna uygun değildir. Rust (`windows` crate'i) ile yazılır, yalnızca
  seçilen yolları uygulamaya iletir ve kendi güvenlik incelemesinden
  geçer: bir hata Gezgin'i çökertir. Eski menü yalnızca yedek.
- Windows paylaşım penceresinde hedef olmak (Share Target) ve uygulamadan
  paylaşmak (`DataTransferManager`).
- Pencereye ve görev çubuğu simgesine sürükle-bırak, tek pencere
  (açıkken ikinci açılış dosyaları açık pencereye verir), işlemli bildirim
  (toast).
- Aktarım sürerken bilgisayarın uyumaması (#340) ve pencere kapatıldığında
  aktarımın ne olacağı Linux'taki kurallarla aynıdır.
- Güvenlik duvarı: QUIC'in UDP üzerinden dinlemesi kural yoksa ilk
  açılışta Windows Defender Güvenlik Duvarı izin penceresini açar. Plan,
  MSIX bildiriminde `desktop2:Extension Category="windows.firewallRules"`
  ile kurulumda gelen bir kuraldır; yalnızca uygulamanın kendi ikilisi ve
  mümkünse yalnızca özel ağ profili için. **Doğrulanacak:** bu uzantının
  Store paketinde kabul edildiği ve kuralın kaldırmada silindiği.

## Dağıtım

- **Seçenekler:**
  1. İmzasız `.exe`/`.msi` sürüm sayfasında. Ücretsiz ama her kurulumda
     uyarı; güven için kabul edilemez.
  2. Certum'un açık kaynak sertifikasıyla imzalı paket (bugünkü plan,
     `don.b4`). Her yıl ödenir; anahtar bir akıllı kartta ya da Certum'un
     bulut anahtarında (SimplySign) durur ve ikisi de CI'dan imzalamayı
     zorlaştırır. Microsoft'a göre OV ve EV sertifikaları artık SmartScreen
     itibarını kendiliğinden getirmez; uyarı itibar birikene kadar sürer.
     Microsoft'un kendi imza hizmeti (Artifact Signing) bireylere yalnızca
     ABD ve Kanada'da açık.
  3. SignPath Foundation'ın ücretsiz açık kaynak imza programı. Yayıncı
     olarak "SignPath Foundation" görünür, TotalDrop değil (PR #347).
  4. **Microsoft Store, MSIX paketiyle; Microsoft imzalar.** winget
     uygulamayı `msstore` kaynağından aynı paketle kurar (Store ürün
     kimliğiyle, `9N…`).
- **Önerilen:** Seçenek 4. Mağaza dışında indirilecek bir Windows paketi
  olmaz. Seçenek 2 ya da 3 gerekirse ayrı bir kararla eklenir.
- winget'in topluluk deposundaki `TotalDrop.TotalDrop` kimliği (PR #347)
  bir kurulum adresi ister; Store dışında paket olmayınca o kimlik
  yayımlanamaz. Önerilen: kimlik boş bırakılır ve başkasınca alınma riski
  kabul edilir; alınırsa marka kuralıyla (0012) winget-pkgs
  inceleyicilerine bildirilir.

### Kurallar

- Paket yalnızca CI'da, etiketli commit'ten, GitHub'ın standart Windows
  koşucularında (`windows-2025`, `windows-11-arm`) derlenir
  ([0007](0007-surec.md), [0009](0009-ios-macos.md) CI kuralları). Her
  PR'da paket derlenir ve testler çalışır.
- Store paketimizi yeniden derlemez, yalnızca yeniden imzalar. Bu yüzden
  Store bir istisna değildir. Ama kullanıcıya giden dosyanın imzası bizim
  yapı kaynağı kanıtımızla (attestation) aynı olmaz; kanıt, CI'ın ürettiği
  `.msixupload` ve içindeki ikililer için verilir.
- Sürüm iş akışı (`windows-release.yml`) Android'deki gibi yalnızca
  artifact üretir; Store'a gönderimi Ozan Partner Center'dan yapar
  ([surec.md](../surec.md) "Sürüm listesi" ve `android-release.yml`'nin
  baş yorumu: CI'ın ürettiği paketi Play Console'a Ozan yükler).
  **Doğrulanacak:** Store gönderim API'si ya da `msstore` CLI bireysel
  hesapta kullanılabiliyor mu; bu, Entra ID kiracısı ve uygulama kaydı
  ister. Kullanılabiliyorsa sonra ayrı bir adımda `windows-release`
  ortamına, onaylı olarak eklenir.
- İmzasız test paketi: geliştirme ve CI denemeleri için paket CI'da her
  çalışmada yeniden üretilen, atılan bir öz imzalı sertifikayla imzalanır
  (ya da bildirimdeki yayıncı imzasız paket işaretini taşır ve
  `Add-AppxPackage -AllowUnsigned` ile kurulur). Bu paket dağıtılmaz.
- Paket kimliği (`Identity Name`, `Publisher`) Partner Center'daki ad
  ayırmadan gelir ve depoya yazılır; gizli bilgi değildir.
- **Korumalı alan yoktur.** WinUI 3 masaüstü uygulaması `runFullTrust`
  ile, kullanıcının bütün haklarıyla çalışır. `internetClient`,
  `internetClientServer` ve `privateNetworkClientServer` yetenekleri
  yalnızca AppContainer içinde uygulanır; burada bildirilir ama bir sınır
  çizmez. Flatpak'teki gibi bir izin sınırı (T15) Windows'ta yoktur; bu,
  kabul edilince tehdit modeline yazılır. `runFullTrust` kısıtlı bir
  yetenektir; gönderimde gerekçesi yazılır.
- Güncellemeyi Store yapar; uygulamanın kendi güncelleyicisi olmaz.

### Hesap

- **Bireysel hesap:** Ücretsiz. Kişisel Microsoft hesabıyla açılır;
  kimlik belgesi ve özçekimle doğrulanır. Microsoft'a göre "kendi adıyla"
  ve "işi, ticareti ya da mesleğiyle ilgili olmadan" yayımlayanlar
  içindir. Mağazada görünen ad "yayıncı görünen adı"dır; hesap türü,
  ülke ve bu ad hesap açıldıktan sonra değiştirilemez.
- **Şirket hesabı:** Ücretsiz. Kayıtlı bir tüzel kişilik ister (D-U-N-S
  numarası ya da resmî kuruluş belgesi) ve alan adıyla eşleşen bir iş
  e-postası. Ekip üyeleri rol alabilir. TotalDrop'un bugün tüzel kişiliği
  yoktur. Bağışlar için Open Collective'e başvuru (PR #354) sürüyor; orada
  emanetçi olacak kuruluşun tüzel kişiliği TotalDrop'unki sayılmaz.
- Bireysel hesap sonradan şirkete çevrilemez; şirket için yeni hesap
  açılır ve uygulama ona taşınır.
- **Bugünkü durum (7 Kasım 2026, Partner Center'dan okundu):** Ozan'ın
  daha önce açtığı bir hesap var. Türü bireysel, Ozan'ın kişisel
  Microsoft hesabına bağlı; yayıncı adı ve Windows yayıncı kimliği kişi
  adıdır, satıcı iletişim bilgileri de kişiseldir. Bu haliyle Store'da
  TotalDrop'u kişi adı yayımlar.
- **İlk gönderimden önce Ozan karar verir** (yanlış ad, yeni hesap ve yeni
  kimlik doğrulaması demektir):
  - Bireysel hesapta yayıncı adı "TotalDrop" olabilir mi. Microsoft'un
    tarifi kişinin kendi adını öngörür; "TotalDrop" reddedilebilir ya da
    şartlara aykırı sayılabilir.
  - Yayıncı olarak kişi adı ("Ozan Özdil") görünmesi marka kuralına
    ([0012](0012-ad-ve-isaret.md): TotalDrop'u yalnızca TotalDrop
    yayımlar) uyar mı. Uymuyorsa yol, tüzel kişilik kurulunca şirket
    hesabıdır ve Store o zamana kalır.
  - Proje bireysel hesap tarifine (ticari olmayan) uyar mı: uygulama
    ücretsiz, reklamsız, bağışla yürür.
- **Önerilen:** Mevcut hesapta yayıncı adını "TotalDrop", satıcı
  iletişim e-postasını `hello@totaldrop.app` yapmayı Ozan dener. Microsoft
  kabul etmezse Store, tüzel kişilikle açılacak şirket hesabına kalır.
  Değişiklikleri Ozan yapar; parola ve kimlik bilgileri depoya ve
  Claude'a verilmez. **Doğrulanacak:** bireysel hesapta yayıncı adının
  sonradan değiştirilip değiştirilemeyeceği ve Windows yayıncı kimliğinin
  (paketteki `Publisher`) adla birlikte değişip değişmediği.
- **Hesabın korunması:** Store kendi imzasıyla, otomatik güncelleme
  olarak ne gönderilirse dağıtır. Partner Center hesabını ele geçiren,
  hiçbir anahtarımıza dokunmadan bütün kullanıcılara güvenilir görünen
  kötü niyetli bir güncelleme gönderebilir. Bu yüzden: Microsoft
  hesabında parolasız giriş (passkey) ya da doğrulayıcı uygulama, SMS ile
  kurtarma yok; `hello@totaldrop.app` posta kutusu da iki aşamalı
  doğrulamalı ve erişimi sınırlı, çünkü hesabı kurtarabilir. Kimin
  kurtarabileceği yazılı tutulur. Bireysel hesaba kullanıcı eklenemediği
  için hesap tek kişiye bağlıdır; Ozan ulaşılamazsa yayın durur.
- **Doğrulanacak:**
  - Avrupa Birliği Dijital Hizmetler Yasası gereği Partner Center'ın
    sorduğu "tacir" (trader) beyanı. Tacir beyan edilirse Ozan'ın adı,
    adresi, e-postası ve telefonu AB'de mağazada herkese açık görünür;
    tek kişilik bir proje için gizlilik riskidir. Yanlış bir "tacir değil"
    beyanı ise hukuki risktir. Beyan Ozan'ındır; markası, sitesi ve
    bağışları olan bir projenin bu teste nasıl girdiği önceden okunur.
  - Bireysel hesaba gerçekten kullanıcı eklenemediği (ikincil kaynak).

### Ad ayırma

- Microsoft'a göre bir ad, uygulama yayımlanmadan en çok üç ay ayrılı
  kalır; sonra düşer. Adım 2–5 büyük olasılıkla üç aydan uzun sürer.
- Önerilen: "TotalDrop" adı, ilk gönderimden yaklaşık üç ay önce, adım 4
  civarında ayrılır. O zamana kadar adın başkasınca alınması riski kabul
  edilir; alınırsa marka kuralıyla (0012) Microsoft'a bildirilir.

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

1. Bu kararın kabulü ve "Hesap" bölümündeki soruların Ozan tarafından
   yanıtlanması: mevcut bireysel hesabın yayıncı adı ve iletişim bilgisi
   TotalDrop'a çevrilir ya da şirket hesabı için tüzel kişilik beklenir.
2. `windows/` klasörü: boş WinUI 3 penceresi, çekirdek bağlaması, CI'da
   x64 ve ARM64 derlemesi, test MSIX'i, birim testleri; bağlama aracı,
   çalışma zamanı dağıtımı ve en düşük Windows sürümü seçilir.
3. Gönderme ve alma ekranları, tema (Mica, başlık çubuğu, yüksek
   karşıtlık), 35 dil, erişilebilirlik; UI Automation testleri CI'da.
4. Gezgin menüsü (Rust COM sunucusu), paylaşım hedefi, bildirimler, tek
   pencere, güvenlik duvarı kuralı. Bu adım civarında ad ayrılır.
5. Sürüm iş akışı, mağaza sayfası metinleri, Store'a ilk gönderim (Ozan),
   kapalı test için Store'un paket uçuşları (package flights).
6. İndirme sayfası, README (35 dil), bağış sayfasında `don.b4` satırı
   ("Windows code signing certificate", 35 dil), tehdit modeli (T12 ve
   Windows için yeni satır) ve gizlilik politikası (35 dil: Microsoft
   Store kurulum ve kullanım verisini kendi sorumluluğunda toplar)
   güncellenir.

## Riskler

- Store sertifikasyonu reddedebilir ya da gecikebilir; Microsoft'a göre
  her gönderim üç iş gününe kadar sürer. Acil güvenlik düzeltmesi de bu
  sürede bekler.
- Partner Center hesabının ele geçirilmesi, imzalı kötü niyetli bir
  güncelleme demektir (yukarıda "Hesabın korunması").
- Store dışından kurmak isteyen (Store'u kapalı kurumsal bilgisayarlar)
  bu kararla kuramaz; talep olursa seçenek 2 ya da 3 ayrı bir kararla
  eklenir.
- Windows makinesi yok: denemeler CI koşucularında ve ekran görüntüleriyle
  yapılır; elde bir Windows bilgisayarı olunca o da kullanılır.
- Yayıncı adı ilk gönderimden önce çözülmezse, uygulama kişi adıyla
  yayımlanır; sonradan düzeltmek yeni hesap ve uygulama taşıma ister.

## Sonuçlar

- Windows için imza sertifikası parası gerekmez; bağış sayfasındaki
  `don.b4` kalemi düşer (macOS için Apple hesabı kalır).
- [tasarim.md](../tasarim.md) platform tablosuna Windows satırı eklenir:
  renk kaynağı sistem vurgu rengi, açık/koyu/yüksek karşıtlık
  (`UISettings`, `ThemeSettings`); denetimler WinUI 3 Fluent; pencere Mica
  ve sistem başlık çubuğu. Windows'un işaret dosyası "Hareket" bölümündeki
  aynı değerleri paylaşan dosyalar listesine girer.
- Tehdit modeline: T12'ye "Store paketini Microsoft yeniden imzalar,
  kullanıcı kanıtı doğrulayamaz; Partner Center hesabı tek nokta" notu ve
  Windows'ta korumalı alan olmadığını söyleyen bir satır.
- `windows/` klasörü ve `windows.yml`, `windows-release.yml` iş akışları
  açılır; CLAUDE.md'deki alan tablosunda `windows/` için
  `windows-app-engineering` zaten yazılı.
