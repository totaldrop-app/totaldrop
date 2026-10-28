# 0009: iOS ve macOS, ücretsiz CI ile; Apple hesabı yalnızca yayında

- **Durum:** Kabul, 28 Ekim 2026
- **Bağlam:** Proje yalnızca bağışla yürür ve para harcamak istemiyoruz.
  iOS ve macOS uygulamaları Xcode ister; Xcode yalnızca macOS'ta çalışır,
  geliştirme makinesi ise Linux. App Store ve TestFlight için yıllık 99 USD'lik
  Apple Developer hesabı gerekir; macOS uygulaması hesapsız da dağıtılabilir.
- **Seçenekler:**
  1. Kiralık bulut Mac. Aylık ücret; gerekmiyor.
  2. **GitHub'ın macOS koşucuları (açık kaynak depoda ücretsiz, Xcode
     kurulu) ile derleme, test ve paketleme; Apple hesabı yalnızca mağaza
     yayınına gelince.**
  3. iOS ve macOS'u bağışlar hesabı karşılayana kadar hiç başlatmamak.
- **Karar:** Seçenek 2.

## Sıra

1. Çekirdek iOS (`aarch64-apple-ios`, `aarch64-apple-ios-sim`) ve macOS
   (`aarch64-apple-darwin`) için CI'da derlenir; macOS'ta testler çalışır.
   Hesap gerekmez.
2. Arayüz için ayrı bir karar: SwiftUI ile yerel uygulama ya da mevcut
   Flutter uygulamasının iOS/macOS hedefi. Ölçüt, sistem görünümüne uyum
   (Dieter Rams ilkeleri, [tasarim.md](../tasarim.md)) ve paylaşım menüsü
   gibi sistem tümleşmeleri.
3. Simülatör testleri CI'da. Hesap gerekmez.
4. macOS: imzasız paket CI'dan çıkar. İlk açılışta Gatekeeper uyarısı
   çıkar; indirme sayfası nasıl açılacağını anlatır.
5. Yayın: Apple Developer hesabı bu adımda alınır. Hesabı proje sahibi
   açar; imza sertifikaları ve App Store Connect anahtarı yalnızca CI gizli
   değişkenlerinde durur, depoya ve geliştirme makinesine yazılmaz. Hesap
   alınınca macOS paketi de imzalanır ve noterden geçirilir.

## Sonuçlar

- Bir Mac olmadan simülatörü canlı görmek ve gerçek cihazda denemek
  mümkün değil; CI günlükleri ve ekran görüntüleri ile çalışılır. Bir Mac
  bağlanırsa o da kullanılır.
- macOS koşucularının kuyruğu zaman zaman uzar; bu bir CI hatası sayılmaz,
  iş yeniden çalıştırılır.
- Yayınlar, diğer platformlarda olduğu gibi, yalnızca CI'dan çıkar
  ([0007](0007-surec.md)).
