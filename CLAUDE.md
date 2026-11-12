# TotalDrop

Bu dosya, bu depoda çalışan Claude Code oturumlarına yol gösterir.

- Proje kıdemli bir yazılım ekibinin disipliniyle yürür: yazılım mühendisliği yöntemi kısaltılmadan uygulanır. Çalışma süreci `docs/surec.md` bağlayıcıdır:
  - her iş bir issue ile başlar, kısa ömürlü bir branch'te yapılır ve PR ile `main`'e girer;
  - bütün CI kontrolleri yeşil olmadan ve kodu yazmamış bir inceleyici (ayrı bir inceleme ajanı) okumadan birleştirme yapılmaz;
  - "bitti tanımı" ve sürüm listesi adım adım izlenir; yayınlanan her paket CI'da derlenir.
- Mimari kararlar `docs/kararlar/`, protokol kararları `docs/protocol-v0.1-notes.md`, tehdit modeli `docs/tehdit-modeli.md` içindedir.
- Belgeler ve kullanıcıya yazılan metinler Türkçe.
- Tasarım kuralları: `docs/tasarim.md`. Bu kurallar bağlayıcıdır, özellikle şu ikisi:
  - Arayüz her platformda sistem temasına yüzde yüz uyar. Linux'ta KDE, Hyprland ve Omarchy desteklenir. Her adımda üçü de kontrol edilir.
  - Aktarım sırasında işaret, web sitesindeki animasyonun aynısını oynatır.
- Gizli bilgiler (`key.properties`, imza anahtarı parolası) okunmaz ve depoya eklenmez.
- Paketler yalnızca CI'da derlenir; CI çıktısı (ör. Play Console için `.aab`) yerelde `~/Projects/totaldrop/dist` altına indirilir.
  Tek istisna Flathub: Flathub, Flatpak'i kendi altyapısında `packaging/flatpak` tarifinden ve etiketli kaynaktan derler ve kendi anahtarıyla imzalar; aynı tarif her PR'da bizim CI'ımızda da derlenir ([karar 0014](docs/kararlar/0014-steam-deck.md)).
- Skill'ler depoda, `.claude/skills/<ad>/SKILL.md` altındadır; Claude Code onları klondan kendiliğinden tanır (#458). Skill'ler ajan yönergesidir ve İngilizce yazılır. Her işin başında, değişen klasörlerin alanına göre aşağıdaki skill'leri yükle (birden çok alan varsa hepsini; branch adı yalnızca ipucudur, klasörler belirler). `software-engineering-discipline` her zaman geçerlidir; tabloda karşılığı olmayan işlerde yalnızca o yüklenir:

  | Alan | Klasör (branch ipucu) | Skill |
  | --- | --- | --- |
  | Android, ileride iOS (Flutter) | `app/` (`*android*`), `app/ios/` (`*ios*`) | `android-app-engineering`; iOS kodu için `ios-app-engineering` |
  | Linux masaüstü (Qt 6, QML, CXX-Qt) | `linux/`, `packaging/` (`*linux*`, `*packaging*`) | `fullstack-engineering` (Qt/QML için ayrı skill yok; arayüz kuralları `docs/tasarim.md`) |
  | Web sitesi | `site/` (`*site*`) | `web-design-pro`; `site/functions/` ve `site/public/_headers` için ayrıca `fullstack-engineering` ve `security-engineering-analyst` |
  | Sunucular, Worker'lar, CI | `server/`, `.github/` | `fullstack-engineering`, `security-engineering-analyst` |
  | Protokol, kriptografi, çekirdek | `crates/totaldrop-crypto`, `-proto`, `-core`, `fuzz/`, `docs/protocol-v0.1-notes.md` (`*core*`) | `applied-cryptography-review`, `security-engineering-analyst` |
  | Komut satırı | `crates/totaldrop-cli` (`*cli*`) | `fullstack-engineering` |
  | Bağımlılıklar ve tedarik zinciri | `Cargo.toml`, `Cargo.lock`, `deny.toml`, Dependabot PR'ları | `security-engineering-analyst` |
  | Güvenlik ve gizlilik | `docs/tehdit-modeli.md`, `site/public/privacy.html`, `SECURITY.md` | `security-engineering-analyst` (açık raporları `SECURITY.md`'deki gizli kanaldan gelir, herkese açık issue'dan değil) |
  | Diller ve çeviriler | `l10n/`, `crates/totaldrop-l10n` | kurallar `docs/kararlar/0008-diller-ve-bolgeler.md` ve `docs/ceviri.md` |
  | macOS, Windows | ileride `macos/`, `windows/` | `macos-app-engineering`, `windows-app-engineering` |
- Tabloda `security-engineering-analyst` istenen her işte (`site/functions/` ve `site/public/_headers` dahil) bu skill'e ek olarak `securitymaxxing` eklentisinin `/securitymaxxing:` komutları da kullanılır (`diff`, `deps`, `secrets`, `crypto` ve konuya göre diğerleri). Eklenti hesap düzeyinde kuruludur, depoya kopyalanmaz; kurulu olmayan oturumda bu adım atlanır ve PR'da belirtilir. Bütün dosyaları okunan sürüm v0.2.1'dir (#422: yalnızca Markdown, hook ya da MCP sunucusu yok); daha yeni bir sürüm, dosyaları okunup bu satır güncellenmeden kullanılmaz. Üç sınır:
  1. Sızan bir anahtar ya da belirteç hiçbir zaman canlı bir çağrıyla doğrulanmaz (`redteam` ve eklentinin `PROMPTS.md`'si bunu önerir). Ajan gizli bilgiyi okumaz ve kullanmaz: `secrets` yalnızca `git ls-files` ile izlenen dosyaları ve git geçmişini tarar, gitignore'daki gizli dosyalar (`key.properties`, `*.jks` gibi) açılmaz. Bulunan bir gizli bilginin yalnızca yeri (`dosya:satır`, değerin hiçbir parçası değil) proje liderine ([GOVERNANCE.md](GOVERNANCE.md)) gizli kanaldan ([SECURITY.md](SECURITY.md)) bildirilir, herkese açık bir issue ya da PR'a yazılmaz; değeri proje lideri değiştirir ([docs/surec.md](docs/surec.md), "Gizli bilgiler").
  2. `redteam` gerçek saldırı gönderir. Yerelde çalıştırılır: loopback'teki Rust sunucularına ya da `wrangler dev --local` ile açılan Worker'lara karşı (`.github/workflows/server.yml`'deki gibi; ayrı bir deneme ortamı yok). totaldrop.app'e ve canlı sunuculara karşı yalnızca proje liderinin açık onayıyla çalıştırılır; üçüncü taraf sistemlere karşı hiçbir zaman çalıştırılmaz.
  3. `fix` kod yazar ama issue → branch → PR → bağımsız inceleme akışı aynen geçerlidir. `crypto` komutu `applied-cryptography-review`, Tamarin modeli ve insan kripto incelemesinin yerini tutmaz, onlara ek olarak kullanılır.

## Komutlar

Zorunlu denetim CI'dır; aşağıdakiler aynı denetimlerin yerel karşılığıdır (ayrıntı: `docs/gelistirme.md`, "Yerel denetimler").

- Rust çekirdeği, CLI ve Rust sunucuları (`core.yml` ile aynı; `linux/` Qt 6 istediği için `linux.yml`'de, Android köprüsü `app/rust` `android.yml`'de derlenir, bu yüzden burada dışarıda). CI `RUSTFLAGS=-D warnings` ile derler; yerelde aynısını görmek için komutların başına `RUSTFLAGS="-D warnings"` ekle (ör. `RUSTFLAGS="-D warnings" cargo test …`):
  ```sh
  cargo fmt --check -p totaldrop-crypto -p totaldrop-proto -p totaldrop-core -p totaldrop-cli -p totaldrop-l10n -p totaldrop-rendezvous -p totaldrop-relay
  cargo clippy --workspace --exclude totaldrop-linux --exclude rust_lib_totaldrop --all-targets --locked -- -D warnings
  cargo test --workspace --exclude totaldrop-linux --exclude rust_lib_totaldrop --locked
  cargo test -p totaldrop-proto <test_adı>            # tek test
  cargo test -p totaldrop-cli --test end_to_end --locked   # CLI ↔ sunucu uçtan uca
  ```
  `end_to_end` yerelde yalnızca Rust sunucularına karşı çalışır. Worker'lara karşı çalıştırmak için `wrangler dev` ile Worker'ları başlatıp `TOTALDROP_TEST_SERVER`, `TOTALDROP_TEST_SERVER_LIMITED` ve `TOTALDROP_TEST_RELAY` değişkenlerini ayarla (`.github/workflows/server.yml`, "End to end against both Workers").
  CI'da `TOTALDROP_REQUIRE_IPV6=1` ile çalışır (çift yığın testleri IPv4'e düşmesin, #403). Kapsam eşiği (`core.yml`): `cargo llvm-cov --locked --summary-only --fail-under-lines 90` ve fmt satırındaki aynı yedi `-p`.
- Worker'lar (`server/rendezvous-worker`, `server/relay-worker`, her birinde): `npm ci && npm run check && npm run --if-present test` (`relay-worker`'da `test` betiği yok).
- Site: `node site/tools/check-i18n.mjs`, `node --test site/tools/test-*.mjs`; üretilen dosyaları denetleyen araçlar (`privacy.mjs`, `team.mjs`, `favicons.mjs`, `brand-files.mjs`, `asset-versions.mjs`, `check-readmes.mjs`) `--write` ile yeniden yazar; CI yalnızca denetler (`.github/workflows/site.yml`).
- Android (`app/`): `flutter pub get --enforce-lockfile`, `flutter gen-l10n`, `flutter analyze`, `flutter test`; köprü crate'i `cargo test -p rust_lib_totaldrop --locked`.
- Linux QML bileşen testleri (Rust derlemesi istemez): `linux/tests/qml/run.sh`.
- Tamarin modeli: `formal/check.sh`, `formal/mutants.py` (`formal/README.md`).
- Depo denetimleri: `node tools/changelog.mjs check`, `tools/check-spdx.sh`, `tools/check-component-versions.sh`, `packaging/check-versions.sh`, `tools/check-advisory-ignores.sh`, `node tools/check-layers.mjs` (crate katmanları), `node tools/check-doc-refs.mjs` (`docs/ARCHITECTURE.md` satır referansları; kayınca `--fix`).
- Hepsi birden: `pre-commit run -c .pre-commit-config-extra.yaml --all-files` (olmayan araçları `SKIP=dart-format,flutter-analyze,worker-tsc` ile atla).

## Her PR'da unutulanlar

- Changelog: her PR `changelog.d/<issue>-<kısa-ad>.<bölüm>.md` ekler; bölüm `added|changed|deprecated|removed|fixed|security|none`. Yayından önce herkese açık issue'su olmayan güvenlik düzeltmesi `ghsa-<id>-<kısa-ad>.security.md` adını alır. Girdi biçimi `tools/changelog.mjs` başındaki açıklamada. `CHANGELOG.md` elle değişmez, yalnızca sürümde `node tools/changelog.mjs release "<başlık>"`.
- Her dosya SPDX başlığı taşır (`tools/check-spdx.sh`, `REUSE.toml`).
- Kullanıcıya görünen her yeni metin 35 dilin hepsine girer, yalnızca TR/EN'e değil: uygulama ve Linux masaüstü metinleri `l10n/app/*.arb`, CLI `l10n/cli/*.arb`, site `site/public/i18n/*.json`, gizlilik `site/privacy/`, README çevirileri `docs/readme/` (`docs/ceviri.md`). İngilizce dosya şablondur; anahtar önce oraya açıklamasıyla eklenir. CI eksik çeviriyi yalnızca raporlar (İngilizce'ye düşer) ama proje kuralı çevirilerin aynı PR'da doldurulmasıdır. Dosya sayısı 35'ten fazladır, çünkü bölge varyantlarının temel dosyası da vardır (`app_pt.arb`, `app_pt_BR.arb`).
- Her platformun kendi sürümü vardır (`docs/versioning.md`); bileşen sürümlerinin tutarlılığını `tools/check-component-versions.sh` denetler.

## Mimari (büyük resim)

Ayrıntı `docs/mimari.md`; dosya ve satır düzeyindeki karşılığı `docs/ARCHITECTURE.md`. Kısaca:

- Tek Cargo çalışma alanı (`Cargo.toml`): `crates/*`, iki Rust sunucusu, `app/rust` (Android köprüsü), `linux`. `fuzz/` ayrı çalışma alanıdır. Çalışma alanı lint'leri `unsafe_code = forbid`; `missing_docs` ve `unwrap/expect/panic/indexing_slicing/arithmetic_side_effects` uyarı (belgesiz bir `pub` öğe de CI'ı kırar); CI uyarıyı hata sayar (`RUSTFLAGS=-D warnings`, clippy `-D warnings`).
- Çekirdek bütün platformlarda aynıdır ve kriptografi yalnızca orada yapılır: `totaldrop-crypto` (CPace, anahtar türetme, STREAM parça şifrelemesi), `totaldrop-proto` (aktarım kodu, sınırlar, yol güvenliği, CBOR kablo mesajları; dışarıdan gelen her veri burada ayrıştırılır, fuzz hedefleri `fuzz/`), `totaldrop-core` (istemci motoru: rendezvous, ham açık anahtarlı QUIC, Noise XXpsk3 + ML-KEM-768, gönder/al, aktarmaya geçiş), `totaldrop-l10n`.
- Arayüzler çekirdeği çağırır, kriptografi yapmaz: Linux Qt 6/QML + CXX-Qt (`linux/`), Android Flutter + flutter_rust_bridge (`app/`, `app/rust`; `frb_generated.rs` üretilmiş dosyadır, elle düzenlenmez), CLI (`crates/totaldrop-cli`).
- Her sunucunun iki uygulaması vardır ve ikisi aynı uçtan uca testten geçer: Rust (`server/totaldrop-rendezvous`, `server/totaldrop-relay`; kendi sunucusunu kuranlar ve testler için) ve Cloudflare Workers + Durable Objects (`server/*-worker`; totaldrop.app). Davranış değişikliği ikisine birden yapılır. Sunucular dosya içeriğini görmez, kayıt tutmaz. Projenin STUN sunucusu yok: varsayılan `stun.cloudflare.com`, ya da kendi kurulan aktarma sunucusunun keşif portu (`server/totaldrop-relay/src/lib.rs:8-10`, `docs/mimari.md`).
- Site (`site/`) Cloudflare Pages'te, dışarıdan hiçbir şey yüklemez; tarayıcıdan gönderme yoktur.
- CI: `ci.yml` değişen klasörlere bakıp platform iş akışlarını çağırır; sondaki `CI` işi `main` için tek zorunlu kontroldür. Sürümler etiketten `android-release.yml` / `linux-release.yml` ile derlenir, dağıtım `deploy.yml` ile.
