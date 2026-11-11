# 0017: Bağışlar şimdilik proje yöneticisinin GitHub Sponsors sayfasından alınır

- **Durum:** Kabul, 11 Kasım 2026 (Sadık önerdi, Ozan kabul etti; #487).
  [0005](0005-bagis.md)'in bağış kanalını kimin adına açtığını ve ne zaman
  değişeceğini belirler.

## Bağlam

[0005](0005-bagis.md) bağışları GitHub Sponsors'a bağladı. Bunun için
proje yöneticisi Ozan'ın kişisel sayfası (github.com/sponsors/ozdil)
kullanıldı. Projenin ekip olarak yürüdüğü düşünülünce bağışları ekip
adına almak daha doğruydu. Bunun için Open Collective'e başvurduk (#352,
PR #368). Open Source Collective başvuruyu 11 Kasım 2026'da reddetti.
Katkıcı sayısı artınca yeniden başvurulabilir.

Sadık şunu önerdi: bağışlar şimdilik Ozan'ın kişisel sayfasından alınsın,
proje büyüyünce TotalDrop adına bir hesaba geçilsin. Ozan bunu kabul etti.

## Karar

1. Bağışlar, proje yöneticisinin kişisel GitHub Sponsors sayfasından
   bir kez ya da her ay alınır. Bu, projenin ekip adına yürümesi
   ilkesine (#352) bilerek yapılmış, süreli bir istisnadır.
2. /donate sayfası bunu 35 dilde açıkça söyler: para proje yöneticisinin
   adına gelir ve proje büyüyünce TotalDrop adına bir hesaba geçilir.
3. Bağışlar yalnızca /donate sayfasındaki maliyetlere harcanır. Bu
   kararla şu kural konur: her harcamayı Ozan önerir, Sadık onaylar.
   Kimse kendi önerisini onaylamaz. Gelen ve giden her ay /donate
   sayfasındaki tabloda ve depoda [bağış dökümünde](../bagis-dokumu.md)
   aynı satırlarla yayımlanır.
4. Bağış gelirinin kişisel gelir olarak doğurduğu yükümlülükler hesabın
   sahibine aittir.

## Ne zaman değişir

Aşağıdakilerden biri olunca TotalDrop adına bir hesaba (bir mali ev
sahibi üzerinden Open Collective ya da GitHub'da bir kuruluşun Sponsors
sayfası) geçmek için yeni bir karar açılır:

- düzenli katkıcı sayısı artar ve bir mali ev sahibine yeniden başvurmak
  anlamlı hâle gelir;
- aylık bağışlar üç ay üst üste projenin aylık maliyetlerini aşar;
- ekipten biri ya da Ozan, paranın kişisel hesapta durmasının artık
  uygun olmadığını söyler.

Geçişte eski sayfanın bağışçılarına yeni hesap duyurulur ve kişisel
sayfadaki aylık bağışlar kapatılır.

## Sonuçlar

- `.github/FUNDING.yml`, README rozeti, site düğmeleri ve Linux
  metainfo'daki bağış adresi github.com/sponsors/ozdil olarak kalır.
- PR #368'deki Open Collective geçişi bu karar geçerli olduğu sürece
  uygulanmaz.
- GitHub Sponsors profil metni, kademeler ve vergi bilgileri Ozan'ın
  kendi hesabında tutulur; depoda değildir.
