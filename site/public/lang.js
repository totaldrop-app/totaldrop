// The site's languages and how one is chosen (ADR 0008). Kept apart from
// app.js so CI can load it without a page (site/tools/check-i18n.mjs) and
// check it against l10n/status.json and l10n/tests/negotiation.json, the
// cases the apps' Rust negotiation passes too.
((root) => {
  "use strict";

  // Each language under its own name (never a flag), in tag order.
  const LANGUAGES = [
    ["ar", "العربية"],
    ["bn", "বাংলা"],
    ["cs", "Čeština"],
    ["da", "Dansk"],
    ["de", "Deutsch"],
    ["el", "Ελληνικά"],
    ["en", "English"],
    ["es", "Español"],
    ["fa", "فارسی"],
    ["fi", "Suomi"],
    ["fr", "Français"],
    ["he", "עברית"],
    ["hi", "हिन्दी"],
    ["hu", "Magyar"],
    ["id", "Bahasa Indonesia"],
    ["it", "Italiano"],
    ["ja", "日本語"],
    ["ko", "한국어"],
    ["ms", "Bahasa Melayu"],
    ["nb", "Norsk bokmål"],
    ["nl", "Nederlands"],
    ["pl", "Polski"],
    ["pt-BR", "Português (Brasil)"],
    ["pt-PT", "Português (Portugal)"],
    ["ro", "Română"],
    ["ru", "Русский"],
    ["sv", "Svenska"],
    ["sw", "Kiswahili"],
    ["th", "ไทย"],
    ["tr", "Türkçe"],
    ["uk", "Українська"],
    ["ur", "اردو"],
    ["vi", "Tiếng Việt"],
    ["zh-Hans", "简体中文"],
    ["zh-Hant", "繁體中文"]
  ];
  // Languages a native speaker has checked (l10n/status.json); every other
  // one shows a note that it is a draft.
  const REVIEWED = ["en", "tr"];

  // The best available language for the ordered preferences: the tag
  // itself, then the same language and script (so zh-HK finds zh-Hant, not
  // zh-Hans), preferring the same region; English when nothing fits.
  // Portuguese outside Brazil falls back to pt-PT, as in CLDR and the apps.
  function negotiate(prefs, available) {
    const tags = available || LANGUAGES.map((l) => l[0]);
    const script = (tag) => { try { return new Intl.Locale(tag).maximize(); } catch (e) { return null; } };
    for (let pref of prefs) {
      if (!pref) continue;
      // tr_TR, and Norwegian as Bokmål, as the apps read them.
      pref = String(pref).replace(/_/g, "-").replace(/^(no|nn)(?=-|$)/i, "nb");
      if (tags.includes(pref)) return pref;
      const p = script(pref);
      if (!p) continue;
      const same = tags.filter((tag) => {
        const a = script(tag);
        return a && a.language === p.language && a.script === p.script;
      });
      const exact = same.find((tag) => script(tag).region === p.region);
      if (exact) return exact;
      if (p.language === "pt" && p.region !== "BR" && same.includes("pt-PT")) return "pt-PT";
      if (same[0]) return same[0];
    }
    return "en";
  }

  root.TDLang = { LANGUAGES, REVIEWED, negotiate };
})(typeof window !== "undefined" ? window : globalThis);
