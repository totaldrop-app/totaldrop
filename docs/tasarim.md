# Tasarım kuralları

Tam marka kitabı TotalDrop tasarım sisteminde durur. Bu dosyada kod yazarken uyulması gereken bağlayıcı kurallar var.

## 1. Sisteme yüzde yüz uyum

TotalDrop her platformda o sistemin bir parçası gibi görünür. Renk, açık ya da koyu tema, vurgu rengi, yazı tipi ölçeği, simgeler ve pencere davranışı sistemden gelir. Aurora paleti yalnızca sistem hiçbir renk vermediğinde kullanılır. Değişmeyen tek şey işarettir (Köşe) ve onun renkleridir.

Sistem temasını değiştirdiğinizde uygulama yeniden başlatılmadan yeni temaya geçer.

| Ortam | Renk kaynağı | Kontrol stili ve simgeler | Pencere |
|---|---|---|---|
| Android 12 ve üstü | Material You (dynamic_color) | Material 3 | Sistem |
| Omarchy | `~/.local/state/omarchy/current/theme/colors.toml`; tema değişince canlı yeniden okunur | Fusion, renkler temadan | Hyprland çizer, uygulama kendi çerçevesini çizmez |
| Hyprland (Omarchy olmadan) | Qt paleti varsa (hyprqt6engine ya da qt6ct) o; yoksa GTK teması (`gtk-theme` adıyla `gtk.css`, üstüne `~/.config/gtk-4.0/gtk.css`), açık/koyu ve vurgu portaldan (temanın kendi vurgusu varsa o kazanır) | Sistem Qt stili, yoksa GTK biçiminde Fusion | Hyprland çizer |
| KDE Plasma | Plasma'nın Qt paleti ve `kdeglobals` (vurgu dahil) | `org.kde.desktop` (qqc2-desktop-style), Breeze simgeler | KWin çizer (sunucu tarafı çerçeve) |
| GNOME ve diğerleri | GNOME'da libadwaita paleti, diğerlerinde GTK teması; ikisinde de üstüne `~/.config/gtk-4.0/gtk.css`. Açık/koyu ve vurgu portaldan; `gtk.css` vurgu verirse o kazanır. Tema ya da `gtk.css` değişince canlı yeniden okunur | GTK biçiminde Fusion (düğme, alan ve anahtar GTK gibi çizilir, simgeler yazı renginde) | Sistem |

Her değişiklikten sonra bu ortamların hepsinde açık ve koyu temada ekran görüntüsü alınarak kontrol edilir.

### Dil ve bölge

Dil, bölge biçimleri ve yazı yönü de sistemden gelir ([karar 0008](kararlar/0008-diller-ve-bolgeler.md)):

- Arayüz sistemin tercih ettiği dilde açılır; çevirisi yoksa en yakın varyanta, o da yoksa İngilizce'ye düşer.
- Sayılar ve boyutlar bölge ayarına, çoğullar gösterilen dilin kurallarına göre biçimlenir. Kodda `toStringAsFixed`, elle ondalık ayırıcı ya da "1 file / 2 files" gibi elle çoğul yazılmaz.
- Sağdan sola dillerde (Arapça, Farsça, İbranice, Urduca) yerleşim aynalanır. Simgelerde yön bildirenler (geri oku) aynalanır, işaret (Köşe) ve logolar aynalanmaz.
- Metin kutuları uzun çevirilere göre büyür: Almanca ve Fince'de metin İngilizce'den %30–40 uzun olabilir; sabit genişlik verilmez.
- Aktarım kodu ve doğrulama kelimeleri her dilde İngilizce gösterilir ve soldan sağa yalıtılır (Flutter'da `Directionality(textDirection: ltr)` ve cümle içinde FSI…PDI; Qt'de `LayoutMirroring` dışında tutulur). Kod alanında otomatik büyük harf ve düzeltme kapalıdır.
- Karşı taraftan gelen dosya adları cümleye yön yalıtımıyla girer; içlerindeki yön denetim karakterleri görünür yapılır.

Arayüz değişiklikleri açık ve koyu temaya ek olarak en az bir sağdan sola (Arapça) ve bir CJK (Japonca) dilinde denenir.

## 2. İşaret: Köşe

Tek köşesi kare olan bir daire. Daire bütünlüktür (*Total*), kare köşe onu damlaya (*Drop*) çevirir ve dosyanın gittiği yönü gösterir. Köşe aynı zamanda bir dosyanın ve bir cihaz ekranının köşesidir. Aktarım sürerken işaret çeyrek çeyrek döner, köşe sırayla her yönü gösterir: her cihazdan her cihaza. Ana dosya `app/assets/logo.svg`; Android simgeleri `packaging/render-icons.sh` ile ondan üretilir.

- **Yapı (112 birimlik tuval):** merkezi (55, 57), yarıçapı 42 olan daire; sağ üst çeyreği kare köşeyle doldurulur (köşe 97, 15). Yol: `M55 15H97V57A42 42 0 1 1 55 15Z`. Daire, tuvalin merkezinden bir birim sola ve aşağı kaydırılmıştır: köşe tarafı ağır olduğu için şekil böyle ortada görünür.
- **Renkler:** açık zeminde mor (#6D4AE0), koyu zeminde parlak mor (#A68BFF). Tek renktir; gölge, degrade ya da kontur yoktur.
- **Uygulama simgesi:** mor zeminde kâğıt rengi (#F4F2FB) işaret; işaret karonun genişliğinin yarısıdır (0,68 ölçek: Linux 128 birimlik simge, eski Android simgesi, Play simgesi). Android uyarlanabilir simgesinde işaret 0,55 ölçekle güvenli alanın içinde kalır (köşe, merkezden en uzak nokta, 33 dp'lik sınırın 1 dp içinde); tek renkli katmanda beyaz. Sembolik simge yalnız işarettir.
- **Bitti rozeti:** dinlenen işaretin dairesine kâğıt rengi bir onay işareti (açık temada; koyu temada mürekkep), kalınlığı 9.
- **Boşluk ve en küçük boy:** her yanda en az yarıçapın yarısı (21) boşluk; en küçük boy 16 px. Küçük boy için ayrı çizim gerekmez.
- **Yapılmaz:** köşeyi yuvarlatmak, daireyi elipse çevirmek, işareti çerçeve ya da halka içine almak, başka renge boyamak, dinlenirken döndürmek.
- **Yön:** sağdan sola dillerde aynalanmaz; dinlenirken köşe hep sağ üstte.

### Hareket

Aktarım sürerken işaret, dairenin merkezi (55, 57) çevresinde saat yönünde çeyrek tur döner ve bekler; dört çeyrekte başladığı yere gelir. Web sitesi (`site/public/styles.css`, `mk-turn`), Android (`mark.dart`, `MarkMotion`) ve Linux (`Mark.qml`) aynı değerleri kullanır:

- **Döngü:** 3,2 saniye, sonsuz.
- **Çeyrekler:** her biri 0,8 saniye: 0,6 saniye `cubic-bezier(0.65, 0, 0.35, 1)` eğrisiyle 90° dönüş, ardından 0,2 saniye bekleme.
- **Hareketi azalt:** sistemde bu ayar açıksa işaret dinlenme hâlinde durur.
