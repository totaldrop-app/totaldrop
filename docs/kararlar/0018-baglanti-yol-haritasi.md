# 0018: Bağlantı katmanı kendi yığınımızda kalır, doğrudan yollar parça parça artırılır

- **Durum:** Önerilen, 10 Kasım 2026 (#401)
- **Bağlam:** Ozan, 10 Kasım 2026'da iroh'un ve libp2p'nin TotalDrop'a
  uyup uymadığını ve bu alandaki açık fikirlerden hangilerini
  birleştirebileceğimizi sordu. Bugünkü bağlantı katmanı şöyle çalışır
  ([0002](0002-protokol-v0.md), [0003](0003-cloudflare-altyapi.md),
  [protokol notları](../protocol-v0.1-notes.md) 9, 10, 20, 21):

  - İki cihaz rendezvous posta kutusunda CPace ile eşleşir ve kimliği
    doğrulanmış `HINTS` ile yerel adreslerini, STUN ile öğrendikleri genel
    adreslerini ve rölelerini paylaşır.
  - Her aktarım her iki tarafta da yeni, tek kullanımlık bir Ed25519
    anahtarıyla QUIC (quinn) açar. Güven, QUIC içindeki Noise XXpsk3 ve
    ML-KEM-768 adımından gelir.
  - Tek bir UDP soketi hem doğrudan yolları hem röle yolunu taşır
    (`crates/totaldrop-core/src/path.rs`). Yollar yarışır, ilk yanıt veren
    kazanır. Doğrudan yol 8 saniye sessiz kalırsa bağlantı röleye geçer
    ve yol açılınca geri döner (#185).
  - Röle, Cloudflare Workers üzerinde her aktarım için bir Durable
    Object'tir ve yalnızca o iki anahtarı kabul eder. Kendi sunucusunda
    çalıştırmak isteyenler için aynı protokolü konuşan bir Rust rölesi de
    vardır. Adres bulma için `stun.cloudflare.com` kullanılır.

  Kodu okurken bir eksik bulundu: QUIC soketi yalnızca IPv4'e bağlanıyor
  (`transport::bind_socket`, `0.0.0.0`). `HINTS` 16 baytlık adresleri
  zaten taşıyor, ama IPv6 yolları hiç denenmiyor. Bugünkü yavaşlık
  bulgularının (#338, #341) nedeni uygulamanın arka plana düşmesi;
  taşıma katmanı değil. Röle yolu ise 3 Kasım'da Ozan'ın Wi-Fi bağlantısında
  yaklaşık 3 MB/s ölçüldü (#185).

- **Seçenekler:**
  1. **iroh** (n0, 1.4.1, MIT veya Apache-2.0; 1.0 sürümü 15 Haziran
     2026). Olgun NAT geçişi, yönlendiricide port açma, QUIC multipath,
     hazır röle. Uymadığı yerler:
     - Rölesi (`iroh-relay`) bir Rust sunucusudur ve Cloudflare
       Workers'ta çalışmaz. Ya bir sunucu işletiriz ya da n0'ın hız
       sınırlı genel rölelerine veya ücretli barındırılan rölesine
       bağlanırız. Bu, 0003 ile ve bağışla yürüyen, ücretli hizmet
       kullanmayan yapıyla ([0005](0005-bagis.md)) çelişir.
     - Kimlik, uzun ömürlü bir `EndpointId`'dir ve DNS (pkarr), mDNS ve
       DHT ile yayınlanır. n0'ın kendisi bu anahtarların gizli
       sayılmaması gerektiğini yazar; gizli adres arama yalnızca bir
       denemedir ([Blinded address lookup](https://iroh.computer/blog/blinded-address-lookup),
       13 Ekim 2026). Bizde anahtar her aktarımda yenidir ve keşif
       yoktur. Keşfi kapatıp her aktarımda anahtar üretirsek iroh'un
       kazandırdığının büyük kısmı gider.
     - Kripto tarafında bir şey kazandırmaz: CPace, Noise ve ML-KEM yine
       üstte kalır.
     - Geçiş, `path.rs`, `relay.rs`, `transport.rs`, iki röle sunucusu ve
       protokol notları 20–21'in yeniden yazılması demektir. Bağımsız
       denetimin kapsamı değişir ve bağımlılık ağacı büyür (Flatpak cargo
       kaynakları, `cargo-deny`). Kararlılık aşamasında bu riski almayız.
     - `iroh-blobs` hâlâ deneme sürümündedir. STREAM ve BLAKE3 ile kendi
       parçalı aktarımımız aynı işi görür.
  2. **libp2p** (`rust-libp2p` 0.57.1, MIT). 1.0 değildir ve sık sık
     kırıcı değişiklik yapar. Circuit relay v2 yalnızca hole punching'i
     koordine etmek içindir; Rust uygulamasının varsayılan devre sınırları
     (2 dakika, 128 KiB) dosya taşımaya yetmez. DCUtR'ın başarı oranı yaklaşık %70, simetrik
     NAT'ta yaklaşık %40'tır
     ([Trautwein vd., büyük ölçekli ölçüm](https://arxiv.org/html/2510.27500v1),
     [ProbeLab](https://probelab.io/blog/can-libp2p-punch-through-nats/)).
     Identify protokolü cihazın adreslerini ve sürümünü karşı tarafa
     duyurur. DHT ve gossip ağları için tasarlanmıştır; bize uyan bir
     parçası yoktur.
  3. **Kendi yığınımızı korumak ve açık fikirleri parça parça eklemek.**
     Bugünkü yapı (yarışan yollar, `HINTS` ile adres paylaşımı, röleye
     düşüp geri dönme) bu parçaların iskeletidir; her biri protokolü
     geriye uyumlu biçimde büyütür.

- **Karar:** Seçenek 3. Kripto, eşleşme, rendezvous ve Workers rölesi
  olduğu gibi kalır. Aşağıdaki adımların her biri kendi issue'su, dalı,
  hatayı ya da eksiği gösteren testi ve cihaz denemesiyle ayrı bir PR olarak
  gelir. Sıra kazanca ve maliyete göredir; ilk üçü kararlılık aşamasında
  yapılır.

  1. **IPv6.** Çift yığınlı soket; IPv4 ve IPv6 yollarının yarışması
     (Happy Eyeballs, [RFC 8305](https://www.rfc-editor.org/rfc/rfc8305)).
     Mobil operatörlerin çoğunda IPv6 vardır ve NAT yoktur; doğrudan yol
     kendiliğinden açılır. Adayların sıralanması ve denenmesi için
     `str0m`'un ICE modülüne (MIT veya Apache-2.0) bakılır; uymazsa bugünkü
     yarış korunur.
  2. **Yönlendiricide port açma.** UPnP-IGD, NAT-PMP
     ([RFC 6886](https://www.rfc-editor.org/rfc/rfc6886)) ve PCP
     ([RFC 6887](https://www.rfc-editor.org/rfc/rfc6887)); kısa süreli
     ve aktarım bitince geri bırakılan eşleme. Önce n0'ın tek başına
     kullanılabilen `portmapper` crate'i (MIT veya Apache-2.0)
     değerlendirilir. Açılan port yalnızca o aktarımın QUIC uç noktasına
     gider ve bu uç nokta yalnızca `HINTS`'te duyurulan anahtarı kabul
     eder; yine de tehdit modeline yazılır.
  3. **Ağ değişimini dinlemek.** 8 saniyelik sessizliği beklemek yerine
     işletim sistemi ağ değiştiğini söyleyince (Android
     `ConnectivityManager`, Linux NetworkManager) adresler yeniden
     öğrenilir ve yollar hemen yeniden denenir. Magic Wormhole'un
     "dilation" fikri ve `netwatch` crate'i (MIT veya Apache-2.0) örnek
     alınır. #340 ve protokol notlarındaki "sonradan açılan doğrudan
     yola geçme" eksiğiyle birlikte ele alınır.
  4. **Ölçüm: ikinci röle olarak TURN.** Cloudflare Realtime TURN UDP
     üzerinden çalışır ve WebSocket rölemizin hızını (#185) aşabilir.
     Fiyatı, Cloudflare'den istemciye giden trafik için GB başına 0,05
     USD'dir ve ayrı bir ücretsiz kota yazılmamıştır; bu, 0005 ile
     çelişir. Bu adım yalnızca bir ölçümdür: hız, maliyet ve sunucunun
     gördükleri (IP adresleri, süre, bayt) yazılır; kullanıma almak ayrı
     bir karar ve gizlilik politikası güncellemesi gerektirir.
  5. **Simetrik NAT için doğum günü delme.** Tailscale'in anlattığı
     yöntem: bir taraf çok sayıda port açar, öbür taraf rastgele portları
     dener. Paket sayısı ve süre sınırlıdır; röle her zaman yedekte kalır.
  6. **Sunucusuz yerel ağ keşfi.** Aynı ağdaki iki cihaz, internet
     yokken de, koddan türetilen bir duyuruyla birbirini bulur
     (LocalSend'in çok noktaya yayın fikri). Duyuru cihaz adı ya da
     kalıcı bir kimlik taşımaz; kod bilinmeden eşleşilemez. Protokol
     notu ve tehdit modeli önce yazılır.
  7. **Gizlilik modunda Tor.** Protokol notlarında "henüz yok" olarak
     duran Tor yolu, `arti` (MIT veya Apache-2.0) ile onion hizmeti
     üzerinden yapılır; OnionShare bunun çalışan örneğidir. Yavaştır,
     yalnızca isteğe bağlıdır.
  8. **Sonra.** QUIC'in adres bulma, NAT geçişi ve multipath taslakları
     quinn'de desteklenince STUN'un ve elle yaptığımız yol değişiminin
     yerini alabilir. İnternetsiz yakınlık için Android'de Wi-Fi Aware ve
     Wi-Fi Direct.

  Bu projelerden yalnızca tasarım alınır, kodları alınmaz: Tailscale,
  Syncthing, LocalSend, Magic Wormhole, OnionShare, Briar, rquickshare.
  İstemci MIT veya Apache-2.0'dır; GPL ya da EUPL lisanslı kod (ör.
  `magic-wormhole` crate'i EUPL-1.2'dir) istemciye girmez. Kullanılacak
  her crate, eklendiği PR'da lisansı, bakım durumu ve alternatifleriyle
  yazılır ve `deny.toml`'dan geçer.

- **Neden:** Gizlilik ve maliyet modelimiz (tek kullanımlık anahtarlar,
  keşif yok, sunucusuz röle) iroh'ta ve libp2p'de baştan yoktur;
  onları bu modele uydurmak, kazandıracaklarından daha çok iş ve risk
  getirir. Doğrudan yol oranını en çok artıracak parçalar (IPv6, port
  açma, ağ değişimine tepki) ise küçüktür ve bugünkü yapıya eklenebilir.

- **Sonuçlar:**
  - Kripto, eşleşme ve röle protokolü değişmez; Tamarin modeli ve 1.0
    öncesi denetimin kapsamı bu karardan etkilenmez. Yeni yol türleri
    (port açma, yerel keşif, Tor) eklendikçe tehdit modeli ve protokol
    notları aynı PR'da güncellenir.
  - Sunucuların gördüğü bir şey değişirse (ör. TURN), gizlilik politikası
    35 dilde, yayından önce güncellenir.
  - Her adım ölçülür: doğrudan yol mu röle mi kullanıldığı ve hız, aynı
    cihazlarla (Jonsbo, POCO, iş PC'si) önce ve sonra denenir.

- **Ne zaman yeniden bakılır:**
  - Ölçümler, bu adımlardan sonra da doğrudan bağlantının sık
    kurulamadığını gösterirse.
  - iroh rölesi sunucusuz bir ortamda (ör. Workers) çalışabilir olursa ya
    da iroh kimliği aktarım başına anahtarla ve keşifsiz, desteklenen bir
    yol olarak sunarsa.
  - quinn, QUIC NAT geçişi ya da multipath taslaklarını desteklerse.
