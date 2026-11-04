# 0012: Tarayıcıdan gönderme, web.totaldrop.app

- **Durum:** Önerilen, 4 Kasım 2026 (#221)
- **Bağlam:** Uygulaması olmayan biri bugün TotalDrop ile dosya
  gönderemez. Ozan, `web.totaldrop.app` adresinde tarayıcıdan gönderme
  sayfası istiyor. Sayfa güvenlikten taviz vermemeli: aynı kod, aynı
  doğrulama kelimeleri, uçtan uca şifreleme. Alıcı tarafta Linux, Android
  ve komut satırı uygulamaları hiç değişmeden çalışmalı.

## Kısıt: tarayıcı UDP konuşamaz

Protokol v0 ([0002](0002-protokol-v0.md)) QUIC üzerinde çalışır ve
cihazlar UDP ile doğrudan bağlanır. Tarayıcı ham UDP soketi açamaz,
WebTransport ise gerçek bir TLS sertifikası olan bir sunucuya bağlanır,
karşıdaki cihaza değil. Bu yüzden tarayıcı doğrudan yolu kullanamaz.

Ama protokolde zaten tarayıcıya uyan bir yol var: **gizlilik modu**
(yalnızca aktarma). O modda cihaz hiçbir adres bildirmez ve QUIC
paketlerini `wss://relay.totaldrop.app` üzerinden WebSocket içinde taşır.
Rendezvous da WebSocket'tir. İkisi de tarayıcıda çalışır.

## Seçenekler

1. **Rust çekirdeği WebAssembly'ye derlemek, tarayıcı gizlilik modundaki
   bir cihaz gibi davranır.** CPace, Noise XXpsk3, ML-KEM-768, BLAKE3 ve
   STREAM aynı Rust kodundan gelir. QUIC için `quinn-proto` (G/Ç'siz
   QUIC durumu, wasm32'de çalışır) kullanılır; paketler aktarma
   sunucusunun WebSocket'ine verilir. Kablodaki her bayt bugünkü
   protokolle aynıdır.
2. **WebRTC veri kanalı.** Tarayıcıda yerleşik ve doğrudan yol açabilir.
   Ama uygulamaların hepsine WebRTC yığını ve ikinci bir aktarım protokolü
   eklemek gerekir; güvenlik tasarımı yeniden yazılır ve denetlenir.
3. **Şifreli dosyayı sunucuya yükleyip bağlantıyla paylaşmak.** Bu,
   v0'da tasarlanan ama henüz yapılmayan "bağlantılar ve çevrimdışı
   teslim" özelliğidir. Depolama maliyeti ve saklama süresi kararı ister;
   canlı aktarımın yerine geçmez.

## Karar

Seçenek 1. Tarayıcı, gizlilik modundaki bir gönderen gibi davranır:

- Yeni `crates/totaldrop-web` paketi (wasm-bindgen) çekirdeğin protokol
  mantığını tarayıcıya taşır. Çekirdekteki el sıkışma ve aktarım mantığı,
  tokio'ya bağlı olmayan parçalara ayrılır; uygulamalar ve tarayıcı aynı
  kodu kullanır. Kriptografi yeniden yazılmaz.
- Tarayıcı `HINTS` içinde adres bildirmez, aktarma adresi olarak
  varsayılan `wss://relay.totaldrop.app/v0/relay` verilir. Alıcının
  aktarma ayarı aynı olmalıdır (bugünkü kural, madde 21); kendi aktarma
  sunucusunu ayarlamış bir alıcıyla aktarım açık bir hatayla durur.
- Kod üretimi, doğrulama kelimeleri, teklif ve dosya parçaları bugünkü
  protokolle birebir aynıdır. Protokolde değişiklik yoktur; uygulamalarda
  güncelleme gerekmez.
- Hesaplama bir Web Worker'da çalışır, arayüz donmaz. Dosyalar
  `Blob.slice` ile 1 MiB'lık parçalar hâlinde okunur, hiçbir zaman bütün
  olarak belleğe alınmaz. Teklif her dosyanın BLAKE3 özetini istediği
  için tarayıcı dosyaları iki kez okur: önce özet (ekranda
  "Hazırlanıyor"), sonra gönderim.
- Rastgelelik `crypto.getRandomValues`'tan gelir (`getrandom` wasm
  arka ucu). Anahtarlar WebAssembly belleğinde tutulur ve iş bitince
  sıfırlanır; JavaScript tarafına hiçbir gizli değer geçmez.

### Kapsam ve aşamalar

1. **Deneme (spike), birleştirilmez:** `quinn-proto`, `rustls` (ring) ve
   kripto paketlerinin wasm32 derlemesi; paket boyutu ve hız ölçümü;
   tarayıcıdan komut satırı alıcısına gerçek aktarma sunucusu üzerinden
   gönderim. Ölçüt: en az 5 MB/s ve sıkıştırılmış paket en çok 2 MB.
   Tutmazsa bu karar yeniden açılır.
2. **Çekirdeği ayırmak:** el sıkışma ve aktarım mantığı G/Ç'siz hâle
   gelir; mevcut testler ve uçtan uca testler değişmeden geçer.
3. **`totaldrop-web` ve gönderme sayfası:** yalnızca gönderme. 35 dil,
   sistemin açık/koyu teması ve dil yönü, aktarımda sitedeki işaret
   animasyonu ([tasarım kuralları](../tasarim.md)).
4. **Yayın:** ayrı Cloudflare Pages projesi, `web.totaldrop.app`;
   dağıtım CI'dan, `cloudflare` ortam onayıyla.
5. **Tarayıcıda alma** ayrı bir karar olarak sonra gelir: büyük
   dosyaları diske yazmak tarayıcıya göre değişir (Chromium'da File
   System Access, diğerlerinde akışlı indirme).

## Güvenlik modeli

Protokolün sağladığı her şey aynen kalır: sunucular içeriği, dosya
adlarını ve boyutlarını göremez; kodu tahmin eden tek deneme hakkı alır;
kaydedip sonra çözmeye karşı ML-KEM-768. Değişen iki şey var:

- **IP adresi:** tarayıcı her zaman gizlilik modundadır, karşı cihaz
  gönderenin IP'sini görmez. Rendezvous ve aktarma sunucusu görür
  (bugünkü gizlilik modu gibi).
- **Kodun bütünlüğü, asıl yeni risk:** uygulama bir kez kurulur ve
  imzalıdır; web sayfası ise her açılışta sunucudan gelir. Sunucuyu ya da
  Cloudflare'i ele geçiren biri (saldırgan 2) belirli bir kişiye
  değiştirilmiş kod verip anahtarları sızdırabilir. Tarayıcıda bunu
  tamamen önlemenin bilinen bir yolu yoktur. Karşılıklar:
  - ayrı köken (`web.totaldrop.app`): ana sitedeki hiçbir betik bu sayfaya
    dokunamaz;
  - sıkı CSP (`script-src 'self' 'wasm-unsafe-eval'`, `connect-src`
    yalnızca rendezvous ve aktarma sunucusu), üçüncü taraf betik, analitik
    ve yazı tipi servisi yok;
  - betik ve WebAssembly dosyalarına alt kaynak bütünlüğü (SRI);
  - paket yalnızca CI'da, etiketli kaynaktan derlenir; dosyaların
    SHA-256 değerleri ve derleme kaynağı kanıtı GitHub sürümünde
    yayımlanır, sayfanın altında sürüm ve özet görünür;
  - sayfada açık not: "En güçlü güvence için uygulamayı kullanın."
  - İleride: tarayıcı eklentisiyle doğrulama ya da kurulabilir uygulama
    (PWA) olarak sabitleme ayrıca değerlendirilir.

Tehdit modeline yeni satır girer: "Değiştirilmiş web kodu (saldırgan 2,
6)". Gizlilik politikası, sunucuların web gönderiminde gördüklerini 35
dilde yazar; sayfa yayına girmeden önce.

## Sınırlar

- Aktarma sınırı aynen geçerli: en çok 32 GiB ve 6 saat.
- Hız aktarma yoluyla sınırlıdır (yerelde ölçülen yaklaşık 12 MB/s,
  gerçek internette daha az); doğrudan yol yoktur.
- Sekme kapanırsa ya da cihaz uykuya geçerse aktarım durur; sayfa
  aktarım sürerken kapatmadan önce uyarır.
- Desteklenen tarayıcılar: Chromium, Firefox ve Safari'nin son iki ana
  sürümü (WebAssembly, Web Worker, WebSocket). Klasör seçimi
  `webkitdirectory` ile.

## Maliyet (tahmin, uygulamadan önce güncel fiyatla doğrulanacak)

- Sayfa Cloudflare Pages'te: ücretsiz.
- Her web gönderimi aktarma sunucusundan geçer. Durable Object süresi
  (128 MB, yaklaşık 10 MB/s'de 1 GB için ~12,5 GB-s) ve WebSocket
  mesajları (gelen mesajlar 20'de 1 istek sayılır) Workers Paid planında
  ayda ~30.000 GB'a kadar dahil kotada kalır. Kotanın üstünde 1 GB
  yaklaşık 0,0003 $. Çıkış trafiği ücretsiz.
- Kötüye kullanıma karşı aktarma sunucusundaki eksik sınırlar (#190) bu
  sayfa yayına girmeden kapatılmalıdır: web, aktarma sunucusunu herkese
  daha kolay açar.

## Sonuçlar

- Uygulamalar ve protokol değişmez; yeni yüzey tarayıcı paketi ve web
  sayfasıdır.
- Çekirdeğin G/Ç'siz parçalara ayrılması, ileride iOS, macOS ve Windows
  için de işe yarar.
- `quinn-proto`'nun wasm desteği ve `ring`'in wasm32 derlemesi denemede
  doğrulanmadan uygulamaya geçilmez.
- Yol haritasında "önce kararlılık" ilkesi geçerli: #185 ve #190 bu işin
  önünde kalır.

## Gözden geçirme koşulu

Deneme ölçütleri tutmazsa ya da tarayıcılar ileride eşler arası QUIC
(ör. WebTransport P2P) sunarsa karar yeniden açılır.
