(() => {
  "use strict";

  const TR = {
    "skip": "İçeriğe geç",
    "nav.why": "Neden TotalDrop",
    "nav.download": "İndir",
    "why.title": "Neden TotalDrop",
    "why.r1.t": "Hiçbir şey yüklenmez",
    "why.r1.d": "Dosyalar sizin cihazınızdan onunkine gider. Bir şirketin sunucusunda bekleyen kopya yok, bir haftada süresi dolan link yok.",
    "why.r2.t": "Hiçbir zaman hesap yok",
    "why.r2.d": "E-posta yok, telefon numarası yok, kayıt yok. İki cihazın ihtiyacı yalnızca kısa bir kod.",
    "why.r3.t": "Her platform, tek uygulama",
    "why.r3.d": "Bugün Android; Linux, Windows, macOS ve iOS yolda. Telefondan bilgisayara, her markadan her markaya.",
    "why.r4.t": "Bizden boyut sınırı yok",
    "why.r4.d": "Bütün klasörler, uzun videolar, orijinal kalitede fotoğraflar. Sınırı yalnızca diskiniz ve bağlantınız belirler.",
    "why.r5.t": "Güvenilmez, doğrulanır",
    "why.r5.d": "İki ekrandaki dört sözcük araya kimsenin girmediğini kanıtlar. Şifreleme uçtan uca ve kuantum sonrası.",
    "why.r6.t": "Ücretsiz ve açık, kalıcı olarak",
    "why.r6.d": "GPL ile açık kaynak, bağışlarla ayakta. Reklam yok, takip yok, ücretli sürüm yok.",
    "why.cmp.caption": "TotalDrop, dosya göndermenin alışılmış yollarıyla karşılaştırıldığında",
    "why.cmp.cloud": "Bulut linkleri",
    "why.cmp.nearby": "Yakındaki paylaşım",
    "why.cmp.q1": "Dosyalar şirket sunucularına gitmez",
    "why.cmp.q2": "Hesap olmadan çalışır",
    "why.cmp.q3": "Her marka cihaz arasında",
    "why.cmp.q4": "Dünyanın her yerine",
    "why.cmp.q5": "Açık kaynak, denetlenebilir",
    "cmp.yes": "Evet",
    "cmp.no": "Hayır",
    "cmp.usually": "Çoğunlukla hayır",
    "cmp.family": "Tek bir marka içinde",
    "cmp.relay": "Aktarma sunucusuyla",
    "cmp.room": "Yalnızca aynı ortamda",
    "cmp.rarely": "Nadiren",
    "dl.yours": "Bu cihaz için",
    "dl.android": "Android",
    "dl.beta": "Beta",
    "dl.ready": "Hazır",
    "dl.planned": "Planlandı",
    "dl.android.d": "Uygulamanın tamamı; Türkçe ve İngilizce, telefonunuzun renklerini izler.",
    "dl.android.r1": "Android 15 ve üstü, 64 bit",
    "dl.android.r2": "Beta kapalıyken Google Play'de davetle test",
    "dl.android.cta": "Betaya katılmak isteyin",
    "dl.cli": "Komut satırı",
    "dl.cli.d": "Linux, macOS ve Windows terminalleri için. Dosya da gönderir, bütün klasör de.",
    "dl.copy": "Komutu kopyala",
    "dl.copied": "Kopyalandı",
    "dl.linux": "Linux uygulaması",
    "dl.linux.d": "Wayland için Qt 6 ile yapıldı. Renklerini, yazı tipini ve simgelerini masaüstünüzden alır: Plasma, Hyprland, Omarchy, GNOME.",
    "dl.linux.r1": "Her dağıtım ailesi için ayrı paket, 64 bit",
    "dl.linux.r2": "totaldrop komutunu da kurar",
    "dl.linux.arch": "Arch, Omarchy, Manjaro",
    "dl.linux.deb": "Debian, Ubuntu (.deb)",
    "dl.linux.rpm": "Fedora (.rpm)",
    "dl.req.linux": "Qt 6.8: Debian 13, Ubuntu 25.04, Fedora 43, Arch",
    "dl.req.linux.n": "Wayland ya da X11, x86-64",
    "dl.new.n4": "İlk Linux uygulaması: kodla ya da QR kodla gönderin ve alın, masaüstünüzün temasını izler.",
    "dl.verify.v4": "Linux'ta: indirdiğiniz dosyayla sudo pacman -U, sudo apt install ./ ya da sudo dnf install ./",
    "dl.windows": "Windows",
    "dl.windows.d": "Windows 11 için. İlk sürümler imzasız olacak; kod imzalama bağışlarla karşılanana kadar Windows ilk çalıştırmada uyarı gösterecek.",
    "dl.macos": "macOS",
    "dl.macos.d": "Apple, uygulamaları imzalamak ve onaylatmak için ücretli bir geliştirici hesabı (yılda 99 USD) istiyor. Bunu bağışlar karşılayacak.",
    "dl.ios": "iPhone ve iPad",
    "dl.ios.d": "App Store üzerinden, macOS ile aynı geliştirici hesabıyla.",
    "dl.web": "Tarayıcıda",
    "dl.web.d": "Hiçbir şey kurmadan alın: linki açın, kodu yazın, dosyaları kaydedin.",
    "dl.version": "Android 0.1.0-beta.4 · Linux 0.1.0-beta.3",
    "dl.date": "28 Ekim 2026",
    "dl.title": "TotalDrop'u indirin",
    "dl.lede": "Ücretsiz, açık kaynak, hesap yok. Cihazınızı seçin; şu an kullandığınız işaretli.",
    "dl.platforms": "Platformlar",
    "dl.new.title": "Yenilikler",
    "dl.new.nr": "Herkese, her yere gönderin: cihazlar internet üzerinden doğrudan bağlanır; arada bir modem ya da güvenlik duvarı varsa şifreli aktarma sunucusundan geçer.",
    "dl.new.n0": "Bir cihazın güvenlik duvarı gelen bağlantıları engellese de gönderme çalışıyor: iki cihaz da birbirine ulaşmayı dener ve ilk kurulan bağlantıyı kullanır.",
    "dl.new.n1": "Yeni işaret Dalga: bir damla, dalgaları ve kanaldan çıkan dosya. Gönderirken dosya kanal boyunca ilerler.",
    "dl.new.n2": "Daha sakin bir arayüz: her ekranda tek vurgulu eylem, gölge yok, daha net listeler.",
    "dl.new.n3": "Boş görünebilen ayarlar ekranı düzeltildi.",
    "dl.all": "Tüm sürümler ve sürüm notları",
    "dl.req.title": "Gereksinimler",
    "dl.req.platform": "Platform",
    "dl.req.min": "En az",
    "dl.req.notes": "Notlar",
    "dl.req.android": "Android 15, 64 bit ARM",
    "dl.req.android.n": "Yalnızca internet izni",
    "dl.req.cli": "Derlemek için Rust 1.99",
    "dl.req.cli.n": "Linux, macOS, Windows",
    "dl.req.net": "Ağ",
    "dl.req.net.m": "İnternet ya da aynı ağ",
    "dl.req.net.n": "Mümkünse doğrudan, değilse aktarma sunucusuyla",
    "dl.verify.title": "Kurduğunuzu doğrulayın",
    "dl.verify.v1": "TotalDrop'u yalnızca Google Play'den, bu sayfadan ya da projenin GitHub sürümlerinden alın. Başka hiçbir yerde yayımlamıyoruz.",
    "dl.verify.v2": "GitHub'daki her sürüm SHA-256 sağlama toplamlarını listeler. Bir dosyayı çalıştırmadan önce karşılaştırın.",
    "dl.verify.v3": "Her şey herkese açık kaynak koddan derlenir; kodu okuyabilir ve kendiniz derleyebilirsiniz.",
    "nav.how": "Nasıl çalışır",
    "nav.security": "Güvenlik",
    "nav.roadmap": "Yol haritası",
    "hero.eyebrow": "Ücretsiz · Açık kaynak · Beta",
    "hero.title": "Dosyayı doğrudan öbür cihaza gönderin.",
    "hero.lede": "TotalDrop dosyaları cihazınızdan karşı cihaza doğrudan, uçtan uca şifreli taşır. Kısa bir kod paylaşırsınız; aradaki hiç kimse gönderdiğinizi okuyamaz. Hesap yok, e-posta yok, telefon numarası yok.",
    "hero.cta": "Betayı indirin",
    "hero.source": "Kodu inceleyin",
    "hero.markTitle": "Merkezden çıkan dosya, dalgalardaki kanaldan geçiyor",
    "code.label": "Aktarım kodu",
    "code.caption": "Kodu sesli söyleyin ya da bir mesaja yapıştırın. Kodu kim girerse dosyaları o alır.",
    "how.title": "Üç adım",
    "how.s1.t": "Dosyaları seçin",
    "how.s1.d": "Her türden, istediğiniz kadar. Alıcı yalnızca seçtiklerinizi alır.",
    "how.s2.t": "Kodu paylaşın",
    "how.s2.d": "Bir sayı ve iki sözcük, 7-harbor-rocket gibi. Bir kez çalışır, sonra geçersiz olur.",
    "how.s3.t": "Dört sözcüğü karşılaştırın",
    "how.s3.d": "İki ekran da aynı dört sözcüğü gösterir. Eşleşiyorsa bağlantı yalnızca sizindir.",
    "how.note": "İnternet üzerinden iki cihaz, mümkün olan her yerde doğrudan bağlanır. Arada bir modem ya da güvenlik duvarı varsa şifreli aktarım, içeriği okuyamayan aktarma sunucumuzdan geçer.",
    "sec.title": "Dosyayı yalnızca siz ve alıcı açabilir",
    "sec.f1.t": "Çıkmadan şifrelenir",
    "sec.f1.d": "Dosyalar cihazınızda şifrelenir ve yalnızca alıcının cihazında açılır. Anahtarlar başka hiçbir yerde bulunmaz.",
    "sec.f2.t": "Kod çevrimdışı tahmin edilemez",
    "sec.f2.d": "Kod, PAKE el sıkışmasıyla bir anahtara dönüşür; yanlış bir tahmin hemen başarısız olur ve o deneme yanar. Anahtar değişimi kuantum sonrası ML-KEM'i de içerir.",
    "sec.f3.t": "Sunucumuz yalnızca tanıştırır",
    "sec.f3.d": "IP adresinizi ve kodun sayısını görür; sözcükleri, dosya adlarını ya da dosyaları asla görmez. Kayıt tutmaz.",
    "sec.f4.t": "Herkesin inceleyebileceği kod",
    "sec.f4.d": "Her satır herkese açık. 1.0 sürümü ancak bağımsız bir güvenlik denetiminden sonra çıkar.",
    "sec.privacy": "Gizlilik politikasını okuyun",
    "road.title": "Neredeyiz",
    "road.done": "Bitti",
    "road.now": "Şimdi",
    "road.next": "Sırada",
    "road.later": "Sonra",
    "road.p0": "Açık protokol, komut satırı istemcisi, Android betası ve Linux uygulaması",
    "road.p1": "Farklı ağlardaki cihazlar için aktarma sunucusu",
    "road.p2": "Windows uygulaması, kamerayla QR okutma, tarayıcıda alma",
    "road.p3": "iOS ve macOS, bağışlarla",
    "road.p4": "Cihazlarınız arasında kişiler için yalnızca rastgele bir numaradan oluşan isteğe bağlı hesap",
    "road.p5": "Bağımsız denetim ve 1.0 sürümü",
    "free.title": "Herkese ücretsiz, gücü yetenlerin desteğiyle",
    "free.d": "Reklam yok, ücretli sürüm yok, yatırımcı yok. Bağışlar para gerektiren işleri karşılar; her ay ne geldiğini ve nereye gittiğini yayımlarız.",
    "free.c1.t": "Aktarma sunucuları",
    "free.c1.d": "Farklı ağlar arası gönderim için bant genişliği",
    "free.c2.t": "Apple geliştirici hesabı",
    "free.c2.d": "Yılda 99 USD, iOS ve macOS için gerekli",
    "free.c3.t": "Güvenlik denetimi",
    "free.c3.d": "1.0'dan önce bağımsız bir inceleme",
    "free.gh": "GitHub'da destek olun",
    "foot.line": "TotalDrop açık kaynaktır ve herkesin gözü önünde geliştirilir.",
    "r.eyebrow": "Biri size dosya gönderiyor",
    "r.title": "TotalDrop ile alın",
    "r.code": "Aktarım kodu",
    "r.lede": "Bu cihazda TotalDrop'u açın; kod sizin için doldurulur. Başka bir cihazda Al'ı seçip kodu yazın.",
    "r.open": "TotalDrop'ta aç",
    "r.copy": "Kodu kopyala",
    "r.copied": "Kopyalandı",
    "r.none": "Bu bağlantıda TotalDrop kodu yok. Gönderenden kodu isteyin, sonra TotalDrop'u açıp Al'ı seçin.",
    "r.get.title": "TotalDrop henüz yok mu?",
    "r.get.body": "Ücretsizdir, hesap gerekmez ve bir dakikada kurulur. Gönderen TotalDrop'u açık tuttuğu sürece kod geçerlidir.",
    "r.get.cta": "TotalDrop'u indirin",
    "r.private": "Kod yalnızca bu sayfada okunur, sunucumuza hiç gönderilmez.",
    "foot.rss": "RSS",
    "foot.privacy": "Gizlilik",
    "don.eyebrow": "Bağışlar · fiyatlar 28 Ekim 2026 itibarıyla",
    "don.title": "TotalDrop neden bağışa ihtiyaç duyuyor",
    "don.lede": "TotalDrop herkes için ücretsiz ve öyle kalacak: reklam yok, ücretli sürüm yok, yatırımcı yok, satılacak veri yok. Yine de bazı şeyler para tutuyor. Bu sayfa hepsini tek tek listeler; bağışınızın tam olarak neye gittiğini görebilirsiniz.",
    "don.cta": "GitHub'da destek olun",
    "don.code": "Koda katkı verin",
    "don.col.item": "Kalem",
    "don.col.for": "Ne için",
    "don.audit.title": "En büyük kalem: bağımsız güvenlik denetimi",
    "don.audit.d1": "TotalDrop, aradaki kimsenin dosyalarınızı okuyamayacağını söylüyor. Böyle bir söz, kodu yazmamış insanlar tarafından denetlenmeli. 1.0 sürümünden önce bağımsız bir firma protokolü, kriptografiyi ve dışarıdan gelen veriyi işleyen kodu inceleyecek; raporun tamamı yayımlanacak.",
    "don.audit.d2": "Bu tür denetimler on binlerce dolarla fiyatlanıyor; benzer işler için yayımlanmış teklifler 64.000 USD'ye kadar çıkıyor. Firmaya ve kapsama göre 20.000 – 60.000 USD bekliyoruz. 1.0 sürümü bunu bekliyor.",
    "don.how.title": "Para nasıl kullanılır",
    "don.how.h1": "Bağışlar GitHub Sponsors üzerinden, bir kez ya da her ay yapılır.",
    "don.how.h2": "Yalnızca bu sayfadaki maliyetler için kullanılır. Bir maliyet eklendiğinde ya da değiştiğinde önce bu sayfa değişir.",
    "don.how.h3": "Her ay ne geldiğini ve nereye gittiğini burada ve projenin deposunda yayımlarız.",
    "don.how.h4": "Bağış özellik, öncelik ya da uygulama üzerinde söz hakkı satın almaz. Herkes aynı TotalDrop'u kullanır.",
    "don.raised.title": "Şimdiye kadar toplanan",
    "don.raised.t1": "0 USD toplandı",
    "don.raised.d1": "Proje başladığından beri gelen bağışlar",
    "don.raised.t2": "39,20 USD harcandı",
    "don.raised.d2": "Alan adı ve Google Play hesabı; şimdiye kadar projeyi yürüten kişi ödedi",
    "don.raised.t3": "Hedef 250 USD",
    "don.raised.d3": "Bütün platformlarda bir yılın maliyeti",
    "don.raised.goal": "İlk yıl için 250 USD'nin 0'ı: %0. Güncelleme: 28 Ekim 2026.",
    "don.ledger.caption": "Her ay, kalem kalem",
    "don.ledger.month": "Ay",
    "don.ledger.in": "Gelen",
    "don.ledger.out": "Giden",
    "don.ledger.what": "Ne için",
    "don.ledger.m1": "Ekim 2026",
    "don.ledger.m1.w": "totaldrop.app alan adı, 14,20 USD; Google Play geliştirici hesabı, 25 USD",
    "don.budget.title": "Bütün maliyetler, kalem kalem",
    "don.col.when": "Ne zaman",
    "don.col.year": "Yıllık",
    "don.b1": "totaldrop.app alan adı",
    "don.b1.f": "Sitenin ve bütün sunucuların adresi; Cloudflare'de maliyet fiyatına kayıtlı",
    "don.b1.w": "Şimdi",
    "don.b2": "Cloudflare Workers Paid",
    "don.b2.f": "Site, rendezvous sunucusu ve aktarma sunucusu. Aktarma trafiği günlük ücretsiz sınırları aşana kadar ücretsiz; sonra ayda 5 USD, içinde 1 milyon istek ve 400.000 GB-saniye",
    "don.b2.w": "Aktarma yoğunlaşınca",
    "don.b3": "Apple Developer Program",
    "don.b3.f": "Apple, iPhone, iPad ve Mac'te yalnızca imzalı ve onaylı uygulamaların uyarısız çalışmasına izin veriyor",
    "don.b3.w": "iOS ve macOS'tan önce",
    "don.b4": "Windows kod imzalama sertifikası",
    "don.b4.f": "İmza olmadan Windows bilinmeyen yayıncı uyarısı gösterir. Certum'un açık kaynak geliştiricilere yönelik sertifikası: ilk yıl kart ve okuyucuyla 69 EUR, sonra 49 EUR. Microsoft'un kendi hizmeti ABD ve Kanada dışındaki bireylere sunulmuyor",
    "don.b4.w": "Windows'tan önce",
    "don.b4.c": "Yaklaşık 75 USD",
    "don.b5": "Cloudflare STUN",
    "don.b5.f": "Her cihaza açık adresini söyler; iki cihaz böylece doğrudan bağlanabilir",
    "don.b5.w": "Şimdi",
    "don.b6": "GitHub",
    "don.b6.f": "Kod, otomatik derlemeler ve sürüm indirmeleri; açık kaynak için ücretsiz",
    "don.b7": "Google Play geliştirici hesabı",
    "don.b7.f": "Android uygulamasını yayımlamak; bir kerelik ücret, ödendi",
    "don.b7.w": "Bir kez ödendi",
    "don.free": "0 USD",
    "don.total": "Toplam",
    "don.total.c": "Yaklaşık 250 USD",
    "don.relay.title": "Aktarma büyürse",
    "don.relay.d": "Aktarımların çoğu doğrudan bağlanır ve hiçbir şeye mal olmaz. Doğrudan bağlanamayanlar aktarma sunucusundan geçer. Daha fazla hız gerekirse Cloudflare Realtime TURN bunları her kullanıcıya yakın bir noktadan UDP ile taşır: ayda ilk 1.000 GB ücretsiz, sonrası GB başına 0,05 USD. Trafik arttıkça bunu bağışlar karşılayacak.",
    "don.relay.c1": "Aylık aktarılan trafik",
    "don.relay.c2": "Aylık",
    "don.relay.c3": "Yıllık",
    "don.m.3920": "39,20 USD",
    "don.m.1420": "14,20 USD",
    "rm.eyebrow": "Yol haritası · güncelleme: 28 Ekim 2026",
    "rm.title": "TotalDrop'a neler geliyor",
    "rm.lede": "Tarihler hedeftir, söz değildir: bir özellik ancak güvenli olduğunda çıkar. Her adım açık depoda izlenebilir.",
    "rm.all": "Bütün platformlar",
    "rm.cta": "Bağışlar neye gidiyor",
    "rm.issues": "Bir fikir önerin",
    "foot.roadmap": "Yol haritası",
    "road.more": "Yol haritasının tamamı",
    "rm.done.state": "Bitti",
    "rm.done.when": "Ekim 2026",
    "rm.d1.t": "Açık bir protokol",
    "rm.d1.d": "TotalDrop Protokolü v0: CPace ile aktarım kodları, kuantum sonrası ML-KEM-768 ile Noise XXpsk3, tek kullanımlık anahtarlarla QUIC. Herkes uyumlu bir uygulama yazabilsin diye yazılı.",
    "rm.d2.t": "Komut satırı istemcisi",
    "rm.d2.d": "Linux, macOS ve Windows'ta terminalden gönderme ve alma.",
    "rm.d3.t": "Android betası",
    "rm.d3.d": "Kodla gönderme ve alma; İngilizce ve Türkçe, telefonun renklerini izler. Google Play'de davetle.",
    "rm.d4.t": "Linux uygulaması",
    "rm.d4.d": "Qt 6 ve yerel Wayland. KDE Plasma, Hyprland ve Omarchy temalarını canlı izler. Arch, Debian, Ubuntu ve Fedora paketleri.",
    "rm.d5.t": "Güvenlik duvarlarından geçiş",
    "rm.d5.d": "İki cihaz da dinler ve arar; bir taraftaki güvenlik duvarı artık aktarımı durdurmaz.",
    "rm.now.state": "Şimdi",
    "rm.now.when": "Kasım 2026",
    "rm.n1.t": "İnternet üzerinden",
    "rm.n1.d": "Cihazlar açık adreslerini STUN ile bulur ve çoğu ev modeminin arkasından doğrudan bağlanır. Bu mümkün değilse şifreli trafik Cloudflare'deki aktarma sunucumuzdan geçer; sunucu içeriğini göremez.",
    "rm.n2.t": "Komut satırında gizlilik modu",
    "rm.n2.d": "Yalnızca aktarma sunucusu: karşı taraf IP adresinizi hiç öğrenmez.",
    "rm.next.state": "Sırada",
    "rm.next.when": "Kış 2026–27",
    "rm.x1.t": "QR kodu kamerayla okutma",
    "rm.x1.d": "Kodu yazmak yerine telefonu gönderenin ekranına tutun.",
    "rm.x2.t": "Tarayıcıda alma",
    "rm.x2.d": "Bir bağlantı açın, kodu yazın, dosyaları kaydedin. Kurulum yok, yine uçtan uca şifreli.",
    "rm.x3.t": "Yarıda kalan aktarımı sürdürme",
    "rm.x3.d": "Kopan bir bağlantı, gelenleri yeniden göndermeden kaldığı yerden devam eder.",
    "rm.x4.t": "Aktarmadan doğrudan yola geçiş",
    "rm.x4.d": "Aktarma sunucusundan başlayan bir aktarım, doğrudan yol açılır açılmaz ona geçer.",
    "rm.x5.t": "Windows uygulaması",
    "rm.x5.d": "Windows 11 için yerel uygulama. Sertifikayı bağışlar karşılayınca imzalı.",
    "rm.x6.t": "Arch User Repository",
    "rm.x6.d": "AUR yardımcınızla kurun ve güncelleyin.",
    "rm.later.state": "Sonra",
    "rm.later.when": "2027",
    "rm.l1.t": "iPhone, iPad ve Mac",
    "rm.l1.d": "Bağışlar Apple geliştirici hesabını karşılayınca yerel uygulamalar.",
    "rm.l2.t": "Bağlantılar ve çevrimdışı teslim",
    "rm.l2.d": "Çevrimiçi olmayan birine gönderin: şifreli bir kopya en çok 7 gün bekler, yalnızca bağlantı açar.",
    "rm.l3.t": "Yalnızca bir numaradan oluşan hesap",
    "rm.l3.d": "İsteğe bağlı, Mullvad tarzı: e-posta, telefon, parola yok. Kendi cihazlarınız birbirine kodsuz gönderir.",
    "rm.l4.t": "Kayıtlı kişiler",
    "rm.l4.d": "Bir kez eşleşin, dört kelimeyi doğrulayın; o kişiye bir daha kodsuz gönderin.",
    "rm.l5.t": "Uygulamalarda tam gizlilik modu",
    "rm.l5.d": "IP adresini de gizlemesi gerekenler için: yalnızca aktarma, Tor üzerinden, boyutları gizlenmiş.",
    "rm.v1.state": "Sürüm 1.0",
    "rm.v1.when": "Denetimden sonra",
    "rm.v1.t": "Bağımsız güvenlik denetimi",
    "rm.v1.d": "Dışarıdan bir firma protokolü, kriptografiyi ve dışarıdan gelen veriyi işleyen kodu inceler. Raporun tamamı yayımlanır.",
    "rm.v2.t": "El sıkışmanın biçimsel modeli",
    "rm.v2.d": "Hibrit el sıkışmanın sözünü tuttuğunu gösteren ProVerif ya da Tamarin modeli.",
    "rm.v3.t": "Kuantum sonrası imzalar",
    "rm.v3.d": "Cihaz anahtarlarına Ed25519'un yanında ML-DSA-65 eklenir; gelecekteki bir kuantum bilgisayar bile sizi taklit edemez.",
    "rm.v4.t": "Topluluk aktarma sunucuları",
    "rm.v4.d": "İsteyenler, imzalı bir listeden başkalarının işlettiği aktarma sunucularını kullanabilir.",
    "foot.donate": "Bağış",
    "free.more": "Bütün maliyetleri görün"
  };

  const PAGE = document.body.dataset.page || "home";
  const META_DOWNLOAD = {
    en: { title: "Download TotalDrop", desc: "Download TotalDrop for Linux, Android and the command line. Windows, macOS, iOS and the browser are on the way." },
    tr: { title: "TotalDrop'u indirin", desc: "TotalDrop'u Linux, Android ve komut satırı için indirin. Windows, macOS, iOS ve tarayıcı yolda." }
  };
  const META_HOME = {
    en: { title: "TotalDrop: send files directly, encrypted end to end", desc: "TotalDrop is a free, open-source app that sends files directly from one device to another, encrypted end to end. No account, no email, no phone number." },
    tr: { title: "TotalDrop: dosyaları doğrudan, uçtan uca şifreli gönderin", desc: "TotalDrop dosyaları bir cihazdan ötekine doğrudan, uçtan uca şifreli gönderen ücretsiz ve açık kaynak bir uygulama. Hesap yok, e-posta yok, telefon numarası yok." }
  };

  const META_RECEIVE = {
    en: { title: "Receive with TotalDrop", desc: "Someone is sending you files with TotalDrop. Open the app and enter the code." },
    tr: { title: "TotalDrop ile alın", desc: "Biri size TotalDrop ile dosya gönderiyor. Uygulamayı açıp kodu girin." }
  };

  const META_DONATE = {
    en: { title: "Why TotalDrop needs donations", desc: "TotalDrop is free and has no ads or investors. Here is every cost of running it, today and on the way to version 1.0, and what donations pay for." },
    tr: { title: "TotalDrop neden bağışa ihtiyaç duyuyor", desc: "TotalDrop ücretsiz; reklamı ve yatırımcısı yok. Bugün ve 1.0'a giden yolda bütün maliyetleri ve bağışların neye gittiği." }
  };

  const META_ROADMAP = {
    en: { title: "TotalDrop roadmap", desc: "What is done, what comes next and what comes later in TotalDrop, platform by platform, on the way to version 1.0." },
    tr: { title: "TotalDrop yol haritası", desc: "TotalDrop'ta neyin bittiği, sırada ne olduğu ve sonra neyin geleceği; platform platform, 1.0 sürümüne giden yol." }
  };

  const META = PAGE === "roadmap" ? META_ROADMAP
    : PAGE === "download" ? META_DOWNLOAD
    : PAGE === "receive" ? META_RECEIVE
    : PAGE === "donate" ? META_DONATE
    : META_HOME;

  const nodes = Array.from(document.querySelectorAll("[data-i18n]"));
  const EN = {};
  nodes.forEach((el) => { EN[el.dataset.i18n] = el.textContent; });

  const store = {
    get() { try { return localStorage.getItem("td-lang"); } catch (e) { return null; } },
    set(v) { try { localStorage.setItem("td-lang", v); } catch (e) { /* storage unavailable */ } }
  };

  const toggle = document.getElementById("lang-toggle");
  const descTag = document.querySelector('meta[name="description"]');

  function apply(lang) {
    const dict = lang === "tr" ? TR : EN;
    nodes.forEach((el) => {
      const text = dict[el.dataset.i18n];
      if (typeof text === "string") el.textContent = text;
    });
    document.documentElement.lang = lang;
    document.title = META[lang].title;
    if (descTag) descTag.setAttribute("content", META[lang].desc);
    if (toggle) {
      const other = lang === "tr" ? "en" : "tr";
      toggle.textContent = other.toUpperCase();
      toggle.lang = other;
      toggle.setAttribute("aria-label", other === "tr" ? "Türkçe" : "English");
    }
  }

  const saved = store.get();
  const initial = saved === "tr" || saved === "en"
    ? saved
    : ((navigator.language || "").toLowerCase().startsWith("tr") ? "tr" : "en");
  if (initial !== "en") apply(initial);

  if (toggle) {
    toggle.addEventListener("click", () => {
      const next = document.documentElement.lang === "tr" ? "en" : "tr";
      if (document.startViewTransition) document.startViewTransition(() => apply(next));
      else apply(next);
      store.set(next);
    });
  }


  // Download page: mark the card for the device in hand, and copy commands.
  const cards = document.querySelectorAll("[data-platform]");
  if (cards.length) {
    const ua = navigator.userAgent || "";
    const hint = (navigator.userAgentData && navigator.userAgentData.platform) || "";
    const p = /android/i.test(ua) ? "android"
      : /iphone|ipad|ipod/i.test(ua) ? "ios"
      : /mac/i.test(hint || ua) ? "macos"
      : /win/i.test(hint || ua) ? "windows"
      : /linux|cros/i.test(hint || ua) ? "linux" : "";
    const mine = document.querySelector('[data-platform="' + p + '"]');
    if (mine) {
      mine.classList.add("is-yours");
      mine.parentElement.prepend(mine);
      // Where the desktop app is still planned, the command line works today.
      const cli = document.querySelector('[data-platform="cli"]');
      if (cli && ["macos", "windows"].includes(p)) mine.after(cli);
    }
  }
  // Receive link (totaldrop.app/r#7-harbor-rocket): the code is after "#",
  // so the browser never sends it to the server, and this page sends it
  // nowhere either. It only hands it to the app.
  if (PAGE === "receive") {
    window.addEventListener("hashchange", () => location.reload());
    const raw = decodeURIComponent(location.hash.slice(1)).trim().toLowerCase();
    const ok = /^[0-9]{1,6}(-[a-z]{3,8}){2,3}$/.test(raw);
    document.getElementById(ok ? "r-has-code" : "r-no-code").hidden = false;
    if (ok) {
      document.getElementById("r-code").textContent = raw;
      document.getElementById("r-open").href = "totaldrop://r/" + raw;
      const copy = document.getElementById("r-copy");
      copy.addEventListener("click", async () => {
        try {
          await navigator.clipboard.writeText(raw);
          copy.textContent = document.documentElement.lang === "tr" ? TR["r.copied"] : "Copied";
        } catch (e) { window.getSelection().selectAllChildren(document.getElementById("r-code")); }
      });
    }
  }

  document.querySelectorAll("button[data-copy]").forEach((b) => {
    b.addEventListener("click", async () => {
      const lang = document.documentElement.lang === "tr" ? "tr" : "en";
      try {
        await navigator.clipboard.writeText(b.dataset.copy);
        b.textContent = lang === "tr" ? TR["dl.copied"] : "Copied";
      } catch (e) {
        const pre = b.previousElementSibling;
        if (pre) window.getSelection().selectAllChildren(pre);
      }
    });
  });
})();
