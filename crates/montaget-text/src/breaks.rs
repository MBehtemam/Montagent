//! Break opportunities — *"a place text **may** break"* — with the segmenter named beside
//! them (ADR-0008).
//!
//! The renderer never computes one: under no-auto-wrap nothing chooses, and the line
//! partition is fixed by the mandatory breaks the author already wrote. **The agent
//! placing the break always needs one**, and ADR-0008 is explicit that these are two
//! different questions that *"look like one question in English, where the break
//! opportunities are the spaces and are free to see"* and *"come apart completely in Thai,
//! which writes no spaces at all"*.
//!
//! So this is an authoring-time answer, and its output is inert: what the agent does with
//! it is freeze a `\n` into a committed file, where a later disagreement between segmenter
//! versions cannot move a shipped video. That is the same argument that admits `measure`
//! at all — and it is why the segmenter is **named in the output**. Two versions of this
//! data legitimately disagree; a set of offsets that did not say which one produced it
//! would be unfalsifiable.
//!
//! **`icu_segmenter` directly, not parley's layout.** ADR-0008: *"`icu_segmenter` can be
//! called directly by `measure` whichever crate renders"*, because the `SA` dictionary fork
//! is an authoring-tool requirement rather than a renderer criterion. A parley layout would
//! report neither the offsets nor the data version, and it is the same shared node parley
//! links either way.
//!
//! **Dictionary-backed**, as ADR-0008 says in as many words. The choice is live rather than
//! nominal: `icu_segmenter` publishes a dictionary model and an LSTM model that disagree
//! about Thai, and it is the dictionary whose offsets reproduce the committed evidence in
//! `docs/research/prototypes/thai-line-breaking/results/` — `parley-on-opportunities.txt`'s
//! Thai break set is this constructor's, not `new_auto`'s.
//!
//! **Where the agent's judgement starts.** ADR-0008: *"Two judges placed a legal break in
//! different places in the same Thai sentence, for good and opposing reasons. `measure`
//! reports what is legal; the agent chooses among the legal ones. No tool should pretend
//! otherwise."*

use icu_segmenter::LineSegmenter;
use icu_segmenter::options::LineBreakOptions;
use serde::Serialize;

/// Which segmenter, and which data, produced a set of offsets.
///
/// ADR-0008 asks for the *"segmenter and data version alongside the offsets"*. All four
/// fields, because all four can move independently: a crate version, the data crate that
/// version happens to carry, and which of its two models was asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Segmenter {
    /// The crate that computed them.
    pub name: &'static str,
    /// Its exact version, as the workspace pins it.
    pub version: &'static str,
    /// The compiled data behind it, and its version.
    pub data: &'static str,
    /// Which model answered for the complex scripts — UAX #14 class `SA`, which the
    /// standard hands to morphological analysis *"beyond the scope of the Unicode
    /// Standard"*.
    pub model: &'static str,
}

/// The segmenter this build reports offsets from.
///
/// The versions are stated here and asserted against the workspace manifest by
/// `tests/segmenter_pin.rs`, so the name in the output cannot drift from the crate that
/// produced it without a test failing.
pub const SEGMENTER: Segmenter = Segmenter {
    name: "icu_segmenter",
    version: "2.3.0",
    data: "icu_segmenter_data 2.3.0",
    model: "dictionary",
};

/// Every place `line` may legally break, as byte offsets into it.
///
/// **Offsets where a break may be *inserted*.** Position 0 and the end of the text are
/// both boundaries under UAX #14 and neither is a place to put a `\n`, so neither is
/// reported: an agent acting on one would either prepend an empty line or append one.
///
/// One line at a time, because a line is an independent paragraph (ADR-0008) — and because
/// running the segmenter over a whole multi-line text would report the mandatory breaks the
/// author already wrote back to them as opportunities.
pub fn opportunities(line: &str) -> Vec<usize> {
    let segmenter = LineSegmenter::new_dictionary(LineBreakOptions::default());
    segmenter
        .segment_str(line)
        .filter(|&at| at != 0 && at != line.len())
        .collect()
}
