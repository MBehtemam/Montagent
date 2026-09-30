//! A tile's label: what it says, and drawing it in the strip beneath the tile (ADR-0098).
//!
//! **The line is `<index> <sigil> <instant>ms <+offset> <±id>`.** A run tile is the
//! unmarked default, so its sigil is absent and the line reads `9 42800ms +37
//! +word-08-bridge`. The offset is the sampled instant less the run's start, always printed,
//! `+0` included (ADR-0098 §3). How big the line is drawn, and whether the id survives, is
//! [`super::sizing`]'s: this module builds both forms of every label before the sheet is
//! sized, and draws whichever form the sizing chose.
//!
//! **The id names what changed at the state's own boundary** (ADR-0098 §8): an element
//! that entered there (`+`), or, where nothing entered, one that departed (`-`); of those,
//! the one on the highest layer, the greatest element id on a tie — never array order,
//! which ADR-0060 keeps meaningless. It is the rule `docs/research/tile-label/` measured
//! ADR-0098 on, so the fixture's labels are the ones that ADR reasoned about. The boundary
//! is the document's, not the range's: a range opening inside a state names the change the
//! state opened with, so the same tile labels the same way in any range that holds it.
//! Elements spanning the whole document are never candidates, or the first tile of every
//! project would name its chrome. A boundary with no candidate left prints [`NO_CHANGE`].
//! ADR-0128 records each of these choices.

use std::collections::BTreeSet;

use montagent_render::canvas::{
    Canvas, Extent, Fill, Glyph, PathEl, Region, Rgba, Shape, Transform,
};
use montagent_text::engine::VerticalOrigin;
use montagent_text::{Align, Fonts, Run, Spec};
use serde_json::Value;

use crate::exact;
use crate::fonts::chrome;
use crate::permissive::Loose;
use crate::stack::Stack;
use crate::verbs::query::Named;
use crate::verbs::query::cuts::{self, Interval};

use super::path_element;
use super::sizing::LABEL_INSET;

/// What a run tile's id slot reads when nothing nameable changed at its boundary: every
/// change there was an element spanning the whole document, or an element with no id.
///
/// Unsigned, and a real id never is: every id prints behind `+` or `-`, so this cannot be
/// read as an element's name, whatever the document calls its elements (ADR-0128 §3).
pub(super) const NO_CHANGE: &str = "=";

/// The ink a run tile's label is drawn in, on the strip's `#1A1A1A`.
const INK: Rgba = Rgba([0xF2, 0xF2, 0xF2, 0xFF]);

/// A keyframe or infill tile's mark: its strip inverted, this ink on [`MARKED_GROUND`]
/// (ADR-0128 §5). A change of luminance rather than hue, so it survives greyscale, and in
/// the strip, so the tile's pixels stay `frame --at`'s.
const MARKED_INK: Rgba = Rgba([0x1A, 0x1A, 0x1A, 0xFF]);
const MARKED_GROUND: Rgba = Rgba([0xE6, 0xE6, 0xE6, 0xFF]);

/// ADR-0128 §5's sigil for a keyframe tile, in the label's second place.
const KEYFRAME_SIGIL: &str = "K";

/// Both forms of one tile's label, before the sizing picks one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Label {
    /// The numeric core: index, instant and offset. Never elided.
    pub(super) core: String,
    /// The core and the identifying field.
    pub(super) whole: String,
}

impl Label {
    /// A run tile's label: `index` counted from 1, sampled at `instant` inside a run that
    /// opens at `start`, whose boundary changed as `change` says.
    pub(super) fn run(index: usize, instant: i64, start: i64, change: Option<&Change>) -> Label {
        let core = format!("{index} {instant}ms +{}", instant - start);
        let field = match change {
            Some(change) => format!("{}{}", change.sign(), change.id),
            None => NO_CHANGE.to_string(),
        };
        Label {
            whole: format!("{core} {field}"),
            core,
        }
    }

    /// A keyframe tile's label: `index` counted from 1, sampled at `instant` for change
    /// points the earliest of which is at `first`. The offset is from that change point, the
    /// boundary this tile exists for, so it says how far past the change the frame is. There
    /// is no identifying field, so the label ends after the offset and both forms are one
    /// (ADR-0106 D7): nothing here can drive the sheet-wide elision.
    pub(super) fn keyframe(index: usize, instant: i64, first: i64) -> Label {
        let core = format!("{index} {KEYFRAME_SIGIL} {instant}ms +{}", instant - first);
        Label {
            whole: core.clone(),
            core,
        }
    }
}

/// The element a label names, and which side of the boundary it is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Change {
    pub(super) id: String,
    pub(super) entered: bool,
}

impl Change {
    fn sign(&self) -> char {
        if self.entered { '+' } else { '-' }
    }
}

/// The document's own visual states, read once, from which each tile's change is named.
pub(super) struct Changes<'a> {
    /// The visual states over `[0, end)`: the whole document, and the range if it runs on.
    states: Vec<Interval>,
    /// Ids of the elements that span the whole document, `[0, extent)` or wider.
    everywhere: BTreeSet<String>,
    stack: Stack<'a>,
}

impl<'a> Changes<'a> {
    /// The changes of `document`, far enough to name every state that opens before `to`.
    pub(super) fn of(document: &'a Loose, to: i64) -> Changes<'a> {
        let extent = exact::extent(document);
        let end = extent.unwrap_or(0).max(to);
        let everywhere = match extent {
            None => BTreeSet::new(),
            Some(extent) => document
                .elements()
                .filter(|element| {
                    let at = |key: &str| element.get(key).and_then(Value::as_i64);
                    matches!((at("start"), at("end")), (Some(start), Some(end)) if start <= 0 && end >= extent)
                })
                .filter_map(|element| element.get("id").and_then(Value::as_str))
                .map(str::to_string)
                .collect(),
        };
        Changes {
            states: cuts::visual_states(&cuts::cuts(document, 0, end)),
            everywhere,
            stack: Stack::of(document),
        }
    }

    /// What changed at the boundary of the document's visual state holding `at`: the
    /// highest-layer candidate, the greatest id on a tie. `None` where no candidate is left.
    pub(super) fn at(&self, at: i64) -> Option<Change> {
        let current = self.states.iter().rposition(|state| state.start <= at)?;
        let empty = Vec::new();
        let before = match current {
            0 => &empty,
            i => &self.states[i - 1].present,
        };
        let now = &self.states[current].present;
        // What entered is on screen in the tile; a departure is named only where nothing
        // entered, and its sign says it is not there.
        let (id, entered) = match self.highest(before, now) {
            Some(id) => (id, true),
            None => (self.highest(now, before)?, false),
        };
        Some(Change {
            id: id.to_string(),
            entered,
        })
    }

    /// Of the candidates in `to` and not in `from`, the one on the highest layer, the
    /// greatest id on a tie.
    fn highest<'s>(&self, from: &'s [Named], to: &'s [Named]) -> Option<&'s str> {
        moved(from, to)
            .filter(|id| !self.everywhere.contains(*id))
            // An element whose layer cannot be resolved ranks below every one whose layer
            // can: its place in the stack is `validate`'s finding, not a height.
            .max_by_key(|&id| (self.stack.layer_of(id).ok(), id))
    }
}

/// The ids in `to` and not in `from`. An element with no id is not a candidate: the label
/// would have nothing to call it.
fn moved<'s>(from: &'s [Named], to: &'s [Named]) -> impl Iterator<Item = &'s str> + 's {
    to.iter()
        .filter(move |named| !from.contains(named))
        .filter_map(|named| named.id.as_deref())
}

/// Draw `text` at `size` px in the chrome face, into `strip` of `sheet`: left-aligned past
/// the inset, centred on the strip's height, and clipped to it, so that no glyph reaches a
/// tile's pixels. A `marked` tile's strip is inverted first. `Err` carries the reason the
/// face would not lay the text out.
pub(super) fn draw(
    sheet: &mut Canvas,
    fonts: &mut Fonts,
    text: &str,
    size: i64,
    strip: Region,
    marked: bool,
) -> Result<(), String> {
    let ink = match marked {
        true => {
            invert(sheet, strip);
            MARKED_INK
        }
        false => INK,
    };
    let runs = [Run {
        text,
        font: None,
        size: None,
        stroke_width: None,
    }];
    let placement = montagent_text::place(
        fonts,
        &Spec {
            runs: &runs,
            font: chrome::KEY,
            size,
            line_height_tenths: 10,
            stroke_width: 0,
            y: 0,
            vertical_origin: VerticalOrigin::Top,
            align: Align::Start,
        },
    )
    .map_err(|e| format!("a tile label could not be laid out in the chrome face: {e}"))?;
    let paint = Fill {
        fill: Some(ink),
        stroke: None,
        stroke_width: 0.0,
    };
    let glyphs: Vec<Glyph> = placement
        .glyphs
        .iter()
        .map(|glyph| Glyph {
            x: glyph.x,
            y: glyph.y,
            outline: glyph.outline,
            paint,
        })
        .collect();
    let outlines: Vec<Vec<PathEl>> = placement
        .outlines
        .iter()
        .map(|outline| outline.iter().copied().map(path_element).collect())
        .collect();
    sheet.text(
        &glyphs,
        &outlines,
        Extent {
            width: placement.width,
            height: placement.height,
        },
        &Transform {
            x: (strip.x + LABEL_INSET) as f64,
            y: strip.y as f64 + (strip.height as f64 - placement.height) / 2.0,
            origin: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation: 0.0,
            opacity: 1.0,
        },
        Some(strip),
        &[],
    );
    Ok(())
}

/// Paint `strip` in [`MARKED_GROUND`]: a keyframe or infill tile's mark (ADR-0128 §5).
fn invert(sheet: &mut Canvas, strip: Region) {
    sheet.shape(
        Shape::Rect { radius: 0.0 },
        Extent {
            width: strip.width as f64,
            height: strip.height as f64,
        },
        &Transform {
            x: strip.x as f64,
            y: strip.y as f64,
            origin: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation: 0.0,
            opacity: 1.0,
        },
        &Fill {
            fill: Some(MARKED_GROUND),
            stroke: None,
            stroke_width: 0.0,
        },
        None,
        &[],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_label_is_index_instant_offset_and_signed_id() {
        let entered = Change {
            id: "word-08-bridge".into(),
            entered: true,
        };
        let label = Label::run(9, 42800, 42763, Some(&entered));
        assert_eq!(label.whole, "9 42800ms +37 +word-08-bridge");
        assert_eq!(label.core, "9 42800ms +37");

        let departed = Change {
            id: "a".into(),
            entered: false,
        };
        assert_eq!(Label::run(1, 0, 0, Some(&departed)).whole, "1 0ms +0 -a");
        assert_eq!(Label::run(2, 40, 40, None).whole, "2 40ms +0 =");
    }

    #[test]
    fn a_keyframe_label_carries_its_sigil_and_no_id() {
        // ADR-0106 D10's worked example, a keyframe at 1013 ms on a 25 fps grid.
        let label = Label::keyframe(12, 1040, 1013);
        assert_eq!(label.whole, "12 K 1040ms +27");
        assert_eq!(label.core, label.whole);
    }
}
