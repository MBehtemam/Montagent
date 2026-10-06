//! What one unit of a stagger is (ADR-0151 §3, amended by ADR-0153 §2), from the text alone,
//! and which placed glyphs move together.
//!
//! - **[`segment`]** counts units in reading order over the element's whole text, ignoring
//!   runs: a `letter` is a grapheme cluster that is not whitespace; a `word` is a UAX #29
//!   word segment holding a letter, a digit or an `Extended_Pictographic` character, with
//!   punctuation given to the word before it on its line and otherwise to the word after
//!   it; a `line` is a `\n` line holding at least one letter. Whitespace and empty lines
//!   take no step. Graphemes come from [`graphemes`], the rule the caption pace check counts
//!   with, so the two can never disagree on what a grapheme is.
//! - **[`pieces`]** are ADR-0153's joined pieces: maximal runs of graphemes that the Unicode
//!   cursive-joining rules connect. No font is read.
//! - **[`bodies`]** groups a placed element's units into what is drawn as one: a joined
//!   piece under `by: letter`, and any units shaping merged into one glyph. Each body has
//!   the box its pivot is picked in.

use std::ops::Range;

use icu_properties::props::{
    ExtendedPictographic, GeneralCategory, GeneralCategoryGroup, JoiningType,
};
use icu_properties::{CodePointMapData, CodePointSetData};
use unicode_segmentation::UnicodeSegmentation;

use crate::place::Placement;

/// What one unit is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    Letter,
    Word,
    Line,
}

/// One grapheme of the element's whole text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grapheme {
    /// Its bytes in the whole text.
    pub range: Range<usize>,
    /// The unit it belongs to, or `None` where it takes no step: whitespace, and under
    /// `by: line` a line with no letter.
    pub unit: Option<usize>,
}

/// The whole text's graphemes, each with its unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segmentation {
    pub graphemes: Vec<Grapheme>,
    /// How many units there are.
    pub count: usize,
}

impl Segmentation {
    /// The grapheme a byte offset into the whole text sits in.
    pub fn grapheme_at(&self, at: usize) -> Option<usize> {
        match self.graphemes.binary_search_by(|g| g.range.start.cmp(&at)) {
            Ok(i) => Some(i),
            Err(0) => None,
            Err(i) => Some(i - 1),
        }
    }
}

/// Every grapheme cluster of `text`, with its byte offset: UAX #29's extended clusters.
///
/// **The one grapheme rule**: `R-CAPTION-PACE` counts with it and [`segment`] steps with it,
/// so a caption's count and a stagger's count can never disagree on what a grapheme is.
pub fn graphemes(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.grapheme_indices(true)
}

fn is_whitespace(grapheme: &str) -> bool {
    grapheme.chars().all(char::is_whitespace)
}

/// A letter, a digit or an `Extended_Pictographic` character: what makes a word segment a
/// word (ADR-0151 §3).
fn is_word_char(c: char) -> bool {
    let category = CodePointMapData::<GeneralCategory>::new().get(c);
    GeneralCategoryGroup::Letter.contains(category)
        || category == GeneralCategory::DecimalNumber
        || CodePointSetData::new::<ExtendedPictographic>().contains(c)
}

/// The units of `text`, in reading order.
pub fn segment(text: &str, by: By) -> Segmentation {
    let mut graphemes: Vec<Grapheme> = graphemes(text)
        .map(|(at, g)| Grapheme {
            range: at..at + g.len(),
            unit: None,
        })
        .collect();
    let mut count = 0;
    match by {
        By::Letter => {
            for g in &mut graphemes {
                if !is_whitespace(&text[g.range.clone()]) {
                    g.unit = Some(count);
                    count += 1;
                }
            }
        }
        By::Word => count = words(text, &mut graphemes),
        By::Line => {
            let mut line_start = 0;
            for line in text.split('\n') {
                let line_end = line_start + line.len();
                let on_line =
                    |g: &Grapheme| g.range.start >= line_start && g.range.start < line_end;
                let lettered = graphemes
                    .iter()
                    .any(|g| on_line(g) && !is_whitespace(&text[g.range.clone()]));
                if lettered {
                    for g in graphemes.iter_mut().filter(|g| on_line(g)) {
                        if !is_whitespace(&text[g.range.clone()]) {
                            g.unit = Some(count);
                        }
                    }
                    count += 1;
                }
                line_start = line_end + 1;
            }
        }
    }
    Segmentation { graphemes, count }
}

/// `by: word`: every word segment takes a unit, and every other non-whitespace segment —
/// punctuation, symbols — is given to the word before it on its line, otherwise to the next
/// word in the text, otherwise to the last word before it. Returns the count.
fn words(text: &str, graphemes: &mut [Grapheme]) -> usize {
    #[derive(Clone, Copy, PartialEq)]
    enum Kind {
        Space,
        Word,
        Attached,
    }
    // Every segment, with its line, its kind and (for a word) its unit.
    let mut segments: Vec<(Range<usize>, usize, Kind, Option<usize>)> = Vec::new();
    let mut count = 0;
    let mut line = 0;
    for (at, s) in text.split_word_bound_indices() {
        let kind = if s.chars().any(is_word_char) {
            Kind::Word
        } else if s.chars().all(char::is_whitespace) {
            Kind::Space
        } else {
            Kind::Attached
        };
        let unit = (kind == Kind::Word).then(|| {
            count += 1;
            count - 1
        });
        segments.push((at..at + s.len(), line, kind, unit));
        line += s.matches('\n').count();
    }
    for i in 0..segments.len() {
        if segments[i].2 != Kind::Attached {
            continue;
        }
        let line = segments[i].1;
        let word = |j: &usize| {
            (segments[*j].2 == Kind::Word)
                .then_some(segments[*j].3)
                .flatten()
        };
        let before_on_line = (0..i)
            .rev()
            .take_while(|j| segments[*j].1 == line)
            .find_map(|j| word(&j));
        let after = (i + 1..segments.len()).find_map(|j| word(&j));
        let before = (0..i).rev().find_map(|j| word(&j));
        segments[i].3 = before_on_line.or(after).or(before);
    }
    for (range, _, kind, unit) in &segments {
        if *kind == Kind::Space {
            continue;
        }
        for g in graphemes
            .iter_mut()
            .filter(|g| range.contains(&g.range.start))
        {
            if !is_whitespace(&text[g.range.clone()]) {
                g.unit = *unit;
            }
        }
    }
    count
}

/// How one end of a grapheme joins: its joining type, skipping transparent marks.
fn joining_end(grapheme: &str, last: bool) -> JoiningType {
    let map = CodePointMapData::<JoiningType>::new();
    let types = grapheme.chars().map(|c| map.get(c));
    let found = if last {
        types.rev().find(|t| *t != JoiningType::Transparent)
    } else {
        types.into_iter().find(|t| *t != JoiningType::Transparent)
    };
    found.unwrap_or(JoiningType::NonJoining)
}

/// Whether grapheme `a`, followed directly by grapheme `b`, joins it (ADR-0153 §1, the
/// rules of `ArabicShaping.txt`): `a` joins what follows it (dual, left or join-causing)
/// and `b` joins what precedes it (dual, right or join-causing). ZWJ and the tatweel are
/// join-causing; ZWNJ and every non-joining character break the join.
fn joins(a: &str, b: &str) -> bool {
    use JoiningType as J;
    matches!(
        joining_end(a, true),
        J::DualJoining | J::LeftJoining | J::JoinCausing
    ) && matches!(
        joining_end(b, false),
        J::DualJoining | J::RightJoining | J::JoinCausing
    )
}

/// ADR-0153's joined pieces of more than one grapheme, as ranges of grapheme indexes
/// (into [`graphemes`]), in reading order. A grapheme that joins nothing is a piece of its
/// own and is not listed. From the text alone.
pub fn pieces(text: &str) -> Vec<Range<usize>> {
    let all: Vec<&str> = graphemes(text).map(|(_, g)| g).collect();
    let mut out = Vec::new();
    let mut start = 0;
    for i in 1..=all.len() {
        if i < all.len() && joins(all[i - 1], all[i]) {
            continue;
        }
        if i - start > 1 {
            out.push(start..i);
        }
        start = i;
    }
    out
}

/// Units drawn as one: one transform, one pivot, and one layer while fading.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    /// Its units, ascending. The first is the one whose timing the body moves on: the
    /// first in reading order (ADR-0151 §3, ADR-0153 §2).
    pub units: Vec<usize>,
    /// Some of its units share a joined piece (ADR-0153).
    pub joined: bool,
    /// Some of its units share a glyph (ADR-0151 §3).
    pub merged: bool,
    /// `[left, top, right, bottom]` in the block's own coordinates: from its first
    /// grapheme's leading edge to its last's trailing edge, each grapheme its own advance
    /// without the added spacing, by the line's slot. `None` where nothing of it was placed.
    pub rect: Option<[f64; 4]>,
}

/// A placed element's bodies.
#[derive(Debug, Clone, PartialEq)]
pub struct Bodies {
    pub segmentation: Segmentation,
    /// In the order of their first unit.
    pub bodies: Vec<Body>,
    /// Which body each unit is drawn in.
    pub body_of_unit: Vec<usize>,
    /// Which body each of [`Placement::glyphs`] is drawn in; `None` for a glyph of no unit
    /// (whitespace), which is drawn where it was placed.
    pub glyph_body: Vec<Option<usize>>,
}

/// Group a placed element's units into bodies.
///
/// Two units share a body when shaping put them in one glyph (a ligature group: parley's
/// ligature start and its components, which may span graphemes of two units), or, under
/// `by: letter`, when they are in one joined piece. `text` is the element's whole text, the
/// one `placement` was laid out from.
pub fn bodies(text: &str, by: By, placement: &Placement) -> Bodies {
    let segmentation = segment(text, by);
    let count = segmentation.count;
    let unit_at = |at: usize| {
        segmentation
            .grapheme_at(at)
            .and_then(|g| segmentation.graphemes[g].unit)
    };

    // Union-find over units, remembering why two were joined.
    let mut parent: Vec<usize> = (0..count).collect();
    fn root(parent: &mut [usize], u: usize) -> usize {
        let mut r = u;
        while parent[r] != r {
            r = parent[r];
        }
        parent[u] = r;
        r
    }
    let union = |parent: &mut Vec<usize>, a: usize, b: usize| {
        let (ra, rb) = (root(parent, a), root(parent, b));
        if ra != rb {
            parent[ra.max(rb)] = ra.min(rb);
        }
    };
    let mut merged_units = vec![false; count];
    let mut joined_units = vec![false; count];

    // Shaping merges: every ligature group's units.
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for cluster in &placement.clusters {
        let (Some(group), Some(range)) = (cluster.ligature, cluster.text.clone()) else {
            continue;
        };
        let Some(unit) = unit_at(range.start) else {
            continue;
        };
        if groups.len() <= group {
            groups.resize(group + 1, Vec::new());
        }
        if !groups[group].contains(&unit) {
            groups[group].push(unit);
        }
    }
    for group in groups.iter().filter(|g| g.len() > 1) {
        for &u in group {
            merged_units[u] = true;
            union(&mut parent, group[0], u);
        }
    }

    // Joined pieces, under `by: letter` only: under `word` and `line` a piece never
    // crosses a unit.
    if by == By::Letter {
        for piece in pieces(text) {
            let units: Vec<usize> = segmentation.graphemes[piece]
                .iter()
                .filter_map(|g| g.unit)
                .collect();
            for &u in &units {
                joined_units[u] = true;
                union(&mut parent, units[0], u);
            }
        }
    }

    let mut bodies: Vec<Body> = Vec::new();
    let mut body_of_root: Vec<Option<usize>> = vec![None; count];
    let body_of_unit: Vec<usize> = (0..count)
        .map(|unit| {
            let r = root(&mut parent, unit);
            let body = *body_of_root[r].get_or_insert_with(|| {
                bodies.push(Body {
                    units: Vec::new(),
                    joined: false,
                    merged: false,
                    rect: None,
                });
                bodies.len() - 1
            });
            bodies[body].units.push(unit);
            body
        })
        .collect();
    for body in &mut bodies {
        if body.units.len() > 1 {
            body.joined = body.units.iter().any(|&u| joined_units[u]);
            body.merged = body.units.iter().any(|&u| merged_units[u]);
        }
    }

    // Each body's box: every cluster of its graphemes, by its line's slot.
    let block_top = placement.measurement.block_top;
    for cluster in &placement.clusters {
        let Some(unit) = cluster.text.clone().and_then(|r| unit_at(r.start)) else {
            continue;
        };
        let line = &placement.measurement.lines[cluster.line];
        let top = line.slot_top - block_top;
        let bottom = top + line.slot_height;
        let (left, right) = (cluster.x, cluster.x + cluster.advance);
        let rect = &mut bodies[body_of_unit[unit]].rect;
        *rect = Some(match *rect {
            None => [left, top, right, bottom],
            Some([l, t, r, b]) => [l.min(left), t.min(top), r.max(right), b.max(bottom)],
        });
    }

    // Each glyph's body: its cluster's unit, or for a ligature the group's.
    let glyph_body = placement
        .glyphs
        .iter()
        .map(|glyph| {
            let cluster = placement.clusters.get(glyph.cluster)?;
            let unit = match cluster.ligature {
                Some(group) => groups.get(group).and_then(|g| g.iter().min().copied()),
                None => None,
            }
            .or_else(|| cluster.text.clone().and_then(|r| unit_at(r.start)))?;
            Some(body_of_unit[unit])
        })
        .collect();

    Bodies {
        segmentation,
        bodies,
        body_of_unit,
        glyph_body,
    }
}
