# 0001: Tek depo, platformlar ayrı

- **Durum:** Kabul, 27 Ekim 2026
- **Bağlam:** TotalDrop Linux, Android, Windows, macOS ve iOS'ta aynı
  protokolü konuşacak. Bir platformu güncellemek ötekileri bozmamalı.
- **Karar:** Tek depo ve tek `main` branch'i. Her platformun kendi klasörü,
  sürüm numarası, etiketi (`linux-v…`, `android-v…`) ve CI iş akışı var.
  Ortak çekirdek (`crates/`) bir kez değişir, bütün platformlara ulaşır.
  Uzun ömürlü platform branch'leri açılmaz.
- **Neden:** Bir güvenlik düzeltmesi tek yerde yapılır ve hiçbir platformda
  unutulmaz. Platform başına ayrı sürüm, hepsinin aynı anda yayınlanmasını
  gerektirmez.
- **Sonuçlar:** Çekirdekteki bir değişiklik bütün platform CI'larını
  çalıştırır ve hepsi geçmeden birleşmez. Ayrıntılar:
  [gelistirme.md](../gelistirme.md), [versioning.md](../versioning.md).
