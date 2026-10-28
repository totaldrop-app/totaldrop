# Aktarma sunucusu (relay)

Bu belge, farklı ağlardaki iki cihazın birbirine nasıl ulaştığını anlatır.
Protokoldeki karşılığı v0 taslağının 6. bölümüdür ("Relay").

## Amaç

Aynı ağdaki iki cihaz birbirini zaten bulur. İnternet üzerinden ise cihazlar
çoğunlukla bir modemin (NAT) ya da güvenlik duvarının arkasındadır ve
birbirlerine doğrudan ulaşamazlar. Aktarma sunucusu iki iş yapar:

1. **Adres bulma:** Cihaz, sunucuya küçük bir UDP paketi gönderir; sunucu
   "seni şu adres ve portta görüyorum" diye cevap verir. Cihaz bu açık
   adresi şifreli `HINTS` mesajında karşı tarafa bildirir. İki cihaz
   birbirinin açık adresine aynı anda paket göndererek çoğu modemi deler
   (NAT delme) ve doğrudan bağlanır.
2. **Aktarma:** Doğrudan yol hiç açılmazsa (ör. sıkı kurumsal ağlar, mobil
   operatörlerin bazı NAT türleri), şifreli QUIC paketleri sunucu üzerinden
   gider. Bağlantı TCP 443 üzerinde WebSocket'tir, bu yüzden neredeyse her
   ağdan geçer.

## Güvenlik ve gizlilik

- Sunucudan geçen her şey uçtan uca şifrelidir (QUIC + Noise + ML-KEM).
  Sunucu dosya adlarını, içeriği ya da kimlik anahtarlarını göremez.
- Cihaz sunucuya her aktarım için yeni üretilen, tek kullanımlık bir uç nokta
  anahtarıyla bağlanır ve anahtarı imzalayarak kanıtlar. Sunucu iki aktarımı
  birbirine bağlayamaz.
- Sunucu adres, anahtar ya da trafik kaydı tutmaz; diske hiçbir şey yazmaz.
  Yalnızca toplam aktarılan bayt sayısını sayar.
- Her bağlantı sınırlıdır: saniyede 8 MiB, en çok 32 GiB ve 6 saat. Bu,
  aylık 20 TB trafik kotasını kötüye kullanıma karşı korur.
- Alıcı, gönderenin `HINTS` içinde bildirdiği aktarma sunucusunu kullanır
  (iki taraf aynı sunucuda buluşur). Bu adres yalnızca `wss://` olabilir.

## Yol seçimi

Her iki cihaz aynı anda şunları dener:

| Yol | Ne zaman |
| --- | --- |
| Yerel adresler | hemen |
| Açık adres (adres bulma ile) | hemen, NAT delme için |
| Aktarma sunucusu | 1,2 saniye sonra, doğrudan yol açılmadıysa |

İlk açılan bağlantı kullanılır. Gizlilik modunda (`--relay-only`) cihaz hiçbir
IP adresi bildirmez ve yalnızca aktarma sunucusunu kullanır.

Bu sürümde bağlantı, kurulduğu yolda kalır: aktarma üzerinden başlayan bir
aktarım sonradan doğrudan yola geçmez. Bu geçiş (QUIC bağlantı göçü) sonraki
adımdır.

## Uygulama

| Parça | Yer |
| --- | --- |
| Mesaj biçimi | `crates/totaldrop-proto/src/relay.rs` |
| İstemci: adres bulma, aktarma bağlantısı | `crates/totaldrop-core/src/relay.rs` |
| İstemci: tek QUIC soketi, iki yol | `crates/totaldrop-core/src/path.rs` |
| Sunucu | `server/totaldrop-relay` |
| Kurulum | `server/totaldrop-relay/deploy/` |

İstemci QUIC'e kendi soketini verir (`PathSocket`). Bu soket normal paketleri
gerçek UDP soketine, `240.84.68.1:1` adresine gidenleri ise aktarma
sunucusuna yollar. 240.0.0.0/4 ayrılmış bir bloktur ve hiçbir yere
yönlendirilmez. QUIC aktarma sunucusunu karşı cihazın bir adresi gibi görür.

## Sunucu

- **Adres:** `wss://relay.totaldrop.app/v0/relay`. Adres bulma aynı ad ve
  portta, UDP 443'te çalışır.
- **Makine:** Hetzner CX22 (2 vCPU, 4 GB, 20 TB trafik), Debian 13.
- **TLS:** Caddy, Let's Encrypt sertifikasıyla TCP 443'ü karşılar ve
  WebSocket'i `127.0.0.1:4100` adresindeki aktarma sunucusuna geçirir.
  Caddy'nin HTTP/3'ü kapalıdır, çünkü UDP 443 adres bulmaya ayrılmıştır.
- **DNS:** Cloudflare'de `relay` için A ve AAAA kaydı, proxy kapalı (gri
  bulut). Cloudflare proxy'si UDP taşımaz.
- **Güncelleme:** `server/totaldrop-relay/deploy/install.sh` derlenmiş
  ikiliyi kopyalar ve servisi yeniden başlatır.
