//! The ordered `effects` list and its closed v1 vocabulary.
//!
//! ADR-0040 settled the attachment — a list field on the element, order significant,
//! because *"blur-then-shadow is a different frame from shadow-then-blur"* — and named
//! `blur`, `shadow` and `mask`. ADR-0049 added four colour scalars. ADR-0068 retired the
//! bare `mask` key that the fixture carried, and fixed where the list sits in key order:
//! **`effects` appends after the type's existing fields.**

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Colour;

/// One member of the closed vocabulary, discriminated by `name`.
///
/// Two effects of the same name are ordinary rather than forbidden (ADR-0040) — the
/// container is a list precisely so a second shadow has somewhere to go.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "name", rename_all = "lowercase", deny_unknown_fields)]
pub enum Effect {
    /// Gaussian blur; one parameter.
    Blur { radius: f64 },
    /// Drop shadow.
    Shadow {
        dx: f64,
        dy: f64,
        radius: f64,
        color: Colour,
        opacity: f64,
    },
    /// A shape mask, shape-only — no image source, no alpha or soft mask.
    ///
    /// ADR-0068 states the param-less form's geometry, which ADR-0040 left as an ellipsis:
    /// `{"name": "mask", "shape": "circle"}` with no geometry parameters means the largest
    /// circle inscribed in the element's own rect, and `rect`/`ellipse` take the element's
    /// rect itself. The full parameter surface — explicit centre and radius, corner radii,
    /// two axes, and what any of them mean under a keyframed `scale` — is graduated to its
    /// own ticket, so this carries the shape and nothing else.
    Mask { shape: MaskShape },
    /// Pushes pixel colour toward `color` by `amount` (0–1).
    ///
    /// `color` is ADR-0049's sole grandfathered exception to *"parameters must be bounded
    /// scalars"*, and the exception is closed: a future colour operation does not get the
    /// same allowance by citing `tint` as precedent.
    Tint { color: Colour, amount: f64 },
    /// `0` = fully desaturated (a bare `grayscale`, which is deliberately not its own
    /// member), `1` = unchanged, `>1` = oversaturated.
    Saturation { amount: f64 },
    /// Signed offset from unchanged at `0`.
    Brightness { amount: f64 },
    /// Signed offset from unchanged at `0`.
    Contrast { amount: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MaskShape {
    Circle,
    Rect,
    Ellipse,
}
