//! Animatable properties, their keyframe records, and the closed easing set.
//!
//! ADR-0012: *"Any transform property may be a value or a list of `{"t","v","ease"}`
//! records on the project's one absolute clock."* A keyframe's `t` is written in timeline
//! milliseconds but **is not a timeline time** — `shift` moves elements and their
//! keyframes are carried.
//!
//! ADR-0086 adds one optional key beside that `t`: [`Derivation`], the recorded-intent
//! declaration saying which rule the author derived the instant by. No renderer reads it
//! and `validate` is its only consumer, so the literal `t` beside it remains the sole
//! author of what renders.

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
    /// Records must be written with strictly increasing `t` (ADR-0082) — array order is
    /// clock order.
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
            positional_t_from(&records).map_err(D::Error::custom)?;
            ascending_t(&records).map_err(D::Error::custom)?;
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
/// published schema's `prefixItems` can express. That reading and the clock's coincide by
/// construction now that [`ascending_t`] is enforced alongside it (ADR-0082, closing
/// [#270](https://github.com/MBehtemam/Montaget/issues/270)) — array position and clock
/// position are the same order for any record this function is ever called with.
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

/// ADR-0086's one positional rule about `t_from`: `after-previous` may not sit on the
/// **first** record of a list, where there is no previous record for it to name.
///
/// Here rather than in a check for [`positional_ease`]'s reason — it is a schema fact about
/// a position, which no field's own type can carry, and this is the one place a keyframe
/// list is read. `element-start` is legal at every position including the first, which is
/// where the fixture's seven ramps carry it.
///
/// `ms` is *not* re-derived here and no arithmetic is checked: a declaration that names a
/// previous record which exists is well-formed whatever integer it states, and whether the
/// arithmetic still holds is `R-DERIVED-T`'s finding (`crate::checks::derived`) rather than
/// a refusal to read the file. ADR-0086 puts it at `error` with an advise-class repair,
/// which a parse failure has no way to carry.
fn positional_t_from<T>(records: &[Keyframe<T>]) -> Result<(), String> {
    match records.first().map(|record| (record.t, record.t_from)) {
        Some((t, Some(Derivation::AfterPrevious { ms }))) => Err(format!(
            "the first keyframe record (`t` {t}) declares `t_from` \
             `{AFTER_PREVIOUS}` (`ms` {ms}): it names the record before it, and nothing \
             comes before the first one \u{2014} `{{\"rule\": \"{ELEMENT_START}\"}}` is the \
             rule a first record can carry"
        )),
        _ => Ok(()),
    }
}

/// ADR-0082: a keyframe list's records must be written with strictly increasing `t`. Enforced
/// here rather than in a check, for the same reason [`positional_ease`] is: it is a schema
/// fact, this is the one place a keyframe list is read, and `#E-SCHEMA` reports schema facts
/// by catching what parsing refuses.
///
/// Two records sharing one `t` are also an inversion under this rule — `t` is not *strictly*
/// increasing — and fire the same error. ADR-0012 and ADR-0038 are silent on same-`t` records
/// and this function does not additionally resolve that question.
///
/// This is what makes [`positional_ease`]'s array-position reading of ADR-0038 and
/// [`crate::resolve::at`]'s clock-position reading the same statement rather than two that
/// can be fed a list where they disagree — the divergence [#270](https://github.com/MBehtemam/Montaget/issues/270)
/// found and ADR-0082 closes.
fn ascending_t<T>(records: &[Keyframe<T>]) -> Result<(), String> {
    for (index, pair) in records.windows(2).enumerate() {
        let (previous, record) = (&pair[0], &pair[1]);
        if record.t <= previous.t {
            return Err(format!(
                "keyframe record {} (`t` {}) is not strictly after keyframe record {} (`t` \
                 {}): a keyframe list must be written with strictly increasing `t`",
                index + 2,
                record.t,
                index + 1,
                previous.t
            ));
        }
    }
    Ok(())
}

/// One `{"t","t_from","v","ease"}` record — `t_from` optional (ADR-0086).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Keyframe<T> {
    /// Milliseconds on the project's one absolute clock. Legal outside the element's own
    /// range — that is how a trimmed move is spelled, and seven of the fixture's photo
    /// elements carry one.
    pub t: i64,
    /// What this record's own `t` was derived from, where the author recorded it
    /// (ADR-0086).
    ///
    /// **Immediately after `t`**, which is the whole of ADR-0086's key-order
    /// specification: ADR-0041 derives canonical order from schema order, and schema order
    /// is this struct's declaration order, so adjacency to the annotated value costs
    /// nothing to state twice and cannot drift.
    ///
    /// Optional, and its absence means *no claim* — never a claim of independence
    /// (ADR-0086's sixth condition of admission). Required was measured and rejected: on
    /// the committed fixture it would put an escape value on 121 instants to check 14.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_from: Option<Derivation>,
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

/// The name of ADR-0086's no-argument rule, as a document spells it.
///
/// The two spellings are consts because four messages and the schema pass
/// (`crate::schema`) all have to name them, and a rule set that is *"finite and
/// published"* (ADR-0086's third condition of admission) is undone by a second answer to
/// what it is called.
pub(crate) const ELEMENT_START: &str = "element-start";

/// The name of ADR-0086's one-argument rule.
pub(crate) const AFTER_PREVIOUS: &str = "after-previous";

/// Both rule names, in schema order — what a message listing the legal set prints.
pub(crate) const RULES: [&str; 2] = [ELEMENT_START, AFTER_PREVIOUS];

/// `` `element-start`, `after-previous` `` — the closed rule set, for a sentence.
fn rules() -> String {
    RULES
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A recorded-intent declaration on a keyframe record's own `t`: the rule the author
/// derived that instant by (ADR-0086).
///
/// **No renderer reads it and `validate` is its only consumer.** The literal `t` beside it
/// remains the sole author of what renders, so a declaration that has gone stale is a
/// finding (`R-DERIVED-T`) and never a different frame.
///
/// The rule set is closed and both members were measured 7-of-7 on the committed fixture
/// before they shipped. Both are **directional** — the document names which value is the
/// source and which the derived one — so every violation is `error` with an advise-class
/// repair stating the re-derived integer. A rule takes at most one argument, whose type is
/// fixed per rule by the schema; rules do not compose, and direction is carried in the rule
/// *name* (`after-`) rather than in the sign of the argument, so no rule needs signed
/// arithmetic.
///
/// The step this type refuses, named so it cannot be taken by drift: a second argument, a
/// signed argument, or a rule whose argument is another rule. Any of those is a new ADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "rule", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Derivation {
    /// This `t` was derived as the element's own `start`. No argument.
    ElementStart,
    /// This `t` was derived as the previous record's `t` plus `ms`.
    ///
    /// The **rate** spelling the evidence forced: it says how far a ramp runs, not where it
    /// stops, which is what the fixture's seven Ken Burns ramps do — every one overruns its
    /// element's `end` and none lands on it. *"Previous"* is positional and not a reference,
    /// well-defined because ADR-0082 requires a keyframe list to be written in ascending
    /// `t`.
    AfterPrevious {
        /// A **non-negative** integer offset in milliseconds. Published as an unsigned
        /// integer, which is where the schema states the bound; the deserializer states it
        /// again in words, because *"invalid value"* tells an agent only that it must guess
        /// again.
        ms: u64,
    },
}

impl Derivation {
    /// The word a document spells this rule with — the one a finding has to print.
    ///
    /// Here rather than at the check that prints it, so the rule set keeps one name apiece:
    /// `crate::checks::derived` reads a declaration back through this type's own
    /// deserializer and then asks it what it is called, instead of matching the string a
    /// second time.
    pub fn rule(self) -> &'static str {
        match self {
            Derivation::ElementStart => ELEMENT_START,
            Derivation::AfterPrevious { .. } => AFTER_PREVIOUS,
        }
    }
}

impl<'de> Deserialize<'de> for Derivation {
    /// Read as a tree and then dispatched on `rule`, rather than by the derive — for the
    /// reason [`Animatable`] and [`Ease`] are: the error text is the mechanism, and ADR-0086
    /// asks for schema errors *"each naming the legal form"*. An internally-tagged derive
    /// answers an unknown rule with `unknown variant`, a missing `ms` with `missing field`,
    /// and a negative one with `invalid value: integer` — three sentences that name the
    /// fault and never the form.
    ///
    /// **Only a genuinely unpublished key is phrased as `serde`'s own unknown field.** That
    /// phrasing is what `crate::checks::schema` classifies as `E-SCHEMA-UNKNOWN-KEY`, which
    /// carries ADR-0016's *"it may belong to a newer format revision — do not delete the key
    /// to make the file validate"*. That is the right thing to say about a key this binary
    /// has never heard of, and the wrong thing to say about an `ms` on `element-start`,
    /// where deleting the key **is** the fix — so that one is an ordinary `E-SCHEMA` naming
    /// the two legal forms.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        let value = serde_json::Value::deserialize(deserializer)?;
        let serde_json::Value::Object(mut body) = value else {
            return Err(D::Error::custom(format!("a `t_from` is {}", legal_forms())));
        };

        let rule = match body.remove("rule") {
            Some(serde_json::Value::String(rule)) => rule,
            Some(other) => {
                return Err(D::Error::custom(format!(
                    "a `t_from`'s `rule` is {other}: it is one of {}, and a `t_from` is {}",
                    rules(),
                    legal_forms()
                )));
            }
            None => {
                return Err(D::Error::custom(format!(
                    "a `t_from` states no `rule`: it is {}",
                    legal_forms()
                )));
            }
        };
        let ms = body.remove("ms");

        // Whatever is left is a key the format does not publish inside a `t_from`, and the
        // two names it *does* publish are already out of the map — so the list this names is
        // the whole of the legal set rather than a per-rule subset.
        if let Some((key, _)) = body.into_iter().next() {
            return Err(D::Error::custom(format!(
                "unknown field `{key}` in a `t_from`, expected one of `rule`, `ms`"
            )));
        }

        match (rule.as_str(), ms) {
            (ELEMENT_START, None) => Ok(Derivation::ElementStart),
            (ELEMENT_START, Some(ms)) => Err(D::Error::custom(format!(
                "a `t_from` declares `{ELEMENT_START}` and an `ms` of {ms}: \
                 `{ELEMENT_START}` derives the instant from the element's own `start` and \
                 takes no argument — drop the `ms`, or state `{AFTER_PREVIOUS}`, which is \
                 the rule that takes one"
            ))),
            (AFTER_PREVIOUS, None) => Err(D::Error::custom(format!(
                "a `t_from` declares `{AFTER_PREVIOUS}` and no `ms`: the offset from the \
                 previous record is the rule's one argument and has no default — state it \
                 as a non-negative integer of milliseconds"
            ))),
            (AFTER_PREVIOUS, Some(ms)) => Ok(Derivation::AfterPrevious { ms: offset(ms)? }),
            (rule, _) => Err(D::Error::custom(format!(
                "a `t_from`'s `rule` is `{rule}`: the rule set is closed and published, and \
                 its two members are {} \u{2014} a new rule is an ADR carrying a census, \
                 never a convenience",
                rules()
            ))),
        }
    }
}

/// `after-previous`'s one argument, as a non-negative integer of milliseconds.
///
/// Negative is refused in the rule's own words rather than `u64`'s: direction is carried in
/// the rule *name*, so a sign here is not a rule pointing the other way — it is a rule with
/// no meaning at all, and the message has to say which of the two it should have been.
fn offset<E: serde::de::Error>(ms: serde_json::Value) -> Result<u64, E> {
    ms.as_u64().ok_or_else(|| {
        E::custom(format!(
            "a `t_from`'s `ms` is {ms}: it is a non-negative integer of milliseconds \u{2014} \
             direction is carried in the rule name (`{AFTER_PREVIOUS}`), so there is no \
             signed arithmetic for a negative offset to mean"
        ))
    })
}

/// The two legal shapes, spelled out — the second half of every message above.
fn legal_forms() -> String {
    format!(
        "an object naming one rule: `{{\"rule\": \"{ELEMENT_START}\"}}`, or \
         `{{\"rule\": \"{AFTER_PREVIOUS}\", \"ms\": <a non-negative integer>}}`"
    )
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
