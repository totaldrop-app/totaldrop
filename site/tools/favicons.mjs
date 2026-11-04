// The site's icons, drawn from the mark (docs/tasarim.md, "İşaret") (#227):
//   node site/tools/favicons.mjs           check (CI): files in shape, every page declares them
//   NODE_PATH=$(npm root -g) node site/tools/favicons.mjs --write
//                                          render the PNG and ICO files (needs Playwright)
//
// Search engines and older browsers do not all read favicon.svg. Google
// shows a site icon only for a square image whose side is a multiple of
// 48 px (SVG has no size rule) that the home page declares with
// rel="icon"; many crawlers and tools also ask for /favicon.ico without
// reading the page. So the site serves, beside favicon.svg:
//   favicon.ico           16, 32 and 48 px, the violet mark on transparent
//   favicon-192.png       192 px (4 × 48), the same
//   apple-touch-icon.png  180 px, the app icon: paper mark on violet, opaque
//                         (iOS paints transparency black and rounds the corners itself)
// The files are committed; the check fails when one is missing, has the
// wrong size, or a page lacks the <link> lines below.
import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const site = join(dirname(fileURLToPath(import.meta.url)), "..", "public");
const MARK = "M55 15H97V57A42 42 0 1 1 55 15Z";
const VIOLET = "#6D4AE0";
const PAPER = "#F4F2FB";

// The mark alone, as in favicon.svg's light colour.
const markSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 112 112"><path fill="${VIOLET}" d="${MARK}"/></svg>`;
// The app icon (render-icons.sh "legacy" layout, without its rounded corners).
const tileSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 112 112"><rect width="112" height="112" fill="${VIOLET}"/><g transform="translate(17.92 17.92) scale(0.68)"><path fill="${PAPER}" d="${MARK}"/></g></svg>`;

export const LINKS = [
  '<link rel="icon" href="/favicon.ico" sizes="48x48">',
  '<link rel="icon" href="/favicon-192.png" type="image/png" sizes="192x192">',
  '<link rel="icon" href="/favicon.svg" type="image/svg+xml">',
  '<link rel="apple-touch-icon" href="/apple-touch-icon.png">',
];

const ICO_SIZES = [16, 32, 48];

// Width and height from a PNG's IHDR chunk, or null when it is not a PNG.
export function pngSize(buf) {
  const sig = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (buf.length < 24 || !sig.every((b, i) => buf[i] === b)) return null;
  if (buf.toString("latin1", 12, 16) !== "IHDR") return null;
  return { width: buf.readUInt32BE(16), height: buf.readUInt32BE(20) };
}

// An ICO file whose entries are PNG images (Windows Vista and later, every
// current browser).
export function buildIco(pngs) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(pngs.length, 4);
  const dir = Buffer.alloc(16 * pngs.length);
  let offset = header.length + dir.length;
  pngs.forEach((png, i) => {
    const size = pngSize(png);
    if (!size || size.width !== size.height || size.width > 256) throw new Error(`entry ${i}: not a square PNG up to 256 px`);
    const e = i * 16;
    dir.writeUInt8(size.width === 256 ? 0 : size.width, e);
    dir.writeUInt8(size.height === 256 ? 0 : size.height, e + 1);
    dir.writeUInt8(0, e + 2); // no palette
    dir.writeUInt8(0, e + 3);
    dir.writeUInt16LE(1, e + 4); // colour planes
    dir.writeUInt16LE(32, e + 6); // bits per pixel
    dir.writeUInt32LE(png.length, e + 8);
    dir.writeUInt32LE(offset, e + 12);
    offset += png.length;
  });
  return Buffer.concat([header, dir, ...pngs]);
}

// The square sizes of an ICO's entries; throws when the file is malformed.
export function icoSizes(buf) {
  if (buf.length < 6 || buf.readUInt16LE(0) !== 0 || buf.readUInt16LE(2) !== 1) throw new Error("not an ICO file");
  const count = buf.readUInt16LE(4);
  if (count === 0 || buf.length < 6 + 16 * count) throw new Error("ICO directory truncated");
  const sizes = [];
  for (let i = 0; i < count; i++) {
    const e = 6 + i * 16;
    const len = buf.readUInt32LE(e + 8);
    const off = buf.readUInt32LE(e + 12);
    if (off < 6 + 16 * count || off + len > buf.length) throw new Error(`ICO entry ${i} outside the file`);
    const png = pngSize(buf.subarray(off, off + len));
    if (!png) throw new Error(`ICO entry ${i} is not a PNG`);
    const w = buf[e] || 256;
    const h = buf[e + 1] || 256;
    if (png.width !== w || png.height !== h) throw new Error(`ICO entry ${i}: directory says ${w}×${h}, image is ${png.width}×${png.height}`);
    if (w !== h) throw new Error(`ICO entry ${i} is not square`);
    sizes.push(w);
  }
  return sizes;
}

function pages(dir = site) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...pages(p));
    else if (name.endsWith(".html")) out.push(p);
  }
  return out.sort();
}

function check() {
  const problems = [];
  const want = (file, side) => {
    let buf;
    try { buf = readFileSync(join(site, file)); } catch { problems.push(`${file} missing; run with --write`); return; }
    const size = pngSize(buf);
    if (!size) problems.push(`${file} is not a PNG`);
    else if (size.width !== side || size.height !== side) problems.push(`${file} is ${size.width}×${size.height}, expected ${side}×${side}`);
  };
  want("favicon-192.png", 192);
  want("apple-touch-icon.png", 180);
  try {
    const sizes = icoSizes(readFileSync(join(site, "favicon.ico")));
    if (sizes.join() !== ICO_SIZES.join()) problems.push(`favicon.ico has ${sizes.join(", ")} px, expected ${ICO_SIZES.join(", ")}`);
  } catch (e) {
    problems.push(`favicon.ico: ${e.code === "ENOENT" ? "missing; run with --write" : e.message}`);
  }
  for (const page of pages()) {
    const html = readFileSync(page, "utf8");
    const head = html.slice(0, html.indexOf("</head>"));
    for (const link of LINKS) if (!head.includes(link)) problems.push(`${relative(site, page)}: <head> lacks ${link}`);
  }
  if (problems.length) {
    console.error(problems.join("\n"));
    process.exit(1);
  }
  console.log(`icons in shape; ${pages().length} pages declare them`);
}

async function write() {
  // CommonJS resolution, so NODE_PATH finds a global install.
  const { chromium } = createRequire(import.meta.url)("playwright");
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({ deviceScaleFactor: 1 });
    const render = async (svg, side) => {
      await page.setViewportSize({ width: side, height: side });
      await page.setContent(`<!doctype html><style>html,body{margin:0;background:transparent}svg{display:block;width:${side}px;height:${side}px}</style>${svg}`);
      return page.screenshot({ omitBackground: true, clip: { x: 0, y: 0, width: side, height: side } });
    };
    const icoPngs = [];
    for (const side of ICO_SIZES) icoPngs.push(await render(markSvg, side));
    writeFileSync(join(site, "favicon.ico"), buildIco(icoPngs));
    writeFileSync(join(site, "favicon-192.png"), await render(markSvg, 192));
    writeFileSync(join(site, "apple-touch-icon.png"), await render(tileSvg, 180));
  } finally {
    await browser.close();
  }
  console.log("wrote favicon.ico, favicon-192.png, apple-touch-icon.png");
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  if (process.argv.includes("--write")) await write();
  else check();
}
