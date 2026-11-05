# Biçimsel model (Tamarin)

Bu klasör, hesap tasarımının ([hesap.md](../docs/hesap.md),
[karar 0013](../docs/kararlar/0013-hayalet-halka-hesap.md)) Tamarin
modelini tutar (#251). Kod yazılmadan önce tasarımın güvenlik iddiaları
burada lemma olarak kanıtlanır ya da çürütülür.

Model parça parça büyür. Her parça ayrı bir PR'dır:

| Parça | İçerik | Durum |
| --- | --- | --- |
| A1 | Üyelik zinciri: `oluştur`, `ekle`, `çıkar`, `döndür` | Bu klasörde |
| A2 | Çok değerli şeffaflık günlüğü, tanıklar, dahil etme zorlaması (P2) | Sırada |
| A3 | Kişi başı, sabitleme, gönderirken ve alırken tazelik (P3) | Sırada |
| A4 | Kurtarma, sıkı mod, `çatal_çöz` (P7) | Sırada |
| A5 | Bekleme süreleri için TLA+ değerlendirmesi | Sırada |

## Çalıştırmak

Tamarin 1.12.0 ve Maude 3.5.1 gerekir. CI ikisini de sabit sürüm ve
SHA-256 ile indirir (`.github/workflows/formal.yml`).

```sh
formal/check.sh          # her lemmayı ayrı ayrı kanıtlar
formal/mutants.py        # bozuk modellerde saldırının bulunduğunu sınar
```

İki komut da bir lemma kanıtlanmazsa ya da bir saldırı bulunmazsa sıfırdan
farklı çıkış koduyla biter. Lemma başına süre sınırı `LEMMA_TIMEOUT`
(saniye, varsayılan 600) ile değiştirilir.

## A1: üyelik zinciri (`hayalet-halka-zincir.spthy`)

### Modelde olanlar

- Cihaz anahtarları: `IK` ve yönetim anahtarı `AK`'nin iki yarısı
  (Ed25519, ML-DSA-65).
- Kayıtlar: `oluştur`, `ekle`, `çıkar`, `döndür`. Her kayıt halka kimliğini,
  önceki başın özetini, işlemi, yükü ve imzalayanı taşır.
- Her cihaz ve her kişi zincirin kendi görünümünü tutar. Görünümler
  ağdan gelen kayıtları aynı kurallarla denetleyerek ilerler.
- Saldırgan ağın tamamıdır (Dolev-Yao). Kötü niyetli posta kutusu sunucusu
  bu yüzden ayrıca modellenmez: her iletiyi okur, düşürür, yeniden oynatır,
  uydurur.
- Her cihaz anahtarı her an sızabilir. Çıkarılmış ya da çalınmış bir
  cihaz, anahtarları sızmış bir cihazdır. `AK`'nin iki yarısı ayrı ayrı
  sızar; böylece tek bir algoritmanın kırılması da modellenir.

### Soyutlamalar ve nedenleri

Model tasarımdan bazı yerlerde daha basit, bazı yerlerde daha güçlüdür.
Daha güçlü olduğu yerler, kanıtın gerçek sistemde neyi göstermediğini
söyler; bunlar sonraki parçalarda kapatılır.

| Soyutlama | Neden | Ne zaman genişler |
| --- | --- | --- |
| Halkada iki cihaz yeri var | Çok kümeli (multiset) üyelik, Tamarin aramasını sonlandırmıyordu; iki yer, üyelik değişikliğini ve eski kaydı yeniden oynatmayı kapsıyor | A2: çatallar için üç yer gerekecek |
| Her cihaz tek bir halkadadır | Bir cihaz yalnızca bir halka kurar ya da bir halkaya katılır. Bu yüzden P1'deki "aynı halkada" kısmı bu modelde kendiliğinden doğrudur; halka kimliğinin kayıtta bulunması burada sınanmaz | A2: iki halkalı senaryo |
| Halka kimliği `h('ring', ilk cihaz)` | Tasarımda `H("halka", oluştur)`; ikisi de ilk kayda bağlı ve değişmez, bu parçada fark etmez | A3: kişi sabitlemesiyle birlikte |
| Karma `AK` imzası iki ayrı imzadır, ikisi de aynı gövdeyi kendi etiketiyle (`'ed'`, `'ml'`) imzalar | Gövde imzalayanı iki açık anahtarıyla birlikte adlandırdığı için her yarı öbürünün anahtarına bağlıdır (hesap.md, "Anahtarlar") | Değişmez |
| `IK` tek bir anahtardır | Tasarımda `IK` de Ed25519 + ML-DSA-65'tir; `döndür` için iki yarı ayrımı bu parçada sınanmaz | A3 |
| `çıkar` kaydı çıkarılan cihazı anahtarlarının özetiyle adlandırır | Gerçek kayıt cihazı `IK` ile adlandırır; özet, Tamarin'in saldırgan bilgisini gereksiz yere çözümlemesini önler | Değişmez |
| Kayıtlar halka sırrı (`r_e`) taşımaz | Gizlilik (P3) A3'ün konusu | A3 |
| **Daha güçlü:** kişi zinciri ilk kayıttan (`oluştur`) itibaren doğrulanmış bir kanaldan sabitler | Tasarımda kişi ilk eşleşmede o anki başı sabitler; doğrulama kelimeleri yalnızca halka kimliğini, yani ilk kaydı doğrular. Üyelik değiştikten sonra eşleşen kişinin sabitlediği baş bu modelde kapsanmaz | A3 |
| **Daha güçlü:** kişi kayıtları açık görür, üyelerin açık anahtarlarını bilir ve yeni üye kümesini kendisi hesaplar | Tasarımda kişi yalnızca başı (`C_e`, işlem türü, önceki başın özeti), `AK` imzalarını ve Merkle kanıtlarını görür; bir üyenin imzaladığı bir `üyelik` başının tam olarak hangi cihazı eklediğini denetleyemez. Buradaki kişi kanıtları bu yüzden gerçek bir kişinin denetleyebileceğinden güçlüdür | A3: kişi, Merkle kökünü opak bir özet olarak gören başla modellenir |
| Bazı lemmalar adım sayısıyla sınırlıdır (`Step`) | Aramanın bitmesi için; sınır lemmanın metninde yazılı | Gerektikçe |

### Lemmalar

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| `sanity_contact_follows_add` | iz var | Kişi bir `ekle` kaydını kabul edebilir (model çalışıyor) |
| `sanity_contact_follows_add_remove` | iz var | Kişi önce `ekle`, sonra aynı üyenin `çıkar`ını kabul edebilir |
| `sanity_contact_follows_remove_by_slot_2` | iz var | İkinci yerdeki üye birinciyi çıkarabilir (ayna kurallar çalışıyor) |
| `compromised_member_can_add` | iz var | Bir üyenin iki `AK` yarısı da sızarsa saldırgan cihaz ekleyebilir. Bu iz olmasaydı aşağıdaki lemma yanlış bir nedenle doğru çıkabilirdi |
| `membership_change_needs_member_ak` | her iz | **P1:** kabul edilen her `ekle` ve `çıkar`, aynı önceki başta, adı geçen üyece imzalanmıştır. İstisnalar: o üyenin iki `AK` yarısı da önceden sızmışsa, ya da imzalayan saldırganın ürettiği bir anahtarsa. Tek yarının sızması yetmez; `IK` hiç yetmez |
| `signer_must_be_member` | her iz, sınırlı | Hiçbir `AK` sızmadıysa kişi, saldırganın ürettiği bir anahtarla imzalanmış kaydı kabul etmez. P1'in son istisnasını kapatır; sınırsız biçimi henüz yok |
| `rotate_needs_member_ik` | her iz | Kabul edilen her `döndür`, adı geçen üyenin `IK`'siyle aynı başta imzalanmıştır (`IK` sızmadıysa). `döndür` üye kümesini değiştiremez; bu, kabul kurallarında yapısal olarak böyledir |

Eski bir `ekle` kaydının yeniden oynatılarak çıkarılmış bir cihazın geri
getirilmesi, hiçbir anahtar sızmadığında P1'in sonucudur: kayıt yeni başı
adlandırmadığı için kabul edilmez. `add_does_not_bind_previous_head`
mutantı, kayıt başı adlandırmazsa bu saldırının bulunduğunu gösterir.

### Mutant sınamaları (`mutants.py`)

Bir kanıt ancak model, dayandığı kural olmadan başarısız oluyorsa bir şey
söyler. Her mutant modelin bir kopyasında tek bir denetimi bozar ve
Tamarin'in saldırı izini bulmasını bekler. Her düzenlemenin kaç yere
uygulanacağı yazılıdır; sayı tutmazsa betik durur.

| Mutant | Bozulan denetim | Beklenen saldırı |
| --- | --- | --- |
| `accept_checks_ed_half_only` | Kişi `AK` imzasının yalnızca Ed25519 yarısını denetler | Yalnızca o yarı sızınca sahte `ekle` kabul edilir |
| `accept_checks_ml_half_only` | Kişi yalnızca ML-DSA-65 yarısını denetler | Yalnızca o yarı sızınca sahte `ekle` kabul edilir |
| `accept_signer_not_member` | Kişi imzalayanın anahtarlarını kendi görünümünden değil kayıttan alır | Hiçbir anahtar sızmadan, saldırganın anahtarıyla `ekle` kabul edilir |
| `add_does_not_bind_previous_head` | `ekle` kaydı uzattığı başı adlandırmaz (dört kuralın hepsinde) | Eski `ekle` yeniden oynatılarak çıkarılmış cihaz geri gelir |
| `remove_signature_not_checked` | Kişi `çıkar` imzasını denetlemez | Hiçbir anahtar sızmadan sahte `çıkar` kabul edilir |
| `rotate_signature_not_checked` | Kişi `döndür` imzasını denetlemez | Hiçbir anahtar sızmadan sahte `döndür` kabul edilir |

### Henüz kanıtlanmayanlar

- **Saldırganın anahtarının üye olamaması, sınırsız:** "imzalayan bir
  cihazdır ya da bir cihazın iki `AK` yarısı da sızmıştır" lemması
  tümevarımla Tamarin'de sonlanmadı; bir yardımcı değişmez gerekiyor. Şimdilik
  sınırlı biçimi `signer_must_be_member`'dır.
- **Anahtarları sızmış, çıkarılmış cihaz:** cihaz çıkarıldıktan sonra
  anahtarları sızarsa, kişinin onu yeniden eklenmeden hiçbir kaydın
  imzalayanı olarak kabul etmemesi. Kişinin görünümü çıkarılan cihazı artık
  içermediği için böyle olması beklenir, ama bunu gösteren lemma (sınırlı
  biçimi de) Tamarin'de sonlanmadı. Bir sahibin birden çok görünümü
  olamayacağını gösteren bir değişmez gerekiyor; A2'de, her başın üye
  kümesini tek bir değere bağlayan günlükle birlikte denenecek. Bu, tasarımın
  "eski yetki" tehdidinin (H3) çekirdeğidir.
- Çatallar, günlük, tazelik, kurtarma ve sıkı mod: A2-A4.
