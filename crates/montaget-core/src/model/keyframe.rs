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
            let records = read_records(value).map_err(D::Error::custom)?;
            positional_ease(&records).map_err(D::Error::custom)?;
            Ok(Animatable::Keyed(records))
        } else {
            serde_json::from_value(value)
                .map(Animatable::Static)
                .map_err(D::Error::custom)
        }
    }
}

/// One record at a time, so a fault inside one says which one.
///
/// `serde_json::from_value::<Vec<Keyframe<T>>>` would answer a negative `v` in the third
/// record with the bound's own sentence and nothing else — *"`volume` is -0.5"*, reading
/// as though the element's flat field were negative. The value bounds live on the value
/// types ([`crate::model::playback::Volume`] and its siblings), which by construction
/// cannot know where in a list they were written; the list is what knows, so the list says
/// it — in the same words [`positional_ease`] already uses for the other fault a record can
/// carry.
fn read_records<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<Vec<Keyframe<T>>, String> {
    let serde_json::Value::Array(items) = value else {
        return Err("a keyframe list is an array of `{\"t\",\"v\",\"ease\"}` records".to_string());
    };
    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            // The record's own `t` before it is parsed, so the one field that locates it on
            // the clock survives the failure that is about some other field.
            let t = item.get("t").and_then(serde_json::Value::as_i64);
            serde_json::from_value(item).map_err(|e| match t {
                Some(t) => format!("keyframe record {} (`t` {t}): {e}", index + 1),
                None => format!("keyframe record {}: {e}", index + 1),
            })
        })
        .collect()
}

/// ADR-0038's rule, which is a rule about **position** and so cannot be a rule about a
/// field: *"`ease` is required on every keyframe record except the first, where it remains a
/// schema error. Presence is a pure function of position in the list."*
///
/// It is enforced here, in the one place a keyframe list is read, rather than in a check —
/// because it is a schema fact, and [`crate::checks::schema`] reports schema facts by
/// parsing. Four agents hit the silence this ADR closed and resolved it three different
/// ways; one of them wrote `"ease":"linear"` on ten hold segments *"purely because the rule
/// was unstated"*, and one reclassified its own correct output as a violation. Both sides of
/// the rule are stated out loud below for that reason: the message names the convention,
/// which is the whole of what ADR-0012 asked the first-record error to do.
///
/// **Position here is position in the array**, which is what both ADRs say and what the
/// published schema's `prefixItems` can express. Whether it should instead be position on
/// the clock — the two coincide on every list anyone writes, and come apart on one written
/// backwards — is open, and is
/// [#270](https://github.com/MBehtemam/Montaget/issues/270).
fn positional_ease<T>(records: &[Keyframe<T>]) -> Result<(), String> {
    for (index, record) in records.iter().enumerate() {
        match (index, &record.ease) {
            (0, Some(_)) => {
                return Err(format!(
                    "the first keyframe record (`t` {}) carries an `ease`: `ease` describes \
                     the segment *arriving at* a record, and nothing arrives at the first \
                     one — drop it",
                    record.t
                ));
            }
            (index, None) if index > 0 => {
                return Err(format!(
                    "keyframe record {} (`t` {}) carries no `ease`: it is required on every \
                     record except the first, and there is no default — state the shape of \
                     the travel from the previous record, `linear` where the value is held",
                    index + 1,
                    record.t
                ));
            }
            _ => {}
        }
    }
    Ok(())
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
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Ease {
    Named(EaseName),
    /// `[x1, y1, x2, y2]`. `x` outside `[0,1]` is a schema error; `y` outside is legal,
    /// because overshoot is a real need beziers give away free.
    Bezier([f64; 4]),
}

impl<'de> Deserialize<'de> for Ease {
    /// ADR-0012: *"`x1`/`x2` outside `[0,1]` is a schema error; `y` outside is legal, because
    /// overshoot is a real need beziers give away free."*
    ///
    /// Enforced on the way in rather than left to a check, for the reason the positional
    /// `ease` rule is: it is a schema fact. It also keeps [`crate::resolve`]'s evaluator
    /// well-posed — the curve's `x` is what a resolved instant is solved against, and `x`
    /// control points outside the unit interval make that `x` non-monotonic, so there can be
    /// several parameters at one instant and no reason to prefer any of them. The refusal is
    /// what keeps the resolver from having to pick.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        let value = serde_json::Value::deserialize(deserializer)?;
        // A name is a string and the control points are an array, so the two forms are told
        // apart on shape — not tried in turn, whose only report is that neither matched.
        if value.is_string() {
            return serde_json::from_value(value)
                .map(Ease::Named)
                .map_err(D::Error::custom);
        }
        let points: [f64; 4] = serde_json::from_value(value).map_err(D::Error::custom)?;
        for (name, x) in [("x1", points[0]), ("x2", points[2])] {
            if !(0.0..=1.0).contains(&x) {
                return Err(D::Error::custom(format!(
                    "the ease's `{name}` is {x}: a bezier's `x` control points are the \
                     curve's own time and must be within `[0, 1]` — only `y` may overshoot"
                )));
            }
        }
        Ok(Ease::Bezier(points))
    }
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
