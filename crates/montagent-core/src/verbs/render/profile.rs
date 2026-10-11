//! `render`'s `profile` (ADR-0194, #931): where the painters' CPU went, by element, element
//! type and effect.
//!
//! Collected only where the request asked, by the painter that paints ([`Painter::paint`]),
//! so a render without `profile` reads one `Option` per element and nothing else. Each
//! painter keeps its own [`Profile`] and the encoder's side merges them with [`Stages`], so
//! the block is summed across painters, like the stage split beside it.
//!
//! **CPU is the painter thread's own time**: the wall clock between an element's first and
//! last paint call, less what the same call spent waiting on decoded frames. The wait is the
//! `decode` stage, which the block reports beside the rest rather than inside any element.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::{Value, json};

use super::Stages;

/// How many element rows the block lists before it folds the rest into one remainder row.
pub(crate) const TOP: usize = 25;

#[derive(Debug, Default, Clone, Copy)]
struct Cost {
    spent: Duration,
    /// Frames the element was drawn in; for an effect, the times it was applied.
    count: u64,
}

impl Cost {
    fn add(&mut self, spent: Duration, count: u64) {
        self.spent += spent;
        self.count += count;
    }
}

/// One painter's costs, or the sum of several.
#[derive(Debug, Default, Clone)]
pub(crate) struct Profile {
    /// By element id, with the element's type.
    elements: BTreeMap<String, (String, Cost)>,
    types: BTreeMap<String, Cost>,
    effects: BTreeMap<&'static str, Cost>,
}

impl Profile {
    /// One element's turn in one frame.
    pub(crate) fn element(&mut self, id: &str, kind: &str, spent: Duration, drawn: bool) {
        let frames = u64::from(drawn);
        self.elements
            .entry(id.to_string())
            .or_insert_with(|| (kind.to_string(), Cost::default()))
            .1
            .add(spent, frames);
        self.types
            .entry(kind.to_string())
            .or_default()
            .add(spent, frames);
    }

    /// One effect's layer unwinding, inside whichever element was being painted.
    pub(crate) fn effect(&mut self, name: &'static str, spent: Duration) {
        self.effects.entry(name).or_default().add(spent, 1);
    }

    pub(crate) fn merge(&mut self, other: &Profile) {
        for (id, (kind, cost)) in &other.elements {
            self.elements
                .entry(id.clone())
                .or_insert_with(|| (kind.clone(), Cost::default()))
                .1
                .add(cost.spent, cost.count);
        }
        for (kind, cost) in &other.types {
            self.types
                .entry(kind.clone())
                .or_default()
                .add(cost.spent, cost.count);
        }
        for (name, cost) in &other.effects {
            self.effects
                .entry(name)
                .or_default()
                .add(cost.spent, cost.count);
        }
    }

    /// The `render.profile` block, with the stage split it is measured against.
    ///
    /// `attributed_pct` is the elements' share of the paint stage, and the rest is named
    /// `unattributed`: the background, the transitions' resolution and the per-frame
    /// bookkeeping, so the share cannot be made large by leaving cost out of sight.
    pub(crate) fn to_json(&self, stages: &Stages) -> Value {
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        let ms1 = |d: Duration| (ms(d) * 10.0).round() / 10.0;
        let attributed: Duration = self.elements.values().map(|(_, cost)| cost.spent).sum();
        let unattributed = stages.paint.saturating_sub(attributed);
        let attributed_pct = if stages.paint.is_zero() {
            100.0
        } else {
            (ms(attributed.min(stages.paint)) / ms(stages.paint) * 1000.0).round() / 10.0
        };

        let mut elements: Vec<(&String, &(String, Cost))> = self.elements.iter().collect();
        elements.sort_by(|a, b| b.1.1.spent.cmp(&a.1.1.spent).then_with(|| a.0.cmp(b.0)));
        let (listed, rest) = elements.split_at(elements.len().min(TOP));
        let rows: Vec<Value> = listed
            .iter()
            .map(|(id, (kind, cost))| {
                json!({
                    "id": id,
                    "type": kind,
                    "cpu_ms": ms1(cost.spent),
                    "frames_painted": cost.count,
                })
            })
            .collect();
        let remainder = (!rest.is_empty()).then(|| {
            let spent: Duration = rest.iter().map(|(_, (_, cost))| cost.spent).sum();
            let frames: u64 = rest.iter().map(|(_, (_, cost))| cost.count).sum();
            json!({"elements": rest.len(), "cpu_ms": ms1(spent), "frames_painted": frames})
        });

        let mut types: Vec<(&String, &Cost)> = self.types.iter().collect();
        types.sort_by(|a, b| b.1.spent.cmp(&a.1.spent).then_with(|| a.0.cmp(b.0)));
        let mut effects: Vec<(&&'static str, &Cost)> = self.effects.iter().collect();
        effects.sort_by(|a, b| b.1.spent.cmp(&a.1.spent).then_with(|| a.0.cmp(b.0)));

        let mut block = json!({
            "unit": "ms",
            "note": "CPU time summed across painters. A profile run is timed, not speed-gated, \
                     and paints the same bytes as one without. `effects` are inside their \
                     element's cost, not added to it.",
            "stages": {
                "decode_ms": ms1(stages.decode),
                "paint_ms": ms1(stages.paint),
                "readback_ms": ms1(stages.readback),
                "paint_wait_ms": ms1(stages.paint_wait),
                "encode_wait_ms": ms1(stages.encode_wait),
                "seal_ms": ms1(stages.seal),
            },
            "attributed_pct": attributed_pct,
            "unattributed": {
                "label": "unattributed / compositor: background, transitions, per-frame work",
                "cpu_ms": ms1(unattributed),
            },
            "elements": rows,
            "types": types.iter().map(|(kind, cost)| json!({
                "type": kind, "cpu_ms": ms1(cost.spent), "frames_painted": cost.count,
            })).collect::<Vec<_>>(),
            "effects": effects.iter().map(|(name, cost)| json!({
                "effect": name, "cpu_ms": ms1(cost.spent), "applications": cost.count,
            })).collect::<Vec<_>>(),
        });
        if let (Some(remainder), Some(object)) = (remainder, block.as_object_mut()) {
            object.insert("elements_remainder".to_string(), remainder);
        }
        block
    }
}
