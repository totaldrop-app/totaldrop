// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// One Pages project serves three host names:
//   totaldrop.app           the site
//   www.totaldrop.app       redirects to totaldrop.app, path kept
//   download.totaldrop.app  its root is the download page; shared assets stay where they are
// The download page in another language is download.totaldrop.app/<tag>/
// (ADR 0021, #513). The pages are files written by site/tools/build.mjs;
// this only picks which file answers on that host name.
const TAGGED_ROOT = /^\/([A-Za-z]{2,3}(?:-[A-Za-z]{2,4})?)\/(?:index\.html)?$/;
export async function onRequest({ request, next, env }) {
  const url = new URL(request.url);
  if (url.hostname === "www.totaldrop.app") {
    url.hostname = "totaldrop.app";
    return Response.redirect(url.toString(), 301);
  }
  if (url.hostname === "download.totaldrop.app" && (url.pathname === "/" || url.pathname === "/index.html")) {
    url.pathname = "/download/";
    return env.ASSETS.fetch(new Request(url, request));
  }
  const tagged = url.hostname === "download.totaldrop.app" ? TAGGED_ROOT.exec(url.pathname) : null;
  if (tagged) {
    // Only a language has a download page under it; anything else that
    // looks like one (a two-letter folder) is answered as before.
    url.pathname = "/" + tagged[1] + "/download/";
    const page = await env.ASSETS.fetch(new Request(url, request));
    if (page.status === 200) return page;
  }
  return next();
}
