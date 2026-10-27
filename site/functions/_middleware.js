// One Pages project serves three host names:
//   totaldrop.app           the site
//   www.totaldrop.app       redirects to totaldrop.app, path kept
//   download.totaldrop.app  its root is the download page; shared assets stay where they are
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
  return next();
}
