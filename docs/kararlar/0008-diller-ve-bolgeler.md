# 0008: Her dil, her bölge ayarı

- **Durum:** Kabul, 28 Ekim 2026
- **Bağlam:** Uygulama yalnızca İngilizce ve Türkçe. Hedef "her dosyayı,
  herkese, her yere"; bu, kullanıcının cihazında seçili dil ve bölge neyse
  onunla çalışmayı gerektirir. Bugün metinler iki yerde ayrı tutuluyor
  (Android ARB dosyaları, Linux `Strings.qml`), çoğul ve boyut biçimi elle
  yazılmış ve yalnızca bu iki dili biliyor; CLI yalnızca İngilizce; sitenin
  sözlüğü `app.js` içinde.
- **Seçenekler:**
  1. Her platform kendi aracıyla (Flutter gen_l10n, Qt Linguist, gettext,
     site için ayrı JSON). Uygulamaların ortak cümleleri iki kez çevrilir,
     Android ile Linux birbirinden kayar.
  2. **Ortak arayüz metinleri tek kaynakta: ICU MessageFormat ile ARB
     dosyaları; biçim kuralları CLDR'dan.**
  3. Bir çeviri hizmetinin kendi biçimi. Biçime ve hizmete bağımlılık.
- **Karar:** Seçenek 2. Açık bir biçim (ARB), hizmete bağımlılık yok.

## Kaynak

- Android ve Linux'un ortak metinleri `l10n/app/`, komut satırının
  metinleri `l10n/cli/` altında ARB dosyalarıdır. Site olduğu gibi
  yayınlandığı için kendi metinlerini `site/public/i18n/` altında JSON
  olarak tutar. Yani üç kaynak var: iki uygulama aynı cümleyi paylaşır;
  CLI ve site kendi cümlelerini taşır, ortak olanları (hata mesajları,
  boyut birimleri) uygulama dosyasından alır.
- Dosya adları Flutter'ın kuralıyla alt çizgilidir (`app_pt_BR.arb`,
  `app_zh_Hant.arb`); belgede dil kodları BCP 47 ile yazılır (`pt-BR`).
  Bölge ya da yazı varyantı olan her dilin bir de temel dosyası vardır
  (`app_pt.arb`, `app_zh.arb`): gen_l10n bunu ister ve en yakın eşleşme
  ona düşer.
- İngilizce şablondur. Şablonda çoğullar `one{…}`/`other{…}` ile yazılır;
  `=0`, `=1` yalnızca o sayıya özel bir cümle gerektiğinde kullanılır
  (Rusçada 21 "one" kategorisindedir, `=1` değildir).

## Mesaj biçimi

- ICU4X'te MessageFormat yok; Flutter mesajları derleme sırasında kendisi
  çözer. Bu yüzden Rust (Linux, CLI) ve site için aynı küçük alt kümenin
  ayrı ama aynı testlerle doğrulanan çözücüleri yazılır: `{ad}`,
  `{n, plural, =N{…} zero/one/two/few/many/other{…}}`,
  `{x, select, …, other{…}}`. `selectordinal`, `#` kısaltması ve kaçış
  yoktur.
- Kesme işareti (`'`) her yerde düz metindir (gen_l10n `use-escaping`
  kapalı); Fransızca ve İtalyanca cümleler her platformda aynı görünür.
- Ortak test vektörleri `l10n/tests/messages.json` içindedir; Rust ve site
  çözücüleri ile Flutter'ın ürettiği kod aynı çıktıyı vermek zorundadır.
- Çoğulu ICU4X (`icu_plurals`), sayıları `icu_decimal` biçimler; sitede
  tarayıcının `Intl.PluralRules` ve `Intl.NumberFormat`'ı. Boyut birimleri
  ("1,5 MB") ARB'de çevrilebilir bir kalıptır, sayı yerel biçimle girer.

## Dil ve bölge seçimi

- **Dil:** Sistemin tercih listesine bakılır. Her tercih için CLDR'ın geri
  düşme zinciri (ICU4X `LocaleFallbacker`, ör. `pt-AO` → `pt-PT` → `pt`)
  ve olası alt etiketler (`LocaleExpander`, ör. `zh-HK` → `zh-Hant`)
  denenir; hiçbiri yoksa İngilizce. Bu tam bir CLDR dil eşleştiricisi
  değil, geri düşme zinciridir. `no` ve `nn`, `nb`'ye eşlenir.
  - Android: cihaz dili ve Android 13+ uygulama başına dil
    (`localeConfig`, `l10n/`'dan üretilir). Flutter'ın varsayılan seçimi
    `zh-HK`'yi Basitleştirilmiş Çince'ye düşürebildiği için özel bir
    `localeListResolutionCallback` aynı kuralları uygular.
  - Linux ve CLI: gettext sırası, `LANGUAGE` (iki nokta ile ayrılmış liste;
    yerel ayar `C` ise yok sayılır) > `LC_ALL` > `LC_MESSAGES` > `LANG`.
  - Site: `navigator.languages`, ayrıca elle seçim (tarayıcıda saklanır).
- **Çoğullar** gösterilen cümlenin dilinin kurallarıyla seçilir; bölge
  ayarı çoğulu değiştirmez.
- **Sayılar ve boyutlar** bölge ayarını izler: Linux'ta `LC_ALL` >
  `LC_NUMERIC` > `LANG` (Qt de böyle yapar), Android'de cihaz bölgesi.
  Bölgenin rakam sistemi (ör. Arap-Hint rakamları) Linux ve CLI'da
  korunur; Android'de ve sitede bu ayrıca test edilmeden iddia edilmez.
- **Yazı yönü:** Dilden çıkarılır (Arapça, Farsça, İbranice, Urduca sağdan
  sola). Linux'ta `QGuiApplication::setLayoutDirection` seçilen dilden
  ayarlanır; Qt'nin kendi iletişim kutusu çevirileri (`qtbase_*.qm`)
  yüklenir.
- **CLI:** Terminaller iki yönlü metni güvenilir göstermediği için sağdan
  sola diller komut satırında şimdilik İngilizce'ye düşer. Makineye yönelik
  çıktı (kodun kendisi, ileride `--json`) hiçbir dilde çevrilmez, ASCII
  rakamla yazılır.

## Çeviriler ve gözden geçirme

- **Taslaklar:** Yeni diller Claude'un (Anthropic) ürettiği taslak
  çevirilerle gelir; proje lisansı altındadır. Kimin, hangi araçla
  çevirdiği `l10n/status.json`'da yazar.
- **Durum dosyası:** `l10n/status.json` her dil için gözden geçirilip
  geçirilmediğini ve gözden geçirmeden sonra taslak olarak eklenen
  anahtarları tutar. Bu dosyayı yalnızca bakımcı PR ile değiştirir; çeviri
  dosyasının içindeki bir işaret buna yetmez. Taslak içeren dillerde
  arayüzde "çeviri gözden geçiriliyor" notu görünür.
- **Güvenlikle ilgili metinler** (doğrulama kelimeleri, yanlış kod uyarısı,
  kabul/ret soruları) `status.json`'da "kritik" listesindedir. Bu
  anahtarların taslakları gösterilmez; gözden geçirilene kadar İngilizce
  görünür.
- **Düzeltmeler:** Anadili konuşanlar Hosted Weblate (açık kaynak projeler
  için başvuruyla ücretsiz) ya da PR ile düzeltir. Weblate'ten gelen her
  değişiklik de PR olarak gelir ve [0007](0007-surec.md)'deki incelemeden
  geçer. Weblate'te `icu-message-format` denetimi açıktır.

## CI denetimleri

- **Hata:** şablonda olmayan anahtar; bozuk ICU mesajı; şablonda olmayan
  yer tutucu; kritik bir yer tutucunun (`{code}`, `{words}`, `{folder}`,
  `{name}`) eksik ya da birden çok olması; çevirilerde yön denetim
  karakterleri (U+202A–U+202E, U+2066–U+2069) ve sıfır genişlikli
  karakterler; URL ve HTML; temel dil dosyası olmayan bölge dosyası.
- **Rapor (hata değil):** eksik çeviriler. Eksik metin İngilizce'ye düşer;
  her yeni İngilizce cümle 30 dilde CI'ı kırmaz.
- Sahte diller: CI `en-XA` (aksanlı ve %40 uzun) ve `ar-XB` (sağdan sola)
  üretir; ekran denetimleri bunlarla da yapılır.

## Güvenlik ve erişilebilirlik

- Karşı taraftan gelen metin (dosya ve klasör adları) cümleye yön
  yalıtımıyla (FSI…PDI; Flutter'da `Directionality`/`BidiFormatter`)
  girer. Dosya adlarındaki yön denetim karakterleri görünür hale getirilir
  ("fatura‮fdp.exe" gibi uzantı gizleme saldırısına karşı).
- Aktarım kodu her dilde İngilizce kelimelerle, soldan sağa yalıtılmış
  olarak gösterilir. Kod girişinde otomatik büyük harf ve düzeltme kapalı;
  harfler dile bağlı olmadan küçültülür (Türkçe `İ` ve `ı` açıkça `i`'ye),
  Arap-Hint ve Farsça rakamlar kabul edilir.
- Ekran okuyucular kodu ve doğrulama kelimelerini İngilizce sesle okur
  (Flutter'da yerel ayarlı `Semantics`); TalkBack ve Orca ile denenir.
- Linux paketleri Noto yazı tiplerini (Arapça, Bengalce, Devanagari, Tay
  vb.) bağımlılık olarak ister; satır yüksekliği Tayca ve Arapça'da
  denenir.

## Gizlilik

- Seçilen dil hiçbir üçüncü tarafa gitmez. Site dil dosyasını kendi alan
  adından indirir; bu istek, tarayıcının zaten gönderdiği
  `Accept-Language` gibi, Cloudflare'e hangi dilin istendiğini gösterir.
  Uygulamalar dili hiçbir sunucuya göndermez.

## Değişmeyenler

- Aktarım kodundaki ve doğrulamadaki kelimeler bütün dillerde aynı
  İngilizce listeden gelir; iki taraf farklı dilde olsa da aynı kodu yazıp
  aynı kelimeleri görmelidir. Protokol değişmez.

## Neden

İki uygulama aynı cümleyi gösterir ve bir kez çevrilir. ICU MessageFormat
ve CLDR her dilin çoğul ve sayı kurallarını bilir; bunları elle yazmak her
dilde hata demektir. ARB, Flutter'ın yerel biçimi ve Weblate'in desteklediği
açık bir biçimdir.

## Sonuçlar

- Linux `Strings.qml`'deki elle yazılmış çoğul ve boyut kodu kalkar, yerine
  `totaldrop-l10n` crate'i gelir; CLI da aynı crate'i kullanır.
- Kapsam yalnızca uygulama içi metinler değil: Android'in kendi metinleri
  (uygulama adı, bildirim kanalları, paylaşım hedefi etiketleri), Linux
  `.desktop` ve AppStream çevirileri, Play Store açıklamaları da bu
  kaynaklardan beslenir.
- Her arayüz değişikliği en az bir sağdan sola ve bir CJK dilinde de
  denenir ([tasarim.md](../tasarim.md)).
- Ayrıntılar ve çevirmen rehberi: [ceviri.md](../ceviri.md).

## Yeniden bakılacağı zaman

Weblate'teki topluluk büyür ve taslakların çoğu gözden geçirilirse, yeni
dilleri yalnızca gözden geçirildikten sonra yayınlamak.
