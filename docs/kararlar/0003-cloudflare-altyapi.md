# 0003: Altyapı Cloudflare'de

- **Durum:** Kabul, 28 Ekim 2026
- **Bağlam:** Farklı ağlardaki cihazlar için adres bulma (STUN) ve doğrudan
  yol açılmadığında şifreli trafiği taşıyan bir aktarma sunucusu gerekli.
  Seçenekler: kendi sunucumuz (Hetzner), Cloudflare Workers, Cloudflare
  Realtime TURN.
- **Karar:** Site, rendezvous ve aktarma Cloudflare'de. Adres bulma için
  Cloudflare'in ücretsiz STUN'u; aktarma için her aktarıma bir Durable
  Object, WebSocket üzerinden. Kendi sunucusunda çalıştırmak isteyenler için
  aynı protokolü konuşan Rust aktarma sunucusu depoda kalır.
- **Neden:** Her şey tek yerden yönetilir; yeni bir makineyi güncellemek ve
  korumak gerekmez. Worker çıkış trafiği ücretsiz, maliyet neredeyse sıfır.
  Yerel ölçümde yalnızca aktarma yoluyla 100 MB 8 saniye; gerçek
  internette 28 saniye (iki uç aynı ev bağlantısındaydı).
- **Gözden geçirme koşulu:** Aktarma hızı yetmezse ya da Cloudflare'in
  kullanım koşulları sorun çıkarırsa Cloudflare Realtime TURN ikinci yol
  olarak eklenir (ayda ilk 1.000 GB ücretsiz, sonrası GB başına 0,05 USD).
  Bunun maliyeti bağışlardan karşılanır.
- **Sonuçlar:** Cloudflare IP adreslerini görür; gizlilik politikasında
  yazılı. Gizlilik modu (yalnızca aktarma, ileride Tor) IP'sini de
  gizlemek isteyenler için.
