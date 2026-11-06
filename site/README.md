# totaldrop.app

TotalDrop'un web sitesi: tanıtım (`totaldrop.app`), indirme
(`download.totaldrop.app`), QR bağlantıları (`totaldrop.app/r#kod`) ve
gizlilik sayfası. Tamamen statiktir: derleme adımı yok, çerez yok, üçüncü
taraf istek yok. Yazı tipleri de sitenin kendisinden gelir.

```
public/
  index.html          Tanıtım sayfası (metni İngilizce; diğer diller i18n/'den)
  download/index.html İndirme sayfası; ziyaretçinin cihazına uyan kart öne gelir
  r/index.html        QR ve paylaşım bağlantıları: kodu # işaretinden sonra okur,
                      uygulamayı totaldrop://r/kod ile açar; kod sunucuya gitmez
  brand/index.html    Marka sayfası: işaret, hareket, renk, yazı, düğmeler, dosyalar
  brand/files/        İndirilen logo dosyaları; site/tools/brand-files.mjs üretir
  team/index.html     Ekip sayfası: kişiler ve disipline göre iş; isimler,
                      biyografiler ve sayılar GitHub'dan (tools/team.mjs üretir)
  team/avatars/       Ekipteki kişilerin GitHub profil resimleri, kopyalanmış;
                      sayfa GitHub'dan hiçbir şey yüklemez
  privacy.html        Gizlilik politikası, İngilizce (esas metin)
  privacy/<dil>.html  Aynı politika diğer 34 dilde; ikisi de üretilir (aşağıda)
  404.html            Bulunamayan her adres için; o olmadan Pages her adrese
                      ana sayfayı 200 ile döner
  styles.css          Aurora tokenları, açık ve koyu tema, dönen işaret
  lang.js             Dil listesi, gözden geçirilmiş diller, dil seçimi
  app.js              Çeviriyi uygulama, taslak notu, platform algılama, /r mantığı
  i18n/<dil>.json     Her dilin metinleri (docs/ceviri.md)
  favicon.svg         Köşe işareti, koyu temada renk değiştirir
  favicon.ico         İşaret, 16, 32 ve 48 px
  apple-touch-icon.png
                      iOS için mor zeminde kâğıt rengi işaret, 180 px; bu
                      ikisi tools/favicons.mjs ile üretilir (aşağıda)
  _headers            Güvenlik başlıkları (katı CSP, HSTS, no-referrer)
  _routes.json        Ara katman (functions/) yalnız sayfalarda çalışır; yazı
                      tipi, stil, betik ve çeviri dosyaları doğrudan sunulur
                      (www'de de yönlendirmesiz; sayfalar yine yönlenir)
  fonts/              Bricolage Grotesque ve Atkinson Hyperlegible (SIL OFL 1.1)
privacy/<dil>.html    Gizlilik politikasının her dildeki metni (yalnızca <article> içi)
tools/privacy.mjs     Gizlilik sayfalarını bu metinlerden üretir (--write) ve denetler
tools/brand-files.mjs Marka dosyalarını işaretin geometrisinden üretir (--write, --png)
                      ve CI'da güncel olduklarını denetler
team/team.json        Ekip sayfasında kim, hangi rolle; disiplinler ve depo yolları
team/github.json      GitHub'dan alınan anlık görüntü (profil, katkı sayıları)
tools/team.mjs        Ekip sayfasını anlık görüntüden üretir (--write), yenisini
                      GitHub'ın herkese açık API'sinden alır (--fetch) ve CI'da
                      denetler; yeni kişi yalnızca kendi izniyle eklenir
tools/asset-versions.mjs Sayfalardaki stil ve betik adreslerine dosyanın özetini
                      ekler (`/styles.css?v=…`, --write) ve CI'da denetler
tools/wordmark.py     "TotalDrop" yazısının çizgilerini sitenin yazı tipinden çıkarır
functions/
  _middleware.js      www → kök yönlendirmesi; download alt alan adı → /download/
wrangler.toml         Cloudflare Pages projesi "totaldrop"
```

## Yayınlama

Site Cloudflare Pages'te `totaldrop` projesidir. Yayın CI'dan, yalnızca
`main`'den ve depo yöneticisinin `cloudflare` ortamındaki onayıyla yapılır
(`.github/workflows/deploy.yml`); ardından `server/tools/probe.mjs site`
ana sayfayı ve indirme sayfasını dener. Kurulum ve geri alma:
[docs/surec.md](../docs/surec.md), "Dağıtım" bölümü.

## Kurallar

- Tasarım [docs/tasarim.md](../docs/tasarim.md)'ye uyar; işaretin animasyonu
  uygulamalardakiyle aynıdır.
- Yeni bir metin sayfaya (İngilizce, `data-i18n` anahtarıyla) ve
  `i18n/en.json`'a eklenir; diğer diller [docs/ceviri.md](../docs/ceviri.md)'ye
  göre doldurulur. `node tools/check-i18n.mjs` hepsini denetler.
- `styles.css`, `app.js`, `lang.js` ya da `msg.js` değişince
  `node tools/asset-versions.mjs --write` ve `node tools/privacy.mjs --write`
  çalıştırın. Sayfalar bu dosyaları özetli adresle ister; Cloudflare `.css`
  ve `.js` dosyalarını tarayıcıda 4 saat tuttuğu için özet değişmezse
  ziyaretçi yeni sayfayı eski stille görür (#291).
- Üçüncü taraf kaynak (CDN, analiz, yazı tipi servisi) eklenmez. CSP bunu
  zaten engeller.
- Gizlilik politikası her dilde kendi adresindedir (`/privacy`, `/privacy/<dil>`).
  Metni `site/privacy/<dil>.html`'de değiştirin, sonra
  `node tools/privacy.mjs --write` ile sayfaları yeniden üretin. Her çevirinin
  ilk satırı, çevrildiği İngilizce metnin özetidir
  (`<!-- Translated from en.html <özet>. -->`). İngilizce metin değişince
  özet tutmaz ve CI 34 çevirinin hepsini, güncellenip özetleri yenilenene
  kadar reddeder; böylece politika değişince 35 dil aynı PR'da güncellenir.
  CI ayrıca her çevirinin İngilizce ile aynı yapıda olduğunu (başlıklar,
  paragraflar, bağlantılar, `code` içerikleri, sürüm numaraları, Linux
  yolları, İngilizcedeki her sayı) ve sayfaların güncel olduğunu denetler.
- Site simgeleri işaretten üretilir. İşaretin yolu ve rengi
  `app/assets/logo.svg`, `favicon.svg` ve `tools/favicons.mjs`'nin başındaki
  sabitlerde aynıdır. İşaret değişirse üçünü birlikte düzeltin, sonra
  `NODE_PATH=$(npm root -g) node tools/favicons.mjs --write` (Playwright
  gerekir) ile `favicon.ico` ve `apple-touch-icon.png`'yi yeniden üretip
  ekleyin. Her sayfanın `<head>`'i üç simge satırını taşır;
  `node tools/favicons.mjs` (CI) dosyaların boyutlarını, satırları ve
  işaretin üç yerde aynı olduğunu denetler. Google simgeyi yalnız kare ve
  48 pikselin katı (ya da SVG) olarak, ana sayfadaki `rel="icon"` ile
  gösterir. Büyük bir PNG `rel="icon"` olarak eklenmez: tarayıcı onu SVG'ye
  tercih edip koyu temadaki rengi kaybedebilir.
