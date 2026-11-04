# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Outlines "TotalDrop" in Bricolage Grotesque 800 for the logo files
# (site/tools/brand-files.mjs, WORDMARK). Run once when the wordmark changes:
#
#   pip install fonttools brotli uharfbuzz
#   python3 site/tools/wordmark.py site/public/fonts/bricolage-grotesque-latin-800-normal.woff2
#
# It uses the site's own font file, shapes the name with HarfBuzz (so the
# kerning is the font's) and tracks it by -0.035em, as .brand on the site.
# Prints the path in font units (1000 per em, baseline at y = 0, y down) and,
# on stderr, the units per em, the advance width and the cap height.
# The OFL allows outlines of the font in a logo; they are not the font.
import sys, io, re
import uharfbuzz as hb
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
path = sys.argv[1]
font = TTFont(path)
data = io.BytesIO(); f2 = TTFont(path); f2.flavor=None; f2.save(data)
blob = hb.Blob(data.getvalue()); face = hb.Face(blob); hbfont = hb.Font(face)
buf = hb.Buffer(); buf.add_str("TotalDrop"); buf.guess_segment_properties()
hb.shape(hbfont, buf, {"kern": True})
upem = font["head"].unitsPerEm
gs = font.getGlyphSet(); order = font.getGlyphOrder()
track = -0.035 * upem
x = 0; parts = []
for info, pos in zip(buf.glyph_infos, buf.glyph_positions):
    name = order[info.codepoint]
    pen = SVGPathPen(gs)
    gs[name].draw(TransformPen(pen, (1, 0, 0, -1, x + pos.x_offset, 0)))
    parts.append(pen.getCommands())
    x += pos.x_advance + track
x -= track
os2 = font["OS/2"]
print(upem, round(x), os2.sCapHeight, file=sys.stderr)
print(re.sub(r"-?\d+\.\d+", lambda m: str(round(float(m.group()))), " ".join(parts)))
