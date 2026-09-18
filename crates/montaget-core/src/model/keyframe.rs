//! Animatable properties, their keyframe records, and the closed easing set.
//!
//! ADR-0012: *"Any transform property may be a value or a list of `{"t","v","ease"}`
//! records on the project's one absolute clock."* A keyframe's `t` is written in timeline
//! milliseconds but **is not a timeline time** — `shift` moves elements and their
//! keyframes are carried.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// A property that is either one value or a list of keyframe records.
///
/// The polymorphism is the format's, not a convenience: ADR-0055 restates it for `volume`
/// as *"the same scalar-or-keyframe-array polymorphism every other animatable property
/// already has."* It is deliberately **not** the same as the `scale` union ADR-0012
/// rejected — that was two shapes for one *value* (`1.08` versus `[1.08, 1.08]`), which
/// puts a shape test in every consumer; this is a value versus its animation over time,
/// which no amount of normalising can collapse.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Animatable<T> {
    /// Constant for the element's whole life.
    Static(T),
    /// Keyframed. Clamps at both ends (ADR-0012): before the first record the value is the
    /// first value, after the last it is the last, so *"and then it holds"* costs no syntax.
    Keyed(Vec<Keyframe<T>>),
}

impl<'de, T: serde::de::DeserializeOwned> Deserialize<'de> for Animatable<T> {
    /// Chosen on shape, then deserialized — rather than `#[serde(untagged)]`, which tries
    /// each variant and reports only that none matched.
    ///
    /// The difference is the error text, and the error text is the mechanism: with no
    /// version number in the file, an unknown key is the only signal an old binary has that
    /// a file was authored against a newer schema (ADR-0016), and ADR-0016's own message —
    /// *"do not delete the key to make the file validate"* — can only be written about a
    /// key it can name. Untagged would answer a misspelled `easing` inside a keyframe with
    /// *"data did not match any variant"*, which is the bare schema-mismatch dump ADR-0042
    /// asked implementations not to produce.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        let value = serde_json::Value::deserialize(deserializer)?;
        // A keyframe record is an object (ADR-0012 rejected positional pairs: an unlabelled
        // 2-array has no room for `ease` and becomes genuinely ambiguous once `v` is itself
        // a list). So an array **of objects** is a keyframe list, and every other array —
        // `scale`'s own `[sx, sy]`, `clip`'s four integers — is a static value.
        let keyed = value
            .as_array()
            .and_then(|items| items.first())
            .is_some_and(serde_json::Value::is_object);

        if keyed {
            serde_json::from_value(value)
                .map(Animatable::Keyed)
                .map_err(D::Error::custom)
        } else {
            serde_json::from_value(value)
                .map(Animatable::Static)
                .map_err(D::Error::custom)
        }
    }
}

/// One `{"t","v","ease"}` record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Keyframe<T> {
    /// Milliseconds on the project's one absolute clock. Legal outside the element's own
    /// range — that is how a trimmed move is spelled, and seven of the fixture's photo
    /// elements carry one.
    pub t: i64,
    pub v: T,
    /// The shape of the interpolation **arriving at** this record from the previous one.
    ///
    /// ADR-0038: required on every record except the first, where it is a schema error.
    /// Presence is a pure function of position in the list, so there is never an absent
    /// value to default, normalise, or keep in sync with SPLIT's equality check. The
    /// `Option` here carries that positional rule, not a default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ease: Option<Ease>,
}

/// The closed easing set: a published name, or the four control points directly.
///
/// ADR-0012: *"A name is sugar over one evaluator, never a second mechanism."* The raw
/// form is forced rather than chosen — splitting `ease-in-out` at an arbitrary instant
/// yields a bezier that is no named ease and is not in the family, and snapping to the
/// nearest name costs more framing than the reading `shift` was disqualified for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Ease {
    Named(EaseName),
    /// `[x1, y1, x2, y2]`. `x` outside `[0,1]` is a schema error; `y` outside is legal,
    /// because overshoot is a real need beziers give away free.
    Bezier([f64; 4]),
}

/// Each name is published in the schema as its cubic bezier (ADR-0012).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EaseName {
    /// `[0, 0, 1, 1]`
    Linear,
    /// `[0.25, 0.1, 0.25, 1]`
    Ease,
    /// `[0.42, 0, 1, 1]`
    EaseIn,
    /// `[0, 0, 0.58, 1]`
    EaseOut,
    /// `[0.42, 0, 0.58, 1]`
    EaseInOut,
    /// Named `step`, not `hold`: *"hold" reads as a claim about what happens next — a
    /// `leaving` intuition sitting in an `entering` slot* (ADR-0012).
    Step,
}

impl EaseName {
    /// The published control points. `step` has none — it is not a bezier.
    pub fn bezier(self) -> Option<[f64; 4]> {
        Some(match self {
            EaseName::Linear => [0.0, 0.0, 1.0, 1.0],
            EaseName::Ease => [0.25, 0.1, 0.25, 1.0],
            EaseName::EaseIn => [0.42, 0.0, 1.0, 1.0],
            EaseName::EaseOut => [0.0, 0.0, 0.58, 1.0],
            EaseName::EaseInOut => [0.42, 0.0, 0.58, 1.0],
            EaseName::Step => return None,
        })
    }
}
