# 0016: Bağışlar kişi adına değil, TotalDrop ekibi adına alınır

- **Durum:** Önerilen, 7 Kasım 2026 (Ozan, #352). Open Source Collective
  başvuruyu onaylayınca "Kabul" olur.
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
    Source Collective. Emanetçi gelen her bağıştan %10 alır. Her giriş ve
    çıkış Open Collective'in herkese açık defterinde görünür.
  - Open Collective sayfasını en az iki ekip üyesi yönetir; hesap
    hello@totaldrop.app adresine bağlıdır.
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
  adrese geçer. Bu değişiklik emanetçi onayı gelmeden birleştirilmez,
  yoksa bağış sayfası para alamaz. Ayrı bir PR'da ve 35 dilde, /donate
  sayfasının maliyet tablosuna emanetçi kesintisi yazılır.
