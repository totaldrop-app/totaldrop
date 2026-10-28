# Çeviri

TotalDrop'un bütün metinleri depo kökündeki `l10n/` klasöründe durur
([karar 0008](kararlar/0008-diller-ve-bolgeler.md)):

| Klasör | Ne | Biçim |
| --- | --- | --- |
| `l10n/app/` | Android ve Linux uygulamalarının ortak metinleri | ARB (`app_<dil>.arb`) |
| `l10n/cli/` | Komut satırı istemcisi | ARB (`cli_<dil>.arb`) |
| `l10n/site/` | totaldrop.app | JSON (`<dil>.json`) |

İngilizce dosyalar şablondur. Yeni bir metin önce İngilizce dosyaya, bir
açıklamayla (`@anahtar` → `description`) eklenir; diğer diller sonra
doldurulur. Eksik bir çeviri İngilizce'ye düşer.

## Çevirmenler için

- En kolay yol Weblate: bir metni düzeltmek için depoyu bilmek gerekmez.
  (Bağlantı, proje Hosted Weblate'e alınınca buraya eklenecek.)
- PR ile de düzeltebilirsiniz: ilgili dosyayı değiştirin, CI biçimi denetler.
- Bir dili baştan sona gözden geçirdiyseniz dosyadaki `"@@x-reviewed"`
  değerini `true` yapın; arayüzdeki "çeviri gözden geçiriliyor" notu kalkar.

### Kurallar

- `{count}`, `{size}`, `{code}` gibi yer tutucular olduğu gibi kalır;
  sıraları cümleye göre değişebilir.
- Çoğullar ICU biçimindedir: `{count, plural, =1{…} other{…}}`. Diliniz
  başka çoğul biçimleri istiyorsa (`zero`, `one`, `two`, `few`, `many`)
  ekleyin; CLDR kuralları uygulanır.
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

Listede olmayan bir dil eklemek için o dilin dosyalarını şablondan
kopyalayıp çevirmek yeterli; dil seçimi ve biçimler otomatik çalışır.
