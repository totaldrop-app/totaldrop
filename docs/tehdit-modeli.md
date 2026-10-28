# Tehdit modeli

Bu belge TotalDrop'un neyi kimden koruduğunu, hangi saldırılara nasıl karşı
durduğunu ve neyi korumadığını yazar. 1.0 öncesi bağımsız denetimin
başlangıç noktasıdır. Protokolün kendisi v0 taslağında, uygulamadaki
kararlar [protocol-v0.1-notes.md](protocol-v0.1-notes.md) içindedir.

Son gözden geçirme: 28 Ekim 2026.

## Korunan varlıklar

| Varlık | Kimden saklanır |
| --- | --- |
| Dosyaların içeriği, adları, boyutları ve klasör yapısı | Sunucular, ağdaki herkes, kodu bilmeyen herkes |
| Kim kime gönderdi | Sunucular, ağdaki gözlemciler |
| Kullanıcının kimliği | Herkes; hesap yalnızca rastgele bir numara |
| IP adresi | Gizlilik modunda: karşı cihaz |
| Alıcının diski | Kötü niyetli gönderen (yol dışına yazma, üzerine yazma) |

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

## Güven sınırları

- İki cihaz birbirine yalnızca kodu bildikleri için güvenir; güven CPace ve
  Noise el sıkışmasından gelir, sunuculardan değil.
- Sunucular (rendezvous, aktarma, STUN) güvenilmez sayılır: yalnızca
  ulaştırırlar.
- Uygulamanın kendisi ve işletim sistemi güvenilir sayılır.

## Tehditler ve karşılıkları

| # | Tehdit | Saldırgan | Karşılık | Kalan risk |
| --- | --- | --- | --- | --- |
| T1 | Trafiği okumak | 1, 2 | QUIC/TLS 1.3 içinde Noise XXpsk3; dosyalar ayrıca STREAM ile şifreli | Yok, kriptografi sağlam kaldıkça |
| T2 | Araya girmek (MITM) | 1, 2 | Anahtarlar koddan CPace ile türer; yanlış kod el sıkışmayı bozar; dört doğrulama kelimesi | Kullanıcı kelimeleri karşılaştırmazsa, kodu bilen biri araya girebilir |
| T3 | Kodu tahmin etmek | 3 | CPace: deneme başına tek tahmin, çevrimdışı deneme yok; iki kelimede 2⁻²² olasılık; başarısız deneme iki tarafa "scary" kapanış olarak görünür; saldırı dalgasında yavaşlatma | Çok sayıda koda aynı anda saldırı; üç kelimelik kod (2⁻³³) seçenek olarak var |
| T4 | Kaydedip sonra çözmek | 5 | ML-KEM-768 hibrit adım; oturum anahtarları hem X25519'dan hem ML-KEM'den türer | Kimlik doğrulama v0'da klasik (Ed25519); v1'de ML-DSA-65 |
| T5 | Aktarımları birbirine bağlamak | 2 | Her aktarımda yeni, tek kullanımlık uç nokta anahtarı; sunucular kayıt tutmaz | Sunucu aynı anda IP ve zamanlamayı görür |
| T6 | IP adresini öğrenmek | 2, 4 | Gizlilik modu: adres bildirilmez, yalnızca aktarma | Varsayılan modda karşı taraf IP'yi görür; Tor henüz yok |
| T7 | Alıcının diskine zarar | 4 | Yol denetimi (NFC, `..`, ayırıcılar, Windows adları, bidi), yalnızca seçilen klasörün altına, var olanın üzerine asla yazmama, sembolik bağ izlememe, BLAKE3 doğrulamasından sonra yerine koyma | Aynı dosya sistemi yarış koşulları; denetim kapsamında |
| T8 | Bellek ya da disk tüketme | 4 | Sınırlar: 100.000 girdi, 16 MiB manifest, yol uzunlukları; reddedilen teklif | 16 MiB manifest çözülürken yaklaşık 1 GiB bellek; v0.1'de akış biçimine geçilecek |
| T9 | Sunucuyu kötüye kullanmak | 2, 3 | Rendezvous: kutu başına mesaj ve boyut sınırları, süreler; aktarma: aktarım başına iki anahtar, 32 GiB ve 6 saat | Rendezvous'ta istemci başına hız sınırı henüz yok |
| T10 | Aktarma sunucusuna başka bir anahtarla girmek | 2, 4 | Aktarım adresi iki anahtarı taşır; sunucu yalnızca bu iki anahtarı, imzalı bir meydan okumayla kabul eder | Yok |
| T11 | Tedarik zinciri | 6 | `cargo-deny` (lisans, güvenlik ilanları), Dependabot, kilitli bağımlılıklar, CI'da derleme ve derleme kaynağı kanıtı | Eylemler henüz commit SHA'sına sabitlenmedi |
| T12 | Sahte uygulama ya da paket | 6 | Yalnızca Google Play, GitHub sürümleri ve totaldrop.app; SHA-256 sağlama toplamları; derleme kaynağı kanıtı | Windows ve macOS imzası bağışları bekliyor |

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
