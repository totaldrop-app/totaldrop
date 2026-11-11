# Copilot kod incelemesi için talimatlar

TotalDrop her platformdan her platforma uçtan uca şifreli dosya gönderen
açık kaynak bir uygulamadır. Bu dosya Copilot'un PR incelemesinde neye
bakacağını ve neyi hata saymayacağını söyler. Bağlayıcı kurallar
`docs/surec.md`, `docs/tasarim.md`, `docs/ceviri.md` ve `docs/kararlar/`
altındadır; burada yalnızca incelemede en sık gereken özetleri var.

## İnceleme nasıl yazılır

- İncelemeyi ve yorumları Türkçe yaz.
- Kodun bir yerinden söz eden her bulgu `yol/dosya.ext:satır` verir.
- Emin olmadığın bir bulguyu "emin değilim" diye işaretle; bir davranışı
  doğrulayamıyorsan nasıl sınanacağını söyle.
- Biçim ve üslup tercihlerini bulgu olarak yazma; `cargo fmt`,
  `dart format`, `prettier` ve linter'lar CI'da çalışır.
- Senin incelemen ek bir okumadır. Bakımcının onayının yerini tutmaz.

## Dil: hangi dosya hangi dilde

Depoda iki dil bilerek bir arada durur. Bir dosyanın dilini, o dosyanın ve
yanındaki dosyaların zaten kullandığı dille karşılaştır; hepsini Türkçe
isteme.

- **Türkçe:** `docs/` altındaki belgelerin çoğu (`surec.md`, `tasarim.md`,
  `tehdit-modeli.md`, `kararlar/`), `CLAUDE.md`, PR açıklamaları ve issue'lar.
- **İngilizce:** kod, kod yorumları, commit iletileri ve PR başlıkları
  (Conventional Commits: `fix(core): …`), `CHANGELOG.md` ve `changelog.d/`
  girdileri (`changelog.d/README.md`: girdi CHANGELOG.md'ye olduğu gibi
  taşınır), `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`,
  `docs/protocol-v0.1-notes.md`.
- **35 dil:** kullanıcıya görünen her metin. İngilizce dosya şablondur
  (`l10n/app/app_en.arb`, `l10n/cli/cli_en.arb`, `site/public/i18n/en.json`,
  `site/privacy/en.html`); yeni ya da değişen bir anahtar bütün dil
  dosyalarında olmalı. Yalnızca İngilizce ve Türkçe eklenmiş bir metin
  eksiktir. README çevirileri `docs/readme/` altındadır.

## Her PR'da bak

- **Bitti tanımı** (`docs/surec.md`): hata düzeltmesinde önce hatayı
  gösteren bir test; `changelog.d/<issue>-<ad>.<bölüm>.md` girdisi ya da
  nedenini söyleyen bir `.none.md`; PR şablonundaki liste dolu.
- **İncelemeden çıkan kurallar** (`docs/surec.md`):
  - Hata bekleyen test hatanın türünü ya da kodunu denetler; "bir hata
    oldu" yetmez.
  - Zamanlama testinde sabit `sleep` yok; koşul bir son süreyle beklenir.
  - Durum bir metin ya da bool çifti değil, adlandırılmış bir türdür;
    `await` sonrası işlem neslin ve aşamanın aynı kaldığını denetler.
  - Belge, gizlilik politikası, site ya da README bir süre, sınır ya da
    maliyet söylüyorsa, o sözü sağlayan kod ve test PR'da görünür.
  - Dışarıdan gelen metin terminale ya da arayüze yazılmadan önce kontrol
    karakterleri görünür hâle getirilir; hata sessizce yutulmaz.
  - Bir araç ya da taban sürümünün tek kaynağı vardır; başka yerde geçen
    kopya bir denetimle karşılaştırılır.
- **Güvenlik ve kriptografi:** gizli değerler sabit zamanlı karşılaştırılır;
  ağdan gelen her uzunluk ve sayım okunmadan önce sınırlanır; yeni bir
  ayrıştırıcının `fuzz/` altında hedefi olur. Protokol davranışı değişiyorsa
  `docs/protocol-v0.1-notes.md`, mimari değişiyorsa `docs/kararlar/` güncel
  olmalı.
- **Gizli bilgiler:** `key.properties`, imza anahtarları, parolalar ve API
  anahtarları depoya hiçbir biçimde girmez. Görürsen yüksek önemde yaz.
- **Arayüz:** her platformda sistem temasına uyar (açık ve koyu); Linux'ta
  KDE Plasma, Hyprland ve Omarchy desteklenir (`docs/tasarim.md`). Sabit
  renk ya da yazı boyutu, sağdan sola dillerde kırılan yerleşim bulgudur.
- **CI ve iş akışları:** `.github/workflows/` altında üçüncü taraf eylemler
  commit SHA'sıyla sabitlenir; izinler en dar hâliyle verilir; paketler
  yalnızca CI'da derlenir.

## Hata sayma

- `changelog.d/` girdisinin, kodun ya da yukarıda İngilizce sayılan
  dosyaların İngilizce olmasını.
- `CHANGELOG.md`'nin PR'da değişmemesini; ona yalnızca sürüm sırasında
  yazılır.
