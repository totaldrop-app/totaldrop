# Tasarım kuralları

Tam marka kitabı TotalDrop tasarım sisteminde durur. Bu dosyada kod yazarken uyulması gereken bağlayıcı kurallar var.

## 1. Sisteme yüzde yüz uyum

TotalDrop her platformda o sistemin bir parçası gibi görünür. Renk, açık ya da koyu tema, vurgu rengi, yazı tipi ölçeği, simgeler ve pencere davranışı sistemden gelir. Aurora paleti yalnızca sistem hiçbir renk vermediğinde kullanılır. Değişmeyen tek şey işarettir (Dalga) ve onun renkleridir.

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
- Sağdan sola dillerde (Arapça, Farsça, İbranice, Urduca) yerleşim aynalanır. Simgelerde yön bildirenler (geri oku) aynalanır, işaret (Dalga) ve logolar aynalanmaz.
- Metin kutuları uzun çevirilere göre büyür: Almanca ve Fince'de metin İngilizce'den %30–40 uzun olabilir; sabit genişlik verilmez.
- Aktarım kodu ve doğrulama kelimeleri her dilde İngilizce gösterilir ve soldan sağa yalıtılır (Flutter'da `Directionality(textDirection: ltr)` ve cümle içinde FSI…PDI; Qt'de `LayoutMirroring` dışında tutulur). Kod alanında otomatik büyük harf ve düzeltme kapalıdır.
- Karşı taraftan gelen dosya adları cümleye yön yalıtımıyla girer; içlerindeki yön denetim karakterleri görünür yapılır.

Arayüz değişiklikleri açık ve koyu temaya ek olarak en az bir sağdan sola (Arapça) ve bir CJK (Japonca) dilinde denenir.

## 2. Hareketli işaret

Aktarım sürerken işaret web sitesindeki animasyonun aynısını oynatır. Değerler `totaldrop-site/public/styles.css` dosyasındaki `travel` ve `ripple` animasyonlarıyla birebir aynıdır:

- **Döngü:** 3,2 saniye, sonsuz.
- **Dosya:** Merkezden (uzaklık 12) kanalın ucuna (uzaklık 52) gider. Yolculuk döngünün ilk %55'inde `cubic-bezier(0.65, 0, 0.35, 1)` eğrisiyle olur. Dosya döngünün kalanında uçta bekler.
- **Halka:** Damla ile aynı merkezde ve aynı yarıçapta (12), vurgu renginde 2 px çizgili bir çemberdir. Döngünün ilk %70'inde `cubic-bezier(0.2, 0.6, 0.3, 1)` eğrisiyle 1 kattan 4,6 kata büyür. Bu sırada saydamlığı 0,7'den 0'a iner. Çizgi kalınlığı büyümeyle birlikte kalınlaşmaz.
- **Dalgalar:** Kıpırdamaz.
- **Hareketi azalt:** Sistemde bu ayar açıksa işaret durur.
