# 0014: Steam Deck, Steam Machine ve Steam Frame, Linux sürümünün bir parçası olarak desteklenir

- **Durum:** Kabul, 6 Kasım 2026 (Ozan, #295)
- **Bağlam:** Steam Deck kullanıcıları ekran görüntülerini, oyun
  videolarını, modları ve oyunların kayıt dosyalarını (save) Deck ile PC
  ve telefon arasında sık taşır; Valve'ın SteamOS çalıştıran diğer
  cihazları (Steam Machine, Steam Frame) için de aynısı geçerlidir. Bugün
  bunu çoğunlukla USB bellek, bulut depolama ya da Warpinator ve KDE
  Connect gibi araçlarla yaparlar. TotalDrop'un bu işi uçtan uca şifreli
  ve hesapsız yapabilmesi bu cihazları doğal bir hedef yapar. Bu belge
  önce Deck'te bugün ne çalıştığını, neyin eksik olduğunu ve hangi sırayla
  tamamlanacağını, sonra diğer iki cihazın farklarını kaydeder. Kod
  değişikliği içermez.

  ### Platform (6 Kasım 2026)

  Bu bölümdeki sürüm bilgileri basın haberlerinden alındı; Valve'ın kendi
  sürüm notlarıyla adım 3'te doğrulanacak.

  - SteamOS 3, Arch Linux tabanlıdır. Son kararlı sürüm 3.9.5'tir (Ekim
    2026): Linux 7.2 çekirdeği, Masaüstü Modu'nda KDE Plasma 6.7.
  - Deck'in iki yüzü vardır:
    - **Masaüstü Modu:** sıradan bir KDE Plasma oturumu. Dolphin, Discover,
      xdg-desktop-portal-kde ve bildirimler çalışır.
    - **Oyun Modu:** Steam'in tam ekran arayüzü, Valve'ın Gamescope
      bileşikleyicisi altında. Uygulamalar tam ekran ve tek tek açılır;
      giriş kumandadan gelir, Steam Input onu klavye ve fareye çevirebilir.
  - Kök dosya sistemi salt okunurdur ve her SteamOS güncellemesi bütün
    imajı değiştirir. `steamos-readonly disable` ile `pacman` kullanmak
    mümkündür ama kurulan paketler ilk güncellemede silinir; Valve bunu
    desteklemez. Kullanıcı yazılımı için desteklenen yol Flatpak'tir:
    Discover Flathub'a bağlı gelir.
  - Ev klasörü `/home/deck`, microSD kart `/run/media/…` altına bağlanır.
  - İşlemci x86_64'tür (AMD Zen 2); bugünkü x86_64 derlemesi mimari olarak
    uyar.

  ### Bugünkü Linux uygulaması Deck'te

  - Uygulama Qt 6 ve QML'dir ([0006](0006-linux-qt.md)). KDE'de sistem
    paletini ve Breeze'i izler. Sistem paketinden kurulduğunda Deck'in
    Masaüstü Modu'nda (Plasma) tema kuralı ([tasarim.md](../tasarim.md))
    ek iş gerektirmeden karşılanır; bu bir beklentidir, Deck'te
    denenmedi. **Flatpak içinde ise karşılanmaz:** tema kodu dosyaları
    doğrudan okur ve bu dosyalar korumalı alanda görünmez. `kde.rs`
    `~/.config/kdeglobals`'ı, `omarchy.rs` Omarchy'nin `~/.config`
    altındaki klasörünü, `gtk.rs` `~/.config/gtk-3.0`, `gtk-4.0` ve
    `/usr/share/themes`'i, `hyprland.rs` `$XDG_RUNTIME_DIR/hypr`'ı okur
    (hepsi `linux/src/theme/`). Flatpak'te `~/.config` uygulamanın kendi
    `~/.var/app/…/config` klasörüne döner. Breeze denetimleri için
    `org.kde.desktop` Qt stili de çalışma zamanında bulunmalıdır. Bu,
    adım 1'in en büyük işidir.
  - Dosyalar xdg-desktop-portal ile seçilir (`linux/src/desktop.rs`).
    Bildirimler `org.freedesktop.Notifications` üzerinden gider. Tek
    pencere oturum veriyolu adıyla sağlanır (`linux/src/instance.rs`).
  - Alınan dosyalar `~/Downloads/TotalDrop` klasörüne yazılır; bugün
    değiştirilemez.
  - Klavye gezintisi var: odak Tab ile dolaşır, odaklanan denetim görünür
    alana kaydırılır (`linux/qml/FocusScroller.qml`); gönderme, alma ve
    ayarlar sayfalarında Esc geri döner (`Shortcut`). Ok tuşlarıyla
    gezinme ve kumanda desteği yok.
  - Linux'ta QR kodu yalnızca gönderme sayfasında gösterilir. Alma sayfası
    QR'ı kameradan değil bir resim dosyasından okur (`readQrImage`);
    Deck'in kamerası da yoktur.
  - Aktarım sırasında sistemin uykuya geçmesini engelleyen bir kod yok.
  - Yayınlanan paketler `.deb`, `.rpm` ve Arch paketidir. SteamOS'ta
    hiçbiri desteklenen biçimde kurulamaz: `.deb`/`.rpm` yabancı paket
    yöneticisidir, Arch paketi de salt okunur köke yazar ve güncellemede
    silinir. **Yani bugün Deck'te kalıcı ve desteklenen bir kurulum yolu
    yok.** Flathub için uygulama kimliği (`app.totaldrop.TotalDrop`) ve
    marka kuralı ([0012](0012-ad-ve-isaret.md)) hazır, ama bir Flatpak
    tarifi henüz yok.

  ### Oyun Modu'nun getirdiği sorular

  1. **Steam'e ekleme:** Masaüstü Modu'nda Steam'de "Add a Non-Steam Game"
     (Türkçe arayüzde "Steam Dışı Bir Oyun Ekle") ile Flatpak'in masaüstü
     kaydı seçilir. Steam, `flatpak run` komutunu kısayol olarak saklar.
     Kütüphanedeki kapak, logo ve simge görsellerini kullanıcı elle
     verebilir; uygulamanın kendisi bunları Steam'e yazamaz.
  2. **Pencere, odak ve tema:** Gamescope pencereleri tek tek, tam ekran
     gösterir. Ana pencere sorun değildir. Belirsiz olanlar: portal dosya
     seçicisinin Oyun Modu'nda açılıp açılmadığı (Oyun Modu oturumunda
     hangi portal arka ucunun çalıştığı) ve açılırsa odağı alıp almadığı;
     bildirimlerin nerede göründüğü. Gamescope uygulamalara genellikle
     yalnızca Xwayland verir; Flatpak'te `fallback-x11` bu yüzden
     gereklidir. Oyun Modu'nda `XDG_CURRENT_DESKTOP` KDE olmadığı için
     bugünkü kod stil olarak Fusion'ı ve GTK/portal renklerini seçer.
     Oyun Modu'nda "sistem teması"nın ne olduğuna (Steam'in koyu arayüzü
     mü, Deck'in Plasma ayarları mı) adım 6'da karar verilir. Bunlar Deck'te
     denenene kadar varsayım olarak kalır.

     **Engelleyici risk:** Oyun Modu'nda dosya seçici portalı yoksa
     gönderme hiç yapılamaz. Bu durumda Oyun Modu'nda yalnızca alma
     desteklenir ve gönderme için kullanıcı Masaüstü Modu'na yönlendirilir.
     Uygulama içine ayrı bir dosya seçici yazılmaz: geniş dosya izni
     ister (aşağıya bakın).
  3. **Kumanda:** Steam Input'un masaüstü düzeni (sağ dokunmatik yüzey
     fare, tetikler tıklama) uygulamayı ek iş olmadan kullanılabilir
     yapar ama rahat değildir. Rahat kullanım için uygulamanın yön
     tuşları ve A/B ile gezilebilmesi gerekir. Steam Input bunları
     klavyenin ok, Enter ve Esc tuşlarına çeviren bir düzen olarak
     sunabilir; uygulama tarafında yapılacak iş, her sayfanın yalnızca
     klavyeyle (oklar, Enter, Esc, Tab) eksiksiz kullanılabilmesidir.
     Odaklanan denetim de uzaktan (televizyonda) ve 7 inçlik ekranda
     açıkça görünmelidir.
  4. **Ekran klavyesi:** Oyun Modu'nda Steam klavyesi, Steam dışı bir
     uygulamada metin alanına dokununca kendiliğinden açılmaz; kullanıcı
     Steam + X ile açar (kendiliğinden açılmama davranışı adım 6'da
     doğrulanacak). Telefondan Deck'e gönderirken Deck'te kod yazılır;
     Deck'in kamerası olmadığından QR ile alma yolu yoktur. Bunu
     hafifletmek için kod alanına odaklanınca kısa bir "Steam + X ile
     klavyeyi açın" ipucu gösterilir (yalnızca Gamescope altında, 35
     dilde). Deck'ten göndermek kod yazmayı gerektirmez: Deck kodu ve
     QR'ı gösterir, telefon tarar.
  5. **Uyku:** Deck, ekran kapanınca ya da güç düğmesine basılınca uyur ve
     aktarım kesilir. Aktarım sürerken portalın `Inhibit` arayüzüyle (ya
     da logind ile) uykuyu ertelemek gerekir. Bu Deck'e özgü değildir,
     bütün dizüstü bilgisayarlar için de doğrudur. Steam'in güç düğmesi
     yolunun bu engeli dikkate alıp almadığı doğrulanacak; almıyorsa
     aktarım sürerken uyutmamasını söyleyen bir uyarı kalır.

  ### Kullanım senaryoları ve dosya yolları

  | Senaryo | Yön | Yer |
  | --- | --- | --- |
  | Steam ekran görüntüleri | Deck → telefon/PC | `~/.local/share/Steam/userdata/<hesap>/760/remote/<uygulama>/screenshots` |
  | Steam oyun videoları | Deck → PC | Steam'in "Oyun kaydı" ayarındaki klasör (varsayılanın `userdata/<hesap>/gamerecordings` olduğu doğrulanacak); paylaşmak için önce Steam'den video olarak dışa aktarılır |
  | Modlar, haritalar | PC → Deck | Oyuna göre değişir; çoğunlukla `~/.local/share/Steam/steamapps/common/<oyun>` ya da Proton öneki |
  | Kayıt dosyaları (Steam Bulut'u olmayan oyunlar, öykünücüler) | iki yön | Proton öneki `steamapps/compatdata/<uygulama>/pfx/drive_c/users/steamuser/…`, öykünücülerin kendi klasörleri |
  | Herhangi bir dosya | iki yön | `~/Downloads`, microSD `/run/media/…` |

  Ekran görüntüsü ve oyun videosu klasörleri derin ve sayısal adlıdır;
  kullanıcı bunları dosya seçicide bulmakta zorlanır. Ama bu klasörleri
  uygulamanın kendisinin taraması, Flatpak'te geniş dosya izni ister ve
  Steam'in iç düzenine bağlılık getirir.

  ### Flatpak'te dosya ve sistem erişimi

  - Gönderme: dosya seçici portalı seçilen dosyaya izin verir; geniş izin
    gerekmez. Bugünkü kod bu yolu kullanır.
  - Alma: `--filesystem=xdg-download` ile yalnızca İndirilenler klasörü
    açılır; `~/Downloads/TotalDrop` yeter.
  - Tema: masaüstünün tema dosyalarına yalnızca okuma izni gerekir
    (`--filesystem=xdg-config/kdeglobals:ro`, Omarchy ve GTK tema
    klasörleri `:ro`, `--filesystem=xdg-run/hypr:ro`), ya da tema okumanın
    tamamı portala taşınır. Tek bir dosyanın bağlanması, KConfig'in
    kaydederken dosyayı yeniden adlandırması yüzünden canlı tema
    değişikliğini kaçırabilir; adım 1'de denenecek.
  - Steam klasörlerine ya da `/run/media`'ya kalıcı erişim
    (`--filesystem=home`, `--filesystem=/run/media`) istemek kolaylık
    sağlar ama Flathub incelemesinde gerekçe ister ve en az yetki ilkesine
    ([tehdit modeli](../tehdit-modeli.md)) aykırıdır. Önerilmez.
  - Ayarlar: sistem paketinin `~/.config/totaldrop/settings.toml` dosyası
    Flatpak'te görünmez; ilk açılış varsayılanlarla başlar.
  - Komut satırı: paketteki `totaldrop` aracı Flatpak'te
    `flatpak run --command=totaldrop app.totaldrop.TotalDrop` ile çalışır;
    yönergeye yazılır.
  - Tek pencere: uygulama kimliğiyle aynı D-Bus adını almak Flatpak'te
    varsayılan olarak serbesttir; `instance.rs` değişmez.

  ### Diğer SteamOS cihazları: Steam Machine ve Steam Frame

  Haberlere göre Valve 2026'da SteamOS çalıştıran iki cihaz daha çıkardı.
  Aşağıdaki donanım bilgilerinin hepsi basın haberlerinden; Valve'ın kendi
  belgeleriyle ya da cihazın kendisiyle (adım 3, 6 ve 8) doğrulanacak.

  - **Steam Machine:** televizyona bağlanan küçük bir PC (doğrulanacak:
    AMD Zen 4, x86_64; SteamOS 3, Masaüstü Modu'nda KDE Plasma). Deck'le
    aynı x86_64 Flatpak'i kullanır. Fark kullanımdadır: dokunmatik ekran
    yoktur, kullanıcı çoğunlukla Oyun Modu'nda, kumandayla ve birkaç
    metre uzaktan kullanır. Bu yüzden adım 5 (klavyeyle tam kullanım) ve
    adım 6 (Oyun Modu) Machine'de zorunlu, Deck'te ise rahatlıktır. Metin
    ve odak çerçevesi televizyon uzaklığından okunmalıdır. Ethernet ve
    microSD yuvası olduğu bildiriliyor (doğrulanacak).
  - **Steam Frame:** bağımsız çalışan VR gözlüğü (doğrulanacak:
    Snapdragon 8 Gen 3, ARM, aarch64; SteamOS). Haberlere göre yerel
    `linuxarm64` uygulamalar çalışır, x86_64 Linux uygulamaları FEX ile
    çevrilerek çalışır. Bugün yalnızca `.deb` ve `.rpm` paketlerimizin
    arm64 derlemesi var; Flatpak'in aarch64 derlemesi adım 1'de gelir,
    Flathub da aarch64 derler. Belirsiz olanlar: Frame'de Flatpak ve
    Discover'ın olup olmadığı, 2B bir Qt penceresinin VR içinde nasıl
    gösterildiği, girişin (VR kumandaları, el izleme, ekran klavyesi)
    uygulamaya nasıl ulaştığı, dosya seçici portalının çalışıp
    çalışmadığı ve VR kabuğunda "sistem teması"nın ne olduğu
    ([tasarim.md](../tasarim.md) kuralı burada da bağlayıcıdır). Frame'in
    kameraları uygulamalara açık değilse QR ile alma burada da yoktur.

  Üç cihaz aynı paketi kullanır; ayrı uygulama yazılmaz.

  ### Ağ

  Deck yalnızca Wi-Fi ile bağlanır (bağlantı istasyonunda Ethernet olur).
  Wi-Fi'nin kısa süre susması ya da ağ değişmesi, 0.3.0-beta.4 ile gelen
  röleye geçişle (#185) karşılanır. Cihazın uykuya geçmesi başka bir
  durumdur: uykuda hiçbir şey çalışmaz, aktarım durur. Bunun çözümü
  protokol değil, adım 4'teki uyku ertelemedir.

- **Seçenekler:**
  1. **Hiçbir şey yapmamak.** Kullanıcı `pacman` ile kurar; ilk güncellemede
     uygulama silinir. Desteklenmeyen bir yolu fiilen önermek olur.
  2. **Ayrı bir Deck uygulaması** (ör. Steam'e özel tam ekran arayüz).
     İkinci bir arayüzün bakımı, 35 dilde ikinci bir metin kümesi; kazanç
     küçük. Reddedildi.
  3. **AppImage.** Salt okunur kökte çalışır, ama güncelleme, masaüstü
     kaydı ve portal entegrasyonu kullanıcıya kalır; Discover'da görünmez;
     Qt eklentileri ve tema uyumu ayrıca dert olur.
  4. **Flathub'da Flatpak ve Linux uygulamasına Deck uyumu.** Deck'in
     desteklenen tek kurulum yolu; aynı paket diğer bütün dağıtımlarda da
     işe yarar. Oyun Modu için ayrı uygulama değil, mevcut arayüzde
     klavye/kumanda, uyku ve Gamescope ayrıntıları.

- **Karar:** Seçenek 4. Steam Deck, Steam Machine ve Steam Frame ayrı
  platformlar değildir; Linux sürümünün hedefleridir ve aşağıdaki sırayla
  desteklenir. Her adım ayrı bir issue ve PR'dır; adımlar 0.3 beta
  sürümlerinin yanında ilerler.

  | Adım | İş | Kabaca efor |
  | --- | --- | --- |
  | 1 | **Flatpak tarifi** (`packaging/flatpak/`): Qt sürümü CXX-Qt'ye uyan KDE çalışma zamanı ve Rust SDK eklentisi, `flatpak-cargo-generator` ile çevrimdışı cargo kaynakları, x86_64 ve aarch64. `finish-args`: `--share=network`, `--socket=wayland`, `--socket=fallback-x11`, `--device=dri`, `--filesystem=xdg-download`, `--talk-name=org.freedesktop.Notifications` (ya da bildirim portalına geçiş) ve yukarıdaki salt okunur tema izinleri; dosya seçici ve uyku portalı izin gerektirmez. CI'da derleme ve `flatpak-builder --run` ile duman testi; Flatpak derlemesi KDE, Hyprland ve Omarchy'de, açık ve koyu temada denenir. Bizim CI'daki derlemeye diğer paketlerdeki Sigstore kanıtı eklenir; Flathub'ın dağıttığı derleme ise Flathub'ın anahtarıyla imzalanır, bizim kanıtımızı taşımaz. | 4–6 gün (belirsiz) |
  | 2 | **Flathub başvurusu:** metainfo'nun Flathub denetimlerinden (`flatpak-builder-lint`) geçmesi, ekran görüntüleri; metainfo özeti ve açıklaması bugün yalnızca İngilizce ve Türkçe, 35 dile çevrilir. Flathub deposunun bakımı. İnceleme genellikle birkaç tur sürer. | 2 gün + bekleme |
  | 3 | **Masaüstü Modu'nda deneme** (her cihazda ayrı; cihaz ya da onu kullanan bir testçi gerekir): kurulum, tema, dosya seçici, bildirim, alma klasörü, aktarım hızı; platform bilgilerinin doğrulanması. Sonuçlar ekran görüntüleriyle issue'ya. | 0,5 gün |
  | 4 | **Aktarım sırasında uykuyu erteleme** (portal `Inhibit`), bütün Linux masaüstleri için. Yeni metin 35 dilde. | 1 gün |
  | 5 | **Yalnızca klavyeyle tam kullanım:** Tab ve Esc var; eksik olan ok tuşlarıyla gezinme ve her temada açıkça görünen odak çerçevesi. KDE, Hyprland ve Omarchy'de de denenir (onlarda da erişilebilirlik kazancıdır). | 2 gün |
  | 6 | **Oyun Modu denemesi ve düzeltmeleri** (Deck'te ve Machine'de ayrı ayrı): Steam'e ekleme, Gamescope altında pencere, dosya seçici (engelleyici risk), bildirim, uyku, tema hedefi ve ekran klavyesi; gerekirse Gamescope algılanınca kod alanında klavye ipucu (35 dil). Bulgulara göre ayrı issue'lar. | 1–2 gün + bulgular |
  | 7 | **Belgeler ve site:** her cihaz için ayrı sayfa ve kurulum yönergesi (Deck, Machine, Frame; her biri o cihazın adı geçerli olduğunda) (Discover'dan kurulum, Steam'e ekleme, önerilen Steam Input düzeni, komut satırı), sitede Linux indirme alanında Flathub bağlantısı. 35 dil. | 1–2 gün |
  | 8 | **Steam Machine ve Steam Frame'e özgü deneme:** Machine'de televizyon uzaklığında okunurluk ve yalnızca kumandayla kullanım; Frame'de aarch64 Flatpak'in kurulumu, VR içinde pencere, giriş, dosya seçici ve tema hedefi. Bulgulara göre ayrı issue'lar; Frame'de Flatpak yoksa bu karar Frame için yeniden açılır. | Machine 1 gün, Frame 2–3 gün + bulgular |

  Toplam kabaca Deck için 12–16, Machine ve Frame ile birlikte 15–20 iş
  günü; Flathub onay süresi ve cihazlara erişim hariç. Cihaz olmadan 1, 2,
  4 ve 5 tamamlanabilir; 3, 6 ve 8 cihaz (ya da o cihazı kullanan bir
  testçi) ister. Steam'e kendi uygulama sayfasıyla (Steamworks) çıkmak bu
  kararın dışındadır: ücretli bir başvuru ve şirket bilgisi ister, bağış
  modeline ([0005](0005-bagis.md)) ve hesapsız yapıya katkısı yoktur.

  **Ad: "TotalDrop for Steam Deck", "TotalDrop for Steam Machine",
  "TotalDrop for Steam Frame"** (Ozan, 6 Kasım 2026). Her cihazdaki sürüm
  kullanıcıya kendi adıyla sunulur: sitedeki cihaz sayfası, kurulum
  yönergesi ve Steam kütüphanesinde gösterilen başlık. Teknik olarak ayrı
  bir uygulama değildir: aynı Flatpak, aynı uygulama kimliği
  (`app.totaldrop.TotalDrop`), aynı sürüm numarası. Flathub kaydının adı
  "TotalDrop" kalır, çünkü aynı paket bütün dağıtımlar içindir.

  - "Steam Deck", "Steam Machine" ve "Steam Frame" Valve'ın ticari
    markalarıdır ve yalnızca uyumu anlatmak için kullanılır. Valve'ın
    onayı ya da ortaklığı ima edilmez. Logo, simge ve kapak görsellerinde
    Valve'ın adları ve logoları yer almaz; görsellerde yalnızca
    TotalDrop'un işareti ve adı olur. Her cihaz sayfasında marka sahibini
    söyleyen bir satır ve "TotalDrop, Valve ile ilişkili değildir" cümlesi
    bulunur. Adım 7'den önce Valve'ın kendi marka kuralları okunur ve
    sonucu bu belgeye eklenir; adlar onlarla çelişirse adlar değişir.
  - Bir cihazın adı, yalnızca o cihazdaki denemeler geçtikten sonra
    sitede ve mağaza metninde kullanılır: Deck için adım 3 ve 6, Machine
    için Machine'de yapılan adım 3 ve 6 ile adım 8'in Machine kısmı,
    Frame için adım 3 ve adım 8'in Frame kısmı.
  - Çeviri: ürün adındaki "TotalDrop" ve cihaz adı çevrilmez. Ad dillere
    göre "Steam Deck için TotalDrop" gibi doğal biçimde yazılabilir; hangi
    dilde nasıl yazılacağı [0008](0008-diller-ve-bolgeler.md) ve
    [ceviri.md](../ceviri.md) kurallarıyla adım 7'de belirlenir.
  - [marka.md](../marka.md)'ye adım 7'nin PR'ında eklenir: bu adlar
    yalnızca TotalDrop'un kendi sürümleri içindir; başkalarının dağıttığı
    paketler "TotalDrop for …" adını kullanamaz ([0012](0012-ad-ve-isaret.md)).

  **Switch 2 kapsam dışıdır.** Nintendo'nun geliştirici programı şirket
  başvurusu, gizlilik sözleşmesi ve kapalı SDK ister; açık kaynak bir
  dosya aktarma uygulamasının kabul edilmesi ve kaynağının yayımlanabilmesi
  beklenmez. Sistem uygulamalara kayıt ve medya klasörlerine serbest erişim
  de vermez.

- **Neden:** Flatpak, SteamOS cihazlarında desteklenen tek kalıcı
  kurulum yoludur ve aynı paket bütün dağıtımlarda kurulum sorununu da
  çözer. Uygulama zaten portal kullanacak biçimde yazıldığı için dosya
  seçme ve tek pencere değişmez; asıl iş temanın korumalı alanda
  okunmasıdır. Oyun Modu için ayrı arayüz yerine klavyeyle tam kullanım
  hedeflenir: kumanda Steam Input ile klavyeye çevrilir ve aynı iş bütün
  Linux kullanıcıları için erişilebilirliği artırır.

- **Sonuçlar:**
  - Flatpak, Linux için dördüncü paket biçimi olur; sürüm listesi
    ([surec.md](../surec.md)) ve marka kuralı Flathub kaydını kapsar.
  - **Kural değişikliği (Ozan onayladı, 6 Kasım 2026):** CLAUDE.md'deki
    "paketler yalnızca CI'da derlenir" kuralına bir istisna eklenir.
    Flathub, uygulamayı kendi altyapısında, bizim depodaki tariften ve
    etiketli kaynaktan derler; o derlemeyi biz yapmayız ve imzalamayız.
    Aynı tarif her PR'da bizim CI'ımızda da derlenir, böylece tarif
    bozulursa önce biz görürüz. İstisna CLAUDE.md'ye ve
    [surec.md](../surec.md)'ye adım 1'in PR'ında yazılır.
  - Tehdit modeline Flatpak izinleri eklenir: uygulama ağa, İndirilenler
    klasörüne, portal yoluyla seçilen dosyalara ve tema dosyalarına
    (yalnızca okuma) erişir.
  - Bir cihaz için "desteklenir" ancak yukarıdaki ad kuralındaki
    denemeler geçince yazılır.
  - Tasarım kuralındaki üç masaüstüne (KDE, Hyprland, Omarchy) Deck ve
    Machine eklenmez; Masaüstü Modu KDE'dir. Oyun Modu ve Frame'in VR
    kabuğu ayrı deneme maddeleri olarak sürüm listesine girer.

- **Yeniden bakılacak zaman:** Gamescope ya da SteamOS, Steam dışı
  uygulamalar için yerel bir portal, bildirim ya da ekran klavyesi
  davranışı getirdiğinde; Flathub başvurusu reddedilirse; Frame'de
  Flatpak yoksa; donanım bilgilerinden biri yanlış çıkarsa; ya da Valve
  adların kullanımına itiraz ederse.

## Kaynaklar

- SteamOS 3.9.5 sürümü: [Phoronix](https://www.phoronix.com/news/SteamOS-3.9.5)
- Steam ekran görüntüsü klasörü: [Dexerto](https://www.dexerto.com/tech/where-to-find-steam-screenshot-folder-2619310/)
- Steam oyun kaydı: [Shacknews](https://shacknews.com/article/142140/how-to-use-steam-game-recording)
- Steam Machine: [Engadget](https://www.engadget.com/gaming/pc/everything-we-know-about-valves-new-steam-machine-200458817.html)
- Steam Frame ve FEX: [GamingOnLinux](https://gamingonlinux.com/2026/01/valves-documentation-highlights-the-different-ways-standalone-games-run-on-steam-frame)
