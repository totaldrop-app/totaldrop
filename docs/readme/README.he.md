<!-- Translated from README.md f62f3fe1ba22. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### כל קובץ, לכל אחד, בכל מקום.

חינמי וקוד פתוח. מוצפן מקצה לקצה. בלי אימייל, בלי מספר טלפון, בלי חשבון.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![רישיון](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![חסות](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[אתר](https://totaldrop.app)** · **[הורדה](https://totaldrop.app/download/)** · [מפת דרכים](https://totaldrop.app/roadmap/) · [תרומה](https://totaldrop.app/donate/) · [גרסאות](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · [Ελληνικά](README.el.md) · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · **עברית** · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="TotalDrop ב־Android: מסך הבית במצב בהיר ובמצב כהה, והעברה שהגיעה ואומתה" width="820">

</div>

> [!NOTE]
> תרגום זה הוא טיוטה ונמצא בבדיקה. הגרסה הקובעת היא [English](../../README.md).

## למה TotalDrop

- **מספיק קוד קצר.** בוחרים קבצים ומקבלים קוד כמו `7-harbor-rocket`. מקריאים אותו בקול או מציגים את קוד ה־QR; הצד השני מקליד אותו והקבצים מגיעים.
- **מוצפן מקצה לקצה, ואפשר לבדוק את זה.** בשני המסכים מופיעות אותן ארבע מילים לפני שמשהו זז. אם הן שונות, עוצרים.
- **אין למה להירשם.** בלי חשבון, אימייל או מספר טלפון. חשבונות אופציונליים יהיו מספרים בלבד, בסגנון Mullvad ([תכנון](../../docs/hesap.md)).
- **ישירות ממכשיר למכשיר, בכל מקום.** שרת המפגש רק מפגיש בין שני המכשירים ולא שומר יומנים. המכשירים מתחברים ישירות דרך האינטרנט בכל מקום שאפשר. כשנתב או חומת אש חוסמים את הדרך, שרת ממסר מעביר תעבורה שהוא אינו יכול לקרוא.
- **מרגיש בבית בכל מערכת.** כל אפליקציה עוקבת אחר ערכת הנושא, הגופנים, צבע ההדגשה, השפה והגדרות התנועה של המערכת שלה. אפליקציית Linux כתובה ב־Qt 6 מקורי על Wayland, ועוקבת בזמן אמת אחר ערכות הנושא של KDE Plasma, Hyprland ו־Omarchy.
- **חינמי, לתמיד.** קוד פתוח במימון תרומות. בלי פרסומות, בלי מעקב, בלי מסלול בתשלום ליכולות הבסיסיות.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="אפליקציית Linux ב־Omarchy, בצבעים, בגופנים ובסמלים של שולחן העבודה" width="560">
  <br><sub>אפליקציית Linux ב־Omarchy: הצבעים, הפינות, הגופנים והסמלים מגיעים משולחן העבודה.</sub>
</p>

## הורדה

| פלטפורמה | איפה להשיג | הערות |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 ומעלה |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 ומעלה |
| **Android** | [הצטרפות לבטא](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 ומעלה, Google Play בהזמנה |
| **שורת פקודה** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | בדרך | חתימת קוד עבורן היא הדבר הראשון שהתרומות משלמות עליו |

כל חבילה נבנית על ידי CI מ־commit מתויג, אף פעם לא על המחשב של מפתח. כדי לבדוק חבילה לפני שמתקינים אותה, משווים אותה ל־`SHA256SUMS` של הגרסה ומאמתים היכן היא נבנתה:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **בטא.** הקוד עדיין לא עבר ביקורת אבטחה עצמאית, לכן נא לא להסתמך עליו לקבצים רגישים.

## איך זה עובד

1. השולח מבקש משרת המפגש קוד קצר. הקוד הוא הסוד היחיד ששני האנשים חולקים.
2. שני המכשירים הופכים את הקוד למפתח חזק באמצעות **CPace**. תוקף מקבל ניחוש אחד לכל העברה ואינו יכול לבדוק קודים במצב לא מקוון.
3. הם מתחברים ישירות דרך **QUIC** (TLS 1.3 עם מפתחות ציבוריים גולמיים מוצמדים) ומריצים **Noise XXpsk3 עם ML-KEM-768**, שנשאר בטוח מפני מחשבים קוונטיים עתידיים.
4. הקבצים מוצפנים בחלקים (STREAM), כך שכל חלק מאומת ברגע שהוא מגיע.

הפרוטוקול מתואר ב־*TotalDrop Protocol v0*. ההחלטות שהתקבלו במהלך המימוש שלו נמצאות ב־[docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), ומה כל שרת יכול ואינו יכול לראות מתואר ב[מודל האיומים](../../docs/tehdit-modeli.md) וב[מדיניות הפרטיות](https://totaldrop.app/privacy).

## בשפה שלך

TotalDrop מדבר 35 שפות ועוקב אחר תבניות המספרים של האזור שלך. אנגלית וטורקית עברו בדיקה; 33 האחרות הן טיוטות שדוברים ילידיים בודקים, ולכן האפליקציות משאירות את טקסטי האבטחה (אימות, קוד שגוי, קבלה ודחייה) באנגלית עד ששפה עוברת בדיקה. באתר כולן זמינות כבר היום; האפליקציות יקבלו אותן בגרסאות הבאות שלהן.

נתקלתם במשפט מסורבל? [לעזור בתרגום](../../docs/ceviri.md).

## תמיכה בפרויקט

TotalDrop נשאר חינמי כי אנשים תורמים. התרומות משלמות על שרת הממסר, על אישורי חתימת הקוד ש־Windows ו־macOS דורשות, ועל ביקורת אבטחה עצמאית לפני 1.0. כל עלות, שורה אחר שורה, וכמה נאסף עד כה: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="תמיכה דרך GitHub"></a>

אפשר לעזור גם ב[דיווח על באגים](https://github.com/totaldrop-app/totaldrop/issues/new/choose), בתרגום או בבדיקה על המחשב שלכם.

## למפתחים

### מבנה המאגר

| נתיב | מה זה | רישיון |
| --- | --- | --- |
| `crates/totaldrop-crypto` | חבילה TD0: CPace, גזירת מפתחות, הצפנת תיבת דואר והצפנת סשן, מפתחות מכשיר, הצפנת חלקים ב־STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | קודי העברה, מגבלות, בטיחות נתיבים, כל הודעות התקשורת (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | מנוע הלקוח: מפגש, QUIC עם מפתחות ציבוריים גולמיים, Noise XXpsk3 + ML-KEM-768, העברה | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | טקסטי ממשק לתוכנות Rust: בחירת שפה, הודעות ICU, צורות רבים, מספרים לפי אזור | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | לקוח שורת הפקודה `totaldrop` | GPL-3.0-or-later |
| `l10n/` | כל מחרוזות הממשק, משותפות ל־Android, ל־Linux ולשורת הפקודה ([תרגום](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | אפליקציית שולחן העבודה ל־Linux: Qt 6 ו־QML מעל `totaldrop-core`, Wayland מקורי | GPL-3.0-or-later |
| `packaging/` | חבילות Linux, כל פורמט בנפרד: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | אפליקציית Android (Flutter); iOS בהמשך | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app והדף שקישורי ה־QR פותחים | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | יעדי fuzzing לכל מנתח | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | שרת המפגש ב־Rust, לאירוח עצמי ולבדיקות | AGPL-3.0-or-later |
| `server/rendezvous-worker` | אותו שרת על Cloudflare Workers, עבור `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

הספריות ברישיונות מתירניים כדי שכל אחד יוכל לבנות על הפרוטוקול; האפליקציות תחת GPL והשרתים תחת AGPL. ראו [LICENSING.md](../../LICENSING.md).

לכל פלטפורמה גרסה, תג (`linux-v…`, `android-v…`) ותהליך CI משלה, כך שאפשר לשחרר אחת בלי לגעת באחרות. איך עובדים ענפים ושחרורים: [docs/gelistirme.md](../../docs/gelistirme.md) ו־[docs/versioning.md](../../docs/versioning.md). כללי עיצוב: [docs/tasarim.md](../../docs/tasarim.md). החלטות ארכיטקטורה: [docs/kararlar](../../docs/kararlar/README.md).

### בנייה

```bash
cargo test
cargo build --release
```

נסו העברה על מחשב אחד, בשלושה מסופים:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

שני הצדדים מציגים את אותן ארבע מילות אימות. `ws://` לא מוצפן מתקבל רק עבור שרת באותו מחשב; כל דבר אחר חייב להיות `wss://`.

הפרויקט תמיד מכוון לגרסת Rust היציבה החדשה ביותר (מוצמדת ב־`rust-toolchain.toml`); rustup מתקין אותה בבנייה הראשונה.

**אפליקציית Linux** (דורשת Qt 6.8 ומעלה: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### השתתפות בפיתוח ואבטחה

קראו את [CONTRIBUTING.md](../../CONTRIBUTING.md) לפני פתיחת pull request, ואת [קוד ההתנהגות](../../CODE_OF_CONDUCT.md) שלנו. שאלות מפנים ל־[SUPPORT.md](../../SUPPORT.md). נא לדווח על חולשות אבטחה באופן פרטי כמתואר ב־[SECURITY.md](../../SECURITY.md), ולעולם לא ב־issue ציבורי.
