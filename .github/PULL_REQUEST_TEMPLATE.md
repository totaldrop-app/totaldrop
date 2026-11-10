<!-- Başlık Conventional Commits biçiminde: feat(linux): ..., fix(core): ... -->

## Ne değişti / What changed

Closes #

## Platform

- [ ] Core (crates/)
- [ ] Linux (linux/, packaging/)
- [ ] Android (app/)
- [ ] Server (server/)
- [ ] Site (site/)
- [ ] Docs

## Bitti tanımı / Definition of done (docs/surec.md)

- [ ] Davranışı sınayan testler var; hata düzeltmesinde önce hatayı gösteren test yazıldı
- [ ] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` ve platform denetimleri temiz
- [ ] Dışarıdan veri okuyan yeni ayrıştırıcının fuzz hedefi var
- [ ] Protokol kararı `docs/protocol-v0.1-notes.md`'de, mimari kararı `docs/kararlar/` altında
- [ ] Kullanıcıya görünen her metin İngilizce ve Türkçe; Türkçe belgeler güncel
- [ ] Arayüz değiştiyse: açık ve koyu temada, Linux'ta KDE Plasma, Hyprland ve Omarchy'de ekran görüntüsü (docs/tasarim.md)
- [ ] Sunucuların gördüğü bilgi değiştiyse gizlilik politikası güncellendi
- [ ] Belgede, politikada ya da sitede verilen her söz (süre, sınır, maliyet) kodla ve testle karşılanıyor; değişen metin bütün dil dosyalarında güncel
- [ ] Hata bekleyen testler hatanın türünü denetliyor; zamanlama testlerinde sabit bekleme yok (docs/surec.md, "İncelemeden çıkan kurallar")
- [ ] Zorunlu olmayanlar dahil bütün kontroller yeşil; kırmızı olanın nedeni yazıldı ve issue'su açıldı
- [ ] Yeni bağımlılık varsa: neden gerektiği ve kimin bakımını yaptığı yazıldı
- [ ] `changelog.d/` altına girdi dosyası eklendi (ya da nedenini söyleyen bir `.none.md`)

## Bağımsız inceleme / Independent review

<!-- İnceleyenin bulguları ve her birine ne yapıldığı -->
