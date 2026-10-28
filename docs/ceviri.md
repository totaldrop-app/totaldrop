# Çeviri

TotalDrop'un bütün metinleri depo kökündeki `l10n/` klasöründe durur
([karar 0008](kararlar/0008-diller-ve-bolgeler.md)):

| Klasör | Ne | Biçim |
| --- | --- | --- |
| `l10n/app/` | Android ve Linux uygulamalarının ortak metinleri | ARB (`app_<dil>.arb`, ör. `app_pt_BR.arb`) |
| `l10n/cli/` | Komut satırı istemcisi | ARB (`cli_<dil>.arb`) |
| `site/public/i18n/` | totaldrop.app (site olduğu gibi yayınlandığı için kendi klasöründe) | JSON (`<dil>.json`) |

İngilizce dosyalar şablondur. Yeni bir metin önce İngilizce dosyaya, bir
açıklamayla (`@anahtar` → `description`) eklenir; diğer diller sonra
doldurulur. Eksik bir çeviri İngilizce'ye düşer; CI eksikleri raporlar ama
kırılmaz. Bölge ya da yazı varyantı olan dillerin temel dosyası da vardır
(`app_pt.arb` ile `app_pt_BR.arb`, `app_pt_PT.arb`).

`l10n/status.json` hangi dilin gözden geçirildiğini, hangi anahtarların
hâlâ taslak olduğunu ve hangi anahtarların güvenlik açısından kritik
olduğunu tutar. Bu dosyayı bakımcı değiştirir.

## Çevirmenler için

- En kolay yol Weblate: bir metni düzeltmek için depoyu bilmek gerekmez.
  (Bağlantı, proje Hosted Weblate'e alınınca buraya eklenecek.)
- PR ile de düzeltebilirsiniz: ilgili dosyayı değiştirin, CI biçimi denetler.
- Bir dili baştan sona gözden geçirdiyseniz PR'da ya da Weblate'te
  belirtin; bakımcı `l10n/status.json`'u günceller ve arayüzdeki "çeviri
  gözden geçiriliyor" notu kalkar. Kritik metinler (doğrulama, yanlış kod,
  kabul/ret) gözden geçirilene kadar İngilizce görünür.

### Kurallar

- `{count}`, `{size}`, `{code}` gibi yer tutucular olduğu gibi kalır;
  sıraları cümleye göre değişebilir.
- Çoğullar ICU biçimindedir: `{count, plural, one{…} other{…}}`. Diliniz
  başka çoğul biçimleri istiyorsa (`zero`, `two`, `few`, `many`) ekleyin;
  CLDR kuralları uygulanır. `=1{…}` yalnızca tam olarak bir için özel bir
  cümle gerekiyorsa kullanılır (Rusçada 21 de `one`'dır).
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
2. `l10n/status.json`'a dili ekleyin.
3. Android `localeConfig` ve site dil listesi bu dosyalardan üretilir;
   CI üretilen dosyaların güncel olduğunu denetler.
4. Yazı sistemi yeni ise Linux paketlerine ilgili Noto yazı tipini ekleyin
   ve ekran görüntüsüyle deneyin.
