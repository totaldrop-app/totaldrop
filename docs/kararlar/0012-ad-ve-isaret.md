# 0012: Kod serbest, ad ve işaret yalnızca TotalDrop'un kendi sürümleri için

- **Durum:** Kabul, 4 Kasım 2026 (#234)
- **Bağlam:** TotalDrop'un kodu açık lisanslıdır
  ([LICENSING.md](../../LICENSING.md)): protokol kütüphaneleri MIT veya
  Apache-2.0, uygulamalar ve site GPL-3.0-or-later, sunucular
  AGPL-3.0-or-later. Bu lisanslar kodu kopyalama, değiştirme ve dağıtma
  izni verir. Ama TotalDrop adını ve işaretini kullanma izni vermez.
  Apache-2.0 bunu 6. maddesinde açıkça söyler. GPL-3.0 ad ve işarete
  izin vermez; 7. maddesi isteyen projenin bunu ek şart olarak lisans
  bildirimine yazmasına (e) ve değiştirilmiş sürümlerin farklı olduğunun
  belirtilmesini istemesine (c) izin verir. Bugüne kadar adın ve işaretin başkalarınca nasıl
  kullanılabileceği hiçbir yerde yazmıyordu. [/brand](https://totaldrop.app/brand)
  sayfası (#220) logo dosyalarını indirtiyor ama bu dosyaları kimin,
  hangi koşulla kullanabileceğini söylemiyor.

  TotalDrop'a güvenmenin sebebi şifrelemedir. Kullanıcı "TotalDrop"
  yazan bir uygulamayı kurduğunda, kodu bu depodan gelen ve aynı güvenlik
  sözlerini veren bir uygulama kurduğunu bilmelidir. Değiştirilmiş bir
  sürüm aynı adı ve işareti taşırsa kullanıcı onu resmî uygulamadan
  ayıramaz. Bu sürüm kötü niyetli olmasa bile şifrelemeyi zayıflatmış
  olabilir.

  Diğer açık kaynak projeler:
  - **Mozilla:** Değiştirilmemiş Firefox'u dağıtan herkes adı kullanabilir.
    Kodu değiştirip dağıtan kendi adını kullanır. Debian bu yüzden yıllarca
    "Iceweasel" adını kullandı.
  - **Rust Foundation:** Adı tanıtım ve topluluk işlerinde geniş ölçüde
    serbest bırakır. Değiştirilmiş derleyicinin "Rust" adıyla dağıtılmasını
    ve resmî görünmeyi yasaklar.
  - **Signal:** Kod AGPL'dir ama fork'lar Signal adını ve logosunu
    kullanamaz, kendi adlarıyla yayımlanır.

- **Seçenekler:**
  1. **Fork'lar kendi adıyla yayımlanır.** Kodu değiştiren her sürüm
     kendi adını ve işaretini kullanır. Kodu değiştirmeden yalnızca
     paketleyenler adı korur. Mozilla ve Signal böyle yapar.
  2. **Küçük değişikliklere izin.** Hata düzeltmesi ve yerel uyarlama yapan
     sürüm adı korur, özellik ekleyen ya da güvenliği etkileyen sürüm kendi
     adını kullanır. "Küçük" değişikliğin sınırını çizmek zordur. Güvenliği
     etkileyen bir değişiklik küçük görünebilir.
  3. **Serbest bırakmak.** Fork'lar adı ve işareti kullanabilir. Kullanıcı
     zayıflatılmış bir sürümü resmî uygulama sanabilir.

- **Karar:** 1. seçenek (Ozan, 4 Kasım 2026, karar kartı). Aşağıdaki
  diğer maddeleri ve mağaza kuralını Ozan aynı gün başlıkta onayladı.
  Kurallar [docs/marka.md](../marka.md) belgesindedir. Özetle:
  - Kod lisansları ne izin veriyorsa o serbesttir. Ad ve işaret için ayrı
    kural geçerlidir. Lisans metinlerine GPL'nin 7. maddesine göre ek şart
    eklenmez: kural lisansın değil, adın ve işaretin kuralıdır, ve kodun
    kullanımına hiçbir kısıt getirmez.
  - İşaret dosyaları (SVG, PNG, simgeler) bulundukları klasörün lisansıyla
    dağıtılır; dosyayı değiştirmek lisansın verdiği haktır. Kural yalnızca
    adın ve işaretin bir ürünü tanıtmak için kullanılmasını düzenler.
  - Bağlantı vermek, haberde, yazıda ve eğitimde adı ve değiştirilmemiş
    işareti kullanmak serbesttir.
  - Kodu değiştirmeden, yalnızca paketleme için gereken yamalarla derleyen
    paketleyiciler (Linux dağıtımları, AUR, F-Droid) adı ve işareti
    kullanabilir. Flathub'daki `app.totaldrop.TotalDrop` kaydını proje
    yayımlar ya da onaylar.
  - Kodu değiştiren sürümler kendi adını ve işaretini kullanır. "TotalDrop
    tabanlıdır" diyebilir.
  - Protokolü uygulayan başka istemciler "TotalDrop ile çalışır" diyebilir.
    Adlarında TotalDrop geçemez, işareti kullanamazlar.
  - Resmî ya da onaylı gibi görünmek, işareti değiştirmek, benzer ad ve
    alan adları kullanmak yasaktır. Yayımlayanın geliştirici olarak
    göründüğü mağazalarda (Google Play, App Store, Microsoft Store, Snap
    Store) TotalDrop'u yalnızca TotalDrop yayımlar.
  - Adı veya işareti taşıyan ürün satmak için yazılı izin gerekir.

- **Neden:** Kural kullanıcıyı korur ve uygulaması kolaydır. "Kodu
  değiştirdin mi?" sorusunun yanıtı nettir, "küçük mü?" sorusununki değildir.
  Kodun özgürlüğüne dokunmaz: herkes kodu alıp istediği gibi değiştirebilir,
  yalnızca başka bir adla yayımlar. Açık protokol hedefi de korunur: uyumlu
  istemciler TotalDrop'la çalıştıklarını söyleyebilir.

- **Sonuçlar:**
  - `docs/marka.md` kuralların tam metnidir. `LICENSING.md` ona bağlanır.
  - /brand sayfasına 35 dilde "Ad ve işaretin kullanımı" bölümü eklenir.
  - Bir fork'un Android'de `app.totaldrop`, Linux'ta
    `app.totaldrop.TotalDrop` dışında bir uygulama kimliği kullanması
    önerilir, şart koşulmaz. Nedenleri teknik: mağazalar ve Flathub aynı
    kimliği iki kez kabul etmez, Android başka anahtarla imzalanmış bir
    güncellemeyi kurmaz, aynı masaüstü kimliğini taşıyan iki uygulama
    birbirinin yerine geçer.
  - Ad tescil edilmedi. Metinler "tescilli marka" demez. Ad tescil
    ettirilirse belgeler güncellenir.
  - İzin istekleri hello@totaldrop.app adresine gelir.
  - Kapsam dışı: fork'ların ve uyumlu istemcilerin totaldrop.app
    sunucularını kullanması. Bu bir hizmet kullanım kuralıdır, gerekirse ayrı
    bir kararla yazılır.

- **Gözden geçirme koşulu:** Ad tescil ettirildiğinde, bir paketleyici ya
  da fork kuralın uygulanamadığı bir durum bildirdiğinde, ya da sunucu
  kullanım kuralı yazıldığında.
