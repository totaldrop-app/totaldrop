# Çeviri

TotalDrop'un bütün metinleri depo kökündeki `l10n/` klasöründe durur
([karar 0008](kararlar/0008-diller-ve-bolgeler.md)):

| Klasör | Ne | Biçim |
| --- | --- | --- |
| `l10n/app/` | Android ve Linux uygulamalarının ortak metinleri | ARB (`app_<dil>.arb`, ör. `app_pt_BR.arb`) |
| `l10n/cli/` | Komut satırı istemcisi | ARB (`cli_<dil>.arb`) |
| `site/public/i18n/` | totaldrop.app (site olduğu gibi yayınlandığı için kendi klasöründe) | JSON (`<dil>.json`) |
| `docs/readme/` | GitHub sayfası: `README.md`'nin çevirileri | Markdown (`README.<dil>.md`) |

`README.md` değişince çevirileri eskir: her çevirinin ilk satırı hangi
`README.md`'den çevrildiğini söyler, `site/tools/check-readmes.mjs` eskiyenleri
uyarı olarak bildirir ve dil listesini `--write` ile yeniden yazar.

İngilizce dosyalar şablondur. Yeni bir metin önce İngilizce dosyaya, bir
açıklamayla (`@anahtar` → `description`) eklenir; diğer diller sonra
doldurulur. Eksik bir çeviri İngilizce'ye düşer; CI eksikleri raporlar ama
kırılmaz. Bölge ya da yazı varyantı olan dillerin temel dosyası da vardır
(`app_pt.arb` ile `app_pt_BR.arb`, `app_pt_PT.arb`).

`l10n/status.json` hangi dilin gözden geçirildiğini, hangi anahtarların
hâlâ taslak olduğunu ve hangi anahtarların güvenlik açısından kritik
olduğunu tutar. Dosya yalnızca gözden geçirilmiş PR'larla değişir; PR'ı
bakımcı birleştirir.

Her dilin `noWords` alanı, o dilde "hayır" anlamına gelen cevapları `|`
ile ayırarak tutar (Türkçe: `"hayır|h|yok"`). Komut satırının "evet"
cevapları (`cliYes`) bunlardan biri olamaz. Kayıt sorusundaki `[e/H]`
gibi köşeli parantezin ikinci cevabı da "hayır" sayılır. Sorunun ilk
cevabı ise `cliYes` içinde bulunmalı ya da İngilizce `y` veya `yes`
olmalıdır; bu ikisini komut satırı her dilde "evet" sayar. CI, cevapları
komut satırının karşılaştırdığı gibi denetler: büyük-küçük harf, noktalı
ve noktasız i ve tam genişlikli harfler aynı sayılır. Aksanlı ve aksansız
yazımlar ise farklıdır, ikisini de yazın (`não|nao`, `όχι|οχι`,
`không|khong`). Eksik bir "hayır" kelimesi görürseniz `noWords` alanına
`|` ile ekleyen bir PR açın ya da PR'da belirtin.

## Çevirmenler için

- En kolay yol Weblate olacak: bir metni düzeltmek için depoyu bilmek
  gerekmez. (Bağlantı, proje Hosted Weblate'e alınınca buraya eklenecek.
  O zamana kadar uygulamalardaki "çeviri gözden geçiriliyor" notu GitHub'ı
  gösterir; Weblate açılınca `draftTranslation` metni her dilde
  güncellenmelidir.)
- PR ile de düzeltebilirsiniz: ilgili dosyayı değiştirin, CI biçimi denetler.
- Bir dili baştan sona gözden geçirdiyseniz PR'da ya da bir issue'da
  (Weblate açılınca orada da) belirtin; bakımcı `l10n/status.json`'u
  günceller ve arayüzdeki "çeviri gözden geçiriliyor" notu kalkar. Kritik
  metinler (doğrulama, yanlış kod, kabul/ret) gözden geçirilene kadar
  İngilizce görünür.

### Kurallar

- `{count}`, `{size}`, `{code}` gibi yer tutucular olduğu gibi kalır;
  sıraları cümleye göre değişebilir.
- Çoğullar ICU biçimindedir: `{count, plural, one{…} other{…}}`. Diliniz
  başka çoğul biçimleri istiyorsa (`zero`, `two`, `few`, `many`) ekleyin;
  CLDR kuralları uygulanır.
- `=0`, `=1` ve `=2`, Flutter'da `zero`, `one` ve `two` ile aynı durumdur;
  ikisini birden yazmayın. Rusça, Ukraynaca gibi dillerde `one` 21, 31…
  için de seçilir, bu yüzden `one` cümlesine sayıyı (`{count}`) yazın.
  Başka tam sayılar (`=5`) kullanılamaz.
- Güvenlik metinlerindeki yer tutucular (`{code}`, `{words}`, `{folder}`,
  `{name}`) her durumda İngilizce'deki kadar geçer ve bir durumu seçmek
  için kullanılamaz.
- Kesme işareti düz metindir; kaçış gerekmez (`L'envoi`).
- Çevirilerde bağlantı, HTML ve görünmez yön karakterleri (U+202A–U+202E,
  U+2066–U+2069) kullanılmaz; CI reddeder.
- Aktarım kodu (`7-harbor-rocket`) ve doğrulama kelimeleri çevrilmez:
  iki cihaz farklı dilde olsa da aynı kelimeleri görmelidir.
- "TotalDrop" çevrilmez ve harf değiştirilmez.
- Ton: kısa, sade, emir kipinde düğmeler ("Gönder", "Kodu oluştur");
  ünlem ve pazarlama dili yok.

## Desteklenen diller

Taslak çeviriyle gelenler (gözden geçirilmedikçe arayüzde belirtilir):
Arapça (`ar`), Bengalce (`bn`), Çekçe (`cs`), Danca (`da`), Almanca (`de`),
Yunanca (`el`), İspanyolca (`es`), Farsça (`fa`), Fince (`fi`), Fransızca
(`fr`), İbranice (`he`), Hintçe (`hi`), Macarca (`hu`), Endonezce (`id`),
İtalyanca (`it`), Japonca (`ja`), Korece (`ko`), Malayca (`ms`), Norveççe
(`nb`), Felemenkçe (`nl`), Lehçe (`pl`), Portekizce (Brezilya `pt-BR`,
Portekiz `pt-PT`), Rumence (`ro`), Rusça (`ru`), İsveççe (`sv`), Svahili
(`sw`), Tayca (`th`), Ukraynaca (`uk`), Urduca (`ur`), Vietnamca (`vi`),
Çince (Basitleştirilmiş `zh-Hans`, Geleneksel `zh-Hant`).

Gözden geçirilmiş: İngilizce (`en`), Türkçe (`tr`).

Taslaklar Claude (Anthropic) ile üretildi ve proje lisansı altındadır.

### Yeni bir dil eklemek

1. `l10n/app/`, `l10n/cli/` ve `site/public/i18n/` için şablondan
   dosyaları kopyalayıp çevirin (varyantlı dillerde temel dosyayla birlikte).
2. `l10n/status.json`'a dili ekleyin. Komut satırı dosyası olan bir dil
   için "hayır" cevaplarını da (`noWords`) yazın; CI, komut satırı dosyası
   olup `noWords` alanı olmayan dili reddeder.
3. Android `localeConfig` bu dosyalardan üretilir; CI üretilen dosyaların
   güncel olduğunu denetler. Site dil listesine (`site/public/lang.js`
   `LANGUAGES`) dili kendi adıyla ekleyin; CI listenin `i18n/` dosyalarıyla,
   `REVIEWED` listesinin de `l10n/status.json` ile aynı olduğunu denetler.
   Gözden geçirilmemiş bir dilde site, başlığın altında kısa bir taslak
   notu gösterir.
4. Yazı sistemi yeni ise Linux paketlerine ilgili Noto yazı tipini ekleyin
   ve ekran görüntüsüyle deneyin.
