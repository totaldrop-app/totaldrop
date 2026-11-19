// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// One Pages project serves three host names:
//   totaldrop.app           the site
//   www.totaldrop.app       redirects to totaldrop.app, path kept
//   download.totaldrop.app  its root is the download page; shared assets stay where they are
// The download page in another language is download.totaldrop.app/<tag>/
// (ADR 0021, #513). /en/... (including download.totaldrop.app/en/, #554) and
// non-canonical case or slash variants of supported language tags redirect
// with 301 to their canonical address.
export const TAGS = [
  "ar", "bn", "cs", "da", "de", "el", "en", "es", "fa", "fi", "fr", "he",
  "hi", "hu", "id", "it", "ja", "ko", "ms", "nb", "nl", "pl", "pt-BR",
  "pt-PT", "ro", "ru", "sv", "sw", "th", "tr", "uk", "ur", "vi",
  "zh-Hans", "zh-Hant",
];
export const SUBPAGES = new Set(["download", "roadmap", "donate", "team", "brand"]);

const CANONICAL_TAG = new Map(TAGS.map((t) => [t.toLowerCase(), t]));
const DOWNLOAD_TAGGED = /^\/([^/]+)(?:\/(index\.html)?)?$/;
const SITE_TAGGED = /^\/([^/]+)(?:\/(?:(index\.html)|([^/]+)(?:\/(index\.html)?)?)?)?$/;

function canonicalSitePath(pathname) {
  const m = SITE_TAGGED.exec(pathname);
  if (!m) return null;
  const canonical = CANONICAL_TAG.get(m[1].toLowerCase());
  if (!canonical) return null;
  const sub = m[3];
  if (sub !== undefined && !SUBPAGES.has(sub)) return null;
  const target = canonical === "en"
    ? (sub ? `/${sub}/` : "/")
    : (sub ? `/${canonical}/${sub}/` : `/${canonical}/`);
  return pathname === target ? null : target;
}

export async function onRequest({ request, next, env }) {
  const url = new URL(request.url);
  const wasWww = url.hostname === "www.totaldrop.app";
  if (wasWww) url.hostname = "totaldrop.app";

  if (url.hostname === "download.totaldrop.app") {
    if (url.pathname === "/" || url.pathname === "/index.html") {
      url.pathname = "/download/";
      return env.ASSETS.fetch(new Request(url, request));
    }
    const dm = DOWNLOAD_TAGGED.exec(url.pathname);
    if (dm) {
      const canonical = CANONICAL_TAG.get(dm[1].toLowerCase());
      if (canonical === "en") {
        url.pathname = "/";
        return Response.redirect(url.toString(), 301);
      }
      if (canonical) {
        const canonicalRoot = `/${canonical}/`;
        if (url.pathname !== canonicalRoot && url.pathname !== `${canonicalRoot}index.html`) {
          url.pathname = canonicalRoot;
          return Response.redirect(url.toString(), 301);
        }
        // Only a language has a download page under it; anything else that
        // looks like one (a two-letter folder) is answered as before. Every
        // other answer is the page's own, also "not modified" to a browser
        // that asks again: falling through then would send the home page.
        url.pathname = `/${canonical}/download/`;
        const page = await env.ASSETS.fetch(new Request(url, request));
        if (page.status !== 404) return page;
      }
    }
    return next();
  }

  const targetPath = canonicalSitePath(url.pathname);
  if (targetPath !== null) {
    url.pathname = targetPath;
    return Response.redirect(url.toString(), 301);
  }
  if (wasWww) return Response.redirect(url.toString(), 301);
  return next();
}

