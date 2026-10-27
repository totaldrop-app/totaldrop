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

## 2. Hareketli işaret

Aktarım sürerken işaret web sitesindeki animasyonun aynısını oynatır. Değerler `totaldrop-site/public/styles.css` dosyasındaki `travel` ve `ripple` animasyonlarıyla birebir aynıdır:

- **Döngü:** 3,2 saniye, sonsuz.
- **Dosya:** Merkezden (uzaklık 12) kanalın ucuna (uzaklık 52) gider. Yolculuk döngünün ilk %55'inde `cubic-bezier(0.65, 0, 0.35, 1)` eğrisiyle olur. Dosya döngünün kalanında uçta bekler.
- **Halka:** Damla ile aynı merkezde ve aynı yarıçapta (12), vurgu renginde 2 px çizgili bir çemberdir. Döngünün ilk %70'inde `cubic-bezier(0.2, 0.6, 0.3, 1)` eğrisiyle 1 kattan 4,6 kata büyür. Bu sırada saydamlığı 0,7'den 0'a iner. Çizgi kalınlığı büyümeyle birlikte kalınlaşmaz.
- **Dalgalar:** Kıpırdamaz.
- **Hareketi azalt:** Sistemde bu ayar açıksa işaret durur.
