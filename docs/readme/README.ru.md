<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Любой файл, кому угодно, куда угодно.

Бесплатно и с открытым кодом. Сквозное шифрование. Без почты, без номера телефона, без аккаунта.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Сайт](https://totaldrop.app)** · **[Скачать](https://totaldrop.app/download/)** · [Планы](https://totaldrop.app/roadmap/) · [Пожертвовать](https://totaldrop.app/donate/) · [Выпуски](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · **Русский** · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop на Android: главный экран в светлой и тёмной теме и полученная и проверенная передача" width="820">

</div>

> [!NOTE]
> Этот перевод — черновик, он проверяется. Эталоном служит [английская версия](../../README.md).

## Почему TotalDrop

- **Достаточно короткого кода.** Выберите файлы и получите код вроде `7-harbor-rocket`. Продиктуйте его или покажите QR-код; собеседник вводит код, и файлы приходят.
- **Сквозное шифрование, которое можно проверить.** Прежде чем что-либо будет передано, на обоих экранах появляются одни и те же четыре слова. Если они различаются, остановитесь.
- **Не нужно нигде регистрироваться.** Ни аккаунта, ни почты, ни номера телефона. Необязательные аккаунты будут состоять только из номера, в стиле Mullvad ([описание](../../docs/hesap.md)).
- **Напрямую с устройства на устройство, где угодно.** Сервер встречи только знакомит два устройства и не ведёт журналов; как и любой сервер, он видит их IP-адреса. Через интернет устройства соединяются напрямую везде, где это возможно. Если мешает роутер или файрвол, трафик передаёт ретранслятор, который не может его прочитать.
- **Как дома в вашей системе.** Приложения следуют светлой или тёмной теме системы, акцентному цвету, языку и настройке уменьшения движения. Приложение для Linux — нативное Qt 6 на Wayland, оно также на лету берёт шрифты, форму углов и значки из тем KDE Plasma, Hyprland и Omarchy.
- **Бесплатно, навсегда.** Открытый код, на средства пожертвований. Без рекламы, без слежки, без платной версии для основных функций.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Приложение для Linux в Omarchy, в цветах, шрифтах и значках рабочего стола" width="560">
  <br><sub>Приложение для Linux в Omarchy: цвета, скругления, шрифты и значки берутся с рабочего стола.</sub>
</p>

## Скачать

| Платформа | Где взять | Примечания |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 или новее |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 или новее |
| **Android** | [Присоединиться к бете](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 или новее, Google Play по приглашению |
| **Командная строка** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Скоро | Подпись кода для этих систем — первое, на что пойдут пожертвования |

Каждый пакет собирается в CI из коммита с тегом, никогда на машине разработчика. Чтобы проверить пакет перед установкой, сверьте его с `SHA256SUMS` выпуска и убедитесь, где он был собран:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Бета.** Код ещё не проходил независимую проверку безопасности, поэтому, пожалуйста, не полагайтесь на него для конфиденциальных файлов.

## Как это работает

1. Отправитель запрашивает у сервера встречи короткий код. Этот код — единственный секрет, общий для двух людей.
2. Оба устройства превращают код в стойкий ключ с помощью **CPace**. У атакующего есть одна попытка угадать на каждую передачу, и проверять коды офлайн он не может.
3. Устройства соединяются напрямую по **QUIC** (TLS 1.3 с закреплёнными необработанными открытыми ключами) и выполняют **Noise XXpsk3 с ML-KEM-768**, который рассчитан на то, чтобы устоять и перед будущими квантовыми компьютерами.
4. Файлы шифруются частями (STREAM), поэтому каждая часть проверяется на подлинность по мере поступления.

Протокол описан в документе *TotalDrop Protocol v0*. Решения, принятые при его реализации, собраны в [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), а что каждый сервер может и не может видеть, описано в [модели угроз](../../docs/tehdit-modeli.md) (на турецком) и [политике конфиденциальности](https://totaldrop.app/privacy).

## На вашем языке

TotalDrop говорит на 35 языках и следует числовым форматам вашего региона. Английский и турецкий проверены; остальные 33 — черновики, которые проверяют носители языка, поэтому приложения оставляют тексты, связанные с безопасностью (проверка, неверный код, принять и отклонить), на английском, пока язык не проверен. На сайте все языки есть уже сегодня; приложения получат их в следующих выпусках.

Заметили неуклюжую фразу? [Помогите с переводом](../../docs/ceviri.md) (руководство на турецком).

## Поддержать проект

TotalDrop остаётся бесплатным, потому что люди помогают. Пожертвования оплачивают ретранслятор, сертификаты подписи кода, которых требуют Windows и macOS, и независимый аудит безопасности до версии 1.0. Все расходы построчно и сколько уже собрано: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Поддержать на GitHub"></a>

Ещё можно помочь, [сообщая об ошибках](https://github.com/totaldrop-app/totaldrop/issues/new/choose), переводя или тестируя на своём компьютере.

## Для разработчиков

### Структура репозитория

| Путь | Что это | Лицензия |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, вывод ключей, шифрование почтового ящика и сеанса, ключи устройств, поблочное шифрование STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Коды передачи, лимиты, безопасность путей, все сетевые сообщения (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Клиентский движок: сервер встречи, QUIC с необработанными открытыми ключами, Noise XXpsk3 + ML-KEM-768, передача | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Тексты интерфейса для программ на Rust: выбор языка, сообщения ICU, формы множественного числа, региональные числа | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Клиент командной строки `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Все строки интерфейса, общие для Android, Linux и CLI ([перевод](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Настольное приложение для Linux: Qt 6 и QML поверх `totaldrop-core`, нативный Wayland | GPL-3.0-or-later |
| `packaging/` | Пакеты для Linux, каждый формат отдельно: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Приложение для Android (Flutter); iOS позже | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app и страница, которую открывают ссылки из QR-кодов | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Цели для фаззинга всех парсеров | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Сервер встречи на Rust, для собственного хостинга и тестов | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Тот же сервер на Cloudflare Workers, для `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Библиотеки распространяются под разрешительными лицензиями, чтобы любой мог строить на основе протокола; приложения — под GPL, серверы — под AGPL. См. [LICENSING.md](../../LICENSING.md).

У каждой платформы своя версия, свой тег (`linux-v…`, `android-v…`) и свой процесс CI, поэтому одну можно выпустить, не затрагивая остальные. Как устроены ветки и выпуски: [docs/gelistirme.md](../../docs/gelistirme.md) (на турецком) и [docs/versioning.md](../../docs/versioning.md). Правила дизайна: [docs/tasarim.md](../../docs/tasarim.md) и архитектурные решения: [docs/kararlar](../../docs/kararlar/README.md), и то и другое на турецком.

### Сборка

```bash
cargo test
cargo build --release
```

Попробуйте передачу на одной машине в трёх терминалах:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Обе стороны показывают одни и те же четыре проверочных слова. Нешифрованный `ws://` допускается только для сервера на той же машине; во всех остальных случаях нужен `wss://`.

Проект всегда ориентируется на новейшую стабильную версию Rust (закреплена в `rust-toolchain.toml`); rustup установит её при первой сборке.

**Приложение для Linux** (нужен Qt 6.8 или новее: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Участие и безопасность

Прежде чем открывать pull request, прочитайте [CONTRIBUTING.md](../../CONTRIBUTING.md) и наш [кодекс поведения](../../CODE_OF_CONDUCT.md). С вопросами — в [SUPPORT.md](../../SUPPORT.md). Об уязвимостях, пожалуйста, сообщайте конфиденциально, как описано в [SECURITY.md](../../SECURITY.md), и никогда — в открытом issue.
