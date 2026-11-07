# 0016: Bağışlar kişi adına değil, TotalDrop ekibi adına alınır

- **Durum:** Önerildi, 7 Kasım 2026 (Ozan, #352). Organizasyonun
  sponsor profili açılınca "Kabul" olur.
- **Bağlam:** [0005](0005-bagis.md) bağışların GitHub Sponsors üzerinden
  alınmasını kararlaştırdı, ama adres proje liderinin kişisel profiliydi
  (github.com/sponsors/ozdil). Proje artık bir ekiple yürüyor
  ([/team](https://totaldrop.app/team/)); bağışçının parayı bir kişiye
  değil projeye verdiğini görmesi gerekir. Öte yandan organizasyona
  doğrudan banka ve vergi bilgisi bağlanmaz.
- **Karar:**
  - Bağışlar `totaldrop-app` organizasyonunun GitHub Sponsors profilinden
    alınır: [github.com/sponsors/totaldrop-app](https://github.com/sponsors/totaldrop-app).
    Tek bağış kanalı ilkesi (0005) sürer.
  - Para bir mali emanetçide (fiscal host) durur: Open Source Collective.
    Organizasyona banka ya da vergi bilgisi bağlanmaz; her giriş ve çıkış
    emanetçinin herkese açık defterinde de görünür.
  - Bağış metinlerinde kişi adı geçmez; "TotalDrop" ya da "TotalDrop ekibi"
    denir.
  - Kişisel profil yeni bağış almaz; mevcut sponsorlara organizasyon
    profili bildirilir.
- **Neden:** Bağışçı projeye güvenerek para verir; adres bir kişiyi
  gösterince paranın kime ve neye gittiği belirsizleşir. Organizasyon
  profili ekibi gösterir. Mali emanetçi, organizasyona ödeme bilgisi
  bağlamadan para almayı sağlar ve /donate sayfasındaki şeffaflık
  sözünü bağımsız bir defterle destekler.
- **Değerlendirilen seçenekler:**
  - Organizasyona doğrudan banka hesabı: kesinti yok, ama organizasyona
    ödeme ve vergi bilgisi bağlanır; bu kurala aykırı.
  - Yalnızca Open Collective: emanetçi aynı, ama bağış GitHub'dan çıkar ve
    tek kanal ilkesi (0005) GitHub Sponsors'tan başka bir yere taşınır.
  - Kişisel profil kalır: ekip yapısını yansıtmaz.
- **Sonuçlar:** README'ler, site (ana sayfa ve /donate), Linux AppStream
  dosyası, `.github/FUNDING.yml` ve [docs/marka.md](../marka.md) yeni
  adrese geçer. Bu değişiklik organizasyon profili yayında olmadan
  birleştirilmez, yoksa bağlantılar kırık olur. Emanetçi kesinleşince,
  ayrı bir PR'da ve 35 dilde, /donate sayfası paranın emanetçide durduğunu
  söyler ve kesintisi maliyet tablosuna yazılır.
