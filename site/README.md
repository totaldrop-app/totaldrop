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
  privacy.html        Gizlilik
  404.html            Bulunamayan her adres için; o olmadan Pages her adrese
                      ana sayfayı 200 ile döner
  styles.css          Aurora tokenları, açık ve koyu tema, dönen işaret
  lang.js             Dil listesi, gözden geçirilmiş diller, dil seçimi
  app.js              Çeviriyi uygulama, taslak notu, platform algılama, /r mantığı
  i18n/<dil>.json     Her dilin metinleri (docs/ceviri.md)
  favicon.svg         Köşe işareti, koyu temada renk değiştirir
  _headers            Güvenlik başlıkları (katı CSP, HSTS, no-referrer)
  _routes.json        Ara katman (functions/) yalnız sayfalarda çalışır; yazı
                      tipi, stil, betik ve çeviri dosyaları doğrudan sunulur
                      (www'de de yönlendirmesiz; sayfalar yine yönlenir)
  fonts/              Bricolage Grotesque ve Atkinson Hyperlegible (SIL OFL 1.1)
functions/
  _middleware.js      www → kök yönlendirmesi; download alt alan adı → /download/
wrangler.toml         Cloudflare Pages projesi "totaldrop"
```

## Yayınlama

Site Cloudflare Pages'te `totaldrop` projesidir. Bu klasörden:

```bash
npx wrangler pages deploy public --project-name totaldrop --branch main
```

Yayın şimdilik elle yapılır; CI yalnızca kontrol eder (`site.yml`).

## Kurallar

- Tasarım [docs/tasarim.md](../docs/tasarim.md)'ye uyar; işaretin animasyonu
  uygulamalardakiyle aynıdır.
- Yeni bir metin sayfaya (İngilizce, `data-i18n` anahtarıyla) ve
  `i18n/en.json`'a eklenir; diğer diller [docs/ceviri.md](../docs/ceviri.md)'ye
  göre doldurulur. `node tools/check-i18n.mjs` hepsini denetler.
- Üçüncü taraf kaynak (CDN, analiz, yazı tipi servisi) eklenmez. CSP bunu
  zaten engeller.
