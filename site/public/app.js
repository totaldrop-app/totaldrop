// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

(() => {
  "use strict";

  // Interface language (ADR 0008, ADR 0021). The choice follows the
  // browser's languages and can be changed with the picker; it is
  // remembered only in this browser.
  // Most pages have an address in every language (/tr/, /ar/roadmap/),
  // written by site/tools/build.mjs: there the address decides the language
  // and the text is already in the page. /r and the 404 page keep a single
  // address: English is the page itself and the other languages come from
  // /i18n/<tag>.json on this same domain.
  // The languages, which are reviewed, and how one is chosen: lang.js.
  // If lang.js did not load, the page stays in English and the rest
  // (downloads, copy buttons, /r) still works.
  const { LANGUAGES, REVIEWED, negotiate } = window.TDLang
    || { LANGUAGES: [["en", "English"]], REVIEWED: ["en"], negotiate: () => "en" };
  // Right to left: languages whose likely script is one of these (the
  // same rule as msg.js).
  const RTL_SCRIPTS = ["Arab", "Hebr", "Thaa", "Syrc", "Nkoo", "Adlm", "Rohg", "Mand", "Samr"];
  const isRtl = (tag) => {
    try { return RTL_SCRIPTS.includes(new Intl.Locale(tag).maximize().script); } catch (e) { return false; }
  };
  const TAGS = LANGUAGES.map((l) => l[0]);
  const isTag = (v) => Boolean(v && window.TDLang && TAGS.includes(v));
  const PAGE = document.body.dataset.page || "home";
  const store = {
    get() { try { return localStorage.getItem("td-lang"); } catch (e) { return null; } },
    set(v) { try { localStorage.setItem("td-lang", v); return true; } catch (e) { return false; } }
  };
  const prefs = navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language];
  // Where this page is among its languages, or null on /r and the 404 page
  // (and when lang.js did not load: the page then stays as it was sent).
  const HERE = window.TDLang && window.TDLang.ADDRESSED.includes(PAGE) ? window.TDLang.locate(location.pathname) : null;
  const currentHost = location.hostname
    || (PAGE === "download" && !(HERE && HERE.rest === "/download/") ? "download.totaldrop.app" : "totaldrop.app");

  const urlParams = new URL(location.search || "", "https://totaldrop.app").searchParams;
  const rawQueryLang = urlParams.get("lang");
  const queryLang = isTag(rawQueryLang) ? rawQueryLang : null;
  const storedLang = store.get();
  let saved = isTag(storedLang) ? storedLang : null;
  let keptStorage = saved !== null;
  if (queryLang) {
    keptStorage = store.set(queryLang);
    if (keptStorage) {
      saved = queryLang;
      urlParams.delete("lang");
    }
  }
  const cleanQs = urlParams.toString();
  const effectiveSearch = queryLang && keptStorage ? (cleanQs ? "?" + cleanQs : "") : location.search;
  const chosen = saved || queryLang;

  // A visitor on an English address whose language is another one, and who
  // has not chosen English, goes to that language's address. The part
  // after "#" goes along and is never sent to the server.
  if (HERE) {
    const to = window.TDLang.redirect(location.pathname, chosen, prefs);
    if (to) { location.replace(to + effectiveSearch + location.hash); return; }
    if (queryLang && keptStorage && window.history && typeof window.history.replaceState === "function") {
      window.history.replaceState(null, "", location.pathname + effectiveSearch + location.hash);
    }
  } else if (queryLang && keptStorage && window.history && typeof window.history.replaceState === "function") {
    window.history.replaceState(null, "", location.pathname + effectiveSearch + location.hash);
  }

  // Carry an explicit language choice across the two host names (localStorage
  // is per origin, ADR 0021), and across same-host pages too when storage is
  // unavailable. Captured once so show() can re-sync links on /r and 404.html.
  const candidateLinks = [];
  if (window.TDLang) {
    const paths = window.TDLang.ADDRESSED.map((p) => (p === "home" ? "/" : "/" + p + "/"));
    document.querySelectorAll("a[href]").forEach((a) => {
      if (a.hasAttribute("hreflang")) return;
      const href = a.getAttribute("href");
      const m = /^(https:\/\/(?:download\.)?totaldrop\.app)?(\/[^#?]*)?(#.*)?$/.exec(href);
      if (!m || (m[1] === undefined && m[2] === undefined)) return;
      const linkHost = m[1] ? m[1].slice("https://".length) : currentHost;
      const loc = window.TDLang.locate(m[2] || "/");
      const addressed = linkHost === "download.totaldrop.app" ? loc.rest === "/" : paths.includes(loc.rest);
      if (!addressed) return;
      candidateLinks.push({
        el: a,
        orig: href,
        prefix: m[1] || "",
        rest: loc.rest,
        hash: m[3] || "",
        crossHost: linkHost !== currentHost,
      });
    });
  }
  function syncChosenLinks(lang, storageOk) {
    candidateLinks.forEach((item) => {
      if (!isTag(lang)) {
        item.el.setAttribute("href", item.orig);
      } else if (item.crossHost || (!storageOk && (!HERE || lang === "en"))) {
        item.el.setAttribute("href", item.prefix + window.TDLang.address(lang, item.rest) + "?lang=" + lang + item.hash);
      } else {
        item.el.setAttribute("href", HERE ? item.orig : item.prefix + window.TDLang.address(lang, item.rest) + item.hash);
      }
    });
  }
  if (chosen && (!HERE || HERE.tag === chosen)) syncChosenLinks(chosen, keptStorage);

  const nodes = Array.from(document.querySelectorAll("[data-i18n]"));
  const descTag = document.querySelector('meta[name="description"]');
  const EN = {};
  nodes.forEach((el) => { EN[el.dataset.i18n] = el.textContent; });
  // Accessible names (aria-label) translate the same way, by key.
  const labelled = Array.from(document.querySelectorAll("[data-i18n-label][aria-label]:not(#lang-select)"));
  labelled.forEach((el) => { EN[el.dataset.i18nLabel] = el.getAttribute("aria-label"); });
  EN["meta." + PAGE + ".title"] = document.title;
  if (descTag) EN["meta." + PAGE + ".desc"] = descTag.getAttribute("content");
  // On a translated page the build wrote this in the page's language.
  EN["dl.copied"] = document.body.dataset.copied || "Copied";
  EN["r.copied"] = "Copied";
  EN["lang.label"] = "Language";
  EN["lang.draft"] = "This translation is a draft and is being reviewed.";
  let dict = EN;
  // An empty or missing translation falls back to English.
  const has = (d, key) => typeof d[key] === "string" && d[key] !== "";
  const t = (key) => (has(dict, key) ? dict[key] : EN[key]);

  const picker = document.getElementById("lang-select");
  const jump = document.querySelector(".lang-jump");
  // The draft note opens the main content, so it is read (and found by
  // landmark navigation) before the translated text. A page with its own
  // address already carries it.
  let note = null;
  const main = document.getElementById("main");
  if (main && !HERE) {
    note = document.createElement("p");
    note.className = "lang-note";
    note.setAttribute("role", "note");
    note.hidden = true;
    main.prepend(note);
  }

  // The privacy policy has a page in every language (/privacy/<tag>, #207);
  // its links follow the language shown.
  const privacyLinks = Array.from(document.querySelectorAll('a[href="/privacy"], a[href="https://totaldrop.app/privacy"]'))
    .map((a) => [a, a.getAttribute("href")]);

  function apply(tag, d) {
    dict = d;
    privacyLinks.forEach(([a, base]) => {
      if (tag === "en") { a.setAttribute("href", base); a.removeAttribute("hreflang"); } else { a.setAttribute("href", base + "/" + tag); a.hreflang = tag; }
    });
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
      // In English only when this language lacks the note.
      if (note.hidden || has(d, "lang.draft")) { note.removeAttribute("lang"); note.removeAttribute("dir"); } else { note.lang = "en"; note.dir = "ltr"; }
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
    if (save) {
      const savedOk = store.set(tag);
      syncChosenLinks(tag, savedOk);
    }
    if (animate && !reduced && document.startViewTransition) document.startViewTransition(() => apply(tag, d));
    else apply(tag, d);
  }

  if (picker && window.TDLang) {
    LANGUAGES.forEach(([tag, name]) => {
      const o = document.createElement("option");
      o.value = tag;
      o.lang = tag;
      o.textContent = name;
      picker.append(o);
    });
    if (jump) jump.hidden = true;
    picker.hidden = false;
    picker.addEventListener("change", () => {
      if (!HERE) return show(picker.value, { save: true, animate: true });
      // The same page, and the same part of it, in the chosen language.
      // When storage throws, ?lang=en carries an English choice to the next page.
      const kept = store.set(picker.value);
      const params = new URL(location.search || "", "https://totaldrop.app").searchParams;
      if (!kept && picker.value === "en") params.set("lang", "en");
      else params.delete("lang");
      const qs = params.toString();
      location.assign(window.TDLang.address(picker.value, HERE.rest) + (qs ? "?" + qs : "") + location.hash);
    });
  }

  if (window.TDLang) {
    const baseOrigin = location.origin || ("https://" + currentHost);
    document.querySelectorAll("#languages a[hreflang]").forEach((a) => {
      a.addEventListener("click", (e) => {
        if (e && (e.defaultPrevented || (typeof e.button === "number" && e.button !== 0) || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey)) return;
        const tag = a.getAttribute("hreflang");
        if (!isTag(tag)) return;
        const kept = store.set(tag);
        const rawHref = a.getAttribute("href");
        const target = new URL(rawHref, baseOrigin);
        const crossHost = target.hostname !== currentHost;
        if ((!kept && tag === "en") || crossHost) {
          target.searchParams.set("lang", tag);
          if (e && typeof e.preventDefault === "function") e.preventDefault();
          location.assign(rawHref.startsWith("https://") ? target.toString() : target.pathname + target.search + target.hash);
        }
      });
    });
  }

  if (HERE) {
    // Also when the browser brings this page back from its history.
    const here = () => { if (picker) picker.value = HERE.tag; };
    here();
    window.addEventListener("pageshow", here);
  } else {
    const pathTag = window.TDLang ? window.TDLang.locate(location.pathname).tag : "en";
    const initial = pathTag !== "en" ? pathTag : (LANGUAGES.some((l) => l[0] === chosen) ? chosen : negotiate(prefs));
    if (picker) picker.value = initial;
    if (initial !== "en") show(initial);
  }

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
})();
