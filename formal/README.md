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
| A3 | Döndürmenin oturması ve yazanın tazeliği (A2'deki açığın düzeltmesi); ardından kişi başı, sabitleme, gönderirken ve alırken tazelik (P3) | İlk kısmı bu klasörde |
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
  altında henüz hiçbir değerin oturmadığı bir baştan yazar. Bu, yazmadan
  önceki tazelik denetimidir: taze bir kök, o başın anahtarı altında çoktan
  yazılmış bir değer göstermiyor olmalı.

İkisi birlikte şunu sağlar: bir `döndür` henüz otururken yazılan bir
`çıkar`, `döndür` oturmadan önce günlüktedir ve çatal kuralı 1 gereği onu
yener. `döndür` oturduktan sonra ise dürüst cihaz o eski baştan yazamaz;
önce `döndür`'ü izler ve çıkarmayı yeni baştan yazar.

`D`'nin değeri A5'te süre sınırlarıyla (1 saat dahil etme, 1 saat kök
tazeliği, yazanın denetimi) birlikte belirlenir. Kabaca bir hesap: bir
`döndür`'den en çok 2 saat sonra yazılan bir kayıt, ondan en geç 3 saat
sonra köke girer ve en çok 1 saat eski kök gören okuyan onu görür. Bu
yüzden `D` en az 4 saat olmalıdır. Haftalık döndürmede bu gecikmenin
kullanıcıya etkisi yoktur.

### Lemmalar

A2'nin "her iz" lemmaları bu modelde de geçerlidir ve yeniden kanıtlanmaz:
model yalnızca sonucu olmayan bir kural (`Log_Age`), yeni olaylar ve
kısıtlar ekler, bu yüzden bu modelin her izi A2'nin de bir izidir. ("Her
iz" lemmalarını burada yeniden kanıtlamak, yeni kısıtlar aramayı
büyüttüğü için 10 dakikada bitmiyordu.) A2'nin iki "iz var" lemması
burada yeniden kanıtlanır, çünkü kısıtlar bir izi ortadan kaldırabilir.
Yeni olanlar:

| Lemma | Tür | Ne söyler |
| --- | --- | --- |
| `sanity_contact_follows_settled_rotate` | iz var | Oturmuş bir `döndür` yine izlenebilir; kural döndürmeyi engellemez |
| `no_rotate_branch_past_removal` | her iz | **H3, A2'deki açık kapandı:** bir üye `x`'i bir baştan çıkarıyorsa hiçbir okuyan o başta `x`'in `döndür`'ünü izlemez; hangisi önce yazılmış olursa olsun. A2'deki saldırı izinin ikinci adımı bu yüzden imkânsızdır |

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
- Halka sırrı, teklifler, gönderirken ve alırken tazelik (P3): A3'ün ikinci
  kısmı.

