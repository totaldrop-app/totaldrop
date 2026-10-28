# 0006: Linux uygulaması Qt 6 ve QML

- **Durum:** Kabul, 27 Ekim 2026
- **Bağlam:** Uygulama KDE Plasma, Hyprland ve Omarchy'de sistem temasına
  yüzde yüz uymalı; yerel Wayland ve en güncel teknoloji isteniyor.
- **Karar:** Qt 6 ve QML, Rust çekirdeğine CXX-Qt ile bağlı. Tema kaynağı
  masaüstüne göre seçilir: KDE'de sistem paleti ve Breeze, Omarchy'de
  `colors.toml` (canlı izlenir), Hyprland'de köşe yuvarlaklığı, diğerlerinde
  XDG portalı. Paketler Arch, Debian ve Fedora için ayrı tariflerden, ayrı
  kaplarda derlenir.
- **Neden:** Qt hem Plasma'nın kendi araç takımı hem Hyprland ve GNOME'da
  iyi çalışan tek olgun seçenek; QML ile tema değişikliği yeniden başlatmadan
  uygulanır.
- **Sonuçlar:** Her arayüz değişikliği üç masaüstünde, açık ve koyu temada
  denenir ([tasarim.md](../tasarim.md)).
