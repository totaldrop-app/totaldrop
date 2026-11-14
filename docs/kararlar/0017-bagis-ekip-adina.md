# 0017: Bağışlar kişi adına değil, TotalDrop ekibi adına alınır

- **Durum:** Kabul, 10 Kasım 2026 (Ozan, #352); harcama kuralı 8 Kasım
  2026'da Sadık'ın önerisiyle eklendi. Ozan, Open Source Collective
  başvurusu sonuçlanmadan kabul etti; başvuru incelemede. Onay gelene
  kadar Open Collective sayfası bağış alamayabilir; başvuru reddedilirse
  karar yeniden açılır. İki kişi koşulu 10 Kasım 2026'da karşılanıyor:
  Open Collective'de ekip hesabının yanında iki kişi kendi hesabıyla
  yönetici. Bu koşul bozulursa karar yeniden açılır.
- **Bağlam:** [0005](0005-bagis.md) bağışların GitHub Sponsors üzerinden
  alınmasını kararlaştırdı, ama adres proje liderinin kişisel profiliydi
  (github.com/sponsors/ozdil). Proje artık bir ekiple yürüyor
  ([/team](https://totaldrop.app/team/)); bağışçının parayı bir kişiye
  değil projeye verdiğini görmesi gerekir. Öte yandan organizasyona
  banka, kart ya da vergi bilgisi bağlanmaz.
- **Karar:**
  - Bağışlar Open Collective'deki TotalDrop sayfasından alınır:
    [opencollective.com/totaldrop](https://opencollective.com/totaldrop).
    Tek bağış kanalı ilkesi (0005) sürer; kanal GitHub Sponsors'tan
    Open Collective'e geçer.
  - Para mali emanetçide (fiscal host) durur: kâr amacı gütmeyen Open
    Source Collective. Emanetçi gelen her bağıştan %10 alır; kart ya da
    PayPal işlemcisi bunun üstüne kendi ücretini keser (ülkeye ve ödeme
    yöntemine göre değişir). Her giriş ve
    çıkış Open Collective'in herkese açık defterinde görünür.
  - Open Collective sayfasını en az iki ayrı kişi, kendi kişisel
    hesaplarıyla yönetir. Collective'in iletişim adresi ve ekip hesabı
    hello@totaldrop.app'tir; ekip hesabı bu iki kişi sayısına girmez.
  - Harcamalar:
    - Her harcama talebini, talebi açandan başka bir yönetici onaylar.
      Kimse kendi talebini onaylamaz.
    - Para yetmediğinde önce ödenecekler, sırayla: sunucular (site,
      buluşma ve aktarma sunucusu), imza sertifikaları, 1.0'dan önce
      bağımsız güvenlik denetimi. Bunların dışındaki her harcama, talep açılmadan
      önce bir issue'da konuşulur; talep o issue'ya bağlanır.
    - Ekip hesabı (hello@totaldrop.app) harcama talebi açmaz ve onay
      vermez. Bu kutu bugün yalnızca proje liderine gider; aynı kişi hem
      kendi hesabıyla hem ekip hesabıyla onay verebilseydi iki kişi kuralı
      anlamını yitirirdi. Ekip hesabı ortak iletişim adresi ve kişisel
      hesapların yanında bir yedek giriş olarak durur; kutu ileride
      birden çok kişiye açılırsa da onay vermez. Kutuya kimin eriştiği
      değişirse bu madde hemen bir PR ile güncellenir.
    - Open Collective'in harcama politikalarında, talebi açan
      yöneticinin kendi talebini onaylamasını engelleyen ayar açık
      tutulur.
  - Bağış metinlerinde kişi adı geçmez; "TotalDrop" ya da "TotalDrop ekibi"
    denir.
  - Kişisel profil yeni bağış almaz; mevcut sponsorlara yeni adres
    bildirilir.
- **Neden:** Bağışçı projeye güvenerek para verir; adres bir kişiyi
  gösterince paranın kime ve neye gittiği belirsizleşir. Open Collective
  sayfası ekibi gösterir. Emanetçi, organizasyona ödeme bilgisi bağlamadan
  para almayı sağlar ve /donate sayfasındaki şeffaflık sözünü bağımsız bir
  defterle destekler.
- **Değerlendirilen seçenekler:**
  - Organizasyonun GitHub Sponsors profili: başvuru yapıldı, ama GitHub
    emanetçiyle bile kimlik doğrulaması için bir sahibin kişisel ödeme
    bilgisinin organizasyona bağlanmasını istiyor. Bu kurala aykırı, o
    yüzden tamamlanmadı.
  - Organizasyona doğrudan banka hesabı: kesinti yok, ama organizasyona
    ödeme ve vergi bilgisi bağlanır; bu da kurala aykırı.
  - Kişisel profil kalır: ekip yapısını yansıtmaz.
- **Sonuçlar:** README'ler, site (ana sayfa ve /donate), Linux AppStream
  dosyası, `.github/FUNDING.yml` ve [docs/marka.md](../marka.md) yeni
  adrese geçer. Ozan bu değişikliğin emanetçi onayından önce
  birleşmesini istedi (10 Kasım 2026); onay gelene kadar bağış sayfası
  para alamayabilir. /donate sayfasının maliyet tablosuna
  aynı PR'da ve 35 dilde "Bağış kesintileri" satırı eklenir (%10 emanetçi
  payı ve işlemci ücreti); sayfa "bir maliyet eklendiğinde önce bu sayfa
  değişir" sözünü böylece tutar.
