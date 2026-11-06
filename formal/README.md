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
| A3 | Döndürmenin oturması ve yazanın tazeliği (A2'deki açığın düzeltmesi); halka sırrı, teklifler, gönderirken ve alırken tazelik (P3) | Bu klasörde |
| A4 | Kurtarma ve P7'nin kurtarma kısmı (A4a); sıkı mod (A4b), `çatal_çöz` (A4c), tasarım kurallarının `hesap.md`'ye yazılması (A4d) | A4a ve A4b bu klasörde, gerisi sırada |
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
- Herkes günlüğe yazabilir (gerçekte ARC jetonuyla). Günlük yalnızca o
  başın bir üyesince imzalanmış değerleri tutar; okuyanlar geçersiz
  değerleri zaten yok saydığı için başka bir değerin yazılması hiçbir şeyi
  değiştirmez.
- Dürüst cihazlar kendi kayıtlarını yazdıkları adımda günlüğe de yazar.
  Bu, tanıkların dahil etme zorlamasıdır: sunucu dürüst bir kaydı
  sansürleyemez. Yazan, anahtarın altında başka bir değer olsa da yazar;
  çatal o zaman okuyanları durdurur.
- Okuyan (cihaz ya da kişi) çatal kuralını uygular. Bir `AK` kaydı, aynı
  anahtar altında daha önce başka bir geçerli `AK` kaydı yazılmamışsa kabul
  edilir; `döndür`'ün önüne geçer (çatal kuralı 1). Bir `döndür`, aynı
  anahtar altında daha önce başka hiçbir geçerli değer yazılmamışsa kabul
  edilir. İzlediği bir daldan geri dönmez (çatal kuralı 4).
- Saldırgan yine ağın tamamıdır ve her anahtar her an sızabilir.

### Soyutlamalar ve nedenleri

| Soyutlama | Neden | Ne zaman genişler |
| --- | --- | --- |
| Zaman yok: dürüst bir kayıt yazıldığı anda günlüktedir | Gerçek sınır 1 saat dahil etme + 1 saat kök tazeliğidir (P3: 2 saat). Burada yalnızca sıra sınanır | A3 (tazelik), A5 (süreler) |
| Günlük tek bir küresel kümedir | Tanık varsayımının yerini tutar. n tanık, kök başına k imza ve sunucuyla iş birliği yapan f tanık için: sansürün olmaması `f < k`, tek görünüm `f < 2k − n` ister. Dürüst çoğunluk tek başına yetmez (n = 5, k = 3, f = 2: iki kötü tanık iki farklı kökü de imzalar, birer dürüst tanık da birini). Bu koşullar tutmazsa P2 ve P3 güvencesi yoktur | Değişmez; eşik tasarımda düzeltilecek (aşağıda) |
| **Daha güçlü:** okuyan, okumadan önce yazılmış her değeri görür | Kusursuz tazelik. Gerçekte kişi en çok 1 saat eski bir kök görür | A3 |
| Değerler şifresiz | Sunucunun içeriği okuyamaması P5'tir, P2 değil | A3 |
| `oluştur` günlüğe yazılmaz | Kişi ilk kaydı A1'deki gibi doğrulanmış kanaldan sabitler (`L_0` anahtarı burada kullanılmaz) | A3 |
| **Daha katı:** bir üyenin imzaladığı her değer, işlemi anlamsız olsa da çatal sayılır | Tasarım geçersiz değerleri yok sayar. Bu yalnızca okuyanın daha sık durmasına yol açar; güvenlik lemmalarını kolaylaştırmaz | A4 |
| **Daha katı:** birbiriyle çatallanan `döndür`'ler en küçük özetle birleşmez; okuyan durur | Zararsız çatal birleşmesi (çatal kuralı 2) güvenliği değil kullanılabilirliği etkiler; birleşme A4'te `çatal_çöz` ile birlikte modellenir | A4 |

### Lemmalar

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| `sanity_contact_follows_add_via_log` | iz var | Kişi günlükten bir `ekle` kabul edebilir |
| `sanity_leaked_device_rotates_before_removal` | iz var | `IK`'si sızmış bir cihazın `döndür`'ü, çıkarma günlüğe yazılmadan önce o başta kabul edilebilir |
| `removed_device_survives_on_rotate_branch` | iz var | **Modelin bulduğu açık** (aşağıda): çıkarılmış cihaz, kendi `döndür` dalını izleyen bir kişiye çıkarmadan sonra da kayıt kabul ettirir |
| `accepted_records_are_logged` | her iz | **P2, görünürlük:** herhangi bir okuyanın kabul ettiği her kayıt, kabulden önce günlükte geçerli bir değer olarak vardır. Kabul kuralları günlükten okuduğu için yapısal olarak doğrudur; `accept_from_network_not_log` mutantı bunun neden önemli olduğunu gösterir |
| `ak_read_was_logged`, `ak_reads_agree` | her iz, yardımcı | Bir `AK` okuması günlükteki bir değeri okur; aynı anahtar altındaki iki `AK` okuması aynı değeri okur |
| `no_split_view_membership` | her iz | **P2, üyelik değişikliklerinde tek görünüm:** iki okuyan aynı başın iki farklı `ekle` ya da `çıkar`'ını kabul etmez; hangi anahtar sızarsa sızsın. `döndür` ile `AK` kaydı arasındaki bölünme bu lemmanın dışındadır (aşağıdaki açık) |
| `removed_device_locked_out_of_old_head` | her iz | **H3, çıkarıldığı başta:** bir üye `x`'i bir baştan çıkardıktan sonra hiçbir okuyan o başta `x`'in imzaladığı bir kaydı kabul etmez; `x`'in bütün anahtarları sızmış, sunucu yardım ediyor ve `x` o başa kendi kaydını önce yazmış olsa bile (yarış). Kabul edilebilen tek kayıt çıkarmanın kendisidir |

### Mutant sınamaları

Mutantlar bozuk modele modelin kendi lemmalarını almaz: bozulmanın yanlış
çıkardığı bir `[reuse]` yardımcısı varsayılsaydı saldırıyı gizlerdi.

| Mutant | Bozulan denetim | Beklenen saldırı |
| --- | --- | --- |
| `honest_remove_not_logged` | Tanıklar dahil etmeyi zorlamaz; sunucu dürüst `çıkar`'ı sansürler | Çıkarılmış cihazın `döndür`'ü eski başta kabul edilir |
| `reader_accepts_rotate_despite_fork` | Okuyan `döndür` için yalnızca başka `döndür`'lere bakar; `AK` kaydının önceliği (çatal kuralı 1) yok | Aynı saldırı, `çıkar` günlükte olsa bile |
| `reader_ignores_ak_fork` | Okuyan, aynı anahtar altında ikinci bir `AK` kaydı aramaz | İki kişi aynı başın iki farklı `ekle`'sini kabul eder (bölünmüş görünüm) |
| `accept_from_network_not_log` | Kişi `ekle` kaydını günlükten değil ağdan alır | Günlükte olmayan bir kayıt kabul edilir |

### Modelin bulduğu açık: `döndür` dalında kalan kişi

Çatal kuralı 1, "çıkarılmak üzere olan bir cihazın aynı anda yazdığı
`döndür`, çıkarmayı engelleyemez" der. Ama kural 4'e göre bir gözlemci,
kendi sabitlediği başın altındaki çatalları yok sayar. İkisi birlikte şu
izi verir (`removed_device_survives_on_rotate_branch`):

1. `d`, `n`'yi ekler; kişi `c` bu `ekle`'yi kabul eder.
2. `n`'nin `IK`'si sızar. Saldırgan `n` adına bir `döndür` yazar; `c` onu
   kabul eder ve yeni başa geçer.
3. `d`, `n`'yi eski baştan çıkarır. Bu baştan sonra okuyan herkes için
   `çıkar` kazanır, ama `c` o başa bir daha bakmaz.
4. Saldırgan `c`'nin dalını `n` adına uzatmaya devam eder. `n`'nin `AK`'si
   de sızmışsa `d`'yi bile çıkarabilir.

Böylece çıkarılmış bir cihaz, çıkarmadan önce kendi dalına geçirdiği
kişiler için halkanın tek yetkilisi olarak kalır. Bu, H3'ün tam
konusudur. Halka (`d`) kendi anahtarı altındaki yabancı `döndür`'ü görüp
uyarır, ama kişi tarafında bir kural gerekir. Bu modelde (A2)
"çıkarılmış cihaz sonraki başlarda da kilitlidir" iddiası **yanlıştır**,
yalnızca kanıtlanmamış değil. Düzeltme A3'te (aşağıda).

### Tasarıma geri bildirim (A4'te `hesap.md`'ye işlenecek)

- Çatal kuralı 1 ile 4 arasındaki çelişki (yukarıdaki açık) ve A3'teki
  düzeltme kuralı (oturma süresi ve yazanın tazeliği).
- `AK` kaydının denetlendiği tanıklı köke bağlanması ve geç köke giren
  kaydın geçersiz sayılması (A3, "Modelin dayandığı varsayım").
- Tanık eşiği: `hesap.md` "k > n/2" diyor. Tek görünüm için `k`, iş birliği
  yapabilecek tanık sayısı `f` ile birlikte `2k − n > f` olacak şekilde
  seçilmeli (ör. n = 5, k = 4 iken en çok f = 2 kötü tanık tolere edilir;
  k = 3 iken yalnızca f = 0).

## A3: döndürmenin oturması (`hayalet-halka-tazelik.spthy`)

A2'nin modeli aynen kalır; A2'nin bulduğu açığı kapatan iki kural eklenir.
Saat yine yoktur: "süre geçti" bir olaydır (`Old`).

### Eklenen kurallar

- **Oturma:** okuyan (cihaz ya da kişi) bir `döndür`'ü ancak o kayıt
  günlükte oturma süresi `D` kadar durduktan sonra izler.
- **Yazanın tazeliği:** dürüst bir cihaz, bir kaydı yalnızca anahtarı
  altında henüz hiçbir değerin oturmadığı bir baştan yazar. Gerçek kural:
  yazanın taze kökü, o başın anahtarı altında `D_w`'den eski bir değer
  göstermemelidir. Daha yeni bir değer (ör. henüz oturmamış bir `döndür`)
  yazmayı **engellemez**: `AK` kaydı yazılır ve çatal kuralı 1 gereği
  kazanır. "Anahtar altında hiç değer olmamalı" diye okunursa, `IK`'si
  sızmış bir cihaz her `D` süresinde bir `döndür` yazarak (birer ARC
  jetonuyla) kendi çıkarılmasını süresiz erteleyebilirdi.

İkisi birlikte şunu sağlar: bir `döndür` henüz otururken yazılan bir
`çıkar`, `döndür` oturmadan önce günlüktedir ve çatal kuralı 1 gereği onu
yener (`sanity_remove_beats_unsettled_rotate`). `döndür` oturduktan sonra
ise dürüst cihaz o eski baştan yazamaz; önce `döndür`'ü izler ve çıkarmayı
yeni baştan yazar.

### Sürelerin hesabı (A5'te kesinleşecek)

Süreler, saldırganın seçtiği yazma anından değil, `döndür`'ün tanıklı bir
köke **ilk girdiği** andan (`T_giriş`) ölçülür. Yazanın kökü en çok 1 saat
eskidir ve `döndür`'ü `D_w`'den eski göstermiyordur; bu yüzden yazanın
kaydı `T_giriş + D_w + 1 saat`'ten önce yazılır ve en geç 1 saat sonra
köke girer, yani `T_giriş + D_w + 2 saat`'ten önce. Okuyan yaşı kendi
kökünün zaman damgasıyla ölçüyorsa `D > D_w + 2 saat + tanık saat farkı`
yeter; yerel saatiyle ölçüyorsa kökün eskiliği için 1 saat daha eklenir.
`D_w = 0` (yazanın kökü değeri hiç göstermiyor) ve birkaç dakikalık saat
farkıyla `D = 4 saat` güvenlidir.

### Modelin dayandığı varsayım: kayıt zamanında günlüğe girer

Model dürüst bir kaydı, yazıldığı adımda günlüğe koyar. A2'de bu yalnızca
sıra içindi; burada yük taşır, çünkü yazanın tazelik denetimi kayıt
**köke girdiğinde** de geçerli olmalıdır. Gerçekte tanıklar 1 saatlik
sınırı kaydı **aldıkları** andan sayar. Ağdaki bir saldırgan dürüst bir
`çıkar`'ın yüklenmesini düşürüp `döndür` oturduktan sonra günlüğe
yazarsa, `döndür`'ü izlemiş okuyanlar onda kalır, sonra gelenler `çıkar`'ı
izler: bölünmüş görünüm (`no_split_view_membership` yalnızca iki üyelik
değişikliğini karşılaştırdığı için bunu yakalamaz). Tasarımın bunu
önlemesi gerekir; A4'te `hesap.md`'ye yazılacak öneri:

- Bir `AK` kaydı, denetlendiği tanıklı kökü (özeti ya da zaman damgası)
  taşır. Okuyanlar ve tanıklar, o kökten `1 saat + saat farkı`'ndan sonra
  köke giren kaydı geçersiz sayar.
- Yazan, kayıt zamanında köke girmezse tazeliği yeniden denetler ve kaydı
  yeniden imzalar.

### Dürüst döndürmeye etkisi

Döndüren cihaz kendi `döndür`'ünü `D` boyunca bekleyen sayar ve eski
`IK`'sini kullanmayı sürdürür. Bu sürede aynı başa bir `AK` kaydı yazılırsa
`döndür` düşer (kural 1) ve cihaz yeni baştan yeniden döndürür. Haftalık
döndürmede bunun kullanıcıya etkisi yoktur. `IK`'sinin sızdığından
şüphelenilen bir cihaz için döndürme 4 saate kadar sürer; o durumda doğru
işlem çıkarmadır.

### Lemmalar

A2'nin "her iz" lemmaları bu modelde de geçerlidir ve yeniden kanıtlanmaz:
model yalnızca sonucu olmayan bir kural (`Log_Age`), yeni olaylar ve
kısıtlar ekler. Bu modelin bir izinden `Log_Age` adımları çıkarılınca
A2'nin bir izi kalır ve A2'nin hiçbir lemması ya da kısıtı `Old`,
`WriteFresh` ya da `Settled`'dan söz etmez. Bu gerekçe iki dosyanın aynı
kalmasına dayanır: A2'de bir kural değişirse A3'te de aynı değişiklik
yapılmalıdır (ileride A3'ü A2'den betikle üretmek bunu güvenceye alır). ("Her
iz" lemmalarını burada yeniden kanıtlamak, yeni kısıtlar aramayı
büyüttüğü için 10 dakikada bitmiyordu.) A2'nin iki "iz var" lemması
burada yeniden kanıtlanır, çünkü kısıtlar bir izi ortadan kaldırabilir.
Yeni olanlar:

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| `sanity_contact_follows_settled_rotate` | iz var | Oturmuş bir `döndür` yine izlenebilir; kural döndürmeyi engellemez |
| `sanity_remove_beats_unsettled_rotate` | iz var | `IK`'si sızmış cihazın `döndür`'ü henüz otururken aynı baştan yazılan `çıkar` geçer ve kişi onu izler; bekleyen bir `döndür` çıkarmayı geciktiremez |
| `no_rotate_branch_past_removal` | her iz | **H3, A2'deki açık kapandı:** bir üye `x`'i bir baştan çıkarıyorsa hiçbir okuyan o başta `x`'in `döndür`'ünü izlemez; hangisi önce yazılmış olursa olsun. Yalnızca çıkarmanın yapıldığı başı kapsar; A2'deki saldırı izinin ikinci adımını engellemeye bu yeter, genel biçim aşağıda açık |

### Mutant sınamaları

| Mutant | Bozulan denetim | Beklenen saldırı |
| --- | --- | --- |
| `rotate_followed_before_settled` | Okuyan `döndür`'ü günlüğe girer girmez izler (A2'nin kuralı) | A2'deki `döndür` dalı saldırısı |
| `writer_not_fresh` | Dürüst cihaz, altında bir `döndür`'ün oturduğu eski baştan yazar | Kişi `n`'nin `döndür`'ünü, bir üyenin `n`'yi çıkardığı başta izler |

### Henüz kanıtlanmayanlar

- **Sınırsız biçim:** "çıkarmadan sonra okuyan, `x`'in yalnızca çıkarmadan
  önce günlüğe girmiş kayıtlarını kabul eder (ya da `x` yeniden eklenmiştir)"
  lemması tümevarımla 450 saniyede sonlanmadı. Bilinen saldırı
  (`döndür` dalı) kapalıdır; genel biçim için bir yardımcı değişmez
  gerekiyor.

## A3b: çıkarılan cihazın yapamadıkları, P3 (`hayalet-halka-gizlilik.spthy`)

A3'ün zinciri, günlüğü, oturma ve yazanın tazeliği aynen kalır. Eklenenler:

- Her kayıt başlattığı dönemin halka KEM anahtarını (`pk(r_e)`) taşır;
  yazan, `r_e`'yi yeni başın üyelerine verir (gerçekte üyelerin `KK`
  anahtarlarına HPKE ile).
- Durumu sızan bir cihaz, üyesi olduğu her başın `r_e`'sini saldırgana
  verir. Çıkarılmış ya da çalınmış cihaz böyle bir cihazdır.
- Kişi teklifi başının KEM anahtarına şifreler. Halkadaki bir cihaz kişiye
  halka adına, `IK`'siyle imzalı teklif gönderir.
- Kişi göndermeden ve bir teklifi almadan önce tazelik denetimi yapar:
  kullandığı başın anahtarı altında `AK` kaydı da, oturmuş bir değer de
  yoktur. Henüz oturmamış bir `döndür` göndermeyi engellemez; o sırada
  eski baş hâlâ günceldir. Bu, `hesap.md`'deki kuraldan gevşektir (orada
  anahtar altındaki her geçerli değer göndermeyi durdurur); gevşek kural
  A4'te `hesap.md`'ye yazılacak, çünkü katı kuralla kişiler her haftalık
  `döndür`'den sonra `D` boyunca gönderemezdi.
- Döndüren cihaz kendi `döndür`'ünü, öbür okuyucular gibi, ancak oturunca
  izler (A3, "Dürüst döndürmeye etkisi"). O zamana kadar `AK` kayıtlarını
  eski baştan yazar; orada `döndür`'ün önüne geçerler (kural 1).

Kayıt gövdesi değiştiği için (KEM anahtarı eklendi) bu yeni bir modeldir:
A1-A3'ün lemmaları buraya taşınmaz ve burada P1 ile P2 iddia edilmez.

### Soyutlamalar

| Soyutlama | Neden | Ne zaman genişler |
| --- | --- | --- |
| `r_e` üyelere güvenli bir olguyla verilir | HPKE ve anahtar onayı (MAC) burada değil, protokol notlarında ve test vektörlerinde sınanır | Aşama C |
| Teklif gizliliği halka anahtarının gizliliğiyle ifade edilir | Teklifin KEM anahtarına şifrelenmesi standart açık anahtarlı şifrelemedir; bu adımı lemma olarak eklemek aramayı bitirmiyordu. Kanıtlanan: kişinin şifrelediği anahtar, yalnızca o başın üyesi bir cihaz sızınca ya da başı yalnızca saldırgan yazmışsa bilinir | Değişmez |
| Kusursuz tazelik: kişi göndermeden önce yazılmış her değeri görür | P3'teki 2 saatlik sınır (1 saat dahil etme + 1 saat kök tazeliği) A5'te | A5 |
| Teklif imzası halkayı, gönderenin başını ve teklif kimliğini bağlar; alıcı kişiyi ve alıcının başını bağlamaz | Bir teklifin başka bir kişiye yeniden gönderilmesi burada dışlanmaz; `hesap.md` "Gönderim"deki alıcı başı yankısı ve kişiye bağlama protokol notlarında (aşama C) kesinleşecek | Aşama C |
| Kişinin tazelik denetimi ile teklifin gönderilmesi iki adımdır | Tek adım aramayı bitirmiyordu; denetim gönderim adımında yeniden yapılır, bu yüzden lemmalar gönderim anı için geçerlidir | Değişmez |
| Kişi ile halka arasındaki el sıkışma (Noise + ML-KEM) ve kişi anahtarı `k_AB` yok | El sıkışma mevcut protokolde sınanıyor; `k_AB` yenilemesi kişi defteriyle birlikte ele alınacak | A4 ya da ayrı bir parça |

### Lemmalar

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| `ring_secret_secret`, `kem_origin` | her iz, yardımcı | Bir `r_e`, yalnızca o başın sızmış bir üyesi üzerinden bilinir; bir başın KEM anahtarı ya bir üyenin `pk(r_e)`'sidir ya da günlüğe ağdan yazılmıştır |
| `at_rotate_head_settled` | her iz, yardımcı (tümevarım) | Bir `döndür`'ün başlattığı başta bulunan herkesten önce, bir önceki başın anahtarı altında bir değer oturmuştur (tazelik kısıtlarına bu yeter) |
| `sanity_contact_sends`, `sanity_contact_receives`, `sanity_contact_sends_after_removal`, `sanity_remove_from_rotate_head` | iz var | Kişi gönderebilir, bir üyeden teklif alabilir ve bir çıkarmayı izledikten sonra yeni başa gönderebilir; bir çıkarma bir `döndür`'ün başından yazılabilir (aşağıdaki lemma boş değildir) |
| `sent_head_key_secret` | her iz | **P3, gizlilik:** kişinin teklifi şifrelediği anahtar, yalnızca o başın bir üyesi durumunu sızdırdıysa ya da başı hiçbir üye yazmadıysa bilinir. İkinci durum, ağdan günlüğe yazılmış ve kişinin izlediği bir baştır; kişinin onu izlemesi bir üyenin `IK`'sinin (`döndür`) ya da `AK`'sinin iki yarısının sızmasını ister. Bu ayrıca kanıtlanmadı (aşağıda) |
| `no_offer_to_head_after_removal` | her iz | **P3, gönderirken tazelik:** bir üye `x`'i bir baştan çıkardıktan sonra hiçbir kişi o başa teklif göndermez |
| `no_offer_from_removed_device` | her iz | **P3, alırken tazelik:** bir üye `x`'i bir baştan çıkardıktan sonra hiçbir kişi o başta `x`'ten teklif kabul etmez; `x`'in bütün anahtarları sızmış olsa bile |
| `no_fresh_check_before_rotate_after_removal` | her iz | **P3, oturmamış `döndür`'den önceki baş:** çıkarma bir `döndür`'ün başlattığı baştan yazıldıysa, o `döndür`'ün önceki başında artık hiçbir kişinin tazelik denetimi geçmez; ne göndermek ne almak için |
| `no_rotate_branch_past_removal`, `no_rotate_branch_of_remover` | her iz | A3'ün lemması bu modelde yeniden kanıtlandı: hiçbir okuyucu, `x`'in çıkarıldığı baştaki `x` imzalı `döndür`'ü izlemez; çıkaranın kendi `döndür`'ünü de izlemez |
| `received_offer_authentic` | her iz | Kişinin halka üyesi `s`'den geldiğini kabul ettiği teklif `s`'nindir; `s`'nin `IK`'si sızmadıysa ve `s` saldırganın anahtarı değilse. Saldırganın anahtarının üye olması P1'in konusudur (A1: `AK`'nin iki yarısının sızması) |

Üç tazelik lemması, kişinin denetimi (kısıt) ve dürüst kayıtların yazıldığı
adımda günlüğe girmesi soyutlamasından kısa yoldan çıkar (4-12 adım).
İçerikleri gerçektir, ama P3'ün tamamı değildir: hangi başların kapsandığı
yukarıda, kapsanmayanlar aşağıdadır.

### Modelin bulduğu açık: çıkarma `döndür`'ün başından yazılınca önceki baş

PR #282'nin ilk sürümünde döndüren cihaz kendi `döndür`'ünün başına hemen
geçiyordu. Bağımsız inceleme şu izi buldu: `d` halkayı kurar ve `x`'i
ekler; kişi `{d, x}` başındadır. `d` haftalık `döndür`'ünü yazar ve yeni
başa geçer; `döndür` henüz oturmamıştır. `d` sonra `x`'i yeni baştan
çıkarır. Kişi hâlâ eski baştadır ve orada yalnızca oturmamış bir `döndür`
vardır: denetim geçer. Kişi `x`'in üyesi olduğu başa şifreler, ve `x` (hiçbir
anahtarı sızmadan, yazılımı çalıştırmaya devam ederek) eski başta teklif
gönderebilir. Pencere `döndür` oturana kadar, yaklaşık 4 saat sürer; P3'ün
2 saatini aşar.

Düzeltme A3'ün zaten yazdığı kuraldır: döndüren cihaz kendi `döndür`'ünü
oturunca izler; o zamana kadar çıkarmayı eski baştan yazar ve çıkarma
`döndür`'ün önüne geçer. `döndür` oturduktan sonra çıkarma yeni baştan
yazılır, ama o zaman eski başta oturmuş bir değer vardır ve kişiler oradan
göndermez. `no_fresh_check_before_rotate_after_removal` bunu kanıtlar;
`rotator_follows_own_rotate_at_once` mutantı izi yeniden bulur.

### Mutant sınamaları

| Mutant | Bozulan denetim | Beklenen saldırı |
| --- | --- | --- |
| `contact_sends_without_freshness` | Kişi göndermeden önce günlüğe bakmaz | Kişi, `x`'in çıkarıldığı başa gönderir (eski başla dosya alma, dondurma saldırısı) |
| `receive_without_freshness` | Kişi bir teklifi almadan önce günlüğe bakmaz | Kişi, `x`'in çıkarıldığı başta `x`'ten teklif kabul eder |
| `rotator_follows_own_rotate_at_once` | Döndüren cihaz kendi `döndür`'ünün başına hemen geçer | Çıkarma yeni baştan yazılır, kişi eski başta denetimi geçer (yukarıdaki açık) |
| `send_offer_no_recheck` | Kişi tazeliği yalnızca başın anahtarına bakarken denetler, gönderirken denetlemez | Denetim geçer, sonra çıkarma yazılır, sonra teklif `x`'in üyesi olduğu başa gider |
| `receive_signature_not_checked` | Kişi teklifin `IK` imzasını denetlemez | Hiçbir anahtar sızmadan sahte bir teklif üyeden gelmiş sayılır |

A3b mutantlarının dördü (`receive_signature_not_checked` dışındakiler), aramanın
bitmesi için saldırının kullanmadığı kuralları da düşürür (`mutants.py`,
`drop`). Kural düşürmek yalnızca iz eksiltir; küçük modelde bulunan saldırı
mutantta da vardır.

### Henüz kanıtlanmayanlar

- **Sonraki başlar:** P3'ün tazelik lemmaları çıkarmanın yapıldığı başı,
  çıkarma bir `döndür`'ün başından yazıldıysa o `döndür`'ün önceki başını
  ve A3'teki `döndür` dalını kapsar. "Çıkarmadan sonra kişinin gönderdiği
  her baş `x`'i içermez" biçimindeki sınırsız lemma, A3'teki sınırsız kilit
  lemmasına dayanır ve henüz yok.
- **`AK`'si çıkarmadan önce sızmış cihaz:** böyle bir cihaz çıkarmadan önce
  kendi dalını (ör. öbür üyeyi çıkaran bir kayıt) yazıp kişileri oraya
  geçirebilir. Bu, tasarımın kabul ettiği "geç çatal, yeniden eşleşme"
  sınırıdır (`hesap.md`, "Biçimsel modelde kesinleşecek kurallar", madde 1)
  ve A4'te ele alınır.
- **İki ya da daha çok adım önceki başlar:** ör. `döndür`, `döndür`, sonra
  çıkarma. Yukarıdaki gerekçe aynen geçerlidir (her önceki başta `AK` kaydı
  ya da oturmuş bir değer vardır) ve ikinci incelemede bu durumlar elle
  izlendi, ama lemması 4 GB bellekte bitmedi. Genel bir "çocuk başta olan,
  önce üst baştaydı" yardımcısı gerekiyor.
- **Üçüncü bir üyenin `döndür`'ü:** model iki cihaz yeri kullanır; döndüren
  ya `x`'tir ya da çıkaran. Gerekçe üçüncü bir üyeye aynen taşınır, ama
  modelde ifade edilmez.
- **A3'ün modeli eski kuralla:** `hayalet-halka-tazelik.spthy`'de döndüren
  cihaz hâlâ kendi `döndür`'ünün başına hemen geçer; A3'ün lemmaları o
  kuralla kanıtlandı. Yeni kural cihazın davranışını değiştirdiği için
  (eski baştan yazmaya devam eder) A3'ün sonuçları A4'te yeni kuralla
  yeniden kanıtlanacak.
- **Saldırganın yazdığı başlar:** `sent_head_key_secret`'in ikinci durumu
  (hiçbir üyenin yazmadığı bir baş) için "kişinin bu başı izlemesi bir
  üyenin anahtar sızıntısını ister" biçimindeki güçlü lemma 8 GB bellekte
  bitmedi; bir yardımcı değişmez gerekiyor.
- **Teklifin kendisinin gizliliği:** kanıtlanan, kişinin şifrelediği
  anahtarın gizliliğidir; teklifin gizliliği standart açık anahtarlı
  şifrelemeden gelir ve ayrı lemma değildir.

### Tasarıma geri bildirim (A4'te `hesap.md`'ye işlenecek)

- **Tazelik kuralı:** `hesap.md` anahtar altındaki her geçerli değerde
  göndermeyi durdurur. Model daha gevşek kuralı kullanır: `AK` kaydı ya da
  oturmuş bir değer durdurur, oturmamış `döndür` durdurmaz. Gevşek kural,
  aşağıdaki maddeyle birlikte güvenlidir ve kişilerin her haftalık
  `döndür`'den sonra `D` boyunca gönderememesini önler.
- **Döndüren cihaz:** kendi `döndür`'ünü ancak oturunca izler; o zamana
  kadar `AK` kayıtlarını eski baştan yazar. Halkanın hiçbir cihazı bir
  `döndür`'ü oturmadan izlemez.
- **"Oturdu" herkes için aynı anlama gelmeli:** modelde oturma tek bir
  olaydır; gerçekte her cihaz ve kişi kendisi karar verir. Kişi `döndür`'ü
  ilk gördüğü andan saymaya başlarsa (ör. 10 saat çevrimdışı kalmış bir
  kişi), çıkaran cihaz çoktan yeni baştan çıkarmışken kişi `D` boyunca daha
  eski başa, `x`'in üyesi olduğu başa şifreler. Kural: herkes oturmayı
  tanıklı dahil etme zamanından ve aynı `D` ile, şimdiki zamana göre
  ölçer; bir cihaz bir `döndür`'ü ancak her kişinin onu oturmuş saymak
  zorunda olduğu andan sonra izler (pay: saat kayması ve kök yaşı). A5'te
  sayısal olarak kesinleşir.
- **Teklif imzası:** alıcı kişiyi (ya da kişiyle paylaşılan bağlamı) ve
  alıcının başını bağlamalı; `hesap.md` neyin bağlandığını açıkça yazmalı.

## A4a: kurtarma (`hayalet-halka-kurtarma.spthy`)

A3'ün zinciri, günlüğü, oturma ve yazanın tazeliği; A3b'nin `döndür`
kuralıyla (döndüren cihaz kendi `döndür`'ünü oturunca izler). Eklenenler:

- Halkanın ilk kaydı kurtarma anahtarının (`RK`) açık yarısını taşır. Gizli
  yarı 24 kelimelik ifadedir; sahibindedir, hiçbir cihazda değildir.
  `Reveal_Phrase` ifadenin çalınmasıdır.
- **Üyelik dönemi:** her görünüm, son üyelik kaydının (oluştur, ekle,
  çıkar, red, kurtarma) başını `m` olarak taşır. `döndür` `m`'yi
  değiştirmez.
- `kurtarma_ekle`: `RK` imzalı, boş yere bir cihaz ekleyen bir öneri;
  bir başa değil, üyelik dönemi `m`'ye karşı yazılır ve kendi anahtarı
  `h('rlog', m)` altında günlüğe girer. Böylece bekleyen kurtarma sonraki
  `döndür`'lere taşınır ("kurtarma çatal açmaz ... sonraki dönemlere
  taşınır") ve `döndür` onu durdurmaz. Sahibi bunu ifade ve yeni bir
  cihazla yazar (`Owner_Recover`); ifadeyi çalan biri ağdan yazar
  (`Log_Write_RK`).
- **Bekleme:** okuyucular (cihazlar ve kişiler) bir kurtarmayı ancak
  oturunca izler. Gerçekte bu 72 saat ya da 7 gündür; modelde A3'ün
  "oturdu" olayıdır.
- **Öncelik:** `m` döneminin bir başından daha önce bir `AK` kaydı
  yazılmışsa kurtarma izlenmez (çatal kuralı, madde 1'in genişletilmesi).
- **İki kurtarma gerçek çataldır:** `m`'ye karşı başka bir kurtarma önce
  günlüğe girmişse okuyucu hiçbirini izlemez.
- `kurtarma_reddi`: bir üyenin, o anki başından yazdığı `AK` kaydı. Öbür
  kayıtlardan farklı olarak yazanın tazeliğine bağlı değildir: red her
  zaman sayılır (`hesap.md`, çatal kuralı 5). Kuralın tek ön koşulu üyenin
  görünümüdür, bu yüzden üye bunu beklemenin her anında yazabilir.
  Öncelik kuralıyla `m`'ye karşı kurtarmayı durdurur.
- **Kişiler** bir kurtarmayı ancak sahibiyle, o kurtarma kaydı için
  kelimeleri yeniden karşılaştırdıktan sonra izler (`!Words(r, kayıt)`).

Kayıt gövdesi ve görünüm değiştiği için bu yeni bir modeldir; A3'ün
çalıştırılabilirlik izleri ve `no_rotate_branch_past_removal` burada
yeniden kanıtlandı.

### Modelin ilk sürümünde incelemenin bulduğu açıklar

PR #283'ün ilk sürümünde kurtarma, bir başın anahtarı altına yazılıyordu.
Bağımsız inceleme iki iz buldu:

- **Reddedilemeyen geç kurtarma:** `d` haftalık `döndür`'ünü yazar, `döndür`
  oturur ama `d` henüz izlememiştir. İfadeyi çalan biri aynı anahtar altına
  kurtarma yazar. Yazanın tazeliği `d`'nin o anahtar altına artık
  yazmasını (reddetmesini) yasaklıyordu; kurtarmadan önceki değer
  `d`'nin kendi `döndür`'ünü de engelliyordu. `d`'nin tek yolu hırsızın
  kurtarmasını izlemekti. Bu, `hesap.md`'deki "cihaz dalı geçerli kalır"
  kuralıyla çelişiyordu.
- **İki kurtarma okuyucuları bölüyordu:** sahip ve hırsız aynı anahtar
  altına iki ayrı kurtarma yazınca bir cihaz birini, bir kişi öbürünü
  izliyordu.

Düzeltme, kurtarmayı üyelik dönemine bağlamak ve iki kurtarmayı çatal
saymaktır. `sanity_reject_after_settled_rotate` ilk izin artık reddedilebildiğini,
`no_split_recovery` ikincisinin kapandığını gösterir.

İkinci inceleme aynı açığın başka bir yoldan döndüğünü gösterdi: tek
üyenin kendi iki `döndür`'ü çatallanıp oturursa (ya da `IK`'sini çalan biri
başın anahtarına iki değer yazarsa) üye ne `döndür`'ü izleyebiliyor ne de
o baştan yazabiliyordu (yazanın tazeliği). Tek hamlesi hırsızın kurtarmasını
izlemekti. Düzeltme: red yazanın tazeliğine bağlı değildir.

### Soyutlamalar

| Soyutlama | Neden | Ne zaman genişler |
| --- | --- | --- |
| Kurtarma yalnızca boş yere ekler | İki cihaz yeri; bütün cihazlar kayıpsa halkada bir yer boştur | Değişmez |
| Kurtarılan cihaz bu parçada yeni kayıt yazmaz, okuyucular reddi izleyip yeni başa geçmez | P7'nin kurtarma kısmı için gerekmiyor; bu yüzden reddin ardından hizmetin sürmesi (canlılık) burada ifade edilmez | A4c |
| Reddedilen `RK`'nin sonraki dönemlerde geçersiz kalması yok | Burada red, yalnızca reddin yazıldığı döneme karşı kurtarmayı durdurur | A4c (`ifade_değiştir` ile) |
| Sahip kurtarmayı günlükten öğrendiği herhangi bir başın dönemine karşı yazar | Gerçekte aylık kurtarma kaydından en yeni başa ilerler; her yazışı saldırgan da yapabildiği için (`Log_Write_RK`) kanıtları etkilemez | Aşama C |
| Bekleme süresi, A3'ün oturma olayıdır | Süreler A5'te | A5 |
| **Kişinin kimlik denetimi güvenilir bir kanaldır** (`!Words`) | Kelimeler yalnızca onları gösteren cihaza giden kanalı kanıtlar; hırsızın cihazı da kelime gösterebilir. Asıl varsayım, kişinin karşısındakini bant dışında (yüz yüze, sesinden) tanımasıdır: insana ve sosyal mühendisliğe dayanan bir varsayım. P7'nin kişiler yarısı tamamen buna dayanır; sansür altında tek koruma budur | Değişmez; kullanıcıya anlatılır |

### Lemmalar

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| A3'ün dört çalıştırılabilirlik izi, `no_rotate_branch_past_removal` | iz var / her iz | Yeni kurallarla da geçerli |
| `sanity_owner_recovers` | iz var | Sahip kurtarır, bir kişi kelimeleri karşılaştırıp izler |
| `sanity_recovery_despite_rotate` | iz var | Kilitli çalınmış bir telefon (`IK`'si sızmış, `AK`'si sızmamış) `döndür` yazar, kişi bu oturmuş `döndür`'ü izler ve aynı dönemde kurtarmayı yine izler |
| `sanity_reject_after_settled_rotate` | iz var | Üye, kendi `döndür`'ü oturup izlendikten sonra da bekleyen kurtarmayı reddedebilir |
| `ak_record_stops_recovery` | her iz | **P7, cihazlar:** dürüst bir üye `m` döneminin bir başından `AK` kaydı (`kurtarma_reddi`, `ekle`, `çıkar`) yazdıktan sonra kimse `m`'ye karşı kurtarmayı izlemez. Okuyucu kuralı `Rec_Priority`'nin yeniden ifadesidir; asıl içerik, reddin her an yazılabilmesidir (kuralın tek ön koşulu görünüm, bkz. yukarı) |
| `recovery_was_pending` | her iz | **P7, cihazlar:** izlenen her kurtarmanın kendisi önce bekleme boyunca günlükte, her üyenin görebileceği yerdeydi (okuyucunun bekleme kuralının yeniden ifadesi) |
| `read_rec_was_logged` | her iz, yardımcı | İzlenen her kurtarma önce günlüğe yazılmıştır (`no_split_recovery` için) |
| `no_split_recovery` | her iz | Aynı döneme karşı iki kurtarmadan farklılarını iki okuyucu izlemez (`Rec_Single`'ın sonucu) |
| `contact_not_moved_by_phrase_alone` | her iz | **P7, kişiler:** yalnızca ifadeyle kişi taşınmaz; kişi bir kurtarmayı yalnızca sahibinin aynı döneme karşı eklediği cihaza, o kayıt için kimliği denetledikten sonra izler. Bu lemma yukarıdaki kimlik denetimi varsayımının yeniden ifadesidir |

### Mutant sınamaları

| Mutant | Bozulan denetim | Beklenen saldırı |
| --- | --- | --- |
| `recovery_without_wait` | Cihaz kurtarmayı beklemeden izler | Kurtarma, günlükte hiç beklemeden izlenir |
| `recovery_beats_ak_record` | Okuyucu öncelik kuralını uygulamaz | Bir üyenin reddinden sonra kurtarma izlenir |
| `contact_skips_words` | Kişi kimliği yeniden denetlemez | Kişi, ifadeyi çalanın cihazına taşınır |
| `recoveries_not_a_fork` | İki kurtarma çatal sayılmaz | Bir cihaz ve bir kişi farklı kurtarmaları izler |

Dördü de aramanın bitmesi için kullanılmayan kuralları düşürür (`drop`).

Lemmaların çoğu okuyucu kurallarını yeniden ifade ettiği için bu mutantlar
kuralların modele bağlı olduğunu sınar, tasarımın doğruluğunu değil.
Tasarımın asıl dayanakları, gerçek bir okuyucunun öncelik kuralını
uygulayabilmesi (aşağıda, tasarıma geri bildirim) ve üyenin her an
reddedebilmesidir; ikincisi kuralın biçiminden okunur, bir lemma olarak
ifade edilmedi.

### Henüz kanıtlanmayanlar

- **Telefon ve ifade birlikte çalınırsa** P7 güvence vermez (`hesap.md`,
  P7); model bunu iddia etmez.
- **`AK`'si elinde olan hırsızın sonsuz reddi:** kilidi açık bir cihazı
  çalan biri öncelik kuralıyla her kurtarmayı reddedebilir. Sahibin
  çıkış yolu `çatal_çöz`'dür: A4c.
- **Sansür:** tanıklar sunucuyla birlikte red kaydını sansürlerse bekleme
  yalnızca gecikmedir; P7'yi o zaman kişilerin kimlik denetimi sağlar.
  Modelin günlüğü tek ve sansürsüzdür (A2 varsayımı).
- **Geç red okuyucuları sessizce böler:** okuyucu A oturmuş kurtarmayı
  izledikten sonra çevrimdışı bir üye aynı dönemden red yazarsa A
  kurtarma dalında, üye red dalında kalır. Modelde okuyucular reddi hiç
  izlemez ve bu çatal algılanmaz (`hesap.md` kural 5 ve 6): A4c.
- **Yalnızca ifadeyi çalan biri sahibin kurtarmasını kalıcı olarak
  durdurabilir:** aynı döneme karşı ikinci bir kurtarma yazmak `Rec_Single`
  ile ikisini de durdurur. `çatal_çöz` de `RK` ile imzalandığından bunu
  çözmez; geriye kişilerle yeniden eşleşme kalır. Bu bir hizmet engellemedir,
  P7'yi bozmaz: A4c ve A4d.
- **Kurtarmadan sonrası:** kurtarılan cihazın görünümü yok; reddetmez,
  kayıp cihazı çıkarmaz, kayıt yazmaz. Kurtarmadan sonra olanlar için lemma
  yok.
- Kurtarmadan sonraki çatallar, `çatal_çöz` ve reddedilen `RK`: A4c.

### Tasarıma geri bildirim (A4d'de `hesap.md`'ye işlenecek)

- Kurtarma bir başa değil, üyelik dönemine karşı yazılan bir öneridir;
  `döndür` onu durdurmaz, ona taşınır. Üye bekleyen kurtarmayı her zaman
  o anki başından reddedebilir.
- Aynı döneme karşı iki kurtarma gerçek çataldır. Yalnızca ifadeyi çalan
  biri bununla sahibin kurtarmasını durdurabilir (hizmet engelleme);
  tasarım bunu söylemeli.
- Red yazanın tazeliğine bağlı değildir: çatallanmış ya da doldurulmuş bir
  başın üyesi de reddedebilmelidir.
- Öncelik için gerçek okuyucu, `m` döneminin **her** başı altında (izlemediği
  `döndür` dalları ve çatallı başlar dahil) `AK` kaydı aramalıdır.
  `hesap.md` bugün yalnızca "aynı anahtar altında" diyor. Model, `op`'u ne
  olursa olsun geçerli `AK` imzalı her değeri sayar; bu P7 için temkinlidir.
- Model kurtarmayı ayrı bir `h('rlog', m)` anahtarı altına yazar;
  `hesap.md:421` ise kurtarmayı `L_{e+1}` altında, en yeni başı ebeveyn
  göstererek tarif ediyor. A4d'de biri seçilmeli.
- Kişinin yeniden doğrulaması kurtarma kaydının özetine bağlanır (yeniden
  bulma kutusu zaten bu özetle adlandırılıyor).
- P7'nin kişiler yarısı kelimelere değil, kişinin karşısındakini bant
  dışında tanımasına dayanır; `hesap.md` ve kullanıcı metinleri bunu açık
  söylemeli.

## A4b: sıkı mod (`hayalet-halka-sikimod.spthy`)

A4a'nın üyelik dönemleri, kurtarması ve öncelik kuralı üzerine kurulur.
Sıkı modda tek bir `AK` imzası üyeliği hemen değiştiremez (`hesap.md`,
"Çok katmanlı onay", K3b). Modelin cevapladığı açık tasarım soruları
(`hesap.md`, "Biçimsel modelde kesinleşecek kurallar"):

- **Bekleyen kayıt nasıl sıralanır:** bekleyen tek imzalı bir `ekle` ya da
  `çıkar`, kurtarma gibi bir başa değil üyelik dönemi `m`'ye karşı yazılan
  bir **öneridir** ve kendi anahtarı `h('plog', m)` altında günlüğe girer.
  Başın anahtarıyla yarışmaz; `döndür` onu durdurmaz. Aynı döneme karşı iki
  `AK` önerisi çataldır, hiçbiri izlenmez.
- **`bekleyen_reddi`:** `kurtarma_reddi` ile aynı kayıttır: bir üyenin
  `m`'den yazdığı `AK` kaydı. `m`'ye karşı bütün önerileri durdurur;
  önerinin hedefi olan cihazınki de dahil. Yani kilidi açık çalınmış bir
  telefon kendi çıkarılmasını veto edebilir.
- **Çalınan cihaz veto ederse `RK` yolu:** `rk_çıkar(x)`, `m`'ye karşı
  `RK` imzalı bir öneridir. Kurtarma gibi bekler; `x` dışındaki her üye onu
  reddedebilir, `x`'in kendi kayıtları ona karşı sayılmaz. Okuyucu onu
  ancak görünümünde `x`'ten başka bir üye varsa izler: `RK` hiçbir zaman
  halkanın tek cihazını çıkaramaz, çünkü o zaman reddedebilecek kimse
  kalmaz. Aynı döneme karşı iki `RK` önerisi (kurtarma ya da `rk_çıkar`)
  çataldır.

### Soyutlamalar

| Soyutlama | Neden | Ne zaman genişler |
| --- | --- | --- |
| Halka ilk kayıttan beri sıkıdır (ilke alanı ilk kayıtta); `ilke` kaydıyla açıp kapatma yok | Kapatmak sıkı modun istediği imza sayısını ister, yani bir öneridir; aynı kurala tabidir | A4c ya da aşama C |
| İlk kayıt iki cihaz taşır | Sıkı mod iki ve daha çok cihaz için önerilir. İki yerle iki imzalı `ekle` olamaz (boş yer ancak tek üyeli halkada vardır), iki imzalı `çıkar` çıkarılan cihazın da imzasını ister; bu yüzden burada her sıkı değişiklik bekleyen öneridir | Değişmez |
| `döndür` yok | Öneriler döneme karşı yazılır ve `döndür` dönemi değiştirmez; A4a, `döndür`'ün ne kurtarmayı ne de reddi engellediğini gösterdi | Değişmez |
| Okuyucular reddi izlemez, öneri izlenince eklenen cihaz görünüm almaz | Bu parçanın özellikleri için gerekmiyor (A4a ile aynı) | A4c |
| Sıkı modda kurtarmanın ek etkeni (YubiKey, Shamir payı) yok | Kurtarma kuralları A4a ile aynı; ek etken yalnızca ifadenin çalınmasını zorlaştırır | Aşama C |

### Lemmalar

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| `sanity_strict_remove` | iz var | Üye öbürünü çıkarmayı önerir, öneri bekler, kişi izler |
| `sanity_stolen_device_removed_with_rk` | iz var | Kilidi açık çalınmış cihaz kendi çıkarılmasını veto eder; sahip `rk_çıkar` yazar ve kişi, çalınan cihazın vetosuna rağmen onu izler |
| `no_change_without_wait` | her iz | **K3b:** sıkı halkanın okuyucusu üyeliği hiçbir zaman hemen değiştirmez; izlediği her değişiklik (öneri, kurtarma, `rk_çıkar`) önce günlükte bekledi. Kilidi açık çalınmış bir telefon tek başına kimseyi taşıyamaz |
| `reject_stops_proposal` | her iz | **K3b:** bir üye dönemden `AK` kaydı yazdıktan sonra o döneme karşı öneri izlenmez (`Prop_Priority`'nin yeniden ifadesi) |
| `rk_remove_stopped_by_other_member` | her iz | **P7, `rk_çıkar`:** `x` dışındaki bir üyenin reddinden sonra `rk_çıkar(x)` izlenmez; yalnızca ifadeyle dürüst bir cihaz, öbür dürüst üye reddedebildiği sürece çıkarılamaz |
| `rk_remove_keeps_a_member` | her iz | `rk_çıkar` hiçbir zaman halkayı boşaltmaz (okuyucu denetimi) |
| `ak_record_stops_recovery`, `contact_not_moved_by_phrase_alone` | her iz | A4a'nın P7 lemmaları yeni kurallarla da geçerli |

Tablodaki "yeniden ifade" uyarısı A4a'daki gibidir: asıl içerik kuralların
biçimindedir. Reddin tek ön koşulu üyenin görünümüdür, bu yüzden üye onu
beklemenin her anında yazabilir; `rk_çıkar`'ın hedefin vetosunu saymaması
ve tek cihazı çıkaramaması, `sanity_stolen_device_removed_with_rk` ve
`rk_remove_keeps_a_member` ile birlikte okunmalıdır.

### Mutant sınamaları

| Mutant | Bozulan denetim | Beklenen saldırı |
| --- | --- | --- |
| `strict_immediate_accept` | Okuyucu tek imzalı `çıkar`ı varsayılan moddaki gibi hemen izler | Değişiklik beklemeden izlenir |
| `proposal_without_wait` | Okuyucu öneriyi oturmadan izler | Bekleyen `çıkar` beklemeden izlenir |
| `veto_ignored` | Okuyucu öncelik kuralını uygulamaz | Bir üyenin reddinden sonra öneri izlenir |
| `rk_remove_ignores_others` | Okuyucu `rk_çıkar` için reddi saymaz | Öbür üyenin reddine rağmen `rk_çıkar` izlenir |
| `rk_remove_lone_device` | Okuyucu ve sahip kalan üyeyi denetlemez | `RK`, halkanın tek cihazını çıkarır |

### Henüz kanıtlanmayanlar

- **Telefon ve ifade birlikte çalınırsa** sıkı mod da güvence vermez:
  hırsız `rk_çıkar` ile sahibin cihazını çıkarabilir (hedef olan sahibin
  cihazının reddi sayılmaz), sahibin karşı `rk_çıkar`'ını ise elindeki
  cihazla reddeder. Bu durum `hesap.md`'nin P7 dışı bıraktığı durumdur;
  çıkış yolu yeniden eşleşmedir.
- **Hizmet engelleme:** kilidi açık çalınmış bir cihaz her öneriyi ve her
  kurtarmayı reddedebilir; ifadeyi çalan biri ikinci bir `RK` önerisiyle
  sahibin `rk_çıkar`'ını durdurabilir. Güvenliği bozmaz, sahibi bekletir:
  A4c (`çatal_çöz`) ve A5 (süreler).
- **Geç red:** öneriyi izlemiş bir okuyucu, ardından yazılan bir reddi
  görmez (A4a'daki gibi): A4c.
- İki ve daha çok üyeli halkalar, `ilke` değişikliği ve öneri izlendikten
  sonraki halka: A4c ve aşama C.

### Tasarıma geri bildirim (A4d'de `hesap.md`'ye işlenecek)

- Bekleyen tek imzalı kayıt bir öneridir: üyelik dönemine karşı yazılır,
  kendi günlük anahtarı vardır, başın anahtarıyla yarışmaz. Aynı döneme
  karşı iki öneri çataldır.
- `bekleyen_reddi` ve `kurtarma_reddi` tek kayıttır: üyenin dönemden yazdığı
  `AK` kaydı.
- "Çalınan cihaz `bekleyen_reddi` yazarsa yol `RK`'den geçer"in kesin
  tanımı: `rk_çıkar(x)`, `RK` imzalı, bekleyen, `x` dışındaki üyelerin
  reddedebildiği, okuyucunun ancak başka bir üye kalıyorsa izlediği öneri.
- K3b'ye "okuyucu, kalan üye olmadan `rk_çıkar`'ı reddeder" eklenmeli.

