# 0013: Hesap sunucuda tutulmaz: Hayalet halka

- **Durum:** Önerilen, 4 Kasım 2026 (#243). Kabul edilirse
  [0004](0004-mullvad-tarzi-hesap.md)'ün yerini alır.

## Bağlam

0004'te hesap, sunucunun ürettiği 16 haneli bir numaraydı. Bu tasarımda
cihaz listesini sunucu tutuyordu. Numarayla eklenen cihaz için konan 24
saatlik bekleme de sunucunun dürüstlüğüne bağlıydı. Oysa
[tehdit modeli](../tehdit-modeli.md) sunucuları güvenilmez sayıyor. Ayrıca
numara taşıyıcı bir sırdı: numarayı bilen her şeyi yapabiliyordu.

30 Ekim 2026'da Ozan numara yerine "cihaz halkası" modelini seçti. 4 Kasım
2026'da daha önce yapılmamış, daha gelişmiş ve güvenli bir sistem istedi.

## Değerlendirilen seçenekler

Seçenekler, Signal, Mullvad, Session, Briar, SimpleX, Keybase, passkey, MLS
ve MLS Virtual Clients taslağıyla karşılaştırılarak değerlendirildi. Bu
değerlendirme iki bağımsız inceleme turundan geçti.

1. **Tanıklı halka:** imzalı cihaz zinciri ve şeffaflık günlüğü. Sunucuda
   hesap kaydı kalır.
2. **Hayalet halka:**
   - sunucuda hesap tablosu yoktur;
   - posta kutusu adları her gün döner;
   - kişiler halkanın başına gönderir ve cihaz sayısını görmez;
   - kurtarma gecikmelidir.
3. **Tam MLS halkası:** standarttır, ama n ≤ 16 cihaz için gereğinden
   ağırdır.

## Karar

Seçenek 2, Hayalet halka. Ayrıntılar [hesap.md](../hesap.md)'dedir.
Bağlayıcı noktalar şunlardır:

- **Sunucu:** hesap, cihaz listesi ya da kişi listesi tutmaz. Yalnızca
  kısa ömürlü, rastgele adlı posta kutularını bilir.
- **Zincir:** üyelik, cihazların imzaladığı bir zincirdir. Her değişiklikte
  halka sırrı yenilenir. Halka başlarının en çok 7 günlük süresi vardır.
- **Şeffaflık günlüğü:** bağımsız tanıklı, anonim bir şeffaflık günlüğü
  **ilk sürümde** vardır (Ozan, 4 Kasım 2026).
- **Kurtarma:**
  - isteğe bağlıdır ve 24 kelimeden oluşur;
  - bekleme süresi 2 ya da daha çok cihazda 72 saat, tek cihazda 7 gündür;
  - kurtarma anahtarı yalnızca cihaz ekleyebilir ve çatal çözebilir;
  - kişiler kelimeleri yeniden karşılaştırır, otomatik kabul yoktur.
- **Kimlik ve adlar:** hiçbir cihazda kimlik yazılmaz. Cihaz ve kişi
  listesi her cihazda aynıdır. Adları kullanıcı verir ve adlar halkanın
  dışına çıkmaz.
- **Katmanlar ve onay:**
  - Üç güven katmanı vardır: benim cihazlarım, bilinen kişiler,
    yakındakiler.
  - Yönetim işlemleri ayrı bir yönetim anahtarı ve kullanıcı doğrulaması
    ister.
  - Sıkı mod (iki cihaz onayı, kurtarmada ikinci etken) bir ayardır.
- **Kötüye kullanıma karşı:** ARC jetonu, iş kanıtı ve IP sınırı (Ozan,
  4 Kasım 2026).
- **Android bildirimi:** UnifiedPush ve uygulama açıkken bağlantı.
  İçeriksiz FCM uyandırma isteğe bağlıdır (Ozan, 4 Kasım 2026).
- **Sınırlar:** en çok 16 cihaz. OHTTP ve Tor sonraki aşamadadır.
- **Gizlilik ile denetim arasındaki seçim:** kişiler üyeliğin hangi dönemde
  değiştiğini görür (cihaz sayısını görmez). Bu, kişilerin ekleme ve
  çıkarma kurallarını denetleyebilmesi için bilinçli olarak kabul edildi.

## Neden

- **Saklanmayan veri sızamaz.** Sunucu veritabanı sızsa bile içinden
  hesaplar, cihaz sayıları ve kişi ilişkileri anahtar olmadan çıkmaz.
- **Güvenlik iddiaları kötü niyetli sunucuya karşı da hedefleniyor.** Bunu
  sağlayacak parçalar şeffaflık günlüğü, gönderim öncesi tazelik denetimi
  ve kişilerin yeniden doğrulamasıdır. Üç inceleme turundaki altı kritik
  bulgunun hepsi "kötü sunucu + eski yetki" desenindeydi. Tasarım her
  birine bir karşılık getiriyor; bunların yeterli olduğu ancak biçimsel
  model ve bağımsız kripto incelemesiyle gösterilecek. Günlükte çöp yazmayla
  bir dönemi kilitleme sorusu açıktır.
- **Yalnızca standart parçalar kullanılır.** Algoritmalar sürüm alanının
  arkasındadır; zayıflık bulunursa değiştirilebilir.

## Sonuçlar

- **Belgeler:** [hesap.md](../hesap.md) yeniden yazıldı. Kabul edildiğinde
  0004'ün durumu "yerini aldı" olur. [Tehdit modeline](../tehdit-modeli.md) hesap
  bölümü eklendi.
- **Doğrulama ve kod sırası:** kod yazılmadan önce Tamarin modeli ve
  bağımsız bir insan kripto incelemesi yapılır. Ardından protokol notları,
  çekirdek, sunucu ve uygulamalar gelir. Her biri ayrı bir issue'dur.
- **İş büyüklüğü:** Tanıklı halkanın yaklaşık 2-2,5 katı. Nedeni günlük,
  tanık ilişkileri ve kişi tarafındaki doğrulama akışlarıdır.
- **Dil:** site, mağaza ve gizlilik politikası metinleri "dünyada ilk"
  demez. Hesap özelliği için "sunucu hesap tutmaz" denir. Gizlilik
  politikası, özellik yayımlanmadan önce 35 dilde güncellenir.
- **Kalan riskler:** IP ve zamanlama, ARC'de kimlik bilgisi alma anı,
  telefon ile ifadenin birlikte çalınması ve kütüphane olgunluğu.
  Hepsi [hesap.md](../hesap.md)'de yazılıdır.

## Gözden geçirme koşulları

Bu karar şu durumlardan biri gerçekleşirse yeniden değerlendirilir:

- Tamarin modeli ya da bağımsız inceleme bir özelliği çürütürse.
- MLS Virtual Clients RFC olursa ve birden çok halkanın ortak alanı (aile
  grupları) istenirse.
- Kuantum sonrası Rust kütüphanelerinden biri denetlenmiş 1.0 sürümüne
  ulaşırsa (kütüphane seçimi o zaman yeniden yapılır).
