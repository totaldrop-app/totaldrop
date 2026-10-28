# Çalışma süreci

Bu belge bir değişikliğin fikirden yayına nasıl gittiğini anlatır. Depo
düzeni ve branch'ler için [gelistirme.md](gelistirme.md), sürüm numaraları
için [versioning.md](versioning.md) bağlayıcıdır.

## Akış

1. **İş kaydı:** Her iş bir GitHub issue'su ile başlar. Issue neyin neden
   yapılacağını ve ne zaman bitmiş sayılacağını söyler. Yol haritasındaki
   her madde bir issue'dur; aşamalar milestone'dur (Şimdi, Sırada, Sonra,
   1.0).
2. **Branch:** İş, adı platformu söyleyen kısa ömürlü bir branch'te yapılır
   (`feat/linux-…`, `fix/core-…`).
3. **Pull request:** PR açıklaması issue'ya bağlanır (`Closes #12`) ve PR
   şablonundaki "bitti" listesi doldurulur.
4. **CI:** Bütün kontroller yeşil olmadan birleştirme yapılmaz. Kırmızı bir
   kontrol "sonra bakarız" diye geçilmez; ya düzeltilir ya da nedeni PR'da
   yazılır ve ayrı bir issue açılır.
5. **Bağımsız inceleme:** Her PR'ı, kodu yazmamış biri okur. Proje tek kişiyle
   yürüdüğü için bu, değişikliği yazan oturumdan ayrı çalışan bir inceleme
   ajanıdır: yalnızca diff'i ve ilgili kodu görür, yazarın gerekçesini
   görmez. Bulduğu her doğruluk ya da güvenlik sorunu birleştirmeden önce
   düzeltilir; düzeltilmeyenler için gerekçe PR'a yazılır.
6. **Birleştirme:** `main`'e yalnızca PR ile, rebase ile birleştirilir;
   geçmiş doğrusal kalır. Doğrudan push ve force push kapalıdır.
7. **Yayın:** [Sürüm listesi](#sürüm-listesi) adım adım izlenir.

## Bitti tanımı

Bir değişiklik ancak şunların hepsi doğruysa bitmiştir:

- Davranışı sınayan testler var ve geçiyor. Hata düzeltmesinde, önce hatayı
  gösteren bir test yazılır.
- `cargo fmt`, `cargo clippy -- -D warnings` ve platformun kendi denetimi
  (`flutter analyze`, `tsc`) temiz.
- Dışarıdan gelen veriyi okuyan her yeni ayrıştırıcının bir fuzz hedefi var.
- Protokolü etkileyen her karar `docs/protocol-v0.1-notes.md`'ye, mimariyi
  etkileyen her karar `docs/kararlar/` altına yazıldı.
- Türkçe belgeler güncel; kullanıcıya görünen her metin İngilizce ve
  Türkçe var.
- Arayüz değiştiyse: açık ve koyu temada; Linux'ta KDE Plasma, Hyprland ve
  Omarchy'de; Android'de ekran görüntüsü PR'da.
- Sunucuların gördüğü bilgi değiştiyse gizlilik politikası, özellik
  yayınlanmadan önce güncellendi.
- `CHANGELOG.md`'nin "Unreleased" bölümünde girdisi var.

## Sürüm listesi

1. Platformun sürümünü ve (Linux'ta) paket tariflerini artır,
   `CHANGELOG.md` girdilerini sürüm başlığına taşı.
2. Sürüm PR'ı: kontroller ve bağımsız inceleme, sonra birleştir.
3. `main`'deki sürüm commit'ine açıklamalı etiket at (`linux-v…`,
   `android-v…`) ve gönder.
4. CI paketleri etiketten derler, her pakete derleme kaynağını gösteren
   imzalı bir kanıt (build provenance) ekler ve taslak bir yayın açar.
   Kişisel bir makinede derlenen hiçbir şey yayınlanmaz.
5. İndirilen paketler sağlama toplamıyla ve temiz kaplarda kurulumla
   (`packaging/test-install.sh`) denenir.
6. Yayın notları İngilizce ve Türkçe yazılır, taslak yayımlanır.
7. Site güncellenir ve indirme bağlantılarının her biri denenir.
8. Android'de CI'ın imzaladığı paket Play Console'a yüklenir.

## Gizli bilgiler

İmza anahtarları, parolalar ve API anahtarları yalnızca GitHub Actions
gizli değerlerinde ve sahibinin bilgisayarında durur. Hiçbir oturum ya da
araç bunları okumaz, yazdırmaz ya da depoya eklemez.
