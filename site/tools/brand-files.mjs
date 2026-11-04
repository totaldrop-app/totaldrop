// The brand page's downloads (site/public/brand/files/), made from the
// mark's geometry (docs/tasarim.md, "İşaret: Köşe") and the wordmark's
// outline, so no file is drawn by hand:
//   node site/tools/brand-files.mjs           check the files are up to date (CI)
//   node site/tools/brand-files.mjs --write   write the SVG files
//   node site/tools/brand-files.mjs --png     also render the PNG files (Playwright)
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const out = join(root, "site", "public", "brand", "files");

// The mark on its 112-unit canvas, and its colours.
export const MARK = "M55 15H97V57A42 42 0 1 1 55 15Z";
const VIOLET = "#6D4AE0", GLOW = "#A68BFF", PAPER = "#F4F2FB", INK = "#22184F";

// "TotalDrop" in Bricolage Grotesque 800, from site/tools/wordmark.py:
// font units, 1000 per em, baseline at y = 0, 4541 units wide, cap height 660.
const WORDMARK = "M208 0V-660H370V0ZM16 -526V-660H564V-526Z M784 14Q704 14 642 -17Q581 -48 546 -110Q511 -172 511 -265Q511 -358 546 -420Q581 -481 643 -512Q705 -542 785 -542Q865 -542 927 -511Q989 -480 1024 -418Q1059 -357 1059 -264Q1059 -169 1023 -107Q987 -45 924 -16Q862 14 784 14ZM789 -103Q825 -103 849 -120Q873 -136 885 -170Q897 -204 897 -254Q897 -307 884 -344Q871 -380 846 -400Q820 -419 780 -419Q746 -419 722 -402Q697 -386 685 -352Q673 -318 673 -267Q673 -185 704 -144Q734 -103 789 -103Z M1324 13Q1231 13 1187 -36Q1143 -86 1143 -195V-402H1068L1071 -527H1125Q1157 -528 1174 -537Q1190 -546 1194 -571L1207 -645H1298V-528H1425V-397H1298V-202Q1298 -168 1314 -152Q1331 -136 1365 -136Q1384 -136 1401 -140Q1418 -144 1429 -151V-2Q1397 8 1370 10Q1344 13 1324 13Z M1613 14Q1569 14 1535 -4Q1501 -21 1482 -54Q1463 -88 1463 -136Q1463 -181 1480 -210Q1498 -240 1532 -258Q1565 -275 1612 -286Q1659 -298 1719 -307Q1748 -312 1768 -316Q1787 -321 1797 -331Q1807 -341 1807 -360Q1807 -385 1789 -403Q1771 -421 1731 -421Q1703 -421 1680 -411Q1658 -401 1642 -382Q1626 -363 1618 -336L1476 -378Q1489 -420 1512 -451Q1536 -482 1569 -502Q1602 -523 1644 -532Q1687 -542 1735 -542Q1815 -542 1866 -516Q1916 -491 1940 -438Q1965 -384 1965 -300V-219Q1965 -183 1966 -146Q1968 -110 1970 -74Q1973 -37 1977 0H1835Q1831 -23 1827 -56Q1823 -88 1821 -122H1802Q1788 -84 1762 -52Q1735 -21 1698 -4Q1660 14 1613 14ZM1686 -100Q1704 -100 1722 -106Q1741 -113 1758 -124Q1775 -136 1789 -153Q1803 -170 1810 -190L1808 -269L1830 -264Q1811 -252 1788 -245Q1765 -238 1742 -234Q1718 -230 1696 -226Q1673 -221 1656 -214Q1638 -207 1628 -195Q1618 -183 1618 -162Q1618 -134 1637 -117Q1656 -100 1686 -100Z M2049 0V-720H2207V0Z M2379 0V-130H2570Q2622 -130 2657 -152Q2692 -174 2710 -218Q2727 -262 2727 -328Q2727 -381 2716 -419Q2705 -457 2684 -482Q2662 -506 2628 -518Q2595 -530 2550 -530H2379V-660H2544Q2667 -660 2745 -621Q2823 -582 2859 -510Q2895 -437 2895 -336Q2895 -258 2876 -201Q2858 -144 2826 -106Q2794 -67 2752 -44Q2711 -20 2666 -10Q2620 0 2575 0ZM2304 0V-660H2465V0Z M2967 0V-245V-528H3099L3102 -349H3122Q3129 -419 3148 -460Q3168 -502 3204 -521Q3241 -540 3295 -540Q3303 -540 3313 -540Q3323 -539 3336 -537L3330 -365Q3314 -373 3296 -376Q3277 -378 3263 -378Q3224 -378 3196 -363Q3167 -348 3150 -317Q3134 -286 3129 -239V0Z M3621 14Q3541 14 3480 -17Q3418 -48 3383 -110Q3348 -172 3348 -265Q3348 -358 3383 -420Q3418 -481 3480 -512Q3542 -542 3622 -542Q3702 -542 3764 -511Q3826 -480 3861 -418Q3896 -357 3896 -264Q3896 -169 3860 -107Q3824 -45 3762 -16Q3699 14 3621 14ZM3626 -103Q3662 -103 3686 -120Q3710 -136 3722 -170Q3734 -204 3734 -254Q3734 -307 3721 -344Q3708 -380 3682 -400Q3657 -419 3617 -419Q3583 -419 3558 -402Q3534 -386 3522 -352Q3510 -318 3510 -267Q3510 -185 3540 -144Q3571 -103 3626 -103Z M3955 169V-258V-528H4087L4089 -378L4107 -376Q4116 -432 4139 -469Q4162 -506 4198 -524Q4235 -542 4282 -542Q4352 -542 4402 -508Q4453 -474 4480 -411Q4507 -348 4507 -260Q4507 -182 4484 -120Q4460 -58 4412 -22Q4365 14 4292 14Q4243 14 4209 -4Q4175 -21 4152 -55Q4128 -89 4112 -140H4093Q4099 -113 4104 -85Q4109 -57 4112 -31Q4116 -5 4116 19V169ZM4233 -113Q4266 -113 4290 -131Q4313 -149 4326 -182Q4339 -215 4339 -259Q4339 -306 4326 -340Q4312 -373 4287 -392Q4262 -410 4229 -410Q4198 -410 4176 -396Q4155 -383 4142 -362Q4128 -340 4122 -316Q4116 -292 4116 -271V-249Q4116 -230 4121 -211Q4126 -192 4136 -174Q4146 -157 4160 -143Q4174 -129 4192 -121Q4211 -113 4233 -113Z";

const svg = (viewBox, body) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}">${body}</svg>\n`;
const mark = (fill) => svg("0 0 112 112", `<path fill="${fill}" d="${MARK}"/>`);

// The logo: the mark, then the name, as in the site's header (a 2rem mark,
// 0.6rem apart, the name at 1.5rem): in font units the mark's box is 1333
// wide and the name starts 400 after it. The box's centre sits on the
// middle of the capitals.
const logo = (markFill, nameFill) => {
  const box = 1333, gap = 400, k = box / 112, top = -330 - box / 2;
  return svg(`0 ${Math.round(top)} ${box + gap + 4541} ${box}`,
    `<path fill="${markFill}" transform="translate(0 ${top}) scale(${k.toFixed(4)})" d="${MARK}"/>` +
    `<path fill="${nameFill}" transform="translate(${box + gap} 0)" d="${WORDMARK}"/>`);
};

// The app icon, as packaging/linux/app.totaldrop.TotalDrop.svg: the paper
// mark at 0.68 on a violet square.
const icon = svg("0 0 128 128", `<rect x="8" y="8" width="112" height="112" rx="28" fill="${VIOLET}"/><g transform="translate(25.92 25.92) scale(0.68)"><path fill="${PAPER}" d="${MARK}"/></g>`);

export const FILES = {
  "totaldrop-mark.svg": mark(VIOLET),
  "totaldrop-mark-dark.svg": mark(GLOW),
  "totaldrop-mark-black.svg": mark("#000000"),
  "totaldrop-mark-white.svg": mark("#FFFFFF"),
  "totaldrop-logo.svg": logo(VIOLET, INK),
  "totaldrop-logo-dark.svg": logo(GLOW, PAPER),
  "totaldrop-icon.svg": icon,
};
// PNG renders: [svg, png, width].
const PNGS = [
  ["totaldrop-mark.svg", "totaldrop-mark.png", 1024],
  ["totaldrop-mark-dark.svg", "totaldrop-mark-dark.png", 1024],
  ["totaldrop-logo.svg", "totaldrop-logo.png", 2400],
  ["totaldrop-logo-dark.svg", "totaldrop-logo-dark.png", 2400],
  ["totaldrop-icon.svg", "totaldrop-icon.png", 1024],
];

const args = process.argv.slice(2);
const errors = [];
for (const [name, text] of Object.entries(FILES)) {
  const path = join(out, name);
  if (args.includes("--write") || args.includes("--png")) writeFileSync(path, text);
  else if (!existsSync(path) || readFileSync(path, "utf8") !== text) errors.push(`${name} is out of date: run node site/tools/brand-files.mjs --write`);
}
// The Linux app icon is the same drawing.
const linuxIcon = readFileSync(join(root, "packaging", "linux", "app.totaldrop.TotalDrop.svg"), "utf8").trim();
if (linuxIcon !== FILES["totaldrop-icon.svg"].trim()) errors.push("totaldrop-icon.svg differs from packaging/linux/app.totaldrop.TotalDrop.svg");

if (args.includes("--png")) {
  // Playwright from this project or, through NODE_PATH, a global install.
  const { chromium } = createRequire(import.meta.url)("playwright");
  const browser = await chromium.launch();
  const page = await browser.newPage({ deviceScaleFactor: 1 });
  for (const [from, to, width] of PNGS) {
    const text = FILES[from];
    const [, , w, h] = /viewBox="([^"]+)"/.exec(text)[1].split(" ").map(Number);
    const height = Math.round((width * h) / w);
    await page.setViewportSize({ width, height });
    await page.setContent(`<style>html,body{margin:0;background:transparent}svg{display:block;width:${width}px;height:${height}px}</style>${text}`);
    await page.screenshot({ path: join(out, to), omitBackground: true });
  }
  await browser.close();
}

for (const [, png] of PNGS) if (!existsSync(join(out, png))) errors.push(`${png} is missing: run node site/tools/brand-files.mjs --png`);

if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
console.log(`brand files ok: ${Object.keys(FILES).length} SVG, ${PNGS.length} PNG`);
