# Biçimsel model (Tamarin)

Bu klasör, hesap tasarımının ([hesap.md](../docs/hesap.md),
[karar 0013](../docs/kararlar/0013-hayalet-halka-hesap.md)) Tamarin
modelini tutar (#251). Kod yazılmadan önce tasarımın güvenlik iddiaları
burada lemma olarak kanıtlanır ya da çürütülür.

Model parça parça büyür. Her parça ayrı bir PR'dır:

| Parça | İçerik | Durum |
| --- | --- | --- |
| A1 | Üyelik zinciri: `oluştur`, `ekle`, `çıkar`, `döndür` | Bu klasörde |
| A2 | Çok değerli şeffaflık günlüğü, tanıklar, dahil etme zorlaması (P2) | Bu klasörde |
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
| Halkada iki cihaz yeri var | Çok kümeli (multiset) üyelik, Tamarin aramasını sonlandırmıyordu; iki yer, üyelik değişikliğini ve eski kaydı yeniden oynatmayı kapsıyor | A4: çatal çözme için üç yer gerekebilir |
| Her cihaz tek bir halkadadır | Bir cihaz yalnızca bir halka kurar ya da bir halkaya katılır. Bu yüzden P1'deki "aynı halkada" kısmı bu modelde kendiliğinden doğrudur; halka kimliğinin kayıtta bulunması burada sınanmaz | A3: iki halkalı senaryo, kişi başıyla birlikte |
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
| `signer_must_be_member` | her iz, sınırlı | Hiçbir `AK` sızmadıysa kişi, ilk kayıttan sonraki `ekle`'de saldırganın ürettiği bir anahtarla imzalanmış kaydı kabul etmez. P1'in son istisnasını yalnızca bu tek adımda gösterir; dürüst adımlar sonrasındaki ve sınırsız biçimi henüz yok |
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
  olamayacağını gösteren bir değişmez gerekiyor. Bu, tasarımın "eski yetki"
  tehdidinin (H3) çekirdeğidir. A2 bunun, çıkarıldığı baştaki kısmını
  günlükle kanıtlar (`removed_device_locked_out_of_old_head`); sonraki
  başlar hâlâ açık (aşağıda, A2).
- Çatallar, günlük, tazelik, kurtarma ve sıkı mod: A2-A4.

## A2: şeffaflık günlüğü (`hayalet-halka-gunluk.spthy`)

A1'in zinciri, anahtarları ve saldırganı aynen kalır. Değişen, kayıtların
nereden okunduğudur.

### Modelde olanlar

- Kayıtlar ağdan değil günlükten okunur. Günlük anahtarı uzatılan başa
  bağlıdır: `L = h('log', baş)`. Bir anahtarın altında birden çok değer
  durabilir ve hiçbiri silinmez.
- Herkes günlüğe yazabilir (gerçekte ARC jetonuyla). Günlük yalnızca o baş
  için geçerli değerleri tutar; okuyanlar geçersiz değerleri zaten yok
  saydığı için geçersiz bir değerin yazılması hiçbir şeyi değiştirmez.
- Dürüst cihazlar kendi kayıtlarını yazdıkları adımda günlüğe de yazar.
  Bu, tanıkların dahil etme zorlamasıdır: sunucu dürüst bir kaydı
  sansürleyemez.
- Okuyan (cihaz ya da kişi) çatal kuralını uygular. Bir `AK` kaydı, aynı
  anahtar altında daha önce başka bir geçerli `AK` kaydı yazılmamışsa kabul
  edilir; `döndür`'ün önüne geçer (çatal kuralı 1). Bir `döndür`, aynı
  anahtar altında daha önce başka hiçbir geçerli değer yazılmamışsa kabul
  edilir.
- Saldırgan yine ağın tamamıdır ve her anahtar her an sızabilir.

### Soyutlamalar ve nedenleri

| Soyutlama | Neden | Ne zaman genişler |
| --- | --- | --- |
| Zaman yok: dürüst bir kayıt yazıldığı anda günlüktedir | Gerçek sınır 1 saat dahil etme + 1 saat kök tazeliğidir (P3: 2 saat). Burada yalnızca sıra sınanır | A3 (tazelik), A5 (süreler) |
| Günlük tek bir küresel kümedir | Tanıkların çoğunluğu (k > n/2) dürüstse hiçbir okuyan farklı bir günlük görmez; bu varsayım modelin kendisidir. Tanıklar sunucuyla iş birliği yaparsa P2 ve P3 güvencesi yoktur (hesap.md) | Değişmez |
| Değerler şifresiz | Sunucunun içeriği okuyamaması P5'tir, P2 değil | A3 |
| `oluştur` günlüğe yazılmaz | Kişi ilk kaydı A1'deki gibi doğrulanmış kanaldan sabitler (`L_0` anahtarı burada kullanılmaz) | A3 |
| **Daha katı:** birbiriyle çatallanan `döndür`'ler en küçük özetle birleşmez; okuyan durur | Zararsız çatal birleşmesi (çatal kuralı 2) güvenliği değil kullanılabilirliği etkiler; kanıtlanan özellikler bu katı kuralla geçerlidir, birleşme A4'te `çatal_çöz` ile birlikte modellenir | A4 |
| Bir `döndür`'ü izlemiş okuyan, sonra aynı anahtar altında yazılan `AK` kaydına geçmez | Tasarımda `AK` kaydı kazanır ve okuyan ona geçer. Burada `AK` kaydının, yazıldıktan sonra okuyan herkes için kazandığı gösterilir (`removed_device_locked_out_of_old_head`); geçiş A4'te | A4 |

### Lemmalar

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| `sanity_contact_follows_add_via_log` | iz var | Kişi günlükten bir `ekle` kabul edebilir |
| `sanity_leaked_device_rotates_before_removal` | iz var | `IK`'si sızmış bir cihazın `döndür`'ü, çıkarma günlüğe yazılmadan önce o başta kabul edilebilir. Aşağıdaki kilit lemmasının kapattığı pencere gerçektir |
| `accepted_records_are_logged` | her iz | **P2, görünürlük:** herhangi bir okuyanın kabul ettiği her kayıt, kabulden önce günlükte geçerli bir değer olarak vardır |
| `ak_read_was_logged`, `ak_reads_agree` | her iz, yardımcı | Bir `AK` okuması günlükteki bir değeri okur; aynı anahtar altındaki iki `AK` okuması aynı değeri okur |
| `no_split_view_membership` | her iz | **P2, tek görünüm:** iki okuyan (cihaz ya da kişi) aynı başın iki farklı üyelik değişikliğini kabul etmez. Hangi anahtar sızarsa sızsın geçerlidir; tek varsayım dürüst tanık çoğunluğudur |
| `removed_device_locked_out_of_old_head` | her iz | **H3 çekirdeği:** bir üye `x`'i bir baştan çıkardıktan sonra hiçbir okuyan o başta `x`'in imzaladığı bir kaydı kabul etmez; `x`'in bütün anahtarları sızmış ve sunucu yardım ediyor olsa bile |

### Mutant sınamaları

Mutantlar bozuk modele modelin kendi lemmalarını almaz: bozulmanın yanlış
çıkardığı bir `[reuse]` yardımcısı varsayılsaydı saldırıyı gizlerdi.

| Mutant | Bozulan denetim | Beklenen saldırı |
| --- | --- | --- |
| `honest_remove_not_logged` | Tanıklar dahil etmeyi zorlamaz; sunucu dürüst `çıkar`'ı sansürler | Çıkarılmış cihazın `döndür`'ü eski başta kabul edilir |
| `reader_accepts_rotate_despite_fork` | Okuyan, aynı anahtar altında `AK` kaydı varken `döndür` kabul eder | Aynı saldırı, `çıkar` günlükte olsa bile |
| `reader_ignores_ak_fork` | Okuyan, aynı anahtar altında ikinci bir `AK` kaydı aramaz | İki kişi aynı başın iki farklı `ekle`'sini kabul eder (bölünmüş görünüm) |
| `accept_from_network_not_log` | Kişi `ekle` kaydını günlükten değil ağdan alır | Günlükte olmayan bir kayıt kabul edilir |

### Henüz kanıtlanmayanlar

- **Çıkarılmış cihaz sonraki başlarda:** `x` çıkarıldığı baştan kilitli
  kalır (yukarıda), ama sonraki başlarda da hiçbir kaydın imzalayanı
  olarak kabul edilmediğini gösteren sınırsız lemma (`x` yeniden
  eklenmedikçe) Tamarin'de sonlanmadı. Geçmişi yakalayan bir kişi `x`'in
  çıkarılmadan önceki kayıtlarını meşru olarak kabul edebildiği için lemma
  "çıkarmadan önce imzalanmış" istisnasını taşır; bu da bir yardımcı
  değişmez ya da hayalet durum gerektiriyor. A3'te kişi başıyla birlikte
  yeniden denenecek.
