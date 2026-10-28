# 0002: Protokol v0

- **Durum:** Kabul, 27 Ekim 2026; dış inceleme 1.0'dan önce
- **Bağlam:** Öncelik güvenlik ve anonimlik. Kod, kısa ve söylenebilir
  olmalı; aradaki sunucular hiçbir şey okuyamamalı.
- **Karar:**
  - Aktarım kodları Magic Wormhole modelinde, CPace ile (balanced PAKE):
    saldırgan tek bir tahmin hakkı alır, kodu çevrimdışı deneyemez.
  - Bağlantı QUIC (TLS 1.3, her aktarım için yeni, tek kullanımlık ham
    anahtarlar). Güven, içindeki Noise XXpsk3 el sıkışmasından gelir.
  - El sıkışmanın hemen ardından ML-KEM-768 ile kuantum sonrası hibrit adım.
  - Dosyalar STREAM yapısıyla parça parça şifrelenir, BLAKE3 ile doğrulanır.
- **Neden:** Bugün kaydedilen trafik, kuantum bilgisayarlar geldiğinde bile
  okunamamalı; tek kullanımlık anahtarlar sunucuların aktarımları
  birbirine bağlamasını engeller.
- **Sonuçlar:** Kendi CPace katmanımız var (güncel bir Rust paketi yok); bu
  katman ve hibrit adım 1.0 öncesi bağımsız denetimin kapsamında. Belgeler:
  protokol taslağı ve [protocol-v0.1-notes.md](../protocol-v0.1-notes.md).
