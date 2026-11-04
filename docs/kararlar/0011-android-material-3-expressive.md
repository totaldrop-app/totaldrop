# 0011: Android Material 3 Expressive'e resmî paketle, adım adım geçer

- **Durum:** Önerilen, 4 Kasım 2026 (#215)
- **Bağlam:** Android uygulaması standart Material 3 kullanıyor
  (`useMaterial3: true`, `app/lib/ui/core/theme.dart`). Renkler Material
  You'dan geliyor (`dynamic_color`, `app/lib/main.dart`). Android 16'nın
  QPR1 güncellemesiyle (Eylül 2025) Google'ın sistem arayüzü (Pixel)
  Material 3 Expressive'e geçti; diğer üreticilerin arayüzleri kendi
  hızlarında izliyor. Expressive tasarım sistemi daha büyük ve değişken
  köşeler, şekiller arasında biçim değiştirme (shape morph), yaylı
  (spring) hareket, düğme grupları ve yeni bir yükleniyor göstergesi
  getiriyor.
  [Tasarım kuralları](../tasarim.md) uygulamanın her platformda sistemin bir
  parçası gibi görünmesini istiyor. Bu yüzden Expressive'i benimseyip
  benimsemeyeceğimize ve nasıl benimseyeceğimize karar vermek gerekiyor.

  Flutter'daki durum (4 Kasım 2026):
  - Flutter ekibi Expressive'i çerçevenin içinde geliştirmiyor. Yeni
    Expressive işleri, çerçeveden ayrılan paketlerde (`material_ui`)
    yapılacak
    ([flutter/flutter#168813](https://github.com/flutter/flutter/issues/168813),
    29 Temmuz 2025 güncellemesi; issue açık). Haziran 2026'da ekip, ayırma
    işi bittikten sonra Expressive planının ayrıntılarını paylaşacağını
    söyledi.
  - Uygulama zaten `material_ui` kullanıyor (`^1.8.0`, son sürüm 1.10.0).
    Paket `StyleVariant.material3Expressive` seçeneğini getirdi (1.2.0)
    ama kendi belgesi bu desteğin "geliştirme aşamasında" olduğunu ve
    seçenek açılınca görünümün sürümler arasında değişebileceğini
    söylüyor. Bugüne kadar gelenler: Expressive `IconButton` (1.5.0) ve
    biçim değiştiren `MaterialShapeBorder` ile `MaterialShapes` (1.8.0);
    ikisi de uygulamanın kilitli sürümünde (1.8.0) var. Hareket şeması
    (motion scheme), yükleniyor göstergesi ve düğme grupları değişiklik
    günlüğünde henüz yok ([değişiklik günlüğü](https://pub.dev/packages/material_ui/changelog)).
  - Toplulukta tüm bileşen setini yeniden yazan paketler var
    (`material_3_expressive`, `material_expressive`). Bunlar Google ya da
    Flutter ekibinin değil, tek tek geliştiricilerin bakımında.

  Uygulamanın kullandığı bileşenler azdır: `FilledButton`,
  `OutlinedButton`, `TextButton`, `IconButton`, `AppBar`, `Card`,
  `AlertDialog`, `TextField`, `ListTile`, `ExpansionTile`,
  `SwitchListTile`, `Divider`, `SnackBar`, `Chip`,
  `LinearProgressIndicator`. Tema bilinçli olarak gölgesizdir: yüzeyler
  renk ve ince çizgiyle ayrılır, her yükseklik 0'dır.

- **Seçenekler:**
  1. **Standart Material 3'te kalmak.** Ek iş gerektirmez, ama Expressive
     arayüzlü cihazlarda uygulama sistemden geride görünür. "Sisteme yüzde yüz
     uyum" kuralıyla çelişir.
  2. **Topluluk paketiyle hemen geçmek.** Bütün Expressive bileşenleri
     bugün gelir. Ama bileşenler Flutter'ınkilerin yerine geçer: erişilebilirlik,
     sağdan sola düzen ve sistem yazı ölçeği gibi davranışların doğruluğu
     tek bir bakımcıya kalır. Resmî paket bu bileşenleri getirdiğinde de
     ikinci kez göç etmek gerekir. Güvenlik ağırlıklı bir uygulamada
     arayüzün tamamını küçük bir bağımlılığa bağlamak tedarik zinciri
     riskini de büyütür.
  3. **Kendi Expressive bileşenlerimizi yazmak.** Bağımlılık eklemez, ama
     bakımı ve erişilebilirlik sınamaları bize kalır. Resmî bileşenler
     gelince bu kod da çöpe gider.
  4. **Resmî `material_ui` paketiyle adım adım geçmek.** Paket bir
     Expressive parçasını getirdikçe onu açarız. Getirmediği parçalar
     için standart Material 3'te kalırız.

- **Karar:** Seçenek 4.
  - `material_ui` güncel tutulur (Dependabot PR'ları). Her yeni sürümün
    değişiklik günlüğü Expressive maddeleri için okunur.
  - Paketin getirdiği her Expressive parçası (bugün `IconButton` ve
    `MaterialShapes`, ikisi de 1.8.0'da var) ayrı bir issue ve PR ile
    açılır. Her adımda Android'de
    açık ve koyu temada, Material You renkleriyle ve yedek Aurora paletiyle,
    soldan sağa ve sağdan sola dillerde ekran görüntüsü alınır.
  - `StyleVariant.material3Expressive` uygulamanın tamamında, paket onu
    kararlı ilan edene kadar açılmaz. Açılmadan önce hangi bileşenleri
    değiştirdiği paketin kaynağından doğrulanır.
  - Topluluk Expressive paketleri eklenmez. Paket henüz getirmedi diye
    bir Expressive bileşenini kendimiz yazmayız.

- **Neden:** Uygulamayı Flutter'ın resmî yoluna bağlı tutar; bileşenler
  geldikçe sistemle aynı görünüme kendiliğinden yaklaşır. Arayüzün
  erişilebilirlik, sağdan sola düzen ve yazı ölçeği davranışı Flutter
  ekibinin sınadığı kodda kalır. Yeni bağımlılık eklenmez ve iki kez göç
  etmek gerekmez. Bekleyen parçalar eksik görünse de kırık görünmez:
  standart Material 3 Expressive arayüzlü cihazlarda da sistemin parçası
  gibi durur.

- **Değişmeyenler:**
  - İşaret (Köşe), onun renkleri ve aktarım sırasındaki dönüşü değişmez
    ([tasarım kuralları](../tasarim.md), "Hareket"). Expressive'in
    yükleniyor göstergesi ya da dalgalı ilerleme çubuğu işaretin yerini
    almaz. İlerleme çubuğu gelirse yalnızca bugünkü
    `LinearProgressIndicator`'ın yerine geçebilir.
  - Gölgesiz tema sürer: yükseklikler 0 kalır, yüzeyler renk ve ince
    çizgiyle ayrılır. Expressive'in gölge ya da yüzey tonu kullanan
    varsayılanları bu kurala göre ayarlanır.
  - Yazı tipleri markanındır (Bricolage Grotesque ve Atkinson
    Hyperlegible); boyut sistemin yazı ölçeğini izler.
  - Her yeni Expressive hareketi, işaret gibi, sistemin hareketi azaltma
    ayarına uyar (Android'de "Animasyonları kaldır"; tasarım kurallarında
    "Hareketi azalt").

- **Sonuçlar:**
  - `docs/tasarim.md`'deki Android satırı bu karara bağlanır.
  - Yeni Expressive adımları için issue'lar `material_ui` sürümleri
    çıktıkça açılır. İlk aday: kilitli sürümde (1.8.0) zaten bulunan
    Expressive `IconButton`'u denemek. `material_ui` yükseltmeleri
    Dependabot'la ayrı PR olarak gelir.
  - Linux, site ve diğer platformlar bu karardan etkilenmez.

- **Gözden geçirme koşulu:** `material_ui` hareket şemasını,
  yükleniyor göstergesini ve düğme gruplarını getirdiğinde ya da
  Expressive desteğini kararlı ilan ettiğinde. Flutter ekibi Expressive
  planını değiştirirse de yeniden bakılır.
