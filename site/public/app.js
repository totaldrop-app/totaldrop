(() => {
  "use strict";

  // Interface language (ADR 0008). English is the page itself; other
  // languages come from /i18n/<tag>.json on this same domain. The choice
  // follows the browser's languages and can be changed with the picker;
  // it is remembered only in this browser.
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
  // Right to left: languages whose likely script is one of these (the
  // same rule as msg.js).
  const RTL_SCRIPTS = ["Arab", "Hebr", "Thaa", "Syrc", "Nkoo", "Adlm", "Rohg", "Mand", "Samr"];
  const isRtl = (tag) => {
    try { return RTL_SCRIPTS.includes(new Intl.Locale(tag).maximize().script); } catch (e) { return false; }
  };
  const PAGE = document.body.dataset.page || "home";

  const nodes = Array.from(document.querySelectorAll("[data-i18n]"));
  const descTag = document.querySelector('meta[name="description"]');
  const EN = {};
  nodes.forEach((el) => { EN[el.dataset.i18n] = el.textContent; });
  // Accessible names (aria-label) translate the same way, by key.
  const labelled = Array.from(document.querySelectorAll("[data-i18n-label][aria-label]:not(#lang-select)"));
  labelled.forEach((el) => { EN[el.dataset.i18nLabel] = el.getAttribute("aria-label"); });
  EN["meta." + PAGE + ".title"] = document.title;
  if (descTag) EN["meta." + PAGE + ".desc"] = descTag.getAttribute("content");
  EN["dl.copied"] = "Copied";
  EN["r.copied"] = "Copied";
  EN["lang.label"] = "Language";
  EN["lang.draft"] = "This translation is a draft and is being reviewed.";
  let dict = EN;
  // An empty or missing translation falls back to English.
  const has = (d, key) => typeof d[key] === "string" && d[key] !== "";
  const t = (key) => (has(dict, key) ? dict[key] : EN[key]);

  const store = {
    get() { try { return localStorage.getItem("td-lang"); } catch (e) { return null; } },
    set(v) { try { localStorage.setItem("td-lang", v); } catch (e) { /* storage unavailable */ } }
  };

  // The best available language for the ordered preferences: the tag
  // itself, then without region, then the same language and script (so
  // zh-HK finds zh-Hant, not zh); English when nothing fits.
  function negotiate(prefs) {
    const tags = LANGUAGES.map((l) => l[0]);
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
      if (exact || same[0]) return exact || same[0];
    }
    return "en";
  }

  const picker = document.getElementById("lang-select");
  // The draft note sits right under the header, so it is read before the
  // translated text.
  let note = null;
  const header = document.querySelector("body > header");
  if (header) {
    note = document.createElement("p");
    note.className = "lang-note";
    note.setAttribute("role", "note");
    note.hidden = true;
    header.after(note);
  }

  function apply(tag, d) {
    dict = d;
    nodes.forEach((el) => {
      const key = el.dataset.i18n;
      const translated = tag === "en" || has(d, key);
      const text = t(key);
      if (typeof text === "string") el.textContent = text;
      // English shown inside another language keeps its own language and
      // direction, for screen readers and punctuation.
      if (translated) { el.removeAttribute("lang"); el.removeAttribute("dir"); } else { el.lang = "en"; el.dir = "ltr"; }
    });
    labelled.forEach((el) => {
      const label = t(el.dataset.i18nLabel);
      if (typeof label === "string") el.setAttribute("aria-label", label);
    });
    document.documentElement.lang = tag;
    document.documentElement.dir = isRtl(tag) ? "rtl" : "ltr";
    document.title = t("meta." + PAGE + ".title");
    if (descTag) descTag.setAttribute("content", t("meta." + PAGE + ".desc"));
    if (note) {
      note.hidden = REVIEWED.includes(tag);
      note.textContent = note.hidden ? "" : t("lang.draft");
    }
    if (picker) {
      picker.value = tag;
      picker.setAttribute("aria-label", t("lang.label"));
    }
  }

  // Only the latest choice is applied, however the downloads finish; the
  // choice is saved once its language has loaded.
  let request = 0;
  const reduced = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  async function show(tag, { save = false, animate = false } = {}) {
    const mine = ++request;
    let d = EN;
    if (tag !== "en") {
      try {
        const res = await fetch("/i18n/" + tag + ".json", { credentials: "omit" });
        if (!res.ok) throw new Error(String(res.status));
        d = await res.json();
      } catch (e) {
        // Could not load it: show English and remember nothing.
        tag = "en";
        d = EN;
        save = false;
      }
    }
    if (mine !== request) return;
    if (save) store.set(tag);
    if (animate && !reduced && document.startViewTransition) document.startViewTransition(() => apply(tag, d));
    else apply(tag, d);
  }

  if (picker) {
    LANGUAGES.forEach(([tag, name]) => {
      const o = document.createElement("option");
      o.value = tag;
      o.lang = tag;
      o.textContent = name;
      picker.append(o);
    });
    picker.hidden = false;
    picker.addEventListener("change", () => show(picker.value, { save: true, animate: true }));
  }

  const saved = store.get();
  const initial = LANGUAGES.some((l) => l[0] === saved)
    ? saved
    : negotiate(navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language]);
  if (picker) picker.value = initial;
  if (initial !== "en") show(initial);

  // Download page: mark the card for the device in hand, and copy commands.
  const cards = document.querySelectorAll("[data-platform]");
  if (cards.length) {
    const ua = navigator.userAgent || "";
    const hint = (navigator.userAgentData && navigator.userAgentData.platform) || "";
    const p = /android/i.test(ua) ? "android"
      : /iphone|ipad|ipod/i.test(ua) ? "ios"
      : /mac/i.test(hint || ua) ? "macos"
      : /win/i.test(hint || ua) ? "windows"
      : /linux|cros/i.test(hint || ua) ? "linux" : "";
    const mine = document.querySelector('[data-platform="' + p + '"]');
    if (mine) {
      mine.classList.add("is-yours");
      mine.parentElement.prepend(mine);
      // Where the desktop app is still planned, the command line works today.
      const cli = document.querySelector('[data-platform="cli"]');
      if (cli && ["macos", "windows"].includes(p)) cli.parentElement.prepend(cli);
    }
  }
  // Receive link (totaldrop.app/r#7-harbor-rocket): the code is after "#",
  // so the browser never sends it to the server, and this page sends it
  // nowhere either. It only hands it to the app.
  if (PAGE === "receive") {
    window.addEventListener("hashchange", () => location.reload());
    // Turkish İ/ı and Arabic-Indic or Persian digits must not make a valid
    // code look invalid; the app then gets the plain ASCII form.
    let hash = "";
    try { hash = decodeURIComponent(location.hash.slice(1)); } catch (e) { hash = ""; }
    const raw = hash.trim()
      .replace(/[İIı]/g, "i")
      .replace(/[\u0660-\u0669]/g, (d) => String(d.charCodeAt(0) - 0x0660))
      .replace(/[\u06F0-\u06F9]/g, (d) => String(d.charCodeAt(0) - 0x06F0))
      .toLowerCase();
    const ok = /^[0-9]{1,6}(-[a-z]{3,8}){2,3}$/.test(raw);
    document.getElementById(ok ? "r-has-code" : "r-no-code").hidden = false;
    if (ok) {
      document.getElementById("r-code").textContent = raw;
      document.getElementById("r-open").href = "totaldrop://r/" + raw;
      const copy = document.getElementById("r-copy");
      copy.addEventListener("click", async () => {
        try {
          await navigator.clipboard.writeText(raw);
          copy.textContent = t("r.copied");
        } catch (e) { window.getSelection().selectAllChildren(document.getElementById("r-code")); }
      });
    }
  }

  document.querySelectorAll("button[data-copy]").forEach((b) => {
    b.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(b.dataset.copy);
        b.textContent = t("dl.copied");
      } catch (e) {
        const pre = b.previousElementSibling;
        if (pre) window.getSelection().selectAllChildren(pre);
      }
    });
  });

  // The hero mark: the drop visits each device in turn (docs/tasarim.md).
  // Without JavaScript, or with reduced motion, the mark stays at rest.
  const hero = document.querySelector(".hero-mark");
  const motion = hero && hero.querySelector("animateMotion");
  if (motion && typeof motion.beginElement === "function" && window.matchMedia) {
    const less = window.matchMedia("(prefers-reduced-motion: reduce)");
    const sync = () => {
      if (less.matches) {
        hero.classList.remove("moving");
        motion.endElement();
      } else {
        hero.classList.add("moving");
        motion.beginElement();
      }
    };
    sync();
    if (less.addEventListener) less.addEventListener("change", sync);
    else if (less.addListener) less.addListener(sync);
  }
})();
