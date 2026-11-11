# Translation

All of TotalDrop's texts live in the `l10n/` folder at the root of the
repository ([decision 0008](kararlar/0008-diller-ve-bolgeler.md)):

| Folder | What | Format |
| --- | --- | --- |
| `l10n/app/` | The texts the Android and Linux apps share; the Linux desktop entry, the file manager menus and the description in software centres are generated from here too (`tools/desktop-entries.mjs`, [details](linux-dosya-yoneticileri.md#texts)) | ARB (`app_<language>.arb`, e.g. `app_pt_BR.arb`) |
| `l10n/cli/` | The command-line client | ARB (`cli_<language>.arb`) |
| `site/public/i18n/` | totaldrop.app (in its own folder, because the site is published as it is) | JSON (`<language>.json`) |
| `site/privacy/` | The totaldrop.app privacy policy; the authoritative text is English (`en.html`), each translation carries the checksum of `en.html` on its first line, and the pages are generated with `site/tools/privacy.mjs --write` | HTML fragment (`<language>.html`) |
| `docs/readme/` | The GitHub page: translations of `README.md` | Markdown (`README.<language>.md`) |

When `README.md` changes, its translations go stale: the first line of each
translation says which `README.md` it was translated from,
`site/tools/check-readmes.mjs` reports the stale ones as warnings, and
rewrites the language list with `--write`.

The English files are the template. A new text is added to the English
file first, with a description (`@key` → `description`); the other
languages are filled in later. A missing translation falls back to English;
CI reports what is missing but does not fail. Languages with a regional or
script variant also have a base file (`app_pt.arb` alongside
`app_pt_BR.arb` and `app_pt_PT.arb`).

`l10n/status.json` records which languages have been reviewed, which keys
are still drafts, and which keys are critical for security. The file
changes only through reviewed PRs; a maintainer merges the PR.

Each language's `noWords` field holds the answers that mean "no" in that
language, separated by `|` (Turkish: `"hayır|h|yok"`). None of the command
line's "yes" answers (`cliYes`) may be one of them. The second answer in
square brackets in a prompt, such as `[e/H]`, also counts as "no". The
prompt's first answer, however, must be in `cliYes` or be the English `y`
or `yes`; the command line counts those two as "yes" in every language. CI
checks the answers the way the command line compares them: upper and lower
case, dotted and dotless i, and full-width letters count as the same.
Spellings with and without accents are different, though, so write both
(`não|nao`, `όχι|οχι`, `không|khong`). If you see a "no" word missing, open
a PR that adds it to the `noWords` field with `|`, or mention it in a PR.

## For translators

- The easiest route will be Weblate: correcting a text will not need any
  knowledge of the repository. (The link will be added here once the project
  is on Hosted Weblate. Until then, the "translation under review" note in
  the apps points to GitHub; once Weblate opens, the `draftTranslation`
  text must be updated in every language.)
- You can also correct through a PR: change the file concerned, and CI
  checks the format.
- If you have reviewed a language from start to finish, say so in a PR or
  an issue (and on Weblate once it opens); a maintainer updates
  `l10n/status.json` and the "translation under review" note in the
  interface goes away. Critical texts (verification, wrong code,
  accept/decline) appear in English until they are reviewed.

### Rules

- Placeholders such as `{count}`, `{size}` and `{code}` stay as they are;
  their order may change to fit the sentence.
- Plurals use the ICU format: `{count, plural, one{…} other{…}}`. If your
  language needs other plural forms (`zero`, `two`, `few`, `many`), add
  them; the CLDR rules apply.
- `=0`, `=1` and `=2` are the same cases as `zero`, `one` and `two` in
  Flutter; do not write both. In languages such as Russian and Ukrainian,
  `one` is also chosen for 21, 31…, so write the number (`{count}`) into the
  `one` sentence. Other exact numbers (`=5`) cannot be used.
- The placeholders in security texts (`{code}`, `{words}`, `{folder}`,
  `{name}`) appear in every case as often as in English, and cannot be used
  to choose a case.
- The apostrophe is plain text; no escaping is needed (`L'envoi`).
- Translations use no links, HTML or invisible directional characters
  (U+202A–U+202E, U+2066–U+2069); CI rejects them.
- The transfer code (`7-harbor-rocket`) and the verification words are not
  translated: even when the two devices use different languages, they must
  see the same words.
- "TotalDrop" is not translated, and its letters are not changed.
- Tone: short, plain, buttons in the imperative ("Send", "Create code"); no
  exclamation marks and no marketing language.

## Supported languages

Arriving with draft translations (marked in the interface until reviewed):
Arabic (`ar`), Bengali (`bn`), Czech (`cs`), Danish (`da`), German (`de`),
Greek (`el`), Spanish (`es`), Persian (`fa`), Finnish (`fi`), French
(`fr`), Hebrew (`he`), Hindi (`hi`), Hungarian (`hu`), Indonesian (`id`),
Italian (`it`), Japanese (`ja`), Korean (`ko`), Malay (`ms`), Norwegian
(`nb`), Dutch (`nl`), Polish (`pl`), Portuguese (Brazil `pt-BR`, Portugal
`pt-PT`), Romanian (`ro`), Russian (`ru`), Swedish (`sv`), Swahili
(`sw`), Thai (`th`), Ukrainian (`uk`), Urdu (`ur`), Vietnamese (`vi`),
Chinese (Simplified `zh-Hans`, Traditional `zh-Hant`).

Reviewed: English (`en`), Turkish (`tr`).

The drafts were produced with Claude (Anthropic). The app texts (`l10n/`) are licensed MIT OR Apache-2.0, the site's texts (`site/public/i18n/`) GPL-3.0-or-later ([LICENSING.md](../LICENSING.md)); your contribution takes the licence of the folder it goes into.

### Adding a new language

1. Copy the files from the template for `l10n/app/`, `l10n/cli/` and
   `site/public/i18n/` and translate them (for languages with variants,
   together with the base file).
2. Add the language to `l10n/status.json`. For a language that has a
   command-line file, also write its "no" answers (`noWords`); CI rejects a
   language that has a command-line file but no `noWords` field.
3. The Android `localeConfig` is generated from these files; CI checks that
   the generated files are up to date. Add the language, under its own
   name, to the site's language list (`LANGUAGES` in
   `site/public/lang.js`); CI checks that the list matches the `i18n/`
   files, and that the `REVIEWED` list matches `l10n/status.json`. For a
   language that has not been reviewed, the site shows a short draft note
   under the heading.
4. If the script is new, add the matching Noto font to the Linux packages
   and try it with a screenshot.
