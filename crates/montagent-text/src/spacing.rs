//! Letter spacing, and the two script rules that come with it (ADR-0151 §1, ADR-0153 §3–§4).
//!
//! - **The gap.** After every grapheme cluster of a line **except the line's last**,
//!   `size × letter_spacing / 1000` pixels, where `size` is the size of the run the grapheme
//!   sits in. Spaces count, and so do run boundaries. Nothing is added after the last
//!   grapheme, so a centred or end-aligned line stays where `align` puts it.
//! - **Suppression.** No gap after a grapheme when it and the next grapheme on the line are
//!   both letters of the same joining script, whether or not they join (ADR-0153 §4, CSS
//!   Text 3 §7.2.1's fallback for a renderer that does not elongate).
//! - **Ligatures by script.** When the file switches optional ligatures off, `liga`, `clig`
//!   and `dlig` go off only in the script runs that are not a joining script (ADR-0153 §3).
//!
//! Everything here comes from the text alone: Unicode's `Script`, `Joining_Type` and
//! `General_Category`, and UAX #29 graphemes by the rule the caption pace check counts with.
//! No font is read, which is what lets `validate` say from the file how an element shapes
//! and where its spacing is suppressed.
//!
//! # Applied after placement, not as parley's letter-spacing style
//!
//! parley's own `letter_spacing` adds after **every** cluster, the line's last included, and
//! a style that changes along a line splits the shaping item there — which would break the
//! kerning of a line's last pair and tear an Arabic join wherever the suppression rule
//! switched the gap off. So the text is shaped exactly as it would be with no spacing, and
//! the gaps are added here, as an offset to every glyph after them in visual order. The
//! typographic block, `align`, `origin` and the ink box all read the spaced advance.

use std::ops::Range;
use std::sync::OnceLock;

use icu_properties::props::{GeneralCategory, GeneralCategoryGroup, JoiningType, Script};
use icu_properties::{CodePointMapData, PropertyNamesLong};
use parley::Layout;
use unicode_segmentation::UnicodeSegmentation;

/// The scripts whose letters carry a `Joining_Type` of `D` (dual) or `R` (right) — Arabic,
/// Syriac, N'Ko, Mongolian, Adlam and the others Unicode lists (ADR-0153 §1).
///
/// Read off the Unicode data rather than kept as a hand list, so a new joining script in a
/// Unicode update joins without an edit here. Common and Inherited are never in it: their
/// characters (the tatweel, ZWJ) count as the script around them.
fn joining_scripts() -> &'static [Script] {
    static SCRIPTS: OnceLock<Vec<Script>> = OnceLock::new();
    SCRIPTS.get_or_init(|| {
        let joining = CodePointMapData::<JoiningType>::new();
        let script = CodePointMapData::<Script>::new();
        let mut found: Vec<Script> = Vec::new();
        for kind in [JoiningType::DualJoining, JoiningType::RightJoining] {
            for range in joining.iter_ranges_for_value(kind) {
                for code in range {
                    let Some(c) = char::from_u32(code) else {
                        continue;
                    };
                    let s = script.get(c);
                    if s != Script::Common
                        && s != Script::Inherited
                        && s != Script::Unknown
                        && !found.contains(&s)
                    {
                        found.push(s);
                    }
                }
            }
        }
        found
    })
}

fn is_joining(script: Script) -> bool {
    joining_scripts().contains(&script)
}

/// Each character's script, with Common and Inherited resolved to the script around them:
/// the one before, or at the start of the text, the first one after. A text with no real
/// script at all stays Common.
fn resolved_scripts(text: &str) -> Vec<(usize, Script)> {
    let map = CodePointMapData::<Script>::new();
    let mut out: Vec<(usize, Script)> = Vec::with_capacity(text.len());
    let mut last: Option<Script> = None;
    let mut pending = 0;
    for (at, c) in text.char_indices() {
        let s = map.get(c);
        if s == Script::Common || s == Script::Inherited || s == Script::Unknown {
            match last {
                Some(last) => out.push((at, last)),
                None => {
                    out.push((at, Script::Common));
                    pending += 1;
                }
            }
        } else {
            if last.is_none() {
                for entry in &mut out[..pending] {
                    entry.1 = s;
                }
            }
            last = Some(s);
            out.push((at, s));
        }
    }
    out
}

fn script_at(scripts: &[(usize, Script)], at: usize) -> Script {
    match scripts.binary_search_by_key(&at, |&(offset, _)| offset) {
        Ok(index) => scripts[index].1,
        Err(index) => scripts[index.saturating_sub(1)].1,
    }
}

/// The joining script a grapheme is a letter of, if it is a joining-script letter: its base
/// character is a letter (`L*`) and its resolved script joins (ADR-0153 §1). Digits,
/// punctuation and spaces are not.
fn joining_letter(grapheme: &str, script: Script) -> Option<Script> {
    let base = grapheme.chars().next()?;
    let category = CodePointMapData::<GeneralCategory>::new().get(base);
    (GeneralCategoryGroup::Letter.contains(category) && is_joining(script)).then_some(script)
}

/// One line's graphemes, as `(range in the line, suppressed-after)`: whether the gap after
/// it is suppressed because it and the next grapheme are letters of one joining script.
fn graphemes(line: &str) -> Vec<(Range<usize>, Option<Script>)> {
    let scripts = resolved_scripts(line);
    let clusters: Vec<(usize, &str)> = line.grapheme_indices(true).collect();
    let letters: Vec<Option<Script>> = clusters
        .iter()
        .map(|&(at, g)| joining_letter(g, script_at(&scripts, at)))
        .collect();
    clusters
        .iter()
        .enumerate()
        .map(|(i, &(at, g))| {
            let suppressed = match (letters[i], letters.get(i + 1).copied().flatten()) {
                (Some(a), Some(b)) if a == b => Some(a),
                _ => None,
            };
            (at..at + g.len(), suppressed)
        })
        .collect()
}

/// One gap: `px` pixels after the grapheme whose **last character** starts at `at`, in the
/// line's own offsets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Gap {
    pub(crate) at: usize,
    pub(crate) px: f64,
}

/// Every gap one line takes.
///
/// `size_at` answers the size of the run a line offset sits in. Empty when the spacing is
/// zero, so a line that sets none is placed from exactly the numbers it always was.
pub(crate) fn gaps(line: &str, letter_spacing: f64, size_at: impl Fn(usize) -> i64) -> Vec<Gap> {
    if letter_spacing == 0.0 {
        return Vec::new();
    }
    let all = graphemes(line);
    let Some((_, before_last)) = all.split_last() else {
        return Vec::new();
    };
    before_last
        .iter()
        .filter(|(_, suppressed)| suppressed.is_none())
        .map(|(range, _)| {
            let last_char = line[range.clone()]
                .char_indices()
                .last()
                .map_or(range.start, |(at, _)| range.start + at);
            Gap {
                at: last_char,
                px: size_at(range.start) as f64 * letter_spacing / 1000.0,
            }
        })
        .collect()
}

/// What one line's glyphs move by, in the order [`parley::GlyphRun::positioned_glyphs`]
/// yields them across the layout, and the width the line gains.
///
/// The gaps are keyed by offsets in the laid-out string (`to_laid` maps a line offset to
/// it). A gap belongs to the cluster whose text holds the grapheme's last character. In a
/// left-to-right cluster it opens on the cluster's right, so the cluster's own glyphs stay
/// and everything after them moves; in a right-to-left one "after" is its left, so the gap
/// opens before the cluster's glyphs in visual order.
pub(crate) fn shifts(
    layout: &Layout<u32>,
    gaps: &[Gap],
    to_laid: impl Fn(usize) -> usize,
) -> (Vec<f64>, f64) {
    let keyed: Vec<(usize, f64)> = gaps.iter().map(|gap| (to_laid(gap.at), gap.px)).collect();
    let mut out = Vec::new();
    let mut moved = 0.0;
    for line in layout.lines() {
        // By line run rather than by glyph run: parley splits one line run into several
        // glyph runs wherever the style changes, and yields their glyphs in exactly this
        // order — each line run's visual clusters, flattened.
        for run in line.runs() {
            for cluster in run.visual_clusters() {
                let range = cluster.text_range();
                let gap: f64 = keyed
                    .iter()
                    .filter(|(at, _)| range.contains(at))
                    .map(|(_, px)| px)
                    .sum();
                let rtl = cluster.is_rtl();
                if rtl {
                    moved += gap;
                }
                for _ in cluster.glyphs() {
                    out.push(moved);
                }
                if !rtl {
                    moved += gap;
                }
            }
        }
    }
    debug_assert_eq!(
        out.len(),
        layout
            .lines()
            .flat_map(|line| line.items())
            .map(|item| match item {
                parley::PositionedLayoutItem::GlyphRun(run) => run.positioned_glyphs().count(),
                parley::PositionedLayoutItem::InlineBox(_) => 0,
            })
            .sum::<usize>(),
        "one shift per positioned glyph"
    );
    (out, moved)
}

/// The stretches of a laid-out string that are **not** in a joining script — where optional
/// ligatures go off when the file asks for it (ADR-0153 §3).
pub(crate) fn non_joining(text: &str) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    for (at, script) in resolved_scripts(text) {
        if is_joining(script) {
            continue;
        }
        let end = at + text[at..].chars().next().map_or(0, char::len_utf8);
        match out.last_mut() {
            Some(last) if last.end == at => last.end = end,
            _ => out.push(at..end),
        }
    }
    out
}

/// The first pair of graphemes in a text whose spacing ADR-0153 §4 suppresses.
///
/// What `R-SPACING-SUPPRESSED` names: the joining script, and the first word it affects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suppressed {
    /// The script's long Unicode name, `Arabic`.
    pub script: String,
    /// The whitespace-separated word the first suppressed pair sits in.
    pub word: String,
}

/// The first suppressed pair in `text`, line by line, or `None` where every gap is added.
///
/// From the text alone, independent of the spacing value: the caller decides whether the
/// element has a non-zero spacing for the review to be about.
pub fn first_suppressed(text: &str) -> Option<Suppressed> {
    for line in crate::lines::partition(text) {
        let all = graphemes(line.text);
        let Some((range, script)) = all
            .iter()
            .take(all.len().saturating_sub(1))
            .find_map(|(range, s)| s.map(|s| (range.clone(), s)))
        else {
            continue;
        };
        let start = line.text[..range.start]
            .rfind(char::is_whitespace)
            .map_or(0, |at| {
                at + line.text[at..].chars().next().map_or(1, char::len_utf8)
            });
        let end = line.text[range.start..]
            .find(char::is_whitespace)
            .map_or(line.text.len(), |at| range.start + at);
        return Some(Suppressed {
            script: PropertyNamesLong::<Script>::new()
                .get(script)
                .unwrap_or("unknown")
                .to_string(),
            word: line.text[start..end].to_string(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_is_a_joining_script_and_latin_is_not() {
        assert!(is_joining(Script::Arabic));
        assert!(is_joining(Script::Syriac));
        assert!(!is_joining(Script::Latin));
        assert!(!is_joining(Script::Common));
    }

    #[test]
    fn a_line_of_one_grapheme_takes_no_gap() {
        assert!(gaps("A", 200.0, |_| 100).is_empty());
    }

    #[test]
    fn zero_spacing_takes_no_gap() {
        assert!(gaps("ABC", 0.0, |_| 100).is_empty());
    }
}
