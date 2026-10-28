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
- [ ] Yeni bağımlılık varsa: neden gerektiği ve kimin bakımını yaptığı yazıldı
- [ ] CHANGELOG.md "Unreleased" girdisi eklendi

## Bağımsız inceleme / Independent review

<!-- İnceleyenin bulguları ve her birine ne yapıldığı -->
