# Montagent format reference — image elements, sizing

*(Published format documentation. This is what the schema enforces.)*

An image element is placed by `x`, `y`, `origin` and a declared size `width` x `height`,
all absolute integer pixels. Example:

```json
{"id":"photo-06","type":"image","start":17472,"end":30603,"source":"images/06.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1912,
 "fit":"cover","clip":[0,0,1080,1300]}
```

## `clip` — the aperture

`clip` is `[x, y, w, h]`, a static rect in frame space. Only the part of the element inside
it is painted. It does not move or scale with the element.

## `width` / `height` — the drawn rect

Required. **At render the source is resampled to exactly this rect.** Whatever integers you
write are what is drawn.

## `fit` — how you derived `width`/`height`

Required on every element carrying a raster source. `fit` records the rule you used to
compute the drawn rect from the source's pixel dimensions and the aperture. It does not
change what the renderer draws.

The box the rule works against is **`clip`'s w and h**.

Closed value set — no other value is legal:

| value | meaning | inequality |
|---|---|---|
| `cover` | drawn rect covers the box; overflow is clipped | `width >= box_w` and `height >= box_h` |
| `contain` | drawn rect fits inside the box | `width <= box_w` and `height <= box_h` |
| `literal` | no rule; the integers are yours | none |

### The arithmetic, for `cover` and `contain`

Source `sw x sh` into box `bw x bh`. Use **integer arithmetic only** — never floats.

- `cover`: width drives when `bw*sh >= bh*sw`. `contain`: width drives when `bw*sh <= bh*sw`.
- The **driving axis takes the box dimension verbatim**.
- The **slack axis** is `(s_slack * b_driving) // s_driving` — integer division, floor.

`cover` and `contain` require `clip`. Under `literal` the box is not used and `clip` is optional.

Source dimensions are the image's **orientation-applied** pixel dimensions — if a JPEG
carries an EXIF orientation flag that transposes the image, use the transposed dimensions.

## What `validate` reports

- **error** — the declared rect disagrees with the rule named by `fit`. The driving axis must
  equal the box dimension exactly; the slack axis must be `floor` or `ceil` of the exact value.
  Not reported under `literal`.
- **error** — aperture coverage: under `cover` the drawn rect must contain `clip`; under
  `contain` `clip` must contain the drawn rect.
- **UNCHECKED** — the source could not be read.

## Not fields

There is no `gravity` field, no `align` on images, no `crop`, and no `box`.
