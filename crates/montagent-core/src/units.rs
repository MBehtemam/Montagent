//! A text element's stagger (ADR-0151 §2–§5, amended by ADR-0153 §2): its units, when each
//! starts, what a run's `unit` override replaces, and each unit's pose at an instant.
//!
//! Read off the permissive JSON tree, as the painter and `query --at` read everything else,
//! so a document the typed model refuses still answers what it can. What one unit *is* comes
//! from [`montagent_text::units`], from the text alone; which units shaping merged needs the
//! placed glyphs, and is [`montagent_text::units::bodies`]'s.

use std::ops::Range;

use montagent_text::units::{By, Segmentation, segment};
use serde_json::Value;

use crate::model::Origin;
use crate::verbs::query::geometry::{number_at, origin_fraction};

/// The five unit lists, in the order ADR-0151 names them.
///
/// Nested inside a text element, so not members of [`crate::animatable`]'s element-level
/// list; a test below holds this to the animatable properties the schema types on `Units`
/// and `UnitOverride`, so it cannot drift from the format.
pub(crate) const LISTS: [&str; 5] = ["x", "y", "rotation", "scale", "opacity"];

#[cfg(test)]
mod tests {
    use super::LISTS;

    #[test]
    fn the_unit_lists_are_the_animatable_properties_the_schema_types_on_units_and_overrides() {
        let schema = crate::schema::generate();
        for def in ["Units", "UnitOverride"] {
            let mut typed: Vec<&str> = schema["$defs"][def]["properties"]
                .as_object()
                .expect("an object schema")
                .iter()
                .filter(|(_, property)| {
                    property["$ref"]
                        .as_str()
                        .is_some_and(|r| r.starts_with("#/$defs/Animatable"))
                })
                .map(|(name, _)| name.as_str())
                .collect();
            let mut listed = LISTS.to_vec();
            typed.sort_unstable();
            listed.sort_unstable();
            assert_eq!(typed, listed, "{def}");
        }
    }
}

/// One unit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Unit {
    pub(crate) index: usize,
    /// Its text: its graphemes, whitespace between them included (a line unit's spaces).
    pub(crate) text: String,
    /// Its bytes in the element's whole text, first grapheme to last.
    pub(crate) span: Range<usize>,
    /// The run its first grapheme sits in.
    pub(crate) run: usize,
    /// Its position in `order` times `every`, or the override's `delay`.
    pub(crate) delay: i64,
    /// The run whose `unit` override singles it out: a run holding exactly this unit.
    pub(crate) override_run: Option<usize>,
}

/// A unit's resolved pose at an instant: offsets inside the element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Pose {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) rotation: f64,
    pub(crate) scale: [f64; 2],
    pub(crate) opacity: f64,
}

impl Pose {
    /// The pose that draws a unit exactly as it was laid out.
    pub(crate) fn is_rest(&self) -> bool {
        self.x == 0.0
            && self.y == 0.0
            && self.rotation == 0.0
            && self.scale == [1.0, 1.0]
            && self.opacity >= 1.0
    }

    /// `T(pivot + offset) · R(rotation) · S(scale) · T(−pivot)`, as `[sx, kx, tx, ky, sy, ty]`
    /// in the block's own coordinates.
    pub(crate) fn matrix(&self, pivot: (f64, f64)) -> [f64; 6] {
        let (px, py) = pivot;
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        let [sx, sy] = self.scale;
        let (a, b, d, e) = (cos * sx, -sin * sy, sin * sx, cos * sy);
        let tx = px + self.x - (a * px + b * py);
        let ty = py + self.y - (d * px + e * py);
        [a, b, tx, d, e, ty]
    }
}

/// Why a run's `unit` override does not single out one unit (`E-UNIT-RUN-NOT-ONE-UNIT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NotOneUnit {
    /// How many units the run touches.
    pub(crate) found: usize,
}

/// An element's whole stagger, before any instant.
#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub(crate) by: By,
    /// The pivot's fraction of each unit's box.
    pub(crate) origin: (f64, f64),
    /// The element's whole text: its runs concatenated.
    pub(crate) text: String,
    pub(crate) segmentation: Segmentation,
    pub(crate) units: Vec<Unit>,
    /// Each run's bytes in [`Plan::text`].
    pub(crate) run_ranges: Vec<Range<usize>>,
    block: Value,
    runs: Vec<Value>,
}

/// `by`'s word, as the file writes it.
pub(crate) fn by_name(by: By) -> &'static str {
    match by {
        By::Letter => "letter",
        By::Word => "word",
        By::Line => "line",
    }
}

impl Plan {
    /// The element's stagger, or `None` where it has no `units` block with a readable `by`
    /// and a positive `every` — the schema check's to report.
    pub(crate) fn of(element: &Value) -> Option<Plan> {
        let block = element.get("units")?.clone();
        let by = match block.get("by")?.as_str()? {
            "letter" => By::Letter,
            "word" => By::Word,
            "line" => By::Line,
            _ => return None,
        };
        let every = block.get("every")?.as_i64().filter(|every| *every > 0)?;
        let reverse = block.get("order").and_then(Value::as_str) == Some("reverse");
        let origin: Origin = block
            .get("origin")
            .and_then(|o| serde_json::from_value(o.clone()).ok())
            .unwrap_or(Origin::Center);
        let runs: Vec<Value> = crate::verbs::measure::runs_array(element).to_vec();
        let mut text = String::new();
        let mut run_ranges = Vec::with_capacity(runs.len());
        for run in &runs {
            let start = text.len();
            text.push_str(run.get("text").and_then(Value::as_str).unwrap_or(""));
            run_ranges.push(start..text.len());
        }
        let segmentation = segment(&text, by);
        let count = segmentation.count;
        let run_of = |at: usize| run_ranges.iter().position(|r| r.contains(&at)).unwrap_or(0);
        let units: Vec<Unit> = (0..count)
            .map(|index| {
                let mut members = segmentation
                    .graphemes
                    .iter()
                    .filter(|g| g.unit == Some(index));
                let first = members.next().map_or(0..0, |g| g.range.clone());
                let last = members
                    .next_back()
                    .map_or(first.clone(), |g| g.range.clone());
                let span = first.start..last.end;
                let position = if reverse { count - 1 - index } else { index } as i64;
                Unit {
                    index,
                    text: text[span.clone()].to_string(),
                    run: run_of(span.start),
                    span,
                    delay: position * every,
                    override_run: None,
                }
            })
            .collect();
        let mut plan = Plan {
            by,
            origin: origin_fraction(origin),
            text,
            segmentation,
            units,
            run_ranges,
            block,
            runs,
        };
        for r in 0..plan.runs.len() {
            let Some(over) = plan.runs[r].get("unit").filter(|o| o.is_object()) else {
                continue;
            };
            let Ok(unit) = plan.singled_out(r) else {
                continue;
            };
            let delay = over
                .get("delay")
                .and_then(Value::as_i64)
                .filter(|delay| *delay >= 0);
            plan.units[unit].override_run = Some(r);
            if let Some(delay) = delay {
                plan.units[unit].delay = delay;
            }
        }
        Some(plan)
    }

    /// The one unit run `r` holds exactly — all of its graphemes, from its first to its last,
    /// and nothing else — or how many units it touches.
    pub(crate) fn singled_out(&self, r: usize) -> Result<usize, NotOneUnit> {
        let range = &self.run_ranges[r];
        let mut touched: Vec<usize> = self
            .segmentation
            .graphemes
            .iter()
            .filter(|g| range.contains(&g.range.start))
            .filter_map(|g| g.unit)
            .collect();
        touched.sort_unstable();
        touched.dedup();
        match touched.as_slice() {
            [unit] if self.units[*unit].span == *range => Ok(*unit),
            _ => Err(NotOneUnit {
                found: touched.len(),
            }),
        }
    }

    /// The `unit` object on run `r`, as written.
    pub(crate) fn override_on(&self, r: usize) -> Option<&Value> {
        self.runs.get(r)?.get("unit").filter(|o| o.is_object())
    }

    /// Every run carrying a `unit` object, by index.
    pub(crate) fn override_runs(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.runs.len()).filter(|&r| self.override_on(r).is_some())
    }

    /// The `units` block as written.
    pub(crate) fn block(&self) -> &Value {
        &self.block
    }

    /// The override singling out `unit`, if any.
    fn override_of(&self, unit: usize) -> Option<&Value> {
        self.override_on(self.units[unit].override_run?)
    }

    /// Whether `unit`'s `property` comes from a run override — or, for `"delay"`, whether
    /// its delay does.
    pub(crate) fn overridden(&self, unit: usize, property: &str) -> bool {
        self.override_of(unit)
            .is_some_and(|o| o.get(property).is_some())
    }

    /// Where `unit`'s `property` list is read from, and how late it runs: a run's own list on
    /// the absolute clock, otherwise the block's list late by the unit's delay.
    pub(crate) fn source(&self, unit: usize, property: &str) -> (&Value, i64) {
        match self.override_of(unit) {
            Some(o) if o.get(property).is_some() => (o, 0),
            _ => (&self.block, self.units[unit].delay),
        }
    }

    /// The unit's pose at `instant`.
    pub(crate) fn pose(&self, unit: usize, instant: i64) -> Pose {
        self.pose_at(unit, (i128::from(instant), 1))
    }

    /// The unit's pose at `t = (numerator, denominator)` ms — a motion-blur sample
    /// (ADR-0155 §3). The unit's delay is shifted in the same rationals, so a stagger is
    /// sampled as finely as the element it belongs to.
    pub(crate) fn pose_at(&self, unit: usize, t: (i128, i128)) -> Pose {
        let (numerator, denominator) = t;
        let at = |property: &str| {
            let (from, late) = self.source(unit, property);
            (
                from,
                (numerator - i128::from(late) * denominator, denominator),
            )
        };
        let (o, t) = at("x");
        let x = number_at::<i64>(o, "x", t, 0.0);
        let (o, t) = at("y");
        let y = number_at::<i64>(o, "y", t, 0.0);
        let (o, t) = at("rotation");
        let rotation = number_at::<f64>(o, "rotation", t, 0.0);
        let (o, t) = at("scale");
        let scale = number_at::<[f64; 2]>(o, "scale", t, [1.0, 1.0]);
        let (o, t) = at("opacity");
        let opacity = number_at::<f64>(o, "opacity", t, 1.0);
        Pose {
            x,
            y,
            rotation,
            scale,
            opacity,
        }
    }

    /// Every unit list's first and last keyframe instants after its delay, with the unit and
    /// the property: what the stagger window and the settle instant are made of.
    pub(crate) fn spans(&self) -> Vec<(usize, &'static str, i64, i64)> {
        let mut out = Vec::new();
        for unit in 0..self.units.len() {
            for property in LISTS {
                let (from, late) = self.source(unit, property);
                let Some(records) = crate::checks::keyframe_records(from, property) else {
                    continue;
                };
                let ts: Vec<i64> = records
                    .iter()
                    .filter_map(|r| r.get("t").and_then(Value::as_i64))
                    .collect();
                if let (Some(first), Some(last)) = (ts.iter().min(), ts.iter().max()) {
                    out.push((unit, property, first + late, last + late));
                }
            }
        }
        out
    }

    /// The stagger window: from the earliest start of any unit list to the latest end, after
    /// delays and including run overrides (ADR-0151 §5). `None` where no list is keyed.
    pub(crate) fn window(&self) -> Option<(i64, i64)> {
        let spans = self.spans();
        let start = spans.iter().map(|s| s.2).min()?;
        let end = spans.iter().map(|s| s.3).max()?;
        Some((start, end))
    }

    /// The pivot of a body's box.
    pub(crate) fn pivot(&self, rect: [f64; 4]) -> (f64, f64) {
        let [l, t, r, b] = rect;
        (l + self.origin.0 * (r - l), t + self.origin.1 * (b - t))
    }
}

/// How far a stagger's units can reach from where they were laid out, from the file alone
/// (ADR-0151 §5, for `R-OFF-CANVAS`): per edge, the most negative or most positive unit `x`
/// or `y` value any unit list writes — the block's and every run override's — and the list
/// that writes it.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Reach {
    /// `(element pixels, the list's name)`, for each edge a value moves past.
    pub(crate) left: Option<(f64, String)>,
    pub(crate) right: Option<(f64, String)>,
    pub(crate) top: Option<(f64, String)>,
    pub(crate) bottom: Option<(f64, String)>,
    /// Which of `scale` and `rotation` some unit list carries: they are not measured.
    pub(crate) unchecked: Vec<&'static str>,
}

/// [`Reach`] for an element with a `units` block, `None` for any other.
pub(crate) fn reach(element: &Value) -> Option<Reach> {
    let block = element.get("units")?;
    let mut lists: Vec<(String, &Value)> = vec![("units".to_string(), block)];
    for (r, run) in crate::verbs::measure::runs_array(element)
        .iter()
        .enumerate()
    {
        if let Some(over) = run.get("unit").filter(|o| o.is_object()) {
            lists.push((format!("runs[{r}].unit"), over));
        }
    }
    let values = |written: &Value| -> Vec<f64> {
        match written {
            Value::Array(records) => records
                .iter()
                .filter_map(|r| r.get("v").and_then(Value::as_f64))
                .collect(),
            other => other.as_f64().into_iter().collect(),
        }
    };
    let mut out = Reach::default();
    for (name, list) in &lists {
        for (axis, low, high) in [("x", 0, 1), ("y", 2, 3)] {
            let Some(written) = list.get(axis) else {
                continue;
            };
            for v in values(written) {
                let edges = [&mut out.left, &mut out.right, &mut out.top, &mut out.bottom];
                let (edge, amount) = if v < 0.0 {
                    (low, -v)
                } else if v > 0.0 {
                    (high, v)
                } else {
                    continue;
                };
                let slot = edges.into_iter().nth(edge).expect("four edges");
                if slot.as_ref().is_none_or(|(seen, _)| amount > *seen) {
                    *slot = Some((amount, format!("{name}.{axis}")));
                }
            }
        }
        for property in ["scale", "rotation"] {
            if list.get(property).is_some() && !out.unchecked.contains(&property) {
                out.unchecked.push(property);
            }
        }
    }
    Some(out)
}

/// Lay a staggered element out as the painter does, with its spacing at `instant`, and group
/// its units into bodies. `Err` carries why it could not be placed: a font.
pub(crate) fn bodies_at(
    document: &crate::permissive::Loose,
    element: &Value,
    plan: &Plan,
    instant: i64,
) -> Result<(montagent_text::Placement, montagent_text::units::Bodies), String> {
    let placement = crate::text_path::place(document, element, (i128::from(instant), 1))?;
    let bodies = montagent_text::units::bodies(&plan.text, plan.by, &placement);
    Ok((placement, bodies))
}

/// Which units move together, and why.
#[derive(Debug, Clone)]
pub(crate) struct Grouping {
    pub(crate) bodies: Vec<montagent_text::units::Body>,
    pub(crate) body_of_unit: Vec<usize>,
}

impl Grouping {
    /// The other units `unit` moves with, ascending.
    pub(crate) fn merged_with(&self, unit: usize) -> Vec<usize> {
        self.bodies[self.body_of_unit[unit]]
            .units
            .iter()
            .copied()
            .filter(|&u| u != unit)
            .collect()
    }

    /// The unit whose timing `unit` is drawn on: its body's first.
    pub(crate) fn lead(&self, unit: usize) -> usize {
        self.bodies[self.body_of_unit[unit]].units[0]
    }
}

/// [`Grouping`] for a staggered element: from its placed glyphs where its fonts open
/// (shaping merges are static, so any instant gives the same answer), otherwise from its
/// joined pieces alone.
pub(crate) fn grouping(
    document: &crate::permissive::Loose,
    element: &Value,
    plan: &Plan,
    instant: i64,
) -> Grouping {
    if let Ok((_, bodies)) = bodies_at(document, element, plan, instant) {
        return Grouping {
            bodies: bodies.bodies,
            body_of_unit: bodies.body_of_unit,
        };
    }
    let count = plan.units.len();
    let mut body_of_unit: Vec<Option<usize>> = vec![None; count];
    let mut bodies: Vec<montagent_text::units::Body> = Vec::new();
    let pieces = if plan.by == By::Letter {
        montagent_text::units::pieces(&plan.text)
    } else {
        Vec::new()
    };
    for unit in 0..count {
        if body_of_unit[unit].is_some() {
            continue;
        }
        let first = plan
            .segmentation
            .graphemes
            .iter()
            .position(|g| g.unit == Some(unit));
        let piece = first.and_then(|g| pieces.iter().find(|p| p.contains(&g)));
        let units: Vec<usize> = match piece {
            Some(piece) => plan.segmentation.graphemes[piece.clone()]
                .iter()
                .filter_map(|g| g.unit)
                .collect(),
            None => vec![unit],
        };
        for &u in &units {
            body_of_unit[u] = Some(bodies.len());
        }
        bodies.push(montagent_text::units::Body {
            joined: units.len() > 1,
            merged: false,
            units,
            rect: None,
        });
    }
    Grouping {
        bodies,
        body_of_unit: body_of_unit.into_iter().map(|b| b.unwrap_or(0)).collect(),
    }
}

/// `query --at`'s summary of a staggered element (ADR-0151 §5).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Stagger {
    /// `letter`, `word` or `line`.
    pub by: &'static str,
    /// How many units there are.
    pub count: usize,
    /// `[start, end]`: from the earliest start of any unit list to the latest end, after
    /// delays and including run overrides. `null` where no list is keyed.
    pub window: Option<[i64; 2]>,
}

/// One unit, as `query --at` reports it (ADR-0151 §5). Every unit is listed, whatever its
/// state, so "settled", "not started" and "missing" never look alike.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UnitRow {
    pub index: usize,
    pub text: String,
    /// The run its first grapheme sits in.
    pub run: usize,
    pub delay: i64,
    /// Per list the `units` block declares, and for `delay`: whether a run's `unit`
    /// override supplies it.
    pub overridden: serde_json::Map<String, Value>,
    /// The units it moves with — sharing a joined piece or a glyph — which all move on the
    /// first one's timing.
    pub merged_with: Vec<usize>,
    /// The pose it is drawn with at the instant: its own, or its body's first unit's.
    pub x: f64,
    pub y: f64,
    pub scale: [f64; 2],
    pub rotation: f64,
    pub opacity: f64,
}

/// `query --at`'s summary and rows for one element, or `None` where it has no stagger.
pub(crate) fn report(
    document: &crate::permissive::Loose,
    element: &Value,
    instant: i64,
) -> Option<(Stagger, Vec<UnitRow>)> {
    let plan = Plan::of(element)?;
    let grouping = grouping(document, element, &plan, instant);
    let rows = plan
        .units
        .iter()
        .map(|unit| {
            let pose = plan.pose(grouping.lead(unit.index), instant);
            let mut overridden = serde_json::Map::new();
            for property in LISTS {
                if plan.block.get(property).is_some() {
                    overridden.insert(
                        property.to_string(),
                        Value::Bool(plan.overridden(unit.index, property)),
                    );
                }
            }
            overridden.insert(
                "delay".to_string(),
                Value::Bool(plan.overridden(unit.index, "delay")),
            );
            UnitRow {
                index: unit.index,
                text: unit.text.clone(),
                run: unit.run,
                delay: unit.delay,
                overridden,
                merged_with: grouping.merged_with(unit.index),
                x: pose.x,
                y: pose.y,
                scale: pose.scale,
                rotation: pose.rotation,
                opacity: pose.opacity,
            }
        })
        .collect();
    Some((
        Stagger {
            by: by_name(plan.by),
            count: plan.units.len(),
            window: plan.window().map(|(a, b)| [a, b]),
        },
        rows,
    ))
}
