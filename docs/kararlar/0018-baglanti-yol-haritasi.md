# 0018: Bağlantı katmanı kendi yığınımızda kalır, doğrudan yollar parça parça artırılır

- **Durum:** Kabul, 10 Kasım 2026 (Ozan, #401)
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
  - QUIC tek bir soket sarmalayıcısı (`PathSocket`,
    `crates/totaldrop-core/src/path.rs`) üzerinden konuşur: doğrudan
    yollar gerçek UDP soketine, röle yolu WebSocket röle bağlantısına
    gider. Yollar yarışır, ilk el sıkışması tamamlanan kazanır. Doğrudan
    yol 8 saniye sessiz kalırsa bağlantı röleye geçer. Karşı taraf hâlâ
    doğrudan gönderiyorsa geri döner; iki taraf da geçtiyse bağlantı
    bitene kadar rölede kalır. Röleden sonradan açılan doğrudan yola
    geçmek henüz yapılmadı (protokol notu 22, #185).
  - Röle, Cloudflare Workers üzerinde her aktarım için bir Durable
    Object'tir ve yalnızca o iki anahtarı kabul eder. Kendi sunucusunda
    çalıştırmak isteyenler için aynı protokolü konuşan bir Rust rölesi de
    vardır. Adres bulma için `stun.cloudflare.com` kullanılır.

  Kodu okurken bir eksik bulundu: QUIC soketi yalnızca IPv4'e bağlanıyor
  (`transport::bind_socket`, `0.0.0.0`). `HINTS` 16 baytlık adresleri
  zaten taşıyor, ama IPv6 yolları hiç denenmiyor. #338'deki duraklamanın
  nedeni uygulamanın arka plana düşmesiydi, taşıma katmanı değil. #341
  (alma yönünün gönderme yönünden yavaş olması) hâlâ açık. Soket
  arabellekleri büyütüldü (7745be5, 614f243; `SOCKET_BUFFER` 4 MiB),
  bunun #341'e etkisi henüz ölçülmedi; konu taşıma katmanıyla ilgili ama
  iroh ya da libp2p'ye bağlı değil. Röle yolu 3 Kasım'da Ozan'ın Wi-Fi
  bağlantısında yaklaşık 3 MB/s ölçüldü (#185).

- **Seçenekler:**
  1. **iroh** (n0, [1.4.1](https://crates.io/crates/iroh/1.4.1), MIT
     veya Apache-2.0; [1.0 sürümü](https://crates.io/crates/iroh/1.0.0)
     15 Haziran 2026). Olgun NAT geçişi, yönlendiricide port açma, QUIC
     multipath, hazır röle. Uymadığı yerler:
     - Rölesi (`iroh-relay`) bir Rust sunucusudur ve Cloudflare
       Workers'ta çalışmaz. Ya bir sunucu işletiriz ya da n0'ın hız
       sınırlı genel rölelerine veya ücretli barındırılan rölesine
       bağlanırız. 0003, rölenin Workers'ta olmasını tam olarak
       güncellenecek ve korunacak yeni bir makine olmasın diye seçti;
       işletilecek bir sunucu ayrıca, [0005](0005-bagis.md) gereği önce
       bağış sayfasına yazılması gereken yeni bir giderdir.
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
       kaynakları, `cargo-deny`). Yol haritası önce kararlılığı istediği
       için 1.0 öncesi denetimden önce bu riski almayız.
     - `iroh-blobs` hâlâ deneme sürümündedir. STREAM ve BLAKE3 ile kendi
       parçalı aktarımımız aynı işi görür.
  2. **libp2p** (`rust-libp2p`
     [0.57.1](https://crates.io/crates/libp2p/0.57.1), MIT). 1.0
     değildir ve sık sık kırıcı değişiklik yapar. Circuit relay v2 yalnızca hole punching'i
     koordine etmek içindir; Rust uygulamasının varsayılan devre sınırları
     (2 dakika, 128 KiB) dosya taşımaya yetmez. DCUtR'ın başarı oranı
     yaklaşık %70, simetrik NAT'ta ise küçük bir örneklemde (536 eş)
     yaklaşık %40'tır
     ([Trautwein vd., büyük ölçekli ölçüm](https://arxiv.org/html/2510.27500v1);
     simetrik NAT oranı [ProbeLab](https://probelab.io/blog/can-libp2p-punch-through-nats/)).
     Identify protokolü cihazın adreslerini ve sürümünü karşı tarafa
     duyurur. DHT ve gossip ağları için tasarlanmıştır; bize uyan bir
     parçası yoktur.
  3. **Kendi yığınımızı korumak ve açık fikirleri parça parça eklemek.**
     Bugünkü yapı (yarışan yollar, `HINTS` ile adres paylaşımı, röleye
     düşüp geri dönme) bu parçaların iskeletidir; her biri protokolü
     geriye uyumlu biçimde büyütür.

- **Karar:** Seçenek 3. Kripto, eşleşme (CPace), rendezvous ve Workers
  rölesi olduğu gibi kalır. Tek istisna 6. adımdır: aynı ağda internet
  yokken CPace mesajları rendezvous posta kutusu yerine yerel ağ
  üzerinden taşınır; mesajlar ve CPace aynı kalır, değişen yalnızca onları
  taşıyan kanaldır. Aşağıdaki adımların her biri kendi issue'su, dalı,
  hatayı ya da eksiği gösteren testi ve cihaz denemesiyle ayrı bir PR olarak
  gelir. Sıra kazanca ve maliyete göredir; ilk üçü "Sırada" milestone'una,
  kalanları "Sonra"ya girer. Gizlilik modunda (yalnızca röle, tehdit
  modeli T6) bu adımların hiçbiri adres duyurmaz ve port açmaz; röle
  yolları (Workers rölesi ve 4. adımdaki TURN) gizlilik modunda da
  kullanılır ve IP adresini görür.

  1. **IPv6.** Çift yığınlı soket; IPv4 ve IPv6 yollarının yarışması
     (Happy Eyeballs, [RFC 8305](https://www.rfc-editor.org/rfc/rfc8305)).
     IPv6'da çoğunlukla NAT yoktur (NPTv6 ve NAT66 var ama seyrek); iki
     cihazın da IPv6'sı olduğunda ve araya giren güvenlik duvarları (operatörlerin ve ev yönlendiricilerinin) iki
     taraf aynı anda gönderdiğinde geçirdiğinde doğrudan yol delmeye gerek
     kalmadan açılır. Bunun ne kadar sık olduğu cihaz denemeleriyle
     ölçülür. Duyurulan IPv6 adresi, kalıcı adres yerine işletim
     sisteminin geçici adresi ([RFC 8981](https://www.rfc-editor.org/rfc/rfc8981))
     olmalıdır; kalıcı adres cihazı aktarımlar arasında tanınır kılabilir.
     Adayların sıralanması ve denenmesi için `str0m`'un ICE modülüne (MIT
     veya Apache-2.0) bakılır; uymazsa bugünkü yarış korunur.
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
     alınır. #185 ve protokol notu 22'deki "sonradan açılan doğrudan
     yola geçme" eksiğiyle birlikte ele alınır.
  4. **Ölçüm: ikinci röle olarak TURN.** Cloudflare Realtime TURN UDP
     üzerinden çalışır ve WebSocket rölemizin hızını (#185) aşabilir.
     0003 bunu zaten "aktarma hızı yetmezse" eklenecek ikinci yol olarak
     yazar. Fiyatı ayda ilk 1.000 GB ücretsiz, sonrası Cloudflare'den
     istemciye giden trafik için GB başına 0,05 USD'dir
     ([Cloudflare Realtime TURN](https://developers.cloudflare.com/realtime/turn/));
     ücretsiz kotayı aşan maliyet 0003 ve 0005 gereği bağışlardan
     karşılanır. Bu adım önce bir ölçümdür: hız, beklenen trafik ve
     sunucunun gördükleri (IP adresleri, süre, bayt) yazılır; kullanıma
     almak ayrı bir karar ve gizlilik politikası güncellemesi gerektirir.
     TURN de bir röledir: kullanıma alınırsa gizlilik modunda da
     kullanılabilir ve Workers rölesi gibi IP adresini, süreyi ve bayt
     sayısını görür; bu, tehdit modeli T6'ya yazılır.
  5. **Simetrik NAT için doğum günü delme.** Tailscale'in anlattığı
     yöntem: bir taraf çok sayıda port açar, öbür taraf rastgele portları
     dener. Paket sayısı ve süre sınırlıdır; röle her zaman yedekte kalır.
  6. **Sunucusuz yerel ağ keşfi.** Aynı ağdaki iki cihaz, internet
     yokken de birbirini bulur (LocalSend'in çok noktaya yayın fikri).
     Duyuru cihaz adı ya da kalıcı bir kimlik taşımaz ve kodu çevrimdışı
     denemeye yarayacak hiçbir şey içermez: koddan türetilen bir değer
     yayınlanırsa aynı ağdaki herkes kısa kodu çevrimdışı deneyip CPace'in
     verdiği tek tahmin hakkı korumasını bozar. Eşleşme yine CPace ile
     yapılır; duyuru yalnızca buluşma yerini söyler ve CPace mesajları
     rendezvous yerine bu yerel kanaldan gider (bkz. Karar). Eşleşme
     mesajlarının yeni bir taşıma yolu olduğu için bu adım denetim
     kapsamına girer. iOS'ta ham çok noktaya yayın ücretli bir izin
     (entitlement) gerektirir
     ([0009](0009-ios-macos.md)). Protokol notu ve tehdit modeli önce
     yazılır.
  7. **Gizlilik modunda Tor.** Protokol notlarında "henüz yok" olarak
     duran Tor yolu ([#39](https://github.com/totaldrop-app/totaldrop/issues/39)),
     `arti` (MIT veya Apache-2.0) ile onion hizmeti
     üzerinden yapılır; OnionShare bunun çalışan örneğidir. Yavaştır,
     yalnızca isteğe bağlıdır.
  8. **Sonra.** QUIC'in adres bulma, NAT geçişi ve multipath taslakları
     quinn'de desteklenince STUN'un ve elle yaptığımız yol değişiminin
     yerini alabilir. İnternetsiz yakınlık için Android'de Wi-Fi Aware ve
     Wi-Fi Direct.

  Bu projelerden yalnızca tasarım alınır, kodları alınmaz: Tailscale,
  Syncthing, LocalSend, Magic Wormhole, OnionShare, Briar, rquickshare.
  Bağlantı kodu protokol kütüphanelerinde (`totaldrop-core`, `-proto`,
  `-crypto`) yaşar ve bunlar MIT veya Apache-2.0'dır
  ([LICENSING.md](../../LICENSING.md)); bunlara copyleft lisanslı bir
  bağımlılık (ör. EUPL-1.2 lisanslı `magic-wormhole` crate'i) girmez.
  Kullanılacak her crate, eklendiği PR'da lisansı, bakım durumu ve
  alternatifleriyle yazılır ve `deny.toml`'dan geçer.

- **Neden:** Gizlilik ve maliyet modelimiz (tek kullanımlık anahtarlar,
  keşif yok, sunucusuz röle) iroh'ta ve libp2p'de baştan yoktur;
  onları bu modele uydurmak, kazandıracaklarından daha çok iş ve risk
  getirir. Doğrudan bağlantı kurma şansını en çok artıracak parçalar
  (IPv6, port açma, ağ değişimine tepki) ise küçüktür ve bugünkü yapıya
  eklenebilir.

- **Sonuçlar:**
  - 1–3. adımlarda kripto, eşleşme ve röle protokolü değişmez; 1.0
    öncesi denetimin kriptografi kapsamı bu adımlardan etkilenmez
    (aktarım protokolünün bir Tamarin modeli yok; mevcut modeller hesap
    tasarımı içindir). 6. adım CPace mesajlarını yeni bir yoldan (yerel
    ağ) taşır, 7. adım yeni bir yol (Tor) ekler; ikisi de denetimin
    kapsamına girer. Yeni yol türleri eklendikçe tehdit modeli ve
    protokol notları aynı PR'da güncellenir.
  - Sunucuların gördüğü bir şey değişirse (ör. TURN), gizlilik politikası
    35 dilde, yayından önce güncellenir.
  - Telemetri yoktur; her adımın etkisi elle cihaz denemeleriyle
    ölçülür: doğrudan yol mu röle mi kullanıldığı ve hız, aynı cihazlarla
    (Jonsbo, POCO, iş PC'si) önce ve sonra denenir.

- **Gözden geçirme koşulu:**
  - Cihaz denemeleri, bu adımlardan sonra da doğrudan bağlantının sık
    kurulamadığını gösterirse.
  - iroh rölesi sunucusuz bir ortamda (ör. Workers) çalışabilir olursa ya
    da iroh kimliği aktarım başına anahtarla ve keşifsiz, desteklenen bir
    yol olarak sunarsa.
  - quinn, QUIC NAT geçişi ya da multipath taslaklarını desteklerse.
