# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later
"""Reads a `uiautomator dump` file for check.sh.

  ui.py find FILE TEXT   prints "x y", the centre of the node that shows
                         TEXT (text or content-desc); exits 1 if none does
  ui.py texts FILE       prints every text and content-desc, one per line
"""

import re
import sys
import xml.etree.ElementTree as ET


def nodes(path):
    try:
        return list(ET.parse(path).getroot().iter("node"))
    except (ET.ParseError, OSError):
        return []


def box(node):
    x1, y1, x2, y2 = map(int, re.findall(r"-?\d+", node.get("bounds", "")))
    return x1, y1, x2, y2


def labels(node):
    return [node.get("text") or "", node.get("content-desc") or ""]


def main():
    cmd, path = sys.argv[1], sys.argv[2]
    found = nodes(path)
    if cmd == "texts":
        for n in found:
            for attr in ("text", "content-desc"):
                v = n.get(attr)
                if v:
                    print(f"{attr}: {v!r}")
        return 0
    want = sys.argv[3]
    exact = [n for n in found if want in labels(n)]
    line = [n for n in found if any(want in v.split("\n") for v in labels(n))]
    loose = [n for n in found if any(want in v for v in labels(n))]
    for group in (exact, line, loose):
        if not group:
            continue

        def rank(n):
            x1, y1, x2, y2 = box(n)
            return (n.get("clickable") != "true", (x2 - x1) * (y2 - y1))

        x1, y1, x2, y2 = box(sorted(group, key=rank)[0])
        print((x1 + x2) // 2, (y1 + y2) // 2)
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
