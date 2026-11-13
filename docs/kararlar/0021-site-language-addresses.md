# 0021: The site has an address for every page in every language, written by a build step

- **Status:** Accepted, 13 November 2026 (Ozan, #513)
- **Context:** The site was written in English and translated in the
  browser: `app.js` chose a language from the browser's settings, fetched
  `/i18n/<tag>.json` and replaced the text at the same address. Checked on
  the live site on 13 November 2026 with curl and headless Chromium:

  - a request without JavaScript and with `Accept-Language: tr` got
    `<html lang="en">`, the English title and no `hreflang`;
  - a browser set to en-US, with JavaScript, stayed in English and never
    asked for a translation, which is what a search engine's renderer sees;
  - a browser set to Turkish or Arabic got the translation, but the address
    and the canonical link stayed `https://totaldrop.app/`;
  - `https://totaldrop.app/tr/` was a 404.

  A search engine indexes what an address returns, so the 34 other
  languages could not be indexed at all, however good the translations.
  Link previews (`og:` tags) were English in every language, because the
  apps that draw them do not run scripts. A link shared by a Turkish
  visitor opened in English for an English one. The loss in search traffic
  was not measured; what was shown is that indexing was impossible.

  The privacy policy already had a page per language with `hreflang`
  (`site/tools/privacy.mjs`, #207).

- **Options considered:**

  1. *Static pages, committed.* A tool writes the pages into the
     repository, as the privacy pages are. Every published byte is in the
     review, but 204 more files change whenever a script or the stylesheet
     does (their `?v=` version, #291), and site pull requests conflict with
     each other on every page.
  2. *Static pages, written by a build step and not committed.* One
     source: the English pages and the translation files.
  3. *Translation at the edge.* A Pages Function rewrites the English HTML
     per request. No generated files, but every page view then depends on
     code that can fail, and the site stops being plain files.
  4. *Signals only.* `?lang=tr` addresses and `hreflang`, translation still
     in the browser. Clients that run no script, and link previews, would
     still get English.

- **Decision:** Option 2.

  - `site/tools/build.mjs` copies `site/public` to `site/dist` and writes
    six pages (home, download, roadmap, donate, team, brand) once per
    language under `/<tag>/`. Cloudflare Pages publishes `site/dist`. CI
    runs the same command on every pull request; the deploy runs it before
    uploading. The tool uses only Node's standard library.
  - English stays at the addresses it had and is `x-default`. The tag is
    the translation file's name (`/pt-BR/`, `/zh-Hans/`), as on the privacy
    pages. The download page is `download.totaldrop.app/<tag>/`; the
    middleware picks the file on that host name.
  - Each generated page says its language in the HTML: `lang`, `dir`,
    title, description, `og:` tags, a canonical of its own. Every version
    of a page, English included, carries the same 35 `hreflang` links and
    `x-default`. The sitemap lists every address.
  - The address decides the language. The picker goes to the same page,
    and the same `#part`, in the chosen language. A visitor on an English
    address whose browser prefers another language, and who has not chosen
    English, is sent to that language's address; a translated address
    never redirects. The choice is still remembered only in the browser.
  - `/r`, the 404 page and the privacy pages do not change. `/r` keeps one
    address and is translated in the browser as before, so every receive
    link ever shared keeps working and the code after `#` still never
    reaches the server.
  - No new user-facing text: titles and descriptions were already
    translated (`meta.<page>.title`, `meta.<page>.desc`).

- **Why:** Plain files are the most robust answer a server can give. The
  only code at request time is the middleware that was already there, with
  one more rule for the download host name; the content security policy
  stays as it is. Building in CI is how every other TotalDrop package is made. One
  source keeps a design change from becoming 35 changes.

- **Consequences:**

  - The site now has a build step. What is published is not in the
    repository; the tests in `site/tools/test-build.mjs` check its shape
    (language and direction, no translated text left in English, self
    canonical, reciprocal `hreflang`, links that exist, the old addresses
    unchanged), and anyone can write the same folder with one command.
  - Translations that no native speaker has reviewed (all but English and
    Turkish today) become pages a search can find. They carry the draft
    note in the page, as they did when shown by the script.
  - Sending a visitor to their language from an English address is what
    the site already did in place; search engines advise against
    redirecting by language when it hides versions from them. Here every
    version is linked with `hreflang` and listed in the sitemap, the
    picker is on every page, and a client whose language is English is
    never redirected.
  - The choice of language is kept per host name (the browser's storage
    is). A visitor whose browser prefers Turkish and who chose English on
    totaldrop.app is sent to Turkish the first time they open
    download.totaldrop.app, and has to choose English there once too.
    Before, that page was also shown in Turkish, but its links led back to
    English addresses. Carrying the choice between the two host names is
    tracked in #513.
  - Not done here, tracked in #513: language links that work without
    JavaScript, `noindex` for the raw translation files, redirects for
    `/en/` and lower-case tags, and moving the `?v=` versions and the
    privacy pages into the build so that generated files leave the
    repository.
  - Not decided here: `og:locale`. Its values name a territory
    (`tr_TR`), which a language tag such as `ar` does not give.
