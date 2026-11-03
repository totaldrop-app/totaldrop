# 0010: Android'de aktarım arka planda kullanıcının başlattığı iş olarak sürer

- **Durum:** Kabul, 3 Kasım 2026
- **Bağlam:** Android, ekranda olmayan bir uygulamanın ağ bağlantısını
  keser. Gönderen kodu başka bir uygulamada paylaşmak için TotalDrop'tan
  çıkınca bağlantı kopuyordu ("Software caused connection abort"). Bunun
  üzerine rendezvous sunucusu buluşma kutusunu kapatıyor ve kod
  geçersiz kalıyordu (#194). Oysa kodu paylaşmanın olağan yolu tam olarak
  budur.
- **Seçenekler:**
  1. `dataSync` türünde bir ön plan hizmeti. Her yerde çalışır, ama Play
     Console'da ön plan hizmeti beyanı ve tanıtım videosu ister. Android 15
     ve sonrasında da günde 6 saatle sınırlıdır.
  2. Kullanıcının başlattığı veri aktarımı işi
     (`JobInfo.Builder.setUserInitiated`, Android 14 ve sonrası). Android'in
     kişinin başlattığı aktarımlar için önerdiği yoldur, Play beyanı
     gerektirmez ve bildirim zorunludur.
  3. Bağlantı koptuğunda yeniden bağlanmak. Arka planda ağ yokken işe
     yaramaz.
- **Karar:** Seçenek 2. Gönderme ve alma başlarken uygulama bir iş
  kaydeder. İş sürdükçe "TotalDrop ile gönderiliyor/alınıyor" bildirimi
  görünür ve bildirimde **Durdur** düğmesi bulunur. Aktarım ne şekilde
  biterse bitsin iş de biter. Aktarımın kendisi yine Rust çekirdeğinde
  çalışır; iş yalnızca süreci ve ağ bağlantısını ayakta tutar.
  En düşük sürüm zaten Android 15 olduğu için başka bir yola gerek yok.
- **Neden:** Kişinin başlattığı bir aktarım için doğru araç budur. Ek Play
  beyanı istemez ve kod, paylaşıldığı süre boyunca geçerli kalır. Güvenlik
  değişmez: kod yine tek kullanımlıktır, kullanılmazsa sunucuda 15 dakikada
  silinir ve doğrulama kelimeleri karşılaştırılır.
- **Sonuçlar:**
  - Yeni izinler: `RUN_USER_INITIATED_JOBS`, `POST_NOTIFICATIONS` (ilk
    aktarımda sorulur; verilmezse aktarım yine sürer, bildirim görünmez)
    ve `ACCESS_NETWORK_STATE`.
  - Bildirim metinleri 35 dilde `l10n/app/` içindedir.
  - Sistem işi durdurursa ya da kişi görev yöneticisinden durdurursa
    aktarım da durur.
  - iOS'ta karşılığı ayrıca ele alınacak.
