"""Draw the pack's brand assets: the mark, the wordmark and the lockup, each as a
transparent PNG in an ink version (for light grounds) and a paper version (for dark).

Run from `assets/`: `uv run --with pillow python ../pack-src/make_brand.py`.

The mark is four primitives on a 2x2 grid, which Montagent can rebuild from `rect`
(with `radius`) and `ellipse` elements. The exact geometry is in `brand/BRAND.md`.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

INK = "#101418"
PAPER = "#F5F0E6"
SIGNAL = "#FF5A36"
SS = 4  # supersampling factor

OUT = Path("brand")
FONT = "fonts/Inter-Bold.ttf"

# Mark geometry, in a 1024 x 1024 box: tiles 460 wide, 40 apart, 32 from the edge.
MARK = 1024
TILE = 460
GAP = 40
EDGE = 32
RADIUS = 96
TILES = [  # (column, row, shape)
    (0, 0, "rect"),
    (1, 0, "circle"),
    (0, 1, "rect"),
    (1, 1, "rect"),
]


def draw_mark(d, ox, oy, size, fg):
    k = size / MARK
    for col, row, shape in TILES:
        x = ox + (EDGE + col * (TILE + GAP)) * k
        y = oy + (EDGE + row * (TILE + GAP)) * k
        box = [x, y, x + TILE * k, y + TILE * k]
        if shape == "circle":
            d.ellipse(box, fill=SIGNAL)
        else:
            d.rounded_rectangle(box, radius=RADIUS * k, fill=fg)


def save(img, name):
    w, h = img.size
    img.resize((w // SS, h // SS), Image.LANCZOS).save(OUT / name, optimize=True)


def mark(fg, name):
    img = Image.new("RGBA", (MARK * SS, MARK * SS), (0, 0, 0, 0))
    draw_mark(ImageDraw.Draw(img), 0, 0, MARK * SS, fg)
    save(img, name)


def text_box(font, text):
    left, top, right, bottom = font.getbbox(text)
    return left, top, right - left, bottom - top


def wordmark(fg, name, cap=360):
    font = ImageFont.truetype(FONT, cap * SS)
    left, top, w, h = text_box(font, "Montagent")
    pad = 40 * SS
    img = Image.new("RGBA", (w + 2 * pad, h + 2 * pad), (0, 0, 0, 0))
    ImageDraw.Draw(img).text((pad - left, pad - top), "Montagent", font=font, fill=fg)
    save(img, name)


def lockup(fg, name, cap=360):
    font = ImageFont.truetype(FONT, cap * SS)
    left, top, w, h = text_box(font, "Montagent")
    mark_size = int(h * 1.6)
    pad = 40 * SS
    space = int(mark_size * 0.28)
    width = pad + mark_size + space + w + pad
    height = pad + mark_size + pad
    img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    draw_mark(d, pad, pad, mark_size, fg)
    d.text((pad + mark_size + space - left, pad + (mark_size - h) / 2 - top), "Montagent", font=font, fill=fg)
    save(img, name)


OUT.mkdir(exist_ok=True)
for fg, suffix in ((INK, ""), (PAPER, "-on-dark")):
    mark(fg, f"mark{suffix}.png")
    wordmark(fg, f"wordmark{suffix}.png")
    lockup(fg, f"lockup{suffix}.png")
