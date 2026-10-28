# 0009: iOS ve macOS, ücretsiz CI ile; Apple hesabı yalnızca yayında

- **Durum:** Kabul, 28 Ekim 2026
- **Bağlam:** Proje yalnızca bağışla yürür ve para harcamak istemiyoruz.
  iOS ve macOS uygulamaları Xcode ister; Xcode yalnızca macOS'ta çalışır,
  geliştirme makinesi ise Linux. App Store, TestFlight ve mağaza dışı
  dağıtımda imza ile Apple noter onayı (notarization) için yıllık 99 USD'lik
  Apple Developer hesabı gerekir. macOS uygulaması hesapsız da dağıtılabilir,
  ama kurulumu zahmetlidir.
- **Seçenekler:**
  1. Kiralık bulut Mac. Aylık ücret; gerekmiyor.
  2. **GitHub'ın standart macOS koşucuları (açık kaynak depoda ücretsiz,
     Xcode ve simülatörler kurulu) ile derleme, test ve paketleme; Apple
     hesabı yalnızca yayına gelince.**
  3. iOS ve macOS'u bağışlar hesabı karşılayana kadar hiç başlatmamak.
- **Karar:** Seçenek 2.

## Sıra

1. Çekirdek iOS (`aarch64-apple-ios`, `aarch64-apple-ios-sim`) ve macOS
   (`aarch64-apple-darwin`, `x86_64-apple-darwin`; `lipo` ile tek paket)
   için CI'da derlenir; macOS'ta testler çalışır. Hesap gerekmez.
2. Arayüz için ayrı bir karar: SwiftUI ile yerel uygulama ya da mevcut
   Flutter uygulamasının iOS/macOS hedefi. Ölçüt, sistem görünümüne uyum
   (Dieter Rams ilkeleri, [tasarim.md](../tasarim.md)) ve paylaşım menüsü
   gibi sistem tümleşmeleri.
3. Simülatör testleri CI'da. Hesap gerekmez.
4. macOS: CI'dan ad-hoc imzalı ama noter onaysız paket çıkar (Apple
   Silicon'da hiç imzası olmayan kod çalışmaz; ad-hoc imzayı derleyici
   kendisi koyar). macOS 15'ten beri sağ tık > Aç yolu yok. İndirme sayfası
   şu adımları anlatır: uygulamayı bir kez açmayı dene, Sistem Ayarları >
   Gizlilik ve Güvenlik > "Yine de Aç", parola ya da Touch ID ile onayla.
   Sayfa, açmadan önce paketin sürümle yayımlanan özetinin ve imzalı yapı
   kaynağının (`gh attestation verify`) doğrulanmasını da anlatır.
5. Yayın: Apple Developer hesabı bu adımda alınır. Hesabı proje sahibi
   açar; imza sertifikaları ve App Store Connect anahtarı yalnızca CI gizli
   değişkenlerinde durur, depoya ve geliştirme makinesine yazılmaz. İmza ve
   yayın işleri yalnızca sürüm etiketlerinde çalışır; çatallardan gelen
   PR'lar gizli değişkenleri görmez. Hesap alınınca macOS paketi de Developer
   ID ile imzalanır ve noter onayından geçer.

## CI kuralları

- Yalnızca standart koşucular; ücretli büyük koşucu etiketleri kullanılmaz.
  Ücretsiz koşucular depo açık kaldıkça ücretsizdir ve bir hizmet sözü
  yoktur.
- `macos-latest` değil, sabit bir imaj (örneğin `macos-15`) ve sabit bir
  Xcode sürümü kullanılır; ikisi de bilinçli olarak yükseltilir.
- Kuyruğun uzaması bir CI hatası sayılmaz; iş yeniden çalıştırılır.

## Riskler

- iOS'ta yerel ağda cihaz bulma Bonjour ile ücretsizdir. Ham multicast ya
  da broadcast ise Apple'ın yalnızca ücretli hesaplara istek üzerine verdiği
  bir yetki ister. Bulma tasarımı bu yetkiye dayanırsa iOS işi 1–3.
  adımlarda takılır; o zaman ayrı bir karar yazılır.
- Hesap gelene kadar macOS'ta kurulum zahmetli olacak; bu, benimsenmeyi ve
  gizlilik odaklı bir uygulamaya güveni azaltır.

## Sonuçlar

- Mac olmadan simülatörü canlı görmek mümkün değil; CI günlükleri ve ekran
  görüntüleri ile çalışılır. Gerçek iPhone'da deneme, ücretsiz bir Apple
  ID ile yedi günlük yan yükleme dışında, hesap gelene kadar kısıtlıdır.
  Bir Mac bağlanırsa o da kullanılır.
- Hesapsız Homebrew kurulumu bir yol sayılmaz: Homebrew noter onaysız
  paketleri artık kabul etmiyor.
- Yayınlar, diğer platformlarda olduğu gibi, yalnızca CI'dan çıkar
  ([0007](0007-surec.md)).
