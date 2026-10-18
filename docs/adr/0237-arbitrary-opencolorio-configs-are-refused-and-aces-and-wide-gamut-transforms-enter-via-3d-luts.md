---
status: accepted
amends: 0017 (no open configuration file escape hatches), 0145 (closed vocabulary), 0206 (3D LUTs via .cube are the standard color transformation container)
---

# Arbitrary OpenColorIO configs are refused, and ACES and wide-gamut transforms enter via 3D LUTs

[#1110](https://github.com/MBehtemam/Montagent/issues/1110), on the map
[#1080](https://github.com/MBehtemam/Montagent/issues/1080), from the census in
[#904](https://github.com/MBehtemam/Montagent/issues/904) (rows 87–93) and
[#908](https://github.com/MBehtemam/Montagent/issues/908).

## The question

After Effects supports OpenColorIO (OCIO) and ACES color management through arbitrary `.ocio`
configuration files and OCIO transform effects.

Montagent enforces a closed format vocabulary ([ADR-0017](0017-closed-schema-no-escape-hatch.md),
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md))
and provides color grading via ASC-CDL and tetrahedral 3D LUTs ([ADR-0206](0206-asc-cdl-and-3d-lut-tetrahedral-shaders-grade-colour-under-closed-arithmetic.md)).

Are arbitrary `.ocio` configuration files refused from schema, and how do wide-gamut / ACES transforms enter?

## The decision

### 1. Arbitrary `.ocio` config files and OCIO runtime engines are refused

1. **Closed schema, no escape hatch ([ADR-0017](0017-closed-schema-no-escape-hatch.md))**: An `.ocio`
   configuration file is a complex hierarchical YAML document that references external LUT tables,
   matrix transforms, and ICC profiles across filesystem paths. Admitting arbitrary `.ocio` files
   into `project.json` creates an uncontrolled configuration dependency that cannot be verified by
   `validate` or reasoned about by AI agents using exact-string replacement.
2. **Deterministic byte-identical runtime**: OCIO evaluation introduces heavy external C++ runtimes
   and variable interpolation algorithms. Montagent's compositor already provides byte-identical
   tetrahedral interpolation on GPU/CPU for standard 3D LUTs ([ADR-0206](0206-asc-cdl-and-3d-lut-tetrahedral-shaders-grade-colour-under-closed-arithmetic.md)).

### 2. What to do instead: ACES and wide-gamut transforms enter via `.cube` 3D LUTs

Any color transform supported by OCIO or ACES (such as `ACEScg` to `Rec.709`, `Sony S-Log3 / S-Gamut3.Cine`
to `Rec.709`, `ARRI LogC3` to `Rec.709`, or creative look LUTs) can be pre-baked into an industry-standard
`.cube` tetrahedral lookup table (e.g. $33 \times 33 \times 33$ or $65 \times 65 \times 65$).

Elements apply the resulting grade directly using the native `lut3d` effect:

```json
{
  "kind": "lut3d",
  "path": "assets/luts/acescg_to_rec709.cube",
  "intensity": 1.0
}
```

Pre-baked `.cube` files are fully portable, static assets verified by `validate`, and evaluated with
fast, byte-identical Skia tetrahedral shaders.
