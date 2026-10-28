# TotalDrop

- Proje kıdemli bir yazılım ekibinin disipliniyle yürür: yazılım mühendisliği yöntemi kısaltılmadan uygulanır. Çalışma süreci `docs/surec.md` bağlayıcıdır:
  - her iş bir issue ile başlar, kısa ömürlü bir branch'te yapılır ve PR ile `main`'e girer;
  - bütün CI kontrolleri yeşil olmadan ve kodu yazmamış bir inceleyici (ayrı bir inceleme ajanı) okumadan birleştirme yapılmaz;
  - "bitti tanımı" ve sürüm listesi adım adım izlenir; yayınlanan her paket CI'da derlenir.
- Mimari kararlar `docs/kararlar/`, protokol kararları `docs/protocol-v0.1-notes.md`, tehdit modeli `docs/tehdit-modeli.md` içindedir.
- Belgeler ve kullanıcıya yazılan metinler Türkçe.
- Tasarım kuralları: `docs/tasarim.md`. Bu kurallar bağlayıcıdır, özellikle şu ikisi:
  - Arayüz her platformda sistem temasına yüzde yüz uyar. Linux'ta KDE, Hyprland ve Omarchy desteklenir. Her adımda üçü de kontrol edilir.
  - Aktarım sırasında işaret, web sitesindeki animasyonun aynısını oynatır.
- Gizli bilgiler (`key.properties`, imza anahtarı parolası) okunmaz ve depoya eklenmez.
- Paketler yalnızca CI'da derlenir; CI çıktısı (ör. Play Console için `.aab`) yerelde `~/Projects/totaldrop/dist` altına indirilir.
- Her işte konuya göre ilgili skill kendiliğinden yüklenir; branch'in ve değişen klasörlerin alanı hangisiyse o (birden çok alan varsa hepsi). `software-engineering-discipline` her zaman geçerlidir:

  | Alan | Branch / klasör | Skill |
  | --- | --- | --- |
  | Android | `*/android-*`, `app/` | `android-app-engineering` |
  | Linux masaüstü | `*/linux-*`, `linux/`, `packaging/` | `fullstack-engineering`, tasarım için `web-design-pro` yerine `docs/tasarim.md` |
  | Web sitesi | `*/site-*`, `site/` | `web-design-pro` |
  | Sunucular, Worker'lar, CI | `server/`, `.github/` | `fullstack-engineering`, `security-engineering-analyst` |
  | Protokol, kriptografi | `crates/totaldrop-crypto`, `crates/totaldrop-proto`, `crates/totaldrop-core` | `applied-cryptography-review`, `security-engineering-analyst` |
  | Güvenlik ve gizlilik (tehdit modeli, gizlilik politikası, açık) | `docs/tehdit-modeli.md`, `site/public/privacy.html`, `type: security` issue'ları | `security-engineering-analyst` |
  | iOS, macOS, Windows | ileride `ios/`, `macos/`, `windows/` | `ios-app-engineering`, `macos-app-engineering`, `windows-app-engineering` |
