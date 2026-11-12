# Karar kayıtları

Her mimari karar burada kısa bir belge olarak durur: bağlam, karar, neden ve
sonuçları. Bir karar değişirse eski belge silinmez; durumu "yerini aldı"
olur ve yenisine bağlanır.
Henüz onaylanmamış bir karar "Önerilen" durumunda açılır; Ozan onaylayınca
birleştirmeden önce "Kabul" olur.
In records written in English the statuses are "Proposed" (Önerilen),
"Accepted" (Kabul) and "Superseded by" (Yerini aldı).

| No | Karar | Durum |
| --- | --- | --- |
| [0001](0001-tek-depo.md) | Tek depo, platformlar ayrı klasör, sürüm ve CI ile | Kabul |
| [0002](0002-protokol-v0.md) | Protokol v0: CPace, Noise XXpsk3 + ML-KEM-768, QUIC | Kabul |
| [0003](0003-cloudflare-altyapi.md) | Site, rendezvous ve aktarma Cloudflare'de | Kabul |
| [0004](0004-mullvad-tarzi-hesap.md) | Hesap yalnızca rastgele bir numara | Yerini aldı: [0013](0013-hayalet-halka-hesap.md) |
| [0005](0005-bagis.md) | Gelir yalnızca bağış, GitHub Sponsors üzerinden | Kabul |
| [0006](0006-linux-qt.md) | Linux uygulaması Qt 6 ve QML, sistem temasına tam uyum | Kabul |
| [0007](0007-surec.md) | Issue, PR, bağımsız inceleme ve CI'da derlenen sürümler | Kabul |
| [0008](0008-diller-ve-bolgeler.md) | Her dil ve bölge: tek kaynak ARB, ICU ve CLDR | Kabul |
| [0009](0009-ios-macos.md) | iOS ve macOS: ücretsiz CI, Apple hesabı yalnızca yayında | Kabul |
| [0010](0010-android-arka-plan-aktarim.md) | Android'de aktarım arka planda kullanıcının başlattığı iş olarak sürer | Kabul |
| [0011](0011-android-material-3-expressive.md) | Android Material 3 Expressive'e resmî paketle, adım adım geçer | Kabul |
| [0012](0012-ad-ve-isaret.md) | Kod serbest, ad ve işaret yalnızca TotalDrop'un kendi sürümleri için | Kabul |
| [0013](0013-hayalet-halka-hesap.md) | Hesap sunucuda tutulmaz: Hayalet halka | Kabul |
| [0014](0014-steam-deck.md) | TotalDrop for Steam Deck, Steam Machine ve Steam Frame: Linux sürümünün parçası, Flathub ve klavyeyle tam kullanım | Kabul |
| [0015](0015-paket-yoneticileri.md) | Paket yöneticileri önce CI'da derlenen CLI ikililerine dayanır | Kabul |
| [0016](0016-windows-microsoft-store.md) | Windows uygulaması WinUI 3 ile yazılır, Microsoft Store'dan dağıtılır | Kabul |
| [0018](0018-baglanti-yol-haritasi.md) | Bağlantı katmanı kendi yığınımızda kalır; IPv6, port açma ve ağ değişimine tepkiyle doğrudan yollar artırılır | Kabul |
| [0019](0019-english-by-default.md) | The repository is written in English | Proposed |

Yeni bir karar için bir sonraki numarayla `NNNN-kisa-ad.md` açın ve bu
tabloya ekleyin.
