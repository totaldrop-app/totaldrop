# totaldrop.app

TotalDrop'un web sitesi: tanıtım (`totaldrop.app`), indirme
(`download.totaldrop.app`), QR bağlantıları (`totaldrop.app/r#kod`) ve
gizlilik sayfası. Tamamen statiktir: derleme adımı yok, çerez yok, üçüncü
taraf istek yok. Yazı tipleri de sitenin kendisinden gelir.

```
public/
  index.html          Tanıtım sayfası (İngilizce varsayılan, Türkçe app.js'te)
  download/index.html İndirme sayfası; ziyaretçinin cihazına uyan kart öne gelir
  r/index.html        QR ve paylaşım bağlantıları: kodu # işaretinden sonra okur,
                      uygulamayı totaldrop://r/kod ile açar; kod sunucuya gitmez
  privacy.html        Gizlilik
  404.html            Bulunamayan her adres için; o olmadan Pages her adrese
                      ana sayfayı 200 ile döner
  styles.css          Aurora tokenları, açık ve koyu tema, hareketli işaret
  app.js              Türkçe metinler, dil düğmesi, platform algılama, /r mantığı
  favicon.svg         Dalga işareti, koyu temada renk değiştirir
  _headers            Güvenlik başlıkları (katı CSP, HSTS, no-referrer)
  _routes.json        Ara katman (functions/) yalnız sayfalarda çalışır; yazı
                      tipi, stil, betik ve çeviri dosyaları doğrudan sunulur
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
- Yeni bir metin hem `index.html`/`download/index.html`'e (İngilizce) hem
  `app.js`'teki `TR` sözlüğüne (Türkçe) eklenir.
- Üçüncü taraf kaynak (CDN, analiz, yazı tipi servisi) eklenmez. CSP bunu
  zaten engeller.
