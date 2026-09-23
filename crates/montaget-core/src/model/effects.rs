//! The ordered `effects` list and its closed v1 vocabulary.
//!
//! ADR-0040 settled the attachment — a list field on the element, order significant,
//! because *"blur-then-shadow is a different frame from shadow-then-blur"* — and named
//! `blur`, `shadow` and `mask`. ADR-0049 added four colour scalars. ADR-0068 retired the
//! bare `mask` key that the fixture carried, and fixed where the list sits in key order:
//! **`effects` appends after the type's existing fields.**
//!
//! # Two rules the derive cannot state
//!
//! ADR-0084 gives `mask` one shape-independent rect and a `radius` whose legality depends
//! on `shape`. Neither is expressible in a `#[derive(Deserialize)]` enum variant: one is a
//! relation between four optional fields, the other makes a declared field an *unknown key*
//! under two of three `shape` values. So the derive is generated against [`Effect`] as a
//! remote (`remote = "Self"`, serde's own name for "write the functions, not the impls")
//! and the trait impls below wrap it — the parse the derive produces, then
//! [`Effect::checked`]. There is no second enumeration of the vocabulary anywhere: the
//! seven members are declared once, here.
//!
//! The same two rules are said again in the published schema, by
//! `crate::schema::publish_mask_rect`, for the reason ADR-0041 and #168 both give: a schema
//! that admitted files this binary refuses is the two-artifact divergence, arriving through
//! under-statement rather than through drift.

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Colour;

/// One member of the closed vocabulary, discriminated by `name`.
///
/// Two effects of the same name are ordinary rather than forbidden (ADR-0040) — the
/// container is a list precisely so a second shadow has somewhere to go.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "name",
    rename_all = "lowercase",
    deny_unknown_fields,
    remote = "Self"
)]
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
    /// **One rect, shared by all three shapes** (ADR-0084). `x`, `y`, `width`, `height`
    /// name the rect the shape is inscribed in — `circle` is the largest circle inscribed
    /// in it, `rect` is it, `ellipse` fills it — and `shape` selects which figure is drawn
    /// there, never which fields exist. The four are **all-or-none**, element-local
    /// integers measured from the element rect's top-left whatever the `origin` keyword
    /// is, and their identity value is the element's own rect: a bare
    /// `{"name": "mask", "shape": "circle"}` (ADR-0068's form) is the same declaration
    /// reached by the same arithmetic, not a legacy spelling beside this one.
    ///
    /// `radius` rounds the corners of a `rect` mask, identity `0`. On `circle` or
    /// `ellipse` it is an unknown key naming its reason, exactly as `radius` is on a drawn
    /// ellipse (ADR-0014): an inscribed ellipse has no corners to round.
    ///
    /// The mask is declared inside the element's box in unscaled units, so the element's
    /// `scale` grows it and its `rotation` turns it — the rule a `blur` radius and a
    /// `stroke_width` already follow. The fields stay literal integers on every frame, so
    /// that is not a keyframed effect parameter.
    Mask {
        shape: MaskShape,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        x: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        y: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        radius: Option<i64>,
    },
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

/// The four names of the mask rect, in ADR-0084's canonical order.
///
/// One array rather than four literals, because the all-or-none message has to name the
/// *other* three and a second listing would be a second answer to "which four".
pub(crate) const MASK_RECT: [&str; 4] = ["x", "y", "width", "height"];

impl Effect {
    /// This effect, if ADR-0084's two relational rules hold of it — or the sentence saying
    /// which one does not.
    ///
    /// Both are about `mask`, and neither can be a field's own type: one relates four
    /// optional fields to each other, the other makes a declared field unknown under two of
    /// three `shape` values.
    fn checked(self) -> Result<Self, String> {
        let Effect::Mask {
            shape,
            x,
            y,
            width,
            height,
            radius,
        } = &self
        else {
            return Ok(self);
        };

        // All-or-none, named in both directions — what was written and what it still needs.
        // "The four are all-or-none" on its own leaves the author counting.
        let present: Vec<&str> = MASK_RECT
            .iter()
            .zip([x, y, width, height])
            .filter(|(_, value)| value.is_some())
            .map(|(name, _)| *name)
            .collect();
        if !present.is_empty() && present.len() != MASK_RECT.len() {
            let missing: Vec<&str> = MASK_RECT
                .iter()
                .copied()
                .filter(|name| !present.contains(name))
                .collect();
            return Err(format!(
                "`mask` states {} and not {}: the mask rect's four fields are all-or-none \
                 — write all of `x`, `y`, `width`, `height`, or none of them and take the \
                 element's own rect (ADR-0084)",
                quoted(&present),
                quoted(&missing),
            ));
        }

        // `radius` is a field of `shape: "rect"` only. On the other two it is an unknown
        // key that says why, rather than one silently ignored (ADR-0084, ADR-0014).
        //
        // **Phrased in serde's own unknown-field form, deliberately.** ADR-0084 does not
        // merely call it an error: it says the key "is an **unknown key** … following
        // ADR-0014's rule for the same word on the drawn `shape` element", and CONTEXT.md
        // promises it behaves "exactly as it is on a drawn ellipse". A drawn ellipse's
        // `radius` is refused by the derive, so it reads `unknown field \`radius\`,
        // expected one of …` and `crate::checks::schema` classifies it as
        // `E-SCHEMA-UNKNOWN-KEY` — which is what carries ADR-0016's guarantee text ("it may
        // belong to a newer format revision … do not delete the key to make the file
        // validate"). A message of this check's own devising would have been `E-SCHEMA`
        // instead, and the two words would have named two different reports.
        //
        // The reason sits between the two markers that check reads, so it is in front of
        // anyone holding the raw parse error while the finding stays the shared one.
        if radius.is_some() && *shape != MaskShape::Rect {
            return Err(format!(
                "unknown field `radius` on a `{shape}` mask: an inscribed {shape} has no \
                 corners to round, so `radius` is a field of `shape: \"rect\"` only \
                 (ADR-0084, ADR-0014) — expected one of `name`, `shape`, {rect}",
                shape = shape.as_str(),
                rect = quoted(&MASK_RECT),
            ));
        }

        Ok(self)
    }
}

/// `` `x`, `width` `` — a list of field names, for a sentence.
fn quoted(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

// The two halves of `remote = "Self"`: the derive above wrote `Effect::serialize` and
// `Effect::deserialize` as inherent functions, and these are the impls that call them. The
// wire form is entirely the derive's — the only thing added is the parse-time check.
impl Serialize for Effect {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Effect::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Effect {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Effect::deserialize(deserializer)?
            .checked()
            .map_err(D::Error::custom)
    }
}

/// The figure an [`Effect::Mask`] draws in its rect (ADR-0084).
///
/// Not a schema discriminator: all three shapes take the same fields, and `shape` says
/// which figure is cut in them. ADR-0049's two-level-lookup objection is why — an agent
/// that has learned `mask` has learned all of `mask`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MaskShape {
    Circle,
    Rect,
    Ellipse,
}

impl MaskShape {
    /// The word a document spells this shape with — the one a message has to use.
    pub fn as_str(self) -> &'static str {
        match self {
            MaskShape::Circle => "circle",
            MaskShape::Rect => "rect",
            MaskShape::Ellipse => "ellipse",
        }
    }
}
