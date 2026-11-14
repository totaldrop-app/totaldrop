# Mimari

Bu belge TotalDrop'un parçalarını ve bir aktarımın bu parçalar arasında
nasıl ilerlediğini üst düzeyde anlatır. Ayrıntılar başka belgelerde durur:
protokol [protocol-v0.1-notes.md](protocol-v0.1-notes.md), tehditler ve
güven sınırları [tehdit-modeli.md](tehdit-modeli.md), kararların nedenleri
[kararlar/](kararlar/README.md), depo düzeni [gelistirme.md](gelistirme.md).

## Parçalar

```
 Gönderen cihaz                                         Alan cihaz
┌──────────────────────┐                       ┌──────────────────────┐
│ Uygulama (arayüz)    │                       │ Uygulama (arayüz)    │
│ Linux: Qt 6 + QML    │                       │ Android: Flutter     │
├──────────────────────┤                       ├──────────────────────┤
│ totaldrop-core       │ ── QUIC + Noise ────▶ │ totaldrop-core       │
│ totaldrop-proto      │   (doğrudan ya da     │ totaldrop-proto      │
│ totaldrop-crypto     │    aktarma üzerinden) │ totaldrop-crypto     │
└──────────┬───────────┘                       └───────────┬──────────┘
           │ WebSocket (TLS); STUN için UDP                │
           ▼                                               ▼
     ┌──────────────────┐   ┌──────────────────┐   ┌──────────────┐
     │ Rendezvous       │   │ Aktarma (relay)  │   │ STUN         │
     │ kod ve posta     │   │ şifreli trafiği  │   │ dış adres    │
     │ kutusu           │   │ iletir           │   │ öğrenme      │
     └──────────────────┘   └──────────────────┘   └──────────────┘
```

**Çekirdek (Rust, `crates/`).** Bütün platformlar aynı kodu kullanır, bu
yüzden protokol ve kriptografi tek bir yerde durur:

- `totaldrop-crypto`: CPace, anahtar türetme, posta kutusu ve oturum
  şifrelemesi, cihaz anahtarları, STREAM parça şifrelemesi.
- `totaldrop-proto`: aktarım kodları, sınırlar, yol güvenliği ve kablo
  mesajları (CBOR; aktarma çerçeveleri kendi ikili biçiminde). İstemcide
  karşı cihazdan ve sunuculardan gelen veri burada ayrıştırılır; bu
  ayrıştırıcıların `fuzz/` altında fuzz hedefleri vardır.
- `totaldrop-core`: istemci motoru. Rendezvous ile konuşur, ham açık
  anahtarlı QUIC bağlantısını kurar, Noise XXpsk3 + ML-KEM-768 el
  sıkışmasını yapar, dosyaları gönderir ve alır, gerekirse aktarma
  sunucusuna geçer.
- `totaldrop-l10n`: Rust programlarının arayüz metinleri (35 dil).

**Uygulamalar.** Arayüz platformun kendi araçlarıyla yazılır, çekirdeği
çağırır, kriptografi yapmaz:

- Linux (`linux/`): Qt 6 ve QML; çekirdeğe CXX-Qt köprüsüyle bağlanır
  ([0006](kararlar/0006-linux-qt.md)).
- Android (`app/`): Flutter; çekirdeğe `app/rust` ve flutter_rust_bridge
  ile bağlanır. Aktarım arka planda kullanıcının başlattığı iş olarak
  sürer ([0010](kararlar/0010-android-arka-plan-aktarim.md)).
- Komut satırı (`crates/totaldrop-cli`): çekirdeğin ince bir kabuğu.

**Sunucular (`server/`).** Projenin iki sunucusu vardır. İkisi de dosya
içeriğini göremez ve kayıt tutmaz. Her birinin iki uygulaması vardır: kendi sunucusunu kurmak ve
testler için Rust, `totaldrop.app` için Cloudflare Workers ve Durable
Objects ([0003](kararlar/0003-cloudflare-altyapi.md)). İki uygulama aynı
uçtan uca test takımından geçer.

- Rendezvous (`totaldrop-rendezvous`, `rendezvous-worker`): kodun numara
  kısmını (nameplate) ayırır ve iki cihaz buluşana kadar şifreli
  mesajları taşıyan posta kutusunu tutar. Kodun geri kalanını cihaz üretir;
  sunucu onu görmez.
- Aktarma (`totaldrop-relay`, `relay-worker`): şifreli QUIC trafiğini iki
  cihaz arasında iletir. Yalnızca o aktarımın iki
  anahtarını, imzalı bir meydan okumayla kabul eder.

Proje adreslerine (`hello@totaldrop.app` gibi) gelen e-postayı
bakımcılara ileten e-posta Worker'ı (`mail-worker`) da `server/`
altındadır. Protokolün parçası değildir ve uygulamalar onu hiç kullanmaz
([Dağıtım](surec.md#e-posta-yönlendirmesi)).

STUN projenin sunucusu değildir: cihazlar dış adreslerini varsayılan
olarak Cloudflare'in STUN sunucusundan (`stun.cloudflare.com`, UDP) ya da
kişinin kendi kurduğu aktarma sunucusundan öğrenir. Ayrıntılar
[aktarma.md](aktarma.md) içinde.

**Site (`site/`).** totaldrop.app, indirme sayfası ve QR bağlantılarının
açtığı sayfa Cloudflare Pages'te durur ve dışarıdan hiçbir şey yüklemez.
Gönderme yalnızca uygulamalardan yapılır, tarayıcıdan yapılmaz.

## Bir aktarım

1. Gönderen rendezvous'tan bir numara (nameplate) ayırır, kodun geri
   kalanını cihazında üretir ve kodu karşı tarafa söyler.
2. İki cihaz posta kutusunda buluşur ve koddan CPace ile ortak bir anahtar
   türetir. Sunucu kodu ve anahtarı bilmez; posta kutusundaki mesajlar bu
   anahtarla şifrelidir.
3. Cihazlar posta kutusu üzerinden adreslerini ve tek kullanımlık uç nokta
   anahtarlarını değiş tokuş eder. Doğrudan yol ile aktarma yolu birlikte
   denenir, ilk yanıt veren kullanılır; bu yüzden aktarma sunucusu doğrudan
   aktarımlarda da iki cihazın IP adresini görebilir
   ([tehdit modeli](tehdit-modeli.md) T5, #63).
4. QUIC içinde Noise XXpsk3 + ML-KEM-768 el sıkışması yapılır; iki taraf
   dört doğrulama kelimesini görür.
5. Gönderen dosya listesini (manifest) yollar, alan kabul eder; dosyalar
   1 MiB'lik parçalar halinde STREAM ile şifreli akar, her parça gelince
   doğrulanır. Dosya, BLAKE3 doğrulamasından sonra alma klasörüne konur.

## Derleme ve dağıtım

Her şey bu depodan, GitHub Actions'ta derlenir: Linux paketleri ve Android
paketi etiketten, sunucular ve site `main`'den, sahibin onayıyla
([surec.md](surec.md)). Kişisel bir makinede derlenen hiçbir şey
yayınlanmaz.
