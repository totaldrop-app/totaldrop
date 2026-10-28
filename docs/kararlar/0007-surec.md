# 0007: Çalışma süreci

- **Durum:** Kabul, 28 Ekim 2026
- **Bağlam:** İlk iki günde hız için PR'lar yazan tarafından birleştirildi
  ve Android paketleri kişisel bilgisayarda derlendi. Bu, kendi
  belgelerimizdeki kurala aykırıydı.
- **Karar:** Her iş bir issue ile başlar; her PR'ı kodu yazmamış bir
  inceleyici okur; bütün CI kontrolleri yeşil olmadan birleştirme yapılmaz;
  `main` doğrudan push'a kapalıdır; yayınlanan her paket CI'da derlenir ve
  derleme kaynağı kanıtı (build provenance) taşır.
- **Neden:** Güvenlik iddiası taşıyan bir yazılımda yazarın kendi işini
  onaylaması yeterli değildir; kullanıcı da indirdiği paketin depodaki
  koddan geldiğini doğrulayabilmelidir.
- **Sonuçlar:** Süreç [surec.md](../surec.md) içinde; PR şablonu bitti
  tanımını içerir.
