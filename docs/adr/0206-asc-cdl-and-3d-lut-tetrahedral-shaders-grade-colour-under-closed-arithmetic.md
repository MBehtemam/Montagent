---
status: accepted
amends: 0049 (colour filter vocabulary), 0145 (four invariants)
---

# ASC CDL and 3D LUT tetrahedral shaders grade colour under closed arithmetic

[#981](https://github.com/MBehtemam/Montagent/issues/981), [#1018](https://github.com/MBehtemam/Montagent/issues/1018), [#1020](https://github.com/MBehtemam/Montagent/issues/1020).

## Context

In Premiere Pro and industry NLEs, cinematic color grading relies on two primary standardized mechanisms (`docs/research/premiere-gap/colour.md`):
1. **ASC CDL (American Society of Cinematographers Color Decision List)**: A standardized primary color grading format defining Slope, Offset, Power, and Saturation.
2. **3D Color Lookup Tables (3D LUTs)**: Industry-standard `.cube` files mapping 3D RGB lattice cubes to target color grades for camera log conversions (Arri LogC, Sony S-Log3) and creative show LUTs.

Montagent previously offered only basic flat color adjustments (`tint`, `saturation`, `brightness`, `contrast` in [ADR-0049](0049-the-colour-filter-vocabulary-takes-four-names-with-a-closed-meaning-each.md)). To achieve parity with Premiere's Lumetri Color engine while upholding the four invariants of [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md), Montagent requires deterministic implementations of ASC CDL and 3D LUT `.cube` processing.

## Decision

1. **ASC CDL Effect (`"name": "cdl"`)**:  
   Admitted to the closed `VisualEffect` vocabulary on elements and `project.effects`:
   ```json
   {
     "name": "cdl",
     "slope": [1.05, 1.0, 0.95],
     "offset": [0.01, 0.0, -0.01],
     "power": [0.95, 1.0, 1.05],
     "saturation": 1.1
   }
   ```
   - **Literal Parameters**:
     - `slope`: Array of 3 floats $[r, g, b]$, each $\ge 0.0$. Identity: `[1.0, 1.0, 1.0]`.
     - `offset`: Array of 3 floats $[r, g, b]$, typically in $[-1.0, 1.0]$. Identity: `[0.0, 0.0, 0.0]`.
     - `power`: Array of 3 floats $[r, g, b]$, each $> 0.0$. Identity: `[1.0, 1.0, 1.0]`.
     - `saturation`: Scalar float $\ge 0.0$. Identity: `1.0`.
   - **Deterministic Mathematical Formula**:  
     For each normalized RGB channel $c \in \{R, G, B\}$:
     $$c_{\text{graded}} = \left[\max\left(0, c \times \text{slope}_c + \text{offset}_c\right)\right]^{\text{power}_c}$$
     Followed by Rec.709 luma-weighted saturation:
     $$Y = 0.2126 \cdot R_{\text{graded}} + 0.7152 \cdot G_{\text{graded}} + 0.0722 \cdot B_{\text{graded}}$$
     $$c_{\text{final}} = \text{clamp}\left(Y + \text{saturation} \times (c_{\text{graded}} - Y), 0.0, 1.0\right)$$

2. **3D LUT Ingestion (`"name": "lut3d"`)**:  
   Admitted to the closed `VisualEffect` vocabulary:
   ```json
   {
     "name": "lut3d",
     "source": "luts/creative_look.cube",
     "intensity": 1.0
   }
   ```
   - `source`: Relative filesystem path to a `.cube` file.
   - `intensity`: Optional float scalar in $[0.0, 1.0]$ blending between the original and LUT-graded pixel (defaults to `1.0`).

3. **Format Qualification & Static Validation**:  
   The `.cube` file is validated statically during `montagent validate` without rendering:
   - **`E-LUT-NOT-FOUND`**: Target `.cube` file does not exist at relative path.
   - **`E-LUT-INVALID`**: File fails standard specification:
     - Header must specify `LUT_3D_SIZE N` where $2 \le N \le 65$.
     - 1D LUTs (`LUT_1D_SIZE`) are refused; they must be expressed as standard tone curves or converted to 3D.
     - Table data must contain exactly $N^3$ space-separated RGB float triplets.
     - Table values outside reasonable bounds or malformed numbers emit descriptive findings.

4. **Tetrahedral Interpolation via Skia Runtime Shader**:  
   Trilinear 3D LUT interpolation introduces diagonal banding and clipping artifacts in high-contrast gradients. Montagent evaluates 3D LUT cubes via **tetrahedral subdivision**:
   - Each cube cell is partitioned into six tetrahedra based on the relative ordering of fractional coordinates:
     - $\Delta r > \Delta g > \Delta b$, $\Delta r > \Delta b > \Delta g$, $\Delta g > \Delta r > \Delta b$, $\Delta g > \Delta b > \Delta r$, $\Delta b > \Delta r > \Delta g$, $\Delta b > \Delta g > \Delta r$.
   - Interpolation within each simplex uses exact barycentric weights.
   - Implemented via a specialized Skia `SkRuntimeEffect` shader compiled once and cached across rendering sessions.
