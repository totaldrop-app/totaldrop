# Copilot kod incelemesi için talimatlar

TotalDrop uçtan uca şifreli, her platformdan her platforma dosya gönderen
açık kaynak bir uygulamadır. Bağlayıcı kurallar `docs/surec.md`,
`docs/tasarim.md`, `docs/ceviri.md` ve `docs/kararlar/` altındadır; burada
incelemede en sık gereken özetler var.

## İnceleme nasıl yazılır

- İncelemeyi ve yorumları Türkçe yaz.
- Koddan söz eden her bulgu `yol/dosya.ext:satır` verir.
- Emin olmadığın bulguyu "emin değilim" diye işaretle ve nasıl
  sınanacağını söyle.
- Biçim ve üslup tercihlerini bulgu yazma; biçimleyiciler CI'da çalışır.
- İncelemen ek bir okumadır, bakımcı onayının yerini tutmaz.

## Dil: hangi dosya hangi dilde

İki dil bilerek bir arada durur. Bir dosyanın dilini o dosyanın ve
yanındakilerin zaten kullandığı dille karşılaştır; hepsini Türkçe isteme.

- **Türkçe:** `docs/` altındaki belgelerin çoğu (`surec.md`, `tasarim.md`,
  `tehdit-modeli.md`, `kararlar/`), `CLAUDE.md`, PR açıklamaları, issue'lar.
- **İngilizce:** kod ve yorumları, commit iletileri, PR başlıkları
  (Conventional Commits: `fix(core): …`), `CHANGELOG.md`, `changelog.d/`
  girdileri (CHANGELOG.md'ye olduğu gibi taşınır, `changelog.d/README.md`),
  `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`,
  `docs/protocol-v0.1-notes.md`. Bunların İngilizce olması hata değildir.
- **35 dil:** kullanıcıya görünen her metin. İngilizce dosya şablondur
  (`l10n/app/app_en.arb`, `l10n/cli/cli_en.arb`, `site/public/i18n/en.json`,
  `site/privacy/en.html`). Hedef, yeni ya da değişen metnin 35 dilde
  gelmesidir; eksik dili düşük önemde yaz. Eksik çeviri İngilizce'ye düşer
  ve CI'ı kırmaz (`docs/ceviri.md`, karar 0008); engelleyici sayma.
- `CHANGELOG.md`'ye yalnızca sürümde yazılır; PR'da değişmemesi doğrudur.

## Her PR'da bak

- **Bitti tanımı** (`docs/surec.md`): hata düzeltmesinde önce hatayı
  gösteren test; `changelog.d/<issue>-<ad>.<bölüm>.md` ya da nedenini
  söyleyen `.none.md`; PR şablonundaki liste dolu.
- **İncelemeden çıkan kurallar** (`docs/surec.md`):
  - Hata bekleyen test hatanın türünü ya da kodunu denetler.
  - Zamanlama testinde sabit `sleep` yok; koşul son süreyle beklenir.
  - Durum adlandırılmış bir türdür (metin ya da bool çifti değil);
    `await` sonrası neslin ve aşamanın aynı kaldığı denetlenir.
  - Belge, gizlilik politikası, site ya da README bir süre, sınır ya da
    maliyet söylüyorsa onu sağlayan kod ve test PR'da görünür.
  - Dış metin terminale ya da arayüze yazılmadan önce kontrol karakterleri
    görünür yapılır; hata sessizce yutulmaz.
  - Araç ve taban sürümünün tek kaynağı vardır; kopyaları bir denetim
    karşılaştırır.
- **Güvenlik ve kriptografi:** gizli değerler sabit zamanlı karşılaştırılır;
  ağdan gelen uzunluk ve sayım okunmadan önce sınırlanır; yeni ayrıştırıcının
  `fuzz/` altında hedefi olur. Protokol değişirse
  `docs/protocol-v0.1-notes.md`, mimari değişirse `docs/kararlar/` güncel
  olmalı.
- **Gizli bilgiler:** `key.properties`, imza anahtarları, parolalar, API
  anahtarları depoya girmez; görürsen yüksek önemde yaz.
- **Arayüz** (`docs/tasarim.md`): sistem temasına uyar (açık ve koyu);
  Linux'ta KDE Plasma, Hyprland ve Omarchy. Sistem temasını ya da yazı
  ölçeğini yok sayan sabit renk ve boyut, sağdan sola dillerde kırılan
  yerleşim bulgudur. İstisna: işaretin (Köşe) renkleri ve sistem renk
  vermediğinde kullanılan Aurora paleti.
- **CI:** `.github/workflows/` altında üçüncü taraf eylemler commit
  SHA'sıyla sabitlenir, izinler en dardır, paketler yalnızca CI'da derlenir.
