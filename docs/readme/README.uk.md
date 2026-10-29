<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Будь-який файл, будь-кому, будь-куди.

Безкоштовно й з відкритим кодом. Наскрізне шифрування. Без пошти, без номера телефону, без облікового запису.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Сайт](https://totaldrop.app)** · **[Завантажити](https://totaldrop.app/download/)** · [Плани](https://totaldrop.app/roadmap/) · [Підтримати](https://totaldrop.app/donate/) · [Випуски](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · **Українська** · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop на Android: головний екран у світлій і темній темі та отримана й перевірена передача" width="820">

</div>

> [!NOTE]
> Цей переклад — чернетка, він перевіряється. Еталоном є [англійська версія](../../README.md).

## Чому TotalDrop

- **Досить короткого коду.** Виберіть файли й отримайте код на кшталт `7-harbor-rocket`. Продиктуйте його або покажіть QR-код; співрозмовник вводить код, і файли надходять.
- **Наскрізне шифрування, яке можна перевірити.** Перш ніж щось буде передано, на обох екранах з'являються ті самі чотири слова. Якщо вони відрізняються, зупиніться.
- **Не треба ніде реєструватися.** Ні облікового запису, ні пошти, ні номера телефону. Необов'язкові облікові записи складатимуться лише з номера, у стилі Mullvad ([опис](../../docs/hesap.md)).
- **Напряму з пристрою на пристрій, будь-де.** Сервер зустрічі лише знайомить два пристрої й не веде журналів; як і будь-який сервер, він бачить їхні IP-адреси. Через інтернет пристрої з'єднуються напряму всюди, де це можливо. Якщо заважає роутер або брандмауер, трафік передає ретранслятор, який не може його прочитати.
- **Як удома у вашій системі.** Застосунки слідують світлій або темній темі системи, кольору акценту, мові й налаштуванню зменшення руху. Застосунок для Linux — нативний Qt 6 на Wayland, він також на льоту бере шрифти, форму кутів і значки з тем KDE Plasma, Hyprland і Omarchy.
- **Безкоштовно, назавжди.** Відкритий код, коштом пожертв. Без реклами, без стеження, без платної версії для основних функцій.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Застосунок для Linux в Omarchy, у кольорах, шрифтах і значках стільниці" width="560">
  <br><sub>Застосунок для Linux в Omarchy: кольори, заокруглення, шрифти й значки беруться зі стільниці.</sub>
</p>

## Завантажити

| Платформа | Де взяти | Примітки |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 або новіший |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 або новіший |
| **Android** | [Приєднатися до бети](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 або новіший, Google Play за запрошенням |
| **Командний рядок** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Незабаром | Підпис коду для цих систем — перше, на що підуть пожертви |

Кожен пакет збирається в CI з коміту з тегом, ніколи на машині розробника. Щоб перевірити пакет перед встановленням, звірте його з `SHA256SUMS` випуску й переконайтеся, де його було зібрано:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Бета.** Код ще не проходив незалежної перевірки безпеки, тож, будь ласка, не покладайтеся на нього для конфіденційних файлів.

## Як це працює

1. Відправник запитує в сервера зустрічі короткий код. Цей код — єдиний секрет, спільний для двох людей.
2. Обидва пристрої перетворюють код на стійкий ключ за допомогою **CPace**. Зловмисник має одну спробу вгадати на кожну передачу й не може перевіряти коди офлайн.
3. Пристрої з'єднуються напряму через **QUIC** (TLS 1.3 із закріпленими необробленими відкритими ключами) і виконують **Noise XXpsk3 з ML-KEM-768**, який розроблено так, щоб він устояв і перед майбутніми квантовими комп'ютерами.
4. Файли шифруються частинами (STREAM), тож кожна частина перевіряється на автентичність щойно надходить.

Протокол описано в документі *TotalDrop Protocol v0*. Рішення, ухвалені під час його реалізації, зібрано в [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), а що кожен сервер може й не може бачити, описано в [моделі загроз](../../docs/tehdit-modeli.md) (турецькою) і [політиці конфіденційності](https://totaldrop.app/privacy).

## Вашою мовою

TotalDrop розмовляє 35 мовами й дотримується числових форматів вашого регіону. Англійську й турецьку перевірено; решта 33 — чернетки, які перевіряють носії мови, тож застосунки залишають тексти, пов'язані з безпекою (перевірка, хибний код, прийняти й відхилити), англійською, доки мову не перевірено. На сайті всі мови є вже сьогодні; застосунки отримають їх у наступних випусках.

Помітили незграбне речення? [Допоможіть з перекладом](../../docs/ceviri.md) (посібник турецькою).

## Підтримати проєкт

TotalDrop залишається безкоштовним, бо люди допомагають. Пожертви оплачують ретранслятор, сертифікати підпису коду, яких вимагають Windows і macOS, і незалежний аудит безпеки до версії 1.0. Усі витрати порядково й скільки вже зібрано: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Підтримати на GitHub"></a>

Ще можна допомогти, [повідомляючи про помилки](https://github.com/totaldrop-app/totaldrop/issues/new/choose), перекладаючи або тестуючи на своєму комп'ютері.

## Для розробників

### Структура репозиторію

| Шлях | Що це | Ліцензія |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, виведення ключів, шифрування поштової скриньки й сеансу, ключі пристроїв, поблокове шифрування STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Коди передачі, ліміти, безпека шляхів, усі мережеві повідомлення (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Клієнтський рушій: сервер зустрічі, QUIC з необробленими відкритими ключами, Noise XXpsk3 + ML-KEM-768, передача | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Тексти інтерфейсу для програм на Rust: вибір мови, повідомлення ICU, форми множини, регіональні числа | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Клієнт командного рядка `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Усі рядки інтерфейсу, спільні для Android, Linux і CLI ([переклад](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Настільний застосунок для Linux: Qt 6 і QML поверх `totaldrop-core`, нативний Wayland | GPL-3.0-or-later |
| `packaging/` | Пакети для Linux, кожен формат окремо: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Застосунок для Android (Flutter); iOS пізніше | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app і сторінка, яку відкривають посилання з QR-кодів | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Цілі для фазингу всіх парсерів | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Сервер зустрічі на Rust, для власного хостингу й тестів | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Той самий сервер на Cloudflare Workers, для `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Бібліотеки мають дозвільні ліцензії, щоб будь-хто міг будувати на основі протоколу; застосунки — під GPL, сервери — під AGPL. Див. [LICENSING.md](../../LICENSING.md).

Кожна платформа має власну версію, тег (`linux-v…`, `android-v…`) і процес CI, тож одну можна випустити, не зачіпаючи інших. Як влаштовано гілки й випуски: [docs/gelistirme.md](../../docs/gelistirme.md) (турецькою) і [docs/versioning.md](../../docs/versioning.md). Правила дизайну: [docs/tasarim.md](../../docs/tasarim.md) і архітектурні рішення: [docs/kararlar](../../docs/kararlar/README.md), обидва — турецькою.

### Збирання

```bash
cargo test
cargo build --release
```

Спробуйте передачу на одній машині у трьох терміналах:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Обидві сторони показують ті самі чотири перевірочні слова. Нешифрований `ws://` допускається лише для сервера на тій самій машині; в усіх інших випадках потрібен `wss://`.

Проєкт завжди орієнтується на найновішу стабільну версію Rust (закріплену в `rust-toolchain.toml`); rustup встановить її під час першого збирання.

**Застосунок для Linux** (потрібен Qt 6.8 або новіший: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Участь і безпека

Перш ніж відкривати pull request, прочитайте [CONTRIBUTING.md](../../CONTRIBUTING.md) і наш [кодекс поведінки](../../CODE_OF_CONDUCT.md). З питаннями — до [SUPPORT.md](../../SUPPORT.md). Про вразливості, будь ласка, повідомляйте конфіденційно, як описано в [SECURITY.md](../../SECURITY.md), і ніколи — у відкритому issue.
