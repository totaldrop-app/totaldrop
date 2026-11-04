# Hesap sistemi: Hayalet halka (tasarım, önerilen)

- **Durum:** Önerilen, 4 Kasım 2026 ([karar 0013](kararlar/0013-hayalet-halka-hesap.md), #243).
  [Karar 0004](kararlar/0004-mullvad-tarzi-hesap.md)'ün (16 haneli numara)
  yerini alır.
- **Kod:** yok. Bu belge davranışı bağlar. Bayt düzeyindeki biçimler, kod
  yazılmadan önce [protokol notlarına](protocol-v0.1-notes.md) eklenir.
- **İnceleme:** tasarım, kod yazılmadan önce biçimsel modelle (Tamarin) ve
  bağımsız bir insan kripto incelemesiyle sınanacak (bkz. "Doğrulama").

TotalDrop'ta hesap isteğe bağlıdır. Kodla gönderme ve alma hesapsız
çalışmaya devam eder. Hesap iki şey sağlar:

- **Benim cihazlarım:** cihazlarınız birbirine kod girmeden gönderir.
- **Bilinen kişiler:** bir kez eşleştiğiniz kişilere kod girmeden
  gönderirsiniz.

## İlkeler

1. **Sunucuda hesap yok.** Sunucu hesap tablosu, cihaz listesi ya da kişi
   listesi tutmaz. Hesap, cihazlarınızın aralarında tuttuğu imzalı bir
   üyelik zinciridir.
2. **Sunucu güvenilmez.** Her güvenlik iddiası, sunucu kötü niyetliyken de
   geçerli olmalıdır ([tehdit modeli](tehdit-modeli.md)).
3. **Yeni şifreleme ilkeli yok.** Yalnızca standart parçalar kullanılır:
   Ed25519, ML-DSA-65, X-Wing (X25519 + ML-KEM-768), HPKE, HKDF-SHA-256,
   CPace, Noise (mevcut el sıkışma), Privacy Pass ARC, Shamir. Bunların
   bileşimi yine de özel bir grup üyelik protokolüdür; risk buna göre
   yönetilir.
4. **Kapalı kalarak başarısız ol.** Çelişki görülürse gönderim durur ve
   kullanıcı uyarılır; sessizce bir tarafa güvenilmez.
5. **Hiçbir hassas işlem tek bir sırrın ya da tek bir cihazın ele
   geçirilmesiyle yapılamaz.**

## Kullanıcının gördüğü

### Kimlik yazmak yok

Hiçbir cihazda hesap numarası, kullanıcı adı ya da parola yazılmaz.

- İlk cihaz halkayı kendisi kurar.
- Yeni bir cihaz eklenirken yeni cihaz bir QR gösterir. Halkadaki bir
  cihaz QR'ı okur ve eklemeyi başlatır.
- İki ekranda aynı doğrulama kelimeleri çıkar. Kullanıcı, birkaç seçenek
  arasından doğru kelimeleri seçer.
- Yerel ağ yalnızca cihazı **bulmak** için kullanılır. Ekleme her zaman QR
  ya da kodla (CPace) yapılır.

### Tek liste

Halkadaki her cihaz aynı cihaz listesini ve aynı kişi listesini görür.
Telefonda eklenen kişi PC'de de hazırdır. Listeler halka içinde uçtan uca
şifreli eşitlenir.

### Adları kullanıcı verir

Kimlikler arka planda rastgele anahtarlardır. Ekranda kullanıcının verdiği
ad görünür: "Annem", "İş PC'si". Bu yaklaşımın adı "petname"dir (Miller ve
Stiegler); küresel ve benzersiz bir ad yerine güvenli ve yerel bir ad
kullanılır.

- Adlar yalnızca halkanın içinde, şifreli eşitlenir. Karşı tarafa ve
  sunucuya gitmez.
- Karşı tarafın önerdiği ad ayrı, soluk bir etiket olarak gösterilir. Ad
  kutusuna otomatik doldurulmaz; yoksa saldırgan kendine "Annem" diyebilir.
- Aynı ya da benzer bir ad verilirse uyarı çıkar. Benzerlik denetimi
  homoglifleri ve yön karakterlerini de kapsar.
- Kişinin anahtarından türeyen **6 kelimelik (66 bit)** bir işaret
  istendiğinde gösterilir. Renk yalnızca süstür, güvenlik iddiası taşımaz.

### Güven katmanları

| Katman | Nasıl eklenir | Gönderince ne olur |
| --- | --- | --- |
| **Benim cihazlarım** | QR ya da kod + doğrulama kelimeleri, bir kez | Soru sormadan alınır (ayarla kapatılabilir) |
| **Bilinen kişiler** | Bir kez üç kelimelik kod + doğrulama kelimeleri | Kodsuz gönderilir. Alıcıya "Annem 3 dosya gönderiyor, kabul?" sorulur. Kişi başına "her zaman kabul et" seçilebilir |
| **Yakındakiler** | Her seferinde kodla (bugünkü sistem) | Her seferinde onay istenir |

- Sistem "tanıdık cihaz" üzerine değil, "tanıdık kişi" üzerine kurulur.
  Bir kişinin hangi cihazından gönderdiği önemli değildir. Kişi kendi
  halkasına cihaz ekleyip çıkardığında karşı tarafın bir şey yapması
  gerekmez.
- Kodla yapılan başarılı bir aktarımdan sonra "Bu kişiyi kaydet" düğmesi
  çıkar. Bu düğme yalnızca iki tarafın da hesabı varsa görünür.
- Engellenen kişinin kayıtları silinir ve iki taraf arasındaki gizli
  anahtar atılır. Engellenen tarafa bildirim gitmez; gönderdiği teklifler
  cevapsız kalır.

### Çok katmanlı onay

| İşlem | Varsayılan | Sıkı mod (ayar) |
| --- | --- | --- |
| Cihaz ekleme | Mevcut cihazda yönetim anahtarı (biyometri ya da PIN) + doğrulama kelimeleri. Diğer cihazlara anında bildirim gider; 24 saat içinde tek dokunuşla geri alınabilir | Halkada 2 ya da daha çok cihaz varsa iki cihazın onayı gerekir. İkinci cihaz çevrimdışıysa 72 saat beklenir |
| Cihaz çıkarma | Yönetim anahtarı | Aynı |
| Kurtarma ile ekleme | 24 kelime + bekleme + kişilerle yeniden doğrulama | 24 kelimeye ek olarak YubiKey ya da 3 kişiden 2'sinin Shamir payı |
| Kurtarma ifadesini değiştirme | Yönetim anahtarı + bekleme | İkinci cihazın onayı da gerekir |
| Kişiye "her zaman kabul et" | Yönetim anahtarı | Aynı |

Sunucu tarafında ikinci etken (SMS, TOTP) yoktur ve olmayacaktır, çünkü
sunucuda hesap yoktur. Halkada 3 ya da daha çok cihaz varsa sıkı modu
açmak önerilir.

## Anahtarlar

| Anahtar | Sahibi | Kullanım |
| --- | --- | --- |
| `IK_i`: Ed25519 + ML-DSA-65 | Her cihaz | Dönem yenileme kayıtlarını ve el sıkışmadaki tek kullanımlık uç anahtarını imzalar |
| `AK_i`: Ed25519, kullanıcı doğrulamasına bağlı | Her cihaz, OS anahtar deposunda | Yönetim anahtarı: cihaz ekleme ve çıkarma, kişi güvenlik alanları, kurtarma ifadesi değişimi |
| `KK_i`: X-Wing | Her cihaz, dönemsel yenilenir | Yeni halka sırrını almak |
| `RK`: Ed25519 + ML-DSA-65 | 24 kelimeden (BIP-39, 256 bit) türer, hiçbir cihazda saklanmaz | Yalnızca gecikmeli cihaz ekleme ve çatal çözme |
| `r_e`: 32 bayt | Dönem e'deki bütün cihazlar | Posta kutusu adları, halka KEM anahtarı |
| `k_AB`: 32 bayt | İki halka (siz ve bir kişi) | Kişi posta kutuları; her cihaz çıkarmada yenilenir |
| `k_günlük`: 32 bayt | Halka ve kişileri | Şeffaflık günlüğündeki etiketler; cihaz çıkarmada yenilenir |

Her türetme `HKDF-SHA-256` ile yapılır. Etiketler uzunluk önekli ve
benzersizdir (alan ayrımı). `gün` gibi değerler sabit genişlikte kodlanır.
Her kayıtta bir algoritma sürümü alanı bulunur.

Karma imzada iki yarının **ikisi de** doğrulanmalıdır. İki yarı alan
ayrımlı mesajlar imzalar ve her biri öbürünün açık anahtarına bağlanır.
Doğrulama yolunun bir yarıyı atlayamadığı ayrı testlerle sınanır.

Anahtar deposu gerçeği: ML-DSA anahtarları Android Keystore'da ve Linux
Secret Service'te dışa aktarılamaz biçimde tutulamaz. Ed25519 yalnızca
yeni Android sürümlerinde donanımdadır. Bu yüzden ML-DSA yarısı, donanım
anahtarıyla sarılı bir blob olarak saklanır. `IK` her aktarımda imza
attığı için biyometriye bağlanamaz. Bu nedenle yönetim işlemleri ayrı bir
`AK` ile yapılır.

## Üyelik zinciri

Bir kayıt şu alanları taşır: sürüm, dönem `e`, önceki kaydın kanonik
kodlamasının özeti, işlem, yük, imzalayan ve imza. İşlemler şunlardır:

| İşlem | İmzalayan | Not |
| --- | --- | --- |
| `oluştur` | İlk cihazın `AK`'si | `RK`'nin açık anahtarını içerir |
| `ekle(IK, AK, KK)` | Mevcut bir cihazın `AK`'si | |
| `çıkar(IK)` | Mevcut bir cihazın `AK`'si | Çıkarma acil işlemdir, hemen yapılır |
| `döndür` | Mevcut bir cihazın `IK`'si | Yalnızca sırrı yeniler |
| `kurtarma_ekle(IK, AK, KK)` | `RK` | Yalnızca ekler, bekler |
| `çatal_çöz` | `RK` | Bekler, reddedilebilir |

Kişi listesi ve adlar bu zincirde değil, halka içindeki ayrı bir **kişi
defterinde** tutulur (bkz. "Kişilere karşı halka"); kişi değişiklikleri
dönem başlatmaz.

- **Her zincir kaydı yeni bir dönem başlatır** ve yeni bir `r_{e+1}` üretir. Bu
  sır, kalan her cihazın `KK` anahtarına HPKE ile ayrı ayrı şifrelenir.
  HPKE `info` alanı şunları bağlar: halka kimliği, dönem ve alıcının `IK`'si.
- **Kaydın imzası her şifreli metni kapsar.** Her alıcı, `r_{e+1}` altında
  bir anahtar onayı (MAC) doğrular. Bozuk bir yük alan cihaz sessizce
  düşmez; hata bildirir.
- **Zincir sunucuya açık değildir.** Halka posta kutusunda uçtan uca
  şifreli dolaşır.
- **Zamanlama:**
  - Halka başlarının süresi en çok 7 gündür.
  - Haftalık `döndür`, rastgele bir sapmayla yapılır.
  - Acil olmayan değişiklikler (ekleme) bir sonraki planlı zamana
    kaydırılır, böylece kişiler gerçek değişikliği ayırt edemez.
  - Bir başın süresini uzatmak her zaman yeni bir dönemle yapılır. Eski bir
    baş yeniden imzalanarak uzatılmaz.

### Çatal kuralı

Aynı dönemin iki farklı devamı görülebilir. Örneğin iki çevrimdışı cihaz
aynı anda döndürmüş olabilir, ya da çalınan bir cihaz kendi devamını
yazmış olabilir. Böyle bir durumda:

1. Üyelikleri aynı olan eşzamanlı `döndür` kayıtları belirlenimci olarak
   birleşir: en küçük özete sahip olan kazanır. Anahtar onayı, bunun bir
   cihazı düşürmek için öğütülmesini engeller.
2. Başka her çatalda cihazlar ve kişiler kapalı kalır. Gönderim durur ve
   bir uyarı gösterilir.
3. Bir gözlemci, kendi sabitlediği başın altında kalan çatalları yok sayar.
4. Tartışmalı dalda bulunan cihazların reddi sayılmaz.
5. Çatalı yalnızca `RK` imzalı bir `çatal_çöz` kaydı çözer. Bu kayıt da
   bekleme süresine tabidir.
6. Çatal çözülemezse (hırsızın elinde hem bir cihaz hem ifade varsa)
   kişiler sizi yeniden eşleştirir. En kötü sonuç hizmet kaybıdır, sessiz
   ele geçirme değildir.

## Posta kutuları

Sunucu yalnızca kısa ömürlü posta kutularını bilir. Kutular Durable
Object'tir; 16 baytlık, rastgele görünen bir adla açılır ve boyut ile ileti
sınırları vardır. Kutuya yazmak bir ARC jetonu ister. Sunucuda başka bir
tablo yoktur.

| Kutu | Ad | Ömür |
| --- | --- | --- |
| Halka | `HKDF(r_e, "kutu halka" ‖ gün)` | 48 saat |
| Kişi, yön başına | `HKDF(k_AB, "kutu A'ya" ‖ gün)` | 48 saat |
| Yetişme | Zincirdeki, cihaza özel gizli bir değerden, dönem başına | 30 gün |

- Cihazlar dünün, bugünün ve yarının kutusunu dinler. Bu, saat farkı payı
  bırakır. Zaman olarak cihazın kalıcı tekdüze saati kullanılır; ağdan
  gelen saat (NITZ gibi) sahtelenebileceği için kullanılmaz.
- İki günden uzun çevrimdışı kalan bir cihaz yetişme kutusundan ve öteki
  cihazlarla doğrudan zincir eşitlemesinden yetişir. Uzun süre çevrimdışı
  kalan bir kişi ise "güncel başı iste" akışını kullanır.
- Her kutu ayrı bir bağlantıyla ve rastgele zamanlarla yoklanır. Aynı
  NAT IP'sinden gelen bağlantılar yine de ilişkilendirilebilir (bkz.
  "Kalan riskler").
- Zincir kayıtları (~23 KB) mevcut kutu gövdesi sınırını (16 KiB) aşar. Bu
  yüzden hesap kutularında parçalı gönderim kullanılır.

## Kişilere karşı halka

- **Halka başı:** dönem e'deki cihazlar 16 yapraklı bir Merkle ağacına
  konur. Her yaprak `H("yaprak", e, nonce_i, IK_i, AK_i)` biçimindedir,
  dönem başına tuzlanır ve sırası rastgeledir. Boş yapraklar rastgele
  dolgudur. Ağacın kökü `C_e`'dir.
- **Kişiye gönderilen baş:** `{e, C_e, önceki_baş_özeti, son_kullanma}`.
  Değişikliği yapan cihaz bu başı imzalar ve kendi Merkle kanıtını ekler.
  Kanıt 4 adımdır, 128 bayt tutar.
- **Sabitleme:** kişi ilk eşleşmede halka başını sabitler. Bu ilk güven
  (TOFU), doğrulama kelimeleriyle korunur. Kişi daha sonra yalnızca bir
  önceki başa bağlanan ve şeffaflık günlüğünde bulunan devamları kabul
  eder. Süresi dolmuş bir başa gönderim yapılmaz.
- **Cihaz sayısı:** baş, cihaz sayısını (1-16) göstermez. Ancak kişi zamanla
  el sıkışmalarda gördüğü farklı cihazları sayabilir. Bu, kabul edilen bir
  sınırdır.
- **Kişi defteri:** bir kişinin güvenlik alanları (`k_AB`, sabitlenmiş
  baş) ve verilen ad, halka içinde bir cihazın `AK`'siyle imzalanmış,
  yalnızca eklenebilen kayıtlardır. Güvenlik alanlarındaki bir değişiklik
  eşleşme dökümüne (transcript) bağlanır. Sıralama, saat sapması sınırlı
  bir karma mantıksal saatle (HLC) yapılır. Defter, halka posta kutusunda
  `r_e` altında şifreli eşitlenir.
- **Çıkarma sonrası:** bir cihaz çıkarılınca, onun eklediği ya da
  değiştirdiği her kişi "yeniden doğrula" diye işaretlenir.

## Anonim şeffaflık günlüğü (ilk sürümde)

Sunucu ile eski bir yetkisi olan birinin (çıkarılmış bir cihaz ya da
çalınmış ifade) iş birliği yaptığını düşünelim. Kişiyle halka arasındaki
tek kanal bu sunucuysa, ikisi yalıtılmış bir kişiye kendi içinde tutarlı
ama sahte bir geçmiş gösterebilir. Bu, çatal tutarlılığının bilinen
sınırıdır (SUNDR). Bunu aşmak için ya tek görünümlü bir günlük ya da bant
dışı doğrulama gerekir. TotalDrop ilk sürümde ikisini de kullanır.

- Günlük, doğrulanabilir bir eşlemedir (verifiable map). Bir anahtar bir
  kez yazılır.
  - Anahtar: `H(k_günlük, e)`
  - Değer: dönem e'nin başının özeti
- Günlük kimseyi tanıtmaz. Günlüğü okuyan biri halkaları tanıyamaz.
  `k_günlük` yalnızca halkada ve kişilerde bulunur ve cihaz çıkarıldığında
  yenilenir.
- **Tanıklar:** bağımsız tanıklar günlüğün köklerini eş-imzalar. Böylece
  sunucu iki kişiye iki farklı günlük gösteremez.
- **Halkanın denetimi:** halka, kendi dönemi için günlükte başka bir değer
  bulursa durumu hemen anlar ve kullanıcıyı uyarır. Kendi kaydı belirli
  bir süre içinde günlüğe girmezse de uyarır, çünkü bu sansür olabilir.
- **Kişinin denetimi:** kişi her yeni başı günlükteki değerle karşılaştırır.
  Günlükte daha yeni bir dönem varsa ama baş kendisine ulaşmadıysa
  "güncel başı iste" akışını başlatır. Bu, sunucunun bir kişiye yeni başı
  göstermemesini (dondurma saldırısını) açığa çıkarır.
- **ARC anahtarları:** ARC düzenleyici anahtarları da aynı günlükte
  yayımlanır. Böylece sunucu her istemciye ayrı bir anahtar vererek
  istemcileri etiketleyemez (anahtar tutarlılığı).
- **Adaylar (doğrulanacak):**
  - Doğrulanabilir eşleme için Meta'nın açık kaynak `akd` kütüphanesi
    (Parakeet, NDSS 2023).
  - Tanıklar için Sigsum ya da transparency.dev tanık ağı.

## Kurtarma

- **İsteğe bağlıdır.** Tek cihazlı halkada belirgin bir uyarıyla önerilir.
- **İfade:** 24 kelime (BIP-39) `RK`'yi türetir.
- **Paylaştırma:** isteğe bağlı olarak ifade Shamir ile t-of-n paylara
  bölünebilir. Örneğin 3 bilinen kişiden 2'si, ya da bir kişi + YubiKey
  (WebAuthn PRF). Paylar `RK`'nin açık anahtarına karşı denetlenir; bozuk
  bir pay yakalanır. Senkronize passkey kullanılmaz, çünkü kullanıcıyı
  Apple ya da Google hesabına bağlar.
- **Kayıt:** `kurtarma_ekle` kaydı günlükteki en son başı ebeveyn olarak
  adlandırmak zorundadır.
- **Bekleme:** halkada 2 ya da daha çok cihaz varsa 72 saat, tek cihaz
  varsa 7 gün.
  - Bekleme, cihazların ve kişilerin her birinin **kendi tekdüze saatine**
    göre, kaydı ilk gördükleri andan başlar.
  - Bekleme sırasında herhangi bir cihaz tek dokunuşla reddedebilir. Red,
    `RK`'yi geçersiz kılar. Yeni bir ifade, kendi beklemesiyle verilir.
  - Bekleme içinde cihaz imzalı daha yeni bir baş görülürse kurtarma
    kaydı geçersiz olur.
  - Bekleme süresince zincir doğrusal kalır. Bu sırada yapılan dürüst bir
    işlem kurtarmayı geçersiz kılar.
- **Kişiler:** bekleme süresince kişiler "bu kişi hesabını kurtarıyor"
  uyarısını görür. Kurtarılan halka her kişiyle doğrulama kelimelerini
  **yeniden karşılaştırır**. Süre dolunca otomatik kabul yoktur.
- **Kalan risk:** telefon ve ifade birlikte çalınırsa bu katman korumaz.

## Gönderim

1. Gönderen, kişinin kutusuna bir **teklif** bırakır.
   - Teklif, halkanın X-Wing anahtarına (`r_e`'den türer) HPKE ile
     şifrelenir.
   - Teklif şunları imzalar: alıcı halkanın kimliği, HPKE çıktısı ve
     gönderenin bildiği alıcı başı (baş yankısı).
   - Teklifte yalnızca buluşma bilgisi bulunur, dosya anahtarı bulunmaz.
2. Alıcı halkadaki çevrimiçi bir cihaz teklifi açar. Mevcut Noise + ML-KEM
   el sıkışmasını başlatır ([protokol notları](protocol-v0.1-notes.md),
   bölüm 6).
3. Her iki taraf da kendi halkasının o anki üyesi olduğunu kanıtlar:
   - tek kullanımlık uç anahtarı `IK` ile imzalanır;
   - imza, Noise el sıkışma özetini, rolü ve iki halkanın kimliğini kapsar;
   - imzaya bir Merkle kanıtı eklenir.
   Bu bağlamalar bilinmeyen anahtar paylaşımı ve yeniden oynatma
   saldırılarını önler. Noise ön paylaşımlı anahtarı (psk), kodun yerine
   iki halkanın ortak sırrından türer. Kesin tanım protokol notlarına
   yazılacak.
4. Oturum anahtarları geçici anahtarlardan gelir. Halka sırrı ya da bir
   cihaz anahtarı sonradan sızsa bile kaydedilmiş bir aktarım çözülemez.

## Kötüye kullanıma karşı

- **ARC jetonları** (Anonymous Rate-limited Credentials, IETF Privacy Pass
  taslağı): bir kez alınan kimlik bilgisinden sabit sayıda, birbirine
  bağlanamayan jeton üretilir. Böylece istemci kimliği olmadan kimlik
  bilgisi başına kota konabilir.
  - Düzenleyen ve doğrulayan aynı Worker'dır.
  - İstemci tarafında da bir Rust uygulaması gerekir.
- **Kimlik bilgisi almak:** birkaç saniyelik bir iş kanıtı (proof of work)
  gerekir ve IP başına sınır vardır.
- **Sahte istemci (Sybil) direnci** iş kanıtına ve IP sınırına dayanır.
  Bunlar botnete karşı zayıftır ve operatör NAT'ı (CGNAT) arkasındaki
  kullanıcıları zorlayabilir. Play Integrity kullanılmaz: Linux'ta yoktur
  ve gizliliği bozar.

## Bildirimler (Android)

- Varsayılan olarak UnifiedPush ve uygulama açıkken WebSocket bağlantısı
  kullanılır.
- **İsteğe bağlı FCM uyandırma:** Google'a yalnızca içeriksiz bir "uyan"
  sinyali gider. Kutu adı ve kimlik gitmez.
- Uyandırma belirteci her kutu için ayrı türetilir ve körleştirilir.
  Sunucuda kutu ile cihaz arasında bir bağ tablosu oluşmaz.

## Matematik

| Konu | Değer |
| --- | --- |
| Kurtarma ifadesi | 2048²⁴ = 2²⁶⁴, sağlamadan sonra 256 bit; Grover altında ~2¹²⁸ |
| Kutu adı çakışması | N ≈ 1,9·10¹⁰ ad/yıl için N²/2¹²⁹ ≈ 5·10⁻¹⁹ |
| Kutu adı tahmini | ~5·10⁷ canlı ad ile deneme başına ~2⁻¹⁰² |
| Kişi eşleşmesi | Üç kelimelik kod: çevrimiçi deneme başına 2⁻³³ (CPace, çevrimdışı deneme yok). Doğrulama kelimeleri: 2⁻⁴⁴, ama yalnızca kullanıcı karşılaştırırsa |
| Ayırt edici işaret | 6 kelime, 66 bit; taklit ~2⁶⁶ işlem |
| Shamir 2-of-3 | Her pay sahibi 0,9 olasılıkla ulaşılabilirse kurtarma olasılığı 0,972. 3-of-5 için 0,991. t−1 pay sır hakkında hiç bilgi vermez |
| Bekleme (dürüst teslim varsayımıyla) | Fark edilmeme olasılığı P = Π(1 − p_i)^T. İki cihaz, p = 0,9, T = 3 gün: 10⁻⁶. Tek cihaz, p = 0,5, T = 7 gün: 0,0078. Kötü niyetli sunucuya karşı güvence bu formülden değil, günlükten ve kişilerin yeniden doğrulamasından gelir |
| Dönem değişim boyutu | En çok 15 × (1120 + 48) ≈ 17,5 KB HPKE; kayıt ~23 KB |
| Yıllık trafik | En az 52 dönem × ~23 KB ≈ 1,2 MB, halka başına |
| Neden MLS değil | n ≤ 16'da TreeKEM'in O(log n) avantajı (4'e karşı 15) anlamsız; MLS'nin karmaşıklığı büyük |

## Güvenlik özellikleri

Her özellik [tehdit modeline](tehdit-modeli.md) yazılır ve bir testle ya
da biçimsel modeldeki bir lemmayla bağlanır.

| No | Özellik |
| --- | --- |
| P1 | **Üyelik bütünlüğü:** zincire yalnızca mevcut bir cihaz ya da (gecikmeli) `RK` kayıt ekler |
| P2 | **Geri alma ve çatal tespiti:** bir dönemin günlükte tek bir değeri vardır. Tanıklar, iki farklı görünümün gösterilmesini engeller |
| P3 | **Çıkarma sonrası gizlilik:** çıkarılan cihaz, sonraki dönemlere gönderilen içeriği okuyamaz. Sunucuyla birlikte çalışsa bile, günlük ve süreli başlar sayesinde |
| P4 | **İleri gizlilik:** aktarım anahtarları geçicidir |
| P5 | **Sunucuda hesap tablosu yok:** saklanan veriden hesaplar, cihaz sayıları ve kişi ilişkileri anahtar olmadan çıkarılamaz. Canlı bağlantıdaki IP ve zamanlama bu kapsamda değildir |
| P6 | **Kişilere karşı en az bilgi:** halka başı cihaz sayısını göstermez |
| P7 | **Sessiz ele geçirme yok:** yalnızca ifade çalınırsa ele geçirme, cihazlarda ya da kişilerde fark edilir. Telefon ve ifade birlikte çalınırsa bu güvence yoktur |
| P8 | **Kuantum sonrası gizlilik:** anahtar dağıtımı ve aktarım X-Wing ya da ML-KEM ile yapılır |

## Kalan riskler (kabul edilen)

- **IP ve zamanlama:** Cloudflare bağlantı anında IP'yi görür. Aynı
  NAT'tan bağlanan cihazlar ve kutular ilişkilendirilebilir. Bunu azaltan
  OHTTP (ayrı bir kuruluşun aracısıyla) ve Tor sonraki aşamadadır. Site ve
  mağaza metinlerinde "sunucu hiçbir şey bilmez" denmez; "sunucu hesap
  tutmaz" denir.
- **ARC kimlik bilgisi alma anı:** düzenleyici IP'yi görür. Ev NAT'ında
  anonimlik kümesi tek bir hanedir.
- **Cihaz sayısı:** bir kişi zamanla el sıkışmalarda gördüğü cihazları
  sayabilir.
- **Birlikte çalınma:** telefon ve ifade birlikte çalınırsa koruma yoktur.
- **Canlı ele geçirme:** kilidi açık, aktif kullanılan bir cihazın canlı
  olarak ele geçirilmesi kapsam dışıdır.
- **Kütüphane olgunluğu:** kuantum sonrası Rust kütüphaneleri 1.0 öncesidir
  ve bağımsız olarak denetlenmemiştir. Karma yapı klasik güvenceyi korur.
  Algoritmalar sürüm alanının arkasında değiştirilebilir. X-Wing test
  vektörleri tek bir taslak sürümüne sabitlenip depoya konur.

## Kabul ölçütleri (kod ve sürüm için)

- **K1. Kötü niyetli sunucu taklidi.** Test sunucusu her şeyi yapabilir.
  Buna rağmen:
  - listeye cihaz ekleyemez;
  - çıkarılmış bir cihazı geri getiremez;
  - iki kişiye iki farklı liste gösteremez, ya da gösterirse fark edilir.
- **K2. Çıkarma sonrası.** Çıkarılan cihaz, çıkarıldıktan sonra
  gönderilen dosyaları açamaz. Sunucu ona yardım etse bile.
- **K3. Kurtarma.** Kurtarma ifadesini bilen biri, cihazlar ve kişiler fark
  etmeden cihaz ekleyemez. Testte yerel saat, bekleme süresi, red ve
  yeniden doğrulama sınanır.
- **K4. Sızan veritabanı.** Sunucu veritabanının tamamı sızsa bile iki kutu
  adının aynı hesaba ait olduğu anahtar olmadan bulunamaz.
- **K5. Biçimsel model.** Protokol Tamarin ile modellenir ve P1-P8 lemma
  olarak kanıtlanır. İlk lemma "kötü sunucu + eski yetki" desenidir.
  Bekleme süreleri için ayrıca bir model denetimi (örneğin TLA+)
  değerlendirilir.
- **K6. Fuzz.** Zincir, baş, teklif ve günlük kanıtı ayrıştırıcıları ile
  çatal birleştirme kuralı için fuzz hedefleri vardır.

## Doğrulama ve uygulama sırası

1. **Bu belge ve [karar 0013](kararlar/0013-hayalet-halka-hesap.md)**
   (kod yok).
2. **Tamarin modeli** ve bağımsız insan kripto incelemesi. İnceleme
   kütüphane seçimini de kapsar.
3. **Protokol notları:** bayt biçimleri ve test vektörleri.
4. **Çekirdek:** `totaldrop-crypto` ve `totaldrop-core`. Bilinen yanıt
   vektörleri, özellik testleri ve fuzz hedefleriyle.
5. **Sunucu:** posta kutusu, günlük ve ARC Worker'ları. Kötü niyetli
   sunucu taklidiyle test edilir.
6. **Uygulamalar:** Linux (KDE, Hyprland, Omarchy) ve Android. 35 dilde
   metinler.
7. **Gizlilik politikası:** 35 dilde, özellik yayımlanmadan önce
   güncellenir.

## Kaynaklar

- **MLS Virtual Clients** (akraba tasarım, cihaz sayısını gizler):
  https://datatracker.ietf.org/doc/draft-ietf-mls-virtual-clients/
- **ARC:** https://datatracker.ietf.org/doc/draft-yun-privacypass-arc/
- **Parakeet** (WhatsApp anahtar şeffaflığı): https://eprint.iacr.org/2023/081
- **ELEKTRA** (çok cihazlı anahtar şeffaflığı): https://julialen.github.io/elektra_ccs23.pdf
- **Signal "bad device"** (zayıf cihaz kaydı dersi): https://eprint.iacr.org/2021/626
- **WhatsApp çok cihazlı grup analizi:** https://eprint.iacr.org/2025/794
- **Önceki örnekler:**
  - Keybase per-user key: cihaz iptalinde yenilenir, her cihaza ayrı
    şifrelenir.
  - Apple iCloud Keychain çemberi.
  - Akıllı sözleşme cüzdanlarında sosyal kurtarma (koruyucular + zaman
    kilidi).
