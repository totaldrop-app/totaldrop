# Linux: dosya yöneticisinden gönderme

Dosya yöneticisinde bir ya da birden çok dosya ve klasör seçip sağ tıklayınca
"TotalDrop ile gönder" görünür (#303, #54'ün ilk adımı). TotalDrop açılır ya
da açık penceresi öne gelir; seçim Gönder ekranında listelenir. Uygulama
açıksa ikinci bir pencere açılmaz, seçim açık pencereye eklenir
(`linux/src/instance.rs`).

| Dosya yöneticisi | Nasıl | Paketle gelir |
| --- | --- | --- |
| Dolphin (KDE Plasma) | KIO servis menüsü, `/usr/share/kio/servicemenus/app.totaldrop.TotalDrop.send.desktop` | Evet |
| Nautilus / Dosyalar (GNOME, Omarchy) | nautilus-python uzantısı, `/usr/share/nautilus-python/extensions/totaldrop_nautilus.py` | Evet; `python-nautilus` (Arch), `python3-nautilus` (Debian), `nautilus-python` (Fedora) kurulu olmalı |
| Nemo (Cinnamon) | `/usr/share/nemo/actions/app.totaldrop.TotalDrop.nemo_action` | Evet |
| Thunar (Xfce) | Kullanıcının kendi özel eylemi (aşağıda) | Hayır |
| Hepsi | "Birlikte aç → Diğer uygulama" ve pencereye sürükle-bırak | Evet |

Menüler yalnızca yerel dosyalarda görünür; ağ konumları (SFTP, SMB) ve Çöp
Kutusu'ndaki öğeler için görünmez, çünkü TotalDrop dosyayı diskten okur.

Seçim uygulamaya kabuk çalıştırılmadan geçer: Dolphin ve Nautilus
masaüstü girdisinin yolunu kullanır (`%U`), Nemo her adresi çift tırnak
içinde verir (`Quote=double`; GLib dosya adreslerinde `'`, `$`, `&`, `(`,
`)` karakterlerini kodlamaz). Adı `-` ile başlayan ya da boşluk, tırnak,
`;` içeren dosyalar bölünmez ve seçenek sanılmaz; bunu
`linux/src/instance.rs` testleri ve Nemo dosyasının üreticideki testi
denetler. Dolphin'de öğe "Eylemler" alt menüsünde değil, menünün kendisinde
görünür (`X-KDE-Priority=TopLevel`). Nautilus'ta "Son Kullanılanlar"
görünümündeki öğeler de gönderilebilir.

## Neden "her dosya türü" kaydedilmedi

Masaüstü girdisi yalnızca `totaldrop://` bağlantılarını kaydeder. Her dosya
türünü (`application/octet-stream`, `inode/directory`) kaydetmek, ayarı az
olan sistemlerde (yalın bir Hyprland) TotalDrop'u bilinmeyen dosyaların ve
klasörlerin varsayılan açıcısı yapabilirdi: `xdg-open klasör` TotalDrop'u
açardı. "Birlikte aç → Diğer uygulama" listesinde TotalDrop yine görünür.

## Flatpak

Flatpak bir uygulamanın başka programların menülerine eklenmesine izin
vermez; Flatpak sürümünde yalnızca "Birlikte aç" ve sürükle-bırak çalışır.
`packaging/linux/install.sh` menü dosyalarını `/app` önekinde kurmaz.

## Thunar

Thunar'ın özel eylemleri kullanıcı başına bir dosyada tutulur
(`~/.config/Thunar/uca.xml`), bir paket onları kuramaz. Elle eklemek için:
Düzen → Özel eylemleri yapılandır → +

- Ad: TotalDrop ile gönder
- Komut: `totaldrop-desktop %F` (Thunar her yolu tırnaklar; yollar `/` ile başladığı için seçenek sanılmaz)
- Simge: `app.totaldrop.TotalDrop`
- Görünme koşulları sekmesinde: dosya deseni `*`, bütün türler ve
  "Klasörler" işaretli.

## Metinler

Menü metinleri ve masaüstü girdisinin adı, açıklaması ve arama kelimeleri
`l10n/app` içindeki `desktop*` anahtarlarındandır (`sendTitle` ve
`receiveTitle` masaüstü girdisinin "Dosya gönder" ve "Dosya al"
eylemlerinde). Bir metin değişince:

```sh
node tools/desktop-entries.mjs write
```

CI (`node --test tools/test-desktop-entries.mjs`) üretilen dosyalar ARB
dosyalarıyla uyuşmazsa kırılır. Nautilus uzantısının mantığı elle yazılır;
yalnızca `GENERATED` satırları arasındaki metin tablosu üretilir.

## Elle deneme

Her sürümden önce, paket kurulu bir sistemde:

1. KDE Plasma (Dolphin), Hyprland/Omarchy (Nautilus), GNOME (Nautilus) ve
   imkân varsa Cinnamon (Nemo):
   bir dosya, birden çok dosya ve bir klasör seçip sağ tıklayın, "TotalDrop
   ile gönder"e basın. TotalDrop kapalıyken ve açıkken ayrı ayrı deneyin.
2. Adı `-a "b".txt` ve `Bob's (1) & $x.txt` olan dosyaları ve Türkçe
   karakterli bir klasörü deneyin.
3. Sistem dili Türkçe ve Arapça iken menü metnine bakın.
4. Görev çubuğunda ya da başlatıcıda TotalDrop'a sağ tıklayıp "Dosya
   gönder" ve "Dosya al"ı deneyin.
5. Dolphin'de yerel dosyaya başka görünümlerden de bakın: masaüstü
   (`desktop:/`), son kullanılanlar (`recentlyused:/`) ve Baloo araması.
   Menü çıkıyorsa doğru dosyayı gönderdiğini, çıkmıyorsa bunu buraya
   bilinen sınır olarak yazın (KF6'nın `X-KDE-Protocols=file` için
   `url()` mi `mostLocalUrl()` mi denetlediğine bağlı).
6. Nautilus'ta menü çalışmazsa nedeni günlüktedir:
   `journalctl --user -b | grep TotalDrop`.

Nautilus menüyü yalnızca yeniden başlayınca okur: `nautilus -q`.
