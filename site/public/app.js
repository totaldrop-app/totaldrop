(() => {
  "use strict";

  // Interface language (ADR 0008). English is the page itself; other
  // languages come from /i18n/<tag>.json on this same domain. The choice
  // follows the browser's languages and can be changed with the picker;
  // it is remembered only in this browser.
  const LANGUAGES = [
    ["en", "English"],
    ["tr", "Türkçe"]
  ];
  const RTL = ["ar", "fa", "he", "ur", "ps", "yi"];
  const PAGE = document.body.dataset.page || "home";

  const nodes = Array.from(document.querySelectorAll("[data-i18n]"));
  const descTag = document.querySelector('meta[name="description"]');
  const EN = {};
  nodes.forEach((el) => { EN[el.dataset.i18n] = el.textContent; });
  EN["meta." + PAGE + ".title"] = document.title;
  if (descTag) EN["meta." + PAGE + ".desc"] = descTag.getAttribute("content");
  EN["dl.copied"] = "Copied";
  EN["r.copied"] = "Copied";
  let dict = EN;
  const t = (key) => (typeof dict[key] === "string" ? dict[key] : EN[key]);

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
    for (const pref of prefs) {
      if (!pref) continue;
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

  function apply(tag, d) {
    dict = d;
    nodes.forEach((el) => {
      const text = t(el.dataset.i18n);
      if (typeof text === "string") el.textContent = text;
    });
    document.documentElement.lang = tag;
    document.documentElement.dir = RTL.includes(tag.split("-")[0]) ? "rtl" : "ltr";
    document.title = t("meta." + PAGE + ".title");
    if (descTag) descTag.setAttribute("content", t("meta." + PAGE + ".desc"));
    if (picker) picker.value = tag;
  }

  async function show(tag) {
    if (tag === "en") { apply("en", EN); return; }
    try {
      const res = await fetch("/i18n/" + tag + ".json", { credentials: "omit" });
      if (!res.ok) throw new Error(String(res.status));
      const d = await res.json();
      if (document.startViewTransition) document.startViewTransition(() => apply(tag, d));
      else apply(tag, d);
    } catch (e) {
      apply("en", EN);
    }
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
    picker.addEventListener("change", () => { store.set(picker.value); show(picker.value); });
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
      if (cli && ["macos", "windows"].includes(p)) mine.after(cli);
    }
  }
  // Receive link (totaldrop.app/r#7-harbor-rocket): the code is after "#",
  // so the browser never sends it to the server, and this page sends it
  // nowhere either. It only hands it to the app.
  if (PAGE === "receive") {
    window.addEventListener("hashchange", () => location.reload());
    // Fold the way the apps do: Turkish İ/ı and Arabic-Indic or Persian
    // digits must not make a valid code look invalid.
    const raw = decodeURIComponent(location.hash.slice(1)).trim()
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
})();
