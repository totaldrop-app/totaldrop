# Hesap sistemi (tasarım, onaylandı 27 Ekim 2026)

TotalDrop'ta hesap Mullvad tarzıdır: e-posta, telefon, kullanıcı adı ve parola
yoktur. Hesap rastgele bir numaradır. Hesap zorunlu değildir; kodla gönderme
ve alma hesapsız çalışmaya devam eder.

## Hesap ne işe yarar

- **Kendi cihazlarım:** hesaba ekli cihazlar birbirine kod girmeden dosya
  gönderir.
- İleride: aktarma sunucusunda (relay) kotalar ve bağışçı ayrıcalıkları.

## Hesap numarası

- Sunucu, rastgele 16 haneli bir numara üretir ve `1234 5678 9012 3456`
  biçiminde gösterir. Son hane bir Damm kontrol hanesidir; yanlış yazılan ya
  da yer değiştiren bir haneyi cihaz sunucuya sormadan yakalar.
- Numara hiçbir zaman URL'de, kayıtta (log) ya da bildirimde görünmez.
  Sunucu yalnızca numaranın tuzlu özetini (hash) saklar.
- Sunucu, numara denemelerini IP'ye ve numaraya göre sınırlar.
- Numara her an yenilenebilir: eski numara anında geçersiz olur, ekli
  cihazlar çalışmaya devam eder.

## Cihaz ekleme: her cihaz bir kez

Bir cihaz hesaba **bir kez** eklenir ve siz çıkarana kadar orada kalır.
Tekrar tekrar onay yoktur.

### Normal yol: numarayı yazmak

Mullvad'daki gibi: yeni cihazda numarayı girersiniz, cihaz hesaba eklenir.
Bu kadar.

### Kolay yol: eski cihazdan QR

Elinizde ekli bir cihaz varsa yeni cihaz bir QR gösterir, eski cihazla
okutursunuz. İki ekranda aynı doğrulama kelimeleri çıkar; onaylarsınız.
Numara yazmak gerekmez. QR'da hesap numarası bulunmaz; birkaç dakika
geçerli, tek kullanımlık bir eşleştirme jetonu bulunur. Ekranın fotoğrafı
çekilse bile hesap ele geçmez.

### Numarayla eklenen cihaz için güvenlik payı

Numara çalınırsa birinin sessizce cihaz ekleyip size gelen dosyaları
almaması için:

- Numarayla eklenen yeni cihaz, hesaptaki diğer cihazlara **anında
  bildirilir** ("Yeni cihaz eklendi"). Tek dokunuşla çıkarılabilir.
- Hesapta başka cihaz varsa, numarayla eklenen cihaz **24 saat** boyunca
  "kendi cihazlarım" gönderimlerini almaz; kendisi gönderebilir. Bu süre,
  eski bir cihazdan QR ile onaylanırsa hemen biter.
- Hesapta hiç cihaz kalmadıysa (kurtarma) bekleme yoktur.

## Cihaz listesi

- En çok 10 cihaz. Liste her cihazdan görülür; istenen cihaz çıkarılır.
- Sunucu cihazların yalnızca rastgele kimliklerini ve açık anahtarlarını
  bilir. Cihaz adları ("Ozan'ın telefonu") yalnızca hesabın cihazları
  arasında, uçtan uca şifreli dolaşır. Sunucu model, konum ya da IP geçmişi
  tutmaz.
- Çıkarılan cihazın anahtarları iptal edilir; o cihaz bir daha hesaba
  gönderilen dosyaları alamaz.

## Uygulama sırası

1. Sunucu: numara üretme, özet saklama, cihaz kaydı, sınırlama, bildirim.
2. İstemci çekirdeği (`totaldrop-core`): hesap ve cihaz anahtarları,
   eşleştirme jetonu, "kendi cihazlarım" gönderimi.
3. Uygulamalar: Ayarlar > Hesap ekranı (numara, cihaz listesi, QR ile
   ekleme), numara girişi (rakam klavyesi, otomatik gruplama, yapıştırma).

Protokolle ilgili ayrıntılar (anahtar türleri, mesaj biçimleri) v0.1
taslağına önerilir; bu belge davranışı bağlar.
