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
     site için ayrı JSON). Her araç kendi biçiminde; aynı cümle dört kez
     çevrilir, platformlar birbirinden kayar.
  2. **Tek kaynak: ICU MessageFormat ile ARB dosyaları; biçim kuralları
     CLDR'dan.** Flutter ARB'yi doğrudan okur; Rust tarafı (Linux, CLI) aynı
     dosyaları ICU4X ile işler; site JSON alır, tarayıcının `Intl` API'sini
     kullanır.
  3. Bir çeviri hizmetinin (Crowdin, Lokalise) kendi biçimi. Kapalı kaynak,
     hesap ve ücret ister.
- **Karar:** Seçenek 2.
  - **Kaynak:** Bütün metinler depo kökündeki `l10n/` altında: `app/`
    (Android ve Linux'un ortak arayüz metinleri), `cli/` (komut satırı),
    `site/` (web sitesi). İngilizce şablondur; her dil bir dosyadır.
  - **Dil seçimi:** Sistemin tercih listesine bakılır, CLDR'ın yakınlık
    kurallarıyla en uygun çeviri seçilir (`pt-BR` yoksa `pt`, `zh-HK` için
    `zh-Hant`); hiçbiri yoksa İngilizce. Android'de cihaz dili ve Android
    13+ uygulama başına dil; Linux ve CLI'da `LANGUAGE`, `LC_ALL`,
    `LC_MESSAGES`, `LANG`; sitede `navigator.languages`, ayrıca elle seçim.
  - **Bölge biçimleri:** Sayılar, boyutlar ve çoğullar dil değil bölge
    ayarından gelir (Linux'ta `LC_NUMERIC`, Android'de sistem bölgesi);
    bölgenin rakam sistemi (ör. Arap-Hint rakamları) korunur.
  - **Yazı yönü:** Arapça, Farsça, İbranice ve Urduca sağdan sola; yön
    dilden çıkarılır, elle tutulmaz.
  - **Çeviri:** Yeni diller önce taslak çeviriyle gelir ve arayüzde
    "çeviri gözden geçiriliyor" diye belirtilir. Anadili konuşanlar
    Hosted Weblate (açık kaynak projelere ücretsiz) ya da PR ile düzeltir.
    Bir dil gözden geçirilince taslak işareti kalkar.
  - **Değişmeyenler:** Aktarım kodundaki ve doğrulamadaki kelimeler bütün
    dillerde aynı İngilizce listeden gelir; iki taraf farklı dilde olsa da
    aynı kodu yazıp aynı kelimeleri görmelidir. Protokol değişmez.
- **Neden:** Tek kaynakla bir cümle bir kez çevrilir ve Android ile Linux
  aynı metni gösterir. ICU MessageFormat ve CLDR her dilin çoğul ve sayı
  kurallarını bilir; bunları elle yazmak her dilde hata demektir. ARB,
  Flutter'ın yerel biçimi ve Weblate'in desteklediği açık bir biçimdir.
- **Gizlilik:** Seçilen dil ve bölge hiçbir sunucuya gönderilmez; site dil
  dosyasını kendi alan adından indirir.
- **Sonuçlar:**
  - Linux `Strings.qml`'deki elle yazılmış çoğul ve boyut kodu kalkar,
    yerine Rust köprüsü gelir.
  - Eksik ya da fazla anahtar, bozuk ICU mesajı ve şablonda olmayan bir
    yer tutucu CI'da hatadır.
  - Her arayüz değişikliği en az bir sağdan sola ve bir CJK dilinde de
    denenir ([tasarim.md](../tasarim.md)).
  - Ayrıntılar ve çevirmen rehberi: [ceviri.md](../ceviri.md).
- **Yeniden bakılacağı zaman:** Weblate'teki topluluk büyür ve taslak
  çevirilerin çoğu gözden geçirilirse taslak dilleri eklemeyi bırakıp
  yalnızca gözden geçirilmiş dilleri yayınlamak.
