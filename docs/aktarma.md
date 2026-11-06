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
  anahtarı kabul eder ve paketleri yalnızca bu ikisi arasında taşır; başka
  biri araya giremez.
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
Bu davranış Android ve Linux 0.2.0-beta.1 ile geldi
([#63](https://github.com/totaldrop-app/totaldrop/issues/63)); daha önceki
sürümler bağlantıyı her aktarımın başında açar.

Aktarma yolu gönderenin aktarma sunucusundan geçer. Alan cihaz onu yalnızca
kendi ayarındaki sunucuyla aynıysa kullanır; değilse yalnızca doğrudan
yolları dener, gizlilik modundaysa hata verip durur
([#68](https://github.com/totaldrop-app/totaldrop/issues/68)). Gönderen de
aynı kuralla, alan cihaz aynı sunucuyu kullanmayacaksa kendi sunucusuna hiç
bağlanmaz ([#301](https://github.com/totaldrop-app/totaldrop/issues/301)).

İlk açılan bağlantı kullanılır. Gizlilik modunda (komut satırında
`--relay-only`, uygulamalarda Ayarlar → "Adresimi karşı cihazdan gizle") cihaz
hiçbir IP adresi bildirmez ve yalnızca aktarma sunucusunu kullanır.

Doğrudan kurulan bir bağlantı, yolu sonradan susarsa aktarma sunucusuna
taşınır ([#185](https://github.com/totaldrop-app/totaldrop/issues/185)):
Wi-Fi kopar, NAT eşlemesi düşer ya da ağ el sıkışmayı geçirip sonra hiçbir
şey taşımaz. İki taraf da boştayken her 2 saniyede bir QUIC canlı tutma
paketi gönderir; bir cihaz bağlantının doğrudan adresinden 8 saniye boyunca
tek bir paket almazsa aktarma bağlantısını açar ve o adrese giden paketleri
aktarma sunucusundan yollar. Aktarma sunucusundan gelen paketler QUIC'e o
doğrudan adresten gelmiş gibi verilir; bu yüzden QUIC açısından adres
değişmez ve iki taraf ayrı ayrı, kendi saatiyle geçer. Yalnızca bir yön
ölürse önce karşıyı duymayı kesen taraf geçer; öbürü de onu duymayı kesince
bir bekleme sonra izler. İkisi de QUIC'in 30 saniyelik boşta kalma süresinin
içinde kalır.

Karşı taraf hâlâ doğrudan yoldan gönderiyorsa (yalnızca duraklamıştı ya da
hiç geçmedi) ondan gelen ilk doğrudan paketle cihaz doğrudan yola döner. İki
taraf da geçtiyse doğrudan adrese artık hiçbir şey gitmez; aktarım bitene
kadar aktarma sunucusunda kalır. Cihaz kendisinin durakladığını (arka planda
dondurulduğunu) saatinin en az 2 saniye geç işlemesinden anlar; o sessizliği
yolun sessizliği saymaz ve geçmeden önce karşı tarafa 2 saniye daha tanır.
QUIC karşı tarafın yeni bir adresine geçerse (NAT eşlemesi değişti) izlenen
adres de onunla değişir.

Geçiş yalnızca iki taraf aynı aktarma sunucusunu kullanıyorsa vardır: alanın
aktarma sunucusu gönderenden farklıysa ya da hiç yoksa iki taraf da geçmez,
çünkü öbür taraf oraya hiç gelmez. Doğrudan yol kazandığında aktarma
bağlantısı bekleme kipine alınır: kurulum sırasında açılmışsa o anda
kapatılır, sonra yalnızca bağlantı aktarma sunucusuna geçtiğinde yeniden
açılır. Bağlantı doğrudan yola dönerse aktarma bağlantısı 30 saniye boyunca
hiçbir şey göndermedikten sonra kapanır. Aktarma bağlantısı vazgeçmişse
(sunucu reddetti, ulaşılamadı ya da sınırlar doldu) cihaz geçmez, geçmişse
doğrudan yola döner. Aktarma sunucusu doğrudan başlamış bir aktarımı ancak
bir taraf geçtiğinde görür. Geçişten sonra QUIC'in tıkanıklık durumu
doğrudan yoldan kalır; ilk saniyelerde paket kaybı olur, QUIC bunu kendisi
toparlar.

Aktarma bağlantısı, karşı taraf henüz gelmemişken gönderdiği paketlere yanıt
alamaz; bu sırada her 2 saniyede bir ping gönderir. Aktarma sunucusu pinge
yanıt verdikçe bağlantı ölü sayılmaz, böylece geç gelen tarafı bekleyebilir.

Aktarma üzerinden başlayan bir aktarım ise sonradan doğrudan yola geçmez.
Bu geçiş (QUIC bağlantı göçü) sonraki adımdır.

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
