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
- Her işin başında, değişen klasörlerin alanına göre aşağıdaki skill'leri yükle (birden çok alan varsa hepsini; branch adı yalnızca ipucudur, klasörler belirler). `software-engineering-discipline` her zaman geçerlidir; tabloda karşılığı olmayan işlerde yalnızca o yüklenir:

  | Alan | Klasör (branch ipucu) | Skill |
  | --- | --- | --- |
  | Android, ileride iOS (Flutter) | `app/` (`*android*`), `app/ios/` (`*ios*`) | `android-app-engineering`; iOS kodu için `ios-app-engineering` |
  | Linux masaüstü (Qt 6, QML, CXX-Qt) | `linux/`, `packaging/` (`*linux*`, `*packaging*`) | `fullstack-engineering` (Qt/QML için ayrı skill yok; arayüz kuralları `docs/tasarim.md`) |
  | Web sitesi | `site/` (`*site*`) | `web-design-pro`; `site/functions/` ve `site/public/_headers` için ayrıca `fullstack-engineering` ve `security-engineering-analyst` |
  | Sunucular, Worker'lar, CI | `server/`, `.github/` | `fullstack-engineering`, `security-engineering-analyst` |
  | Protokol, kriptografi, çekirdek | `crates/totaldrop-crypto`, `-proto`, `-core`, `fuzz/`, `docs/protocol-v0.1-notes.md` (`*core*`) | `applied-cryptography-review`, `security-engineering-analyst` |
  | Komut satırı | `crates/totaldrop-cli` (`*cli*`) | `fullstack-engineering` |
  | Bağımlılıklar ve tedarik zinciri | `Cargo.toml`, `Cargo.lock`, `deny.toml`, Dependabot PR'ları | `security-engineering-analyst` |
  | Güvenlik ve gizlilik | `docs/tehdit-modeli.md`, `site/public/privacy.html`, `SECURITY.md` | `security-engineering-analyst` (açık raporları `SECURITY.md`'deki gizli kanaldan gelir, herkese açık issue'dan değil) |
  | Diller ve çeviriler | `l10n/`, `crates/totaldrop-l10n` | kurallar `docs/kararlar/0008-diller-ve-bolgeler.md` ve `docs/ceviri.md` |
  | macOS, Windows | ileride `macos/`, `windows/` | `macos-app-engineering`, `windows-app-engineering` |
