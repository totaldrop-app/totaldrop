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
    "dl.version": "Sürüm 0.1.0-beta.2",
    "dl.date": "27 Ekim 2026",
    "dl.title": "TotalDrop'u indirin",
    "dl.lede": "Ücretsiz, açık kaynak, hesap yok. Cihazınızı seçin; şu an kullandığınız işaretli.",
    "dl.platforms": "Platformlar",
    "dl.new.title": "0.1.0-beta.2'de yeni",
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
    "dl.req.net.m": "İki cihaz aynı ağda",
    "dl.req.net.n": "Farklı ağlar arası, sıradaki aktarma sunucusuyla",
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
    "how.note": "Bugün iki cihazın aynı ağda olması gerekiyor. İnternet üzerinden gönderim, yol haritasında sıradaki aktarma sunucusuyla gelecek.",
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
    "road.p0": "Açık protokol, komut satırı istemcisi ve Android betası",
    "road.p1": "Farklı ağlardaki cihazlar için aktarma sunucusu",
    "road.p2": "Linux ve Windows uygulamaları, tarayıcıdan alma",
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
    "foot.privacy": "Gizlilik"
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

  const META = PAGE === "download" ? META_DOWNLOAD : PAGE === "receive" ? META_RECEIVE : META_HOME;

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
