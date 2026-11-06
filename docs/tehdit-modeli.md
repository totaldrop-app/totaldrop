# Tehdit modeli

Bu belge TotalDrop'un neyi kimden koruduğunu, hangi saldırılara nasıl karşı
durduğunu ve neyi korumadığını yazar. 1.0 öncesi bağımsız denetimin
başlangıç noktasıdır. Protokolün kendisi v0 taslağında, uygulamadaki
kararlar [protocol-v0.1-notes.md](protocol-v0.1-notes.md) içindedir.

Son gözden geçirme: 4 Kasım 2026 (hesap bölümü, #243).

## Korunan varlıklar

Bugün korunanlar ile henüz yalnızca tasarımda olanlar ayrı yazılır.

| Varlık | Kimden saklanır | Durum |
| --- | --- | --- |
| Dosyaların içeriği, adları ve klasör yapısı | Sunucular, ağdaki herkes, kodu bilmeyen herkes | Var |
| Dosya boyutları | Sunucular ve ağ gözlemcisi şifreli trafiğin toplam boyutunu görür; tek tek dosya boyutları gizli kalır | Kısmen |
| Kullanıcının kimliği | Herkes: hesap gerekmez | Var; isteğe bağlı, sunucuda tutulmayan hesap (Hayalet halka) [tasarımda](hesap.md) |
| Kim kime gönderdi | Sunucular iki cihazın IP adreslerini aynı anda görür | Yok; gizlilik modu ve Tor ile azaltılacak |
| IP adresi | Gizlilik modunda karşı cihaz | Var: komut satırında `--relay-only`, uygulamalarda Ayarlar → "Adresimi karşı cihazdan gizle" |
| Alıcının diski | Kötü niyetli gönderen (yol dışına yazma, üzerine yazma) | Var |

## Saldırganlar

1. **Ağ gözlemcisi ya da aradaki kişi:** trafiği okur, değiştirir, keser
   (kafe Wi-Fi'ı, internet sağlayıcısı, devlet düzeyinde gözetim).
2. **Kötü niyetli ya da ele geçirilmiş sunucu:** rendezvous, STUN, aktarma
   sunucusu ya da Cloudflare'in kendisi.
3. **Kodu tahmin etmeye çalışan:** gönderen ile alıcı arasına girmek ister.
4. **Kötü niyetli karşı taraf:** kodu bilir, ama alıcının cihazına zarar
   vermek ya da gönderenin IP'sini öğrenmek ister.
5. **Kaydedip sonra çözen:** bugünkü trafiği saklar, ileride kuantum
   bilgisayarla çözmeye çalışır.
6. **Tedarik zinciri:** bir bağımlılığa ya da derleme sürecine kötü kod
   sokar.
7. **Oturumdaki başka bir program:** kişinin kendi hesabında çalışır
   (Linux'ta oturum veriyoluna erişir), uygulamanın penceresine istek
   gönderebilir.

## Güven sınırları

- İki cihaz birbirine yalnızca kodu bildikleri için güvenir; güven CPace ve
  Noise el sıkışmasından gelir, sunuculardan değil.
- Sunucular (rendezvous, aktarma, STUN) güvenilmez sayılır: yalnızca
  ulaştırırlar.
- Uygulamanın kendisi ve işletim sistemi güvenilir sayılır.
- Linux'ta oturum veriyolundaki (D-Bus) her süreç, açık pencereye
  `org.freedesktop.Application` üzerinden dosya, bağlantı ya da ekran
  verebilir (#178). Bu süreçler kişinin kendi hesabında çalışır; onun
  dosyalarına zaten erişebilirler.
- Flatpak paketi (`packaging/flatpak`, [0014](kararlar/0014-steam-deck.md))
  korumalı alanda çalışır. Erişebildikleri: ağ, `~/Downloads/TotalDrop`,
  dosya seçici portalıyla kişinin seçtiği dosyalar, bildirim servisi ve
  masaüstünün tema dosyaları (yalnızca okuma: `kdeglobals`, Omarchy, GTK
  ayarları ve temaları, Hyprland soketi). Sistem paketlerinde (`.deb`,
  `.rpm`, Arch) bu sınır yoktur.

## Tehditler ve karşılıkları

| # | Tehdit | Saldırgan | Karşılık | Kalan risk |
| --- | --- | --- | --- | --- |
| T1 | Trafiği okumak | 1, 2 | QUIC/TLS 1.3 içinde Noise XXpsk3; dosyalar ayrıca STREAM ile şifreli | Bilinen yok; bağımsız denetime kadar ([#40](https://github.com/totaldrop-app/totaldrop/issues/40)) doğrulanmadı |
| T2 | Araya girmek (MITM) | 1, 2 | Anahtarlar koddan CPace ile türer; yanlış kod el sıkışmayı bozar; dört doğrulama kelimesi | Kullanıcı kelimeleri karşılaştırmazsa, kodu bilen biri araya girebilir |
| T3 | Kodu tahmin etmek | 3 | CPace: deneme başına tek tahmin, çevrimdışı deneme yok; iki kelimede 2⁻²² olasılık; başarısız deneme iki tarafa "scary" kapanış olarak görünür; saldırı dalgasında yavaşlatma | Çok sayıda koda aynı anda saldırı; üç kelimelik kod (2⁻³³) seçenek olarak var |
| T4 | Kaydedip sonra çözmek | 5 | ML-KEM-768 hibrit adım; oturum anahtarları hem X25519'dan hem ML-KEM'den türer | Kimlik doğrulama v0'da klasik (Ed25519); ML-DSA-65 plan aşamasında ([#42](https://github.com/totaldrop-app/totaldrop/issues/42)) |
| T5 | Aktarımları birbirine bağlamak | 2 | Her aktarımda yeni, tek kullanımlık uç nokta anahtarı; sunucular kayıt tutmaz | Sunucu aynı anda IP ve zamanlamayı görür; yayımlanmış sürümlerde aktarma sunucusu da, doğrudan aktarımlarda bile, her aktarımda iki cihazın IP'sini görür; bir sonraki sürümden itibaren yalnızca doğrudan yol 1,2 saniye içinde açılmazsa, o cihazınkini ([#63](https://github.com/totaldrop-app/totaldrop/issues/63)) |
| T6 | IP adresini öğrenmek | 2, 4 | Gizlilik modu: adres bildirilmez, yalnızca aktarma (komut satırında `--relay-only`, uygulamalarda Ayarlar'da) | Varsayılan modda karşı taraf IP'yi görür; gizlilik modunda da buluşma ve aktarma sunucuları IP'yi görür (sınıf 2'ye karşı koruma değildir); gizlilik modundaki taraf, adresini bilen bir karşı tarafın doğrudan bağlantısını reddeder; Tor ve boyut doldurma yok ([#39](https://github.com/totaldrop-app/totaldrop/issues/39)) |
| T7 | Alıcının diskine zarar | 4 | Yol denetimi (NFC, `..`, ayırıcılar, Windows adları, bidi), yalnızca seçilen klasörün altına, var olanın üzerine asla yazmama, sembolik bağ izlememe, BLAKE3 doğrulamasından sonra yerine koyma | Aynı dosya sistemi yarış koşulları; denetim kapsamında |
| T8 | Bellek ya da disk tüketme | 4 | Sınırlar: 100.000 girdi, 16 MiB manifest, yol uzunlukları; reddedilen teklif | 16 MiB manifest çözülürken yaklaşık 1 GiB bellek; v0.1'de akış biçimine geçilecek |
| T9 | Sunucuyu kötüye kullanmak | 2, 3 | Rendezvous: kutu başına mesaj ve boyut sınırları, süreler; aktarma: aktarım başına iki anahtar, 32 GiB ve 6 saat | Rendezvous'ta istemci başına hız sınırı henüz yok. Aktarma sunucuları 32 GiB / 6 saati bağlantı başına sayar; bağlantı yeniden kurulunca sayaç sıfırlanır (TotalDrop istemcisi sınırı kendi bağlantıları boyunca toplam tutar, değiştirilmiş bir istemci tutmaz); Cloudflare aktarma sunucusu gönderme tamponunu göremez; her bağlantıya 4 MiB'ta bir yoklama koyar, son yanıtlanan yoklamadan sonra 32 MiB'ı (bir anahtarın bütün bağlantılarında 64 MiB'ı) geçen veriyi düşürür. Bu sınır aktarım başınadır: bir istemcinin açabileceği aktarım sayısı sınırlı değildir, birçok aktarım birlikte bir yalıtımın (isolate) belleğini hâlâ doldurabilir. Yoklamayı yanıtlamayan eski istemciler (0.3.0-beta.4 öncesi) bu sunucu üzerinden bağlantı başına en fazla 32 MiB alabilir; gönderirken de onay iletileri bu sınıra sayılır. Aynı anahtarla yeni bir giriş eski bağlantının yerini alır; bunu yalnızca anahtarın sahibi yapabilir. İki ortak anahtarı (aktarım adresini) bilen biri, giriş bekleyen iki yeri sürekli doldurarak bir tarafın yeniden bağlanmasını engelleyebilir; adresi yalnızca iki cihaz ve aktarma sunucusu bilir ([#190](https://github.com/totaldrop-app/totaldrop/issues/190)) |
| T10 | Aktarma sunucusuna başka bir anahtarla girmek | 2, 4 | Aktarım adresi iki anahtarı taşır; sunucu yalnızca bu iki anahtarı, imzalı bir meydan okumayla kabul eder | Bilinen yok; bağımsız denetime kadar doğrulanmadı |
| T11 | Tedarik zinciri | 6 | `cargo-deny` (lisans, güvenlik ilanları), Dependabot, kilitli bağımlılıklar, SHA'ya sabitli CI eylemleri, CI'da derleme ve derleme kaynağı kanıtı, CodeQL kod taraması, OpenSSF Scorecard | Sabitlenen sürümün kendisi kötü niyetliyse korumaz; her güncelleme PR olarak gelir ve incelenir. Arch kapsayıcısı özete, Flutter sürüme (3.47.8) sabit; Node (`24`, `lts/*`) ve fuzz için Rust nightly hâlâ değişebilen etiketle çağrılıyor |
| T12 | Sahte uygulama ya da paket | 6 | Yalnızca Google Play, GitHub sürümleri ve totaldrop.app; SHA-256 sağlama toplamları; Linux paketlerinde derleme kaynağı kanıtı | Play paketini Google yeniden imzalar, kullanıcı kanıtı doğrulayamaz; Windows ve macOS imzası bağışları bekliyor |
| T13 | Açık pencereye istenmeyen dosya ya da kod vermek (Linux) | 7 | Hiçbir şey kendiliğinden gönderilmez ya da alınmaz: gönderme "Kod oluştur", alma teklifin kabulünü ister. Bir çağrı en çok 1.024 öğe ve 1 MiB taşıyabilir; yalnızca yerel `file://` yolları ve bilinen ekranlar alınır; etkinleştirme belirteci denetlenir; aktarım sürerken gelen istek beklemeye alınır | Gönderilecekler listesine eklenen dosya ayrıca işaretlenmez; bu süreç kişinin dosyalarını zaten okuyabilir |
| T14 | Aktarımı aktarma sunucusuna zorlamak ya da orada tutmamak | 1, 2 | Doğrudan yol 8 saniye susunca bağlantı aktarma sunucusuna geçer, doğrudan adresten yeniden paket gelince döner ([#185](https://github.com/totaldrop-app/totaldrop/issues/185)); içerik iki yolda da aynı uçtan uca şifreyle korunur | Yolun üzerindeki biri doğrudan paketleri düşürerek aktarımı aktarma sunucusuna taşıyabilir; bu da aktarma sunucusunun cihazların IP adresini görmesi demektir (bağlantı kurulurken de aynı yolla yapılabiliyordu). Karşı tarafın IP ve bağlantı noktasını bilen biri, o adresten sahte paket göndererek geçişi geciktirebilir ya da bağlantıyı ölü doğrudan yola geri döndürebilir: paketler QUIC şifresi çözülmeden önce sayılıyor. Bu yalnızca hizmet kesintisidir, içeriğe erişim değildir |
| T15 | Flatpak'te gereğinden geniş izin ya da sahte Flatpak | 4, 6 | İzinler en az olanla sınırlı (yukarıdaki güven sınırı); gönderilecek dosyalar portaldan gelir, ev klasörüne genel erişim yok; tema dosyalarına yalnızca okuma; manifest ve izinleri her PR'da `flatpak-builder-lint` denetler; bizim CI'da derlenen Flatpak paketi diğer Linux paketleriyle aynı derleme kaynağı kanıtını alır | Hyprland soketine erişen uygulama ona komut da gönderebilir (TotalDrop yalnızca köşe yuvarlaklığını sorar); bildirim servisine doğrudan erişim portaldan geniştir. Flatpak'te canlı KDE tema değişikliği ve sistem GTK temaları görünmez (#299); bu bir uyum eksiğidir, güvenlik değil. Flathub'ın derlediği paket Flathub'ın anahtarıyla imzalanır, bizim kanıtımızı taşımaz: kullanıcı Flathub'a güvenir. Uygulama kimliği `app.totaldrop.TotalDrop` Flathub'da projeye aittir ([0012](kararlar/0012-ad-ve-isaret.md)) |
| T16 | Paylaşım menüsüyle TotalDrop'a kendi gizli dosyalarını gönderttirmek (Android) | 7 | Herhangi bir uygulama paylaşım isteğine her adresi koyabilir; yalnızca `content://` adresleri okunur ve adresin sağlayıcısı sistemin çözdüğü biçimde aranır: TotalDrop'un kendi paketine ya da kendi kullanıcı kimliğine (UID) ait bir sağlayıcı (cihaz anahtarının bulunduğu özel klasörü gösterebilecek olanlar), hiçbir uygulamanın sunmadığı bir adres, `file://` adresleri ve `0@sağlayıcı` gibi kullanıcı önekli adresler reddedilir; okuma izni kalıcı yapılmaz; kopyanın adı klasör ayırıcısı, gizli karakter ve `..` içeremez; hiçbir şey kendiliğinden gönderilmez, "Kod oluştur" gerekir; kopyalar yedeğe girmeyen uygulama klasöründe tutulur, seçim temizlenince ya da paylaşım alınmayınca silinir; aktarım sürerken gelen paylaşım alınmaz ([#53](https://github.com/totaldrop-app/totaldrop/issues/53)) | Paylaşan uygulamanın verdiği dosya adı ve içerik gösterildiği gibidir; kişi listeyi görmeden "Kod oluştur"a basarsa istemediği bir dosyayı gönderebilir; başka bir uygulamanın sağlayıcısı kendi erişebildiği her şeyi sunabilir (bu, o uygulamanın verisidir, TotalDrop'unki değil) |

## Güvence gerekçesi

Bu bölüm, [SECURITY.md](../SECURITY.md)'deki güvenlik gereksinimlerinin
neden karşılandığını savunur. Bağımsız bir denetimin yerini tutmaz; denetçi
için başlangıç noktasıdır.

**Tehdit modeli ve güven sınırları** yukarıda: korunan varlıklar,
saldırganlar, güven sınırları ve T1–T13. Mimari ve sınırların nerede
durduğu [mimari.md](mimari.md) içinde.

**Güvenli tasarım ilkeleri nasıl uygulanıyor:**

- *En az yetki ve en az bilgi:* sunucular yalnızca ulaştırır; içeriği,
  kodu ve anahtarları görmez, kayıt tutmaz. Hesap yoktur. CI iş akışları
  en az yetkiyle çalışır, eylemler SHA'ya sabitlidir.
- *Güven kullanıcıdan gelir, sunucudan değil:* iki cihaz yalnızca kodu
  bildikleri için birbirine güvenir (CPace), araya girme dört doğrulama
  kelimesiyle görünür olur (T2).
- *Hata durumunda güvenli kapanma (fail closed):* yanlış kod, bozuk mesaj, sınır aşımı
  ya da doğrulanmayan parça bağlantıyı kapatır; dosya ancak BLAKE3
  doğrulamasından sonra yerine konur (T7).
- *Derinlemesine savunma:* QUIC/TLS 1.3 içinde Noise, onun içinde dosya
  başına STREAM şifrelemesi (T1); klasik ve kuantum sonrası anahtarların
  ikisinden birden türeyen oturum anahtarları (T4).
- *Basitlik ve tek kaynak:* protokol ve kriptografi tek çekirdekte durur;
  uygulamalar kriptografi yapmaz. Yeni kriptografik yapı icat edilmez:
  yayımlanmış yapılar (CPace, STREAM) bakımlı kütüphanelerin ilkelleri
  üzerine kurulur ve taslakların test vektörleriyle sınanır.
- *Girdiye güvenmeme:* istemcide karşı cihazdan ve sunuculardan gelen her
  kablo mesajı `totaldrop-proto`'da, izin listesiyle ve sınırlarla
  ayrıştırılır (T7, T8); bu ayrıştırıcıların fuzz hedefi vardır. Linux'ta
  oturum veriyolundan gelen istekler `linux/` içinde sınırlanır (T13).

**Yaygın açık sınıflarına karşı:**

| Sınıf | Karşılık |
| --- | --- |
| Bellek güvenliği (taşma, serbest bırakılmış belleği kullanma) | Rust; `crates/` ve `server/` içinde `unsafe_code = "forbid"`. İstisnalar: Linux uygulamasındaki Qt köprüsü bildirimleri ve Qt başlamadan önce ortam değişkeni ayarlayan tek satır, köprünün C++ başlıkları ve Android köprüsünün (`app/rust`) üretilmiş kodu |
| Yol geçişi, sembolik bağ, üzerine yazma | T7: yol denetimi, yalnızca alma klasörünün altı, var olanın üzerine yazmama |
| Kaynak tüketme | T8, T9: girdi, manifest, mesaj, süre ve bayt sınırları |
| Tekrar oynatma ve sürüm düşürme | Her aktarımda yeni, tek kullanımlık anahtarlar; protokolün tek bir sürümü ve şifre takımı var (TD0; `totaldrop-core/src/pairing.rs` içinde `VERSIONS = [0]`), düşürülecek eski bir sürüm yok |
| Zayıf ya da yanlış kullanılan kriptografi | Protokol katmanında (TD0) yalnızca CPace (ristretto255, SHA-512), Noise XXpsk3 (X25519, ChaCha20-Poly1305, BLAKE2b), ML-KEM-768, Ed25519 ve BLAKE3. Taşıma katmanında QUIC ve WebSocket için TLS 1.3 (rustls, `ring` sağlayıcısı; AES-GCM ve ChaCha20-Poly1305, X25519 ve P-256/P-384). SHA-1, MD5, CBC yok; rastgelelik işletim sisteminden |
| Web açıkları (XSS, tıklama korsanlığı) | Sitede dışarıdan betik yok, sıkı CSP, `frame-ancestors 'none'`, HSTS (`site/public/_headers`) |
| Tedarik zinciri | T11, T12: kilitli bağımlılıklar, `cargo-deny`, Dependabot, CodeQL, CI'da derleme ve imzalı derleme kanıtı |
| Gizli bilgi sızması | Gizli değerler (Play yükleme anahtarı ve parolası, Cloudflare API belirteci) yalnızca korumalı CI ortamlarında ve bakımcıda; Linux derleme kanıtı anahtarsız (Sigstore); gizli bilgi taraması ve push koruması açık |

**Kanıt:** güvenlik kurallarının testleri depoda durur ve her PR'da
çalışır; ayrıştırıcıların fuzz hedefleri her hafta çalışır. CodeQL her PR'ı
tarar; OpenSSF Scorecard sonucu herkese açıktır (README'deki rozet). Kalan
riskler yukarıdaki tabloda; issue'su olanlar bağlantılıdır, hepsi 1.0
öncesi bağımsız denetimde ayrıca incelenir.

## Hesap (tasarımda, kod yok)

İsteğe bağlı hesap henüz yalnızca tasarımdır ([hesap.md](hesap.md),
[karar 0013](kararlar/0013-hayalet-halka-hesap.md)). Kod yazılmadan önce bu
iddialar Tamarin modelinde lemma, kodda test olur.

Ek saldırganlar:

8. **Çalınan ya da çıkarılmış cihaz:** bir zamanlar halkanın üyesiydi.
9. **Kurtarma ifadesini ele geçiren.**
10. **Kötü niyetli bilinen kişi:** sizinle eşleşmiş, cihazlarınızı ya da
    öteki kişilerinizi öğrenmek ister.

| # | Tehdit | Saldırgan | Karşılık | Kalan risk |
| --- | --- | --- | --- | --- |
| H1 | Halkaya sahte cihaz eklemek | 2, 8 | Üyeliği yalnızca mevcut cihazın yönetim anahtarı (`AK`) ya da gecikmeli kurtarma anahtarı değiştirir; kişiler her başta işlem türünü, imzayı, sıkı mod ilkesini ve önceki köke karşı Merkle kanıtını denetler; `döndür`de kök değişmez (P1) | Kurtarma yolu: bkz. H4; çıkarılmadan önce kilidi açık ele geçirilen cihaz, `AK` ile cihaz ekleyebilir |
| H2 | Kişiye eski ya da farklı cihaz listesi göstermek (geri alma, çatal, dondurma) | 2 + 8 | Bağımsız tanıklı anonim şeffaflık günlüğü; anahtar başına her geçerli değer saklanır, birden çoğu çataldır; `çıkar`, `döndür`ün önüne geçer; her gönderimden önce en çok 1 saatlik tanıklı kök ve sonraki dönemin yokluk kanıtı (P2, P3) | Yeter sayıda tanık sunucuyla iş birliği yaparsa korumaz; anahtarı bilen biri çöp değerlerle bir dönemi geçici olarak kilitleyebilir |
| H3 | Çıkarılan cihazın yeni dosyaları alması ya da halka adına göndermesi | 2 + 8 | Çıkarmada halka sırrı yenilenir; kişi anahtarları karşı halkanın güncel anahtarına HPKE ile yenilenir; tanıkların dahil etme zorlaması (sansüre karşı); gönderirken ve alırken tazelik denetimi (P3) | Çıkarmaya ve sonraki 2 saate kadar geçen süre; eski başı sabitlemiş kişiler geç bir çatalla kapalı bırakılabilir (hizmet kaybı, `RK` ya da yeniden eşleşme çözer); yeter sayıda tanık iş birliği yaparsa korumaz; tanıklı kök gecikirse gönderim durur (hizmet kaybı) |
| H4 | İfadeyle sessizce cihaz eklemek ya da kişileri öğrenmek | 9, 2 + 9, 8 + 9 | Bekleme (2+ cihazda 72 saat, tek cihazda 7 gün) tanık zamanıyla; kurtarma çatal açmaz, herhangi bir cihazın `AK` reddi her zaman sayılır; `çatal_çöz` sonrası da kişiler yeniden doğrular; aylık kurtarma kaydında yalnızca son baş var, kişi defteri yok; kişilerle doğrulama kelimelerini yeniden karşılaştırma, otomatik kabul yok (P7) | Telefon (ya da çıkarılmış eski bir cihaz) ve ifade birlikte ele geçirilirse korumaz; kişiler yeniden doğrulamada fark edebilir; sunucu ve tanıklar reddi sansürlerse bekleme yalnızca gecikmedir, koruma kişilerin yeniden doğrulamasıdır |
| H5 | Kişi listesini zehirlemek (bir kişinin anahtarını değiştirmek) | 8 | Kişi defteri imzalı ve yalnızca eklenebilir; değişiklik eşleşme dökümüne bağlı; çıkarılan cihazın dokunduğu kişiler yeniden doğrulanır | Çıkarılmadan önceki değişiklikler, yeniden doğrulamaya kadar |
| H6 | Hesapları, cihazları ya da kişi ilişkilerini çıkarmak | 2 | Sunucuda hesap tablosu yok; günlük dönen kutu adları; yön başına kutular; günlük anahtarları ve değerleri yalnızca başı bilenlerce hesaplanır ve açılır (P5) | IP ve zamanlama: aynı NAT'tan bağlanan cihazlar ve kutular ilişkilendirilebilir; tanıklar bir halkanın kayıt bildirimlerini IP ve zamanla ilişkilendirebilir; OHTTP/Tor sonraki aşamada |
| H7 | Cihaz sayınızı ya da halkanızın değişimlerini öğrenmek | 10, 8 | Halka başı 16'ya dolgulu, tuzlu Merkle ağacı; dönem başına tek cihaz döndürür (P6) | Kişi zamanla el sıkışmalarda gördüğü cihazları sayabilir; bir baş bilen (engellenmiş kişi, çıkarılmış cihaz da) ileriki başların zamanlarını ve işlem türünü izleyebilir |
| H8 | Bir kişiyi adla taklit etmek | 10 | Ad yerel; önerilen ad otomatik dolmaz; benzer ad uyarısı; 88 bitlik işaret | Kullanıcı uyarıyı ve işareti yok sayarsa |
| H9 | Kutu açma ile kaynak tüketmek | 1, 10 | ARC jetonu, iş kanıtı, IP sınırı, kutu boyutu ve ömrü | Botnet; operatör NAT'ı arkasındaki kullanıcılar sınıra takılabilir |
| H10 | Her istemciye ayrı ARC anahtarı vererek etiketlemek | 2 | ARC anahtarları şeffaflık günlüğünde yayımlanır (anahtar tutarlılığı) | Kimlik bilgisi alma anında IP görülür |
| H11 | Yeni cihaz eklerken araya girmek | 1 | QR ya da kod (CPace) zorunlu; yerel ağ yalnızca bulma için; kelime seçme | Kullanıcı yanlış kelimeyi seçerse |

## Kapsam dışı

- Ele geçirilmiş bir cihaz: kötü amaçlı yazılım dosyaları gönderilmeden
  önce ya da alındıktan sonra okuyabilir.
- Gönderilen dosyanın içeriği: TotalDrop zararlı bir dosyayı tanımaz;
  alıcı açmadan önce karar verir.
- Trafiğin varlığı ve hacmi: bir gözlemci bir aktarımın olduğunu ve
  büyüklüğünü görebilir (gizlilik modundaki boyut gizleme ileride).

## Açık işler

Bu belgedeki "kalan risk" sütunundaki her madde bir issue ile izlenir ve
1.0 öncesi bağımsız denetimde ayrıca incelenir.
