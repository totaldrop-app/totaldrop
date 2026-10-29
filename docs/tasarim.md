# Tasarım kuralları

Tam marka kitabı TotalDrop tasarım sisteminde durur. Bu dosyada kod yazarken uyulması gereken bağlayıcı kurallar var.

## 1. Sisteme yüzde yüz uyum

TotalDrop her platformda o sistemin bir parçası gibi görünür. Renk, açık ya da koyu tema, vurgu rengi, yazı tipi ölçeği, simgeler ve pencere davranışı sistemden gelir. Aurora paleti yalnızca sistem hiçbir renk vermediğinde kullanılır. Değişmeyen tek şey işarettir (Ağ) ve onun renkleridir.

Sistem temasını değiştirdiğinizde uygulama yeniden başlatılmadan yeni temaya geçer.

| Ortam | Renk kaynağı | Kontrol stili ve simgeler | Pencere |
|---|---|---|---|
| Android 12 ve üstü | Material You (dynamic_color) | Material 3 | Sistem |
| Omarchy | `~/.local/state/omarchy/current/theme/colors.toml`; tema değişince canlı yeniden okunur | Fusion, renkler temadan | Hyprland çizer, uygulama kendi çerçevesini çizmez |
| Hyprland (Omarchy olmadan) | xdg-desktop-portal `org.freedesktop.appearance` (color-scheme, accent-color) ve Qt paleti (hyprqt6engine ya da qt6ct) | Sistem Qt stili, yoksa Fusion | Hyprland çizer |
| KDE Plasma | Plasma'nın Qt paleti ve `kdeglobals` (vurgu dahil) | `org.kde.desktop` (qqc2-desktop-style), Breeze simgeler | KWin çizer (sunucu tarafı çerçeve) |
| GNOME ve diğerleri | xdg-desktop-portal color-scheme ve accent-color | Fusion | Sistem |

Her değişiklikten sonra bu ortamların hepsinde açık ve koyu temada ekran görüntüsü alınarak kontrol edilir.

### Dil ve bölge

Dil, bölge biçimleri ve yazı yönü de sistemden gelir ([karar 0008](kararlar/0008-diller-ve-bolgeler.md)):

- Arayüz sistemin tercih ettiği dilde açılır; çevirisi yoksa en yakın varyanta, o da yoksa İngilizce'ye düşer.
- Sayılar ve boyutlar bölge ayarına, çoğullar gösterilen dilin kurallarına göre biçimlenir. Kodda `toStringAsFixed`, elle ondalık ayırıcı ya da "1 file / 2 files" gibi elle çoğul yazılmaz.
- Sağdan sola dillerde (Arapça, Farsça, İbranice, Urduca) yerleşim aynalanır. Simgelerde yön bildirenler (geri oku) aynalanır, işaret (Ağ) ve logolar aynalanmaz.
- Metin kutuları uzun çevirilere göre büyür: Almanca ve Fince'de metin İngilizce'den %30–40 uzun olabilir; sabit genişlik verilmez.
- Aktarım kodu ve doğrulama kelimeleri her dilde İngilizce gösterilir ve soldan sağa yalıtılır (Flutter'da `Directionality(textDirection: ltr)` ve cümle içinde FSI…PDI; Qt'de `LayoutMirroring` dışında tutulur). Kod alanında otomatik büyük harf ve düzeltme kapalıdır.
- Karşı taraftan gelen dosya adları cümleye yön yalıtımıyla girer; içlerindeki yön denetim karakterleri görünür yapılır.

Arayüz değişiklikleri açık ve koyu temaya ek olarak en az bir sağdan sola (Arapça) ve bir CJK (Japonca) dilinde denenir.

## 2. İşaret: Ağ

Üç eşit cihaz, her biri öbür ikisine bağlı; alttaki yolda, gittiği yöne bakan bir damla. "Her cihazdan her cihaza" fikrini çizer: merkezde sunucu yoktur, cihazlar eşittir. Ana dosya `app/assets/logo.svg`; Android simgeleri `packaging/render-icons.sh` ile ondan üretilir.

- **Yapı (112 birimlik tuval):** cihazlar merkezi (56, 62) olan r 38 çemberin üstünde, 120° aralıkla: (56, 24), (88,91, 81), (23,09, 81); yarıçapları 11. Yollar r 60 yaylardır, dışa doğru bombelidir, kalınlıkları 7. Damlanın gövdesi r 6,5, ucu gövde merkezinden 11 uzakta; dinlenirken alt yolun ortasında (56, 90,83) sağa bakar.
- **Renkler:** açık zeminde yollar mürekkep (#22184F), cihazlar ve damla mor (#6D4AE0); koyu zeminde yollar kâğıt (#F4F2FB), cihazlar ve damla parlak mor (#A68BFF).
- **Küçük boy (24 px ve altı; favicon, sembolik simge):** damla yoktur, yol kalınlığı 10, cihaz yarıçapı 15.
- **Boşluk ve en küçük boy:** işaretin her yanında en az bir cihaz yarıçapı (11) boşluk; en küçük boy 16 px (küçük boy çizimiyle).
- **Yön:** sağdan sola dillerde aynalanmaz.

### Hareket

Aktarım sürerken damla cihazları sırayla dolaşır. Web sitesi (`site/public/index.html`, `animateMotion`), Android (`mark.dart`, `MarkMotion`) ve Linux (`Mark.qml`) aynı değerleri kullanır:

- **Döngü:** 4,8 saniye, sonsuz.
- **Yol:** alt yolun ortasından saat yönünün tersine: sağ cihaz, üst cihaz, sol cihaz ve yeniden yerine. Damla hep gittiği yöne bakar.
- **Bacaklar:** döngünün %0–12,5'inde sağ cihaza, %20–45'inde üst cihaza, %52,5–77,5'inde sol cihaza, %85–97,5'inde yerine gider; her bacak `cubic-bezier(0.65, 0, 0.35, 1)` eğrisiyle. Aradaki sürelerde damla cihazın üstünde bekler.
- **Yollar ve cihazlar:** kıpırdamaz.
- **Hareketi azalt:** sistemde bu ayar açıksa işaret dinlenme hâlinde durur.
