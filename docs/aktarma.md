# Aktarma sunucusu (relay)

Bu belge, farklı ağlardaki iki cihazın birbirine nasıl ulaştığını anlatır.
Protokoldeki karşılığı v0 taslağının 6. bölümüdür ("Relay").

## Amaç

Aynı ağdaki iki cihaz birbirini zaten bulur. İnternet üzerinden ise cihazlar
çoğunlukla bir modemin (NAT) ya da güvenlik duvarının arkasındadır ve
birbirlerine doğrudan ulaşamazlar. Aktarma sunucusu iki iş yapar:

1. **Adres bulma:** Cihaz, bir STUN sunucusuna (varsayılan olarak
   Cloudflare'in ücretsiz `stun.cloudflare.com` sunucusu) küçük bir UDP
   paketi gönderir; sunucu "seni şu adres ve portta görüyorum" diye cevap
   verir. Cihaz bu açık
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
- İki cihaz, iki tek kullanımlık anahtarın adını taşıyan adreste buluşur
  (`/v0/relay/<anahtar A>-<anahtar B>`). Sunucu orada yalnızca bu iki
  anahtarı kabul eder; başka biri araya giremez.
- Sunucu adres, anahtar ya da trafik kaydı tutmaz; diske hiçbir şey yazmaz.
- Her aktarım sınırlıdır: en çok 32 GiB ve 6 saat.
- Alıcı, gönderenin `HINTS` içinde bildirdiği aktarma sunucusunu kullanır.
  Bu adres yalnızca `wss://` olabilir.

## Yol seçimi

Her iki cihaz aynı anda şunları dener:

| Yol | Ne zaman |
| --- | --- |
| Yerel adresler | hemen |
| Açık adres (adres bulma ile) | hemen, NAT delme için |
| Aktarma sunucusu | 1,2 saniye sonra, doğrudan yol açılmadıysa |

Aktarma sunucusuna WebSocket bağlantısı da ancak bu 1,2 saniye dolup hâlâ
doğrudan bağlantı yoksa açılır; gizlilik modunda hemen. Her cihaz bu süreyi
kendi saatiyle, eşleşme bittiği andan sayar; bu yüzden biri aktarma
sunucusuna bağlanıp öbürü bağlanmayabilir. Doğrudan yol bu süre içinde
kazanırsa aktarma sunucusuna hiç bağlanılmaz ve sunucu o aktarımı görmez.
Doğrudan yol daha geç gelip kazanırsa açılmış aktarma bağlantısı, yol seçilir
seçilmez kapatılır; sunucu o kısa sürede cihazın IP adresini görmüş olur.
Bu davranış bir sonraki sürümle gelir
([#63](https://github.com/totaldrop-app/totaldrop/issues/63)); daha önceki
sürümler bağlantıyı her aktarımın başında açar.
Alan cihaz, gönderenin bildirdiği aktarma sunucusunu kullanır; gizlilik
modunda da ([#68](https://github.com/totaldrop-app/totaldrop/issues/68)).

İlk açılan bağlantı kullanılır. Gizlilik modunda (komut satırında
`--relay-only`, uygulamalarda Ayarlar → "Adresimi karşı cihazdan gizle") cihaz
hiçbir IP adresi bildirmez ve yalnızca aktarma sunucusunu kullanır.

Bu sürümde bağlantı, kurulduğu yolda kalır: aktarma üzerinden başlayan bir
aktarım sonradan doğrudan yola geçmez. Bu geçiş (QUIC bağlantı göçü) sonraki
adımdır.

## Uygulama

| Parça | Yer |
| --- | --- |
| Mesaj biçimi | `crates/totaldrop-proto/src/relay.rs` |
| İstemci: adres bulma, aktarma bağlantısı | `crates/totaldrop-core/src/relay.rs` |
| İstemci: tek QUIC soketi, iki yol | `crates/totaldrop-core/src/path.rs` |
| Sunucu (Cloudflare) | `server/relay-worker` |
| Sunucu (kendi makinende) | `server/totaldrop-relay`, kurulum dosyaları `deploy/` altında |

İstemci QUIC'e kendi soketini verir (`PathSocket`). Bu soket normal paketleri
gerçek UDP soketine, `240.84.68.1:1` adresine gidenleri ise aktarma
sunucusuna yollar. 240.0.0.0/4 ayrılmış bir bloktur ve hiçbir yere
yönlendirilmez. QUIC aktarma sunucusunu karşı cihazın bir adresi gibi görür.

## Sunucu

- **Adres:** `wss://relay.totaldrop.app/v0/relay`, Cloudflare Workers
  üzerinde. Her aktarım kendi Durable Object'inde iki WebSocket'i birbirine
  bağlar. Birkaç QUIC paketi tek bir WebSocket mesajında taşınır; böylece
  hem hız artar hem istek sayısı düşer.
- **Ölçüm:** yerel test ortamında, yalnızca aktarma yoluyla 100 MB, 8 saniye
  (yaklaşık 12 MB/s).
- **Adres bulma:** Cloudflare'in STUN sunucusu, `stun.cloudflare.com:3478`.
- **Kendi sunucunda çalıştırmak isteyenler için:** `server/totaldrop-relay`
  aynı protokolü konuşur ve STUN'u da kendisi cevaplar. `deploy/` altında
  systemd, Caddy ve nftables dosyaları var; kurulum betiği sunucunun adını
  ister: `sh install.sh relay.example.org`.
- **İleride:** kullanım büyür ya da hız yetmezse Cloudflare Realtime TURN
  ikinci bir aktarma yolu olarak eklenebilir (ayda 1.000 GB ücretsiz, sonrası
  GB başına 0,05 $).
