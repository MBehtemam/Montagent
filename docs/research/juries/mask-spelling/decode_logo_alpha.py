#!/usr/bin/env python3
"""Re-derive the badge's alpha geometry from the committed PNG.

    python3 docs/research/juries/mask-spelling/decode_logo_alpha.py

Exits non-zero, naming the defect, when it stops reproducing.

ADR-0068 rests on one measured claim: `brand/logo-en.png` already *is* the
inscribed circle, so migrating `handle-logo`'s bare `mask: "circle"` key to an
`effects` member changes no pixel. That claim is what makes the migration safe to
land in the same change as the decision, on the one artifact #168 calls the only
test capable of falsifying the format.

Decodes the PNG with zlib + the PNG filter algorithms only -- no third-party
imaging library, so this runs on a bare interpreter the way the other committed
scans do.
"""

import os
import struct
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
PNG = os.path.join(
    HERE, "..", "..", "..", "..",
    "fixtures", "en-halloween-decorating", "brand", "logo-en.png",
)


def decode_rgba(path):
    raw = open(path, "rb").read()
    assert raw[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"

    pos, idat, width, height, depth, colour = 8, b"", None, None, None, None
    while pos < len(raw):
        (length,) = struct.unpack(">I", raw[pos:pos + 4])
        kind = raw[pos + 4:pos + 8]
        body = raw[pos + 8:pos + 8 + length]
        if kind == b"IHDR":
            width, height, depth, colour = struct.unpack(">IIBB", body[:10])
        elif kind == b"IDAT":
            idat += body
        elif kind == b"IEND":
            break
        pos += 12 + length

    assert depth == 8, f"expected 8-bit channels, got {depth}"
    assert colour == 6, f"expected RGBA (colour type 6), got {colour}"

    data = zlib.decompress(idat)
    stride = width * 4
    out, prev, pos = [], bytearray(stride), 0
    for _ in range(height):
        ftype = data[pos]
        line = bytearray(data[pos + 1:pos + 1 + stride])
        pos += 1 + stride
        for i in range(stride):
            a = line[i - 4] if i >= 4 else 0
            b = prev[i]
            c = prev[i - 4] if i >= 4 else 0
            if ftype == 1:
                line[i] = (line[i] + a) & 0xFF
            elif ftype == 2:
                line[i] = (line[i] + b) & 0xFF
            elif ftype == 3:
                line[i] = (line[i] + (a + b) // 2) & 0xFF
            elif ftype == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        out.append(bytes(line))
        prev = line
    return width, height, out


def main():
    w, h, rows = decode_rgba(PNG)
    print(f"{os.path.relpath(PNG, os.path.join(HERE, '..', '..', '..', '..'))}: "
          f"{w}x{h} RGBA")

    problems = []
    if (w, h) != (800, 800):
        problems.append(f"expected 800x800, got {w}x{h}")

    def alpha(x, y):
        return rows[y][x * 4 + 3]

    corners = {
        "top-left": alpha(0, 0),
        "top-right": alpha(w - 1, 0),
        "bottom-left": alpha(0, h - 1),
        "bottom-right": alpha(w - 1, h - 1),
    }
    for name, a in corners.items():
        if a != 0:
            problems.append(f"{name} corner alpha is {a}, expected 0")

    centre = alpha(w // 2, h // 2)
    if centre != 255:
        problems.append(f"centre alpha is {centre}, expected 255")

    # The claim: opacity tracks the inscribed circle. Sample on a 4px grid and
    # allow a 2px annulus either side of the boundary for antialiasing.
    cx, cy, r = (w - 1) / 2.0, (h - 1) / 2.0, min(w, h) / 2.0
    opaque_outside = transparent_inside = sampled = 0
    for y in range(0, h, 4):
        for x in range(0, w, 4):
            d = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5
            if abs(d - r) <= 2.0:
                continue
            sampled += 1
            a = alpha(x, y)
            if d > r and a != 0:
                opaque_outside += 1
            elif d < r and a == 0:
                transparent_inside += 1

    print(f"  corners: {corners}")
    print(f"  centre alpha: {centre}")
    print(f"  sampled {sampled} px on a 4px grid, excluding a 2px boundary annulus")
    print(f"  opaque outside the inscribed circle: {opaque_outside}")
    print(f"  transparent inside the inscribed circle: {transparent_inside}")

    if opaque_outside:
        problems.append(f"{opaque_outside} opaque px outside the inscribed circle")
    if transparent_inside:
        problems.append(f"{transparent_inside} transparent px inside the inscribed circle")

    if problems:
        print("\nFAILED — the stored badge is no longer the inscribed circle:")
        for p in problems:
            print(f"  - {p}")
        return 1

    print("\nOK — the asset is already the inscribed circle, so a circle mask over the\n"
          "element's square rect selects every pixel it already shows.\n"
          "\n"
          "This measures the *stored* 800x800 asset, and that is all it measures. The\n"
          "renderer minifies it 11.8x into a 68x68 slot, which carries the badge a\n"
          "little past the circle the stored one keeps to, and the mask trims that rim:\n"
          "ADR-0075 retires ADR-0068's 'the rendered frame is unchanged' on the strength\n"
          "of it. The frame-level measurement lives in the test suite, where a renderer\n"
          "is available -- montaget-core, tests/effects.rs.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
