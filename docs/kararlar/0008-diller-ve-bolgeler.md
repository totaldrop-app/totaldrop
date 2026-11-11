# 0008: Every language, every regional setting

- **Status:** Accepted, 28 October 2026
- **Context:** The app is only in English and Turkish. The goal is "every
  file, to everyone, everywhere"; that requires working with whatever
  language and region the user has chosen on their device. Today the texts
  are kept in two separate places (the Android ARB files, Linux
  `Strings.qml`), plurals and size formatting are written by hand and only
  know these two languages; the CLI is English only; the site's dictionary
  is inside `app.js`.
- **Options:**
  1. Each platform with its own tool (Flutter gen_l10n, Qt Linguist,
     gettext, separate JSON for the site). The sentences the apps share are
     translated twice, and Android and Linux drift apart.
  2. **The shared interface texts in one source: ARB files with ICU
     MessageFormat; the formatting rules from CLDR.**
  3. A translation service's own format. Dependence on the format and the
     service.
- **Decision:** Option 2. An open format (ARB), no dependence on a service.

## Source

- The texts that Android and Linux share are ARB files under `l10n/app/`,
  the command line's texts under `l10n/cli/`. Because the site is published
  as it is, it keeps its own texts as JSON under `site/public/i18n/`. So
  there are three sources: the two apps share the same sentences; the CLI
  and the site carry their own sentences and take the shared ones (error
  messages, size units) from the app file.
- File names use underscores, by Flutter's rule (`app_pt_BR.arb`,
  `app_zh_Hant.arb`); in documents, language codes are written in BCP 47
  (`pt-BR`). Every language with a regional or script variant also has a
  base file (`app_pt.arb`, `app_zh.arb`): gen_l10n requires it, and the
  closest match falls back to it.
- English is the template. In the template, plurals are written with
  `one{…}`/`other{…}`. Flutter uses `=0`, `=1`, `=2` as the `zero`, `one`,
  `two` cases; the Rust resolver does the same, and rejects other exact
  numbers and the same case written twice. Numbers are grouped on both
  sides (`decimalPattern`).

## Message format

- ICU4X has no MessageFormat; Flutter resolves the messages itself at
  build time. So for Rust (Linux, CLI) and the site, separate resolvers are
  written for the same small subset, checked by the same tests: `{arg}`,
  `{n, plural, =N{…} zero/one/two/few/many/other{…}}`,
  `{x, select, …, other{…}}`. There is no `selectordinal`, no `#`
  shorthand and no escaping.
- The apostrophe (`'`) is plain text everywhere (gen_l10n `use-escaping`
  off); French and Italian sentences look the same on every platform.
- The shared test vectors are in `l10n/tests/messages.json`; the Rust and
  site resolvers and the code Flutter generates must give the same output.
- Plurals are formatted by ICU4X (`icu_plurals`), numbers by
  `icu_decimal`; on the site by the browser's `Intl.PluralRules` and
  `Intl.NumberFormat`. Size units ("1.5 MB") are a translatable pattern in
  the ARB, and the number goes in with the local format.

## Choosing the language and the region

- **Language:** The system's preference list is read. For each preference,
  CLDR's fallback chain (ICU4X `LocaleFallbacker`, e.g. `pt-AO` → `pt-PT` →
  `pt`) and the likely subtags (`LocaleExpander`, e.g. `zh-HK` →
  `zh-Hant`) are tried; if none is available, English. This is a fallback
  chain, not a full CLDR language matcher. `no` and `nn` map to `nb`.
  - Android: the device language and, on Android 13+, the per-app language
    (`localeConfig`, generated from `l10n/`). Because Flutter's default
    choice can fall back from `zh-HK` to Simplified Chinese, a custom
    `localeListResolutionCallback` applies the same rules.
  - Linux and the CLI: the gettext order, `LANGUAGE` (a colon-separated
    list; ignored when the locale is `C`) > `LC_ALL` > `LC_MESSAGES` >
    `LANG`.
  - Site: `navigator.languages`, plus a manual choice (kept in the
    browser).
- **Plurals** are chosen by the rules of the language of the sentence
  shown; the regional setting does not change the plural.
- **Numbers, sizes and percentages** follow the regional setting: on Linux
  `LC_ALL` > `LC_NUMERIC` > `LANG` (as Qt does), on Android the device
  region. The percent sign's position comes from the region too ("%50",
  "50 %"); Android writes it that way (#483), while Linux still uses the
  translated pattern (#491). The region's digit system (e.g. Arabic-Indic
  digits) is kept on Linux and in the CLI; on Android and on the site this
  is not claimed until it is tested separately (#483, 11 November 2026;
  the earlier text did not mention percentages, and Android wrote the
  percentage with the app language's pattern).
- **Text direction:** Derived from the language (Arabic, Persian, Hebrew
  and Urdu are right to left). On Linux the interface is mirrored in QML
  with `LayoutMirroring`; for Qt's own windows (e.g. menus) the app's
  translator gives `QT_LAYOUT_DIRECTION` from the chosen language. The
  texts that Qt and the desktop style draw themselves (the right-click
  menu of text fields) also come from `l10n/app` through this translator;
  Qt's `qtdeclarative_*.qm` files have these texts in only a few
  languages. KDE's `org.kde.desktop` style loads its own translation
  afterwards, and in the languages it has, that one shows (#307,
  6 November 2026; the earlier text foresaw loading `qtbase_*.qm` and
  `setLayoutDirection`, and neither had been implemented).
- **CLI:** Because terminals do not show bidirectional text reliably,
  right-to-left languages fall back to English on the command line for
  now. Output meant for machines (the code itself, later `--json`) is never
  translated in any language and is written with ASCII digits.

## Translations and review

- **Drafts:** New languages arrive with draft translations produced by
  Claude (Anthropic); they are under the project's licence. Who translated
  with which tool is written in `l10n/status.json`.
- **Status file:** `l10n/status.json` records for each language whether it
  has been reviewed, and the keys added as drafts after the review. Only a
  maintainer changes this file, through a PR; a marker inside the
  translation file is not enough. In languages that contain drafts, the
  interface shows a "translation under review" note.
- **Security-related texts** (verification words, the wrong-code warning,
  accept/decline questions) are on the "critical" list in `status.json`.
  Drafts of these keys are not shown; they appear in English until they are
  reviewed.
  The description in software centres (`metainfo*`) is not on this list: it
  describes a feature and does not guide a decision during a transfer. So
  its drafts are published too (#324).
- **Corrections:** Native speakers correct translations through Hosted
  Weblate (free on application for open-source projects) or through a PR.
  Every change from Weblate also arrives as a PR and goes through the
  review in [0007](0007-surec.md). The `icu-message-format` check is on in
  Weblate.

## CI checks

- **Error:** a key that is not in the template; a broken ICU message; a
  placeholder that is not in the template; a critical placeholder
  (`{code}`, `{words}`, `{folder}`, `{name}`) missing or present more than
  once; directional control characters (U+202A–U+202E, U+2066–U+2069) and
  zero-width characters in translations; URLs and HTML; a regional file
  without a base language file.
- **Report (not an error):** missing translations. Missing text falls back
  to English; a new English sentence does not break CI in 30
  languages.
- Pseudo-languages: CI generates `en-XA` (accented and 40% longer) and
  `ar-XB` (right to left); the screen checks are done with these too.

## Security and accessibility

- Text from the other side (file and folder names) enters a sentence with
  direction isolation (FSI…PDI; in Flutter `Directionality`/
  `BidiFormatter`). Directional control characters in file names are made
  visible (a name made of `fatura` + U+202E + `fdp.exe` looks like
  "faturaexe.pdf" on screen; this guards against extension-hiding
  attacks).
- The transfer code is shown in English words in every language, isolated
  left to right. In code entry, automatic capitalisation and correction
  are off; letters are lowercased independently of the language (Turkish
  `İ` and `ı` explicitly to `i`), and Arabic-Indic and Persian digits are
  accepted.
- Screen readers read the code and the verification words with an English
  voice (`Semantics` with a locale in Flutter); tried with TalkBack and
  Orca.
- The Linux packages require the Noto fonts (Arabic, Bengali, Devanagari,
  Thai and others) as dependencies; line height is tried in Thai and
  Arabic.

## Privacy

- The chosen language goes to no third party. The site downloads the
  language file from its own domain; this request, like the
  `Accept-Language` the browser already sends, shows Cloudflare which
  language was requested. The apps send the language to no server.

## What does not change

- The words in the transfer code and in the verification come from the same
  English list in every language; even when the two sides use different
  languages, they must type the same code and see the same words. The
  protocol does not change.

## Why

The two apps show the same sentence, and it is translated once. ICU
MessageFormat and CLDR know every language's plural and number rules;
writing them by hand means errors in every language. ARB is Flutter's
native format and an open format that Weblate supports.

## Consequences

- The hand-written plural and size code in Linux `Strings.qml` goes away,
  replaced by the `totaldrop-l10n` crate; the CLI uses the same crate.
- The scope is not only the in-app texts: Android's own texts (the app
  name, notification channels, share target labels), the Linux `.desktop`
  and AppStream translations and the Play Store descriptions are fed from
  these sources too.
- Every interface change is also tried in at least one right-to-left and
  one CJK language ([tasarim.md](../tasarim.md)).
- Details and the translator's guide: [ceviri.md](../ceviri.md).

## When to look again

If the community on Weblate grows and most drafts are reviewed, publish new
languages only after they have been reviewed.
