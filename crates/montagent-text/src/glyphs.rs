//! What a font file can actually draw — its character map, read off the file — and how to
//! spell a character in a report.
//!
//! ADR-0007 makes the answer an `error`: *"A character with no glyph in any chain entry
//! renders `.notdef` and is a `validate` **error** — under ADR-0006's definition it is
//! guaranteed wrong, and unlike a gap there is no intent it could express."* The check
//! that says so is the core's ([`montagent_core::checks::fonts`]); what is here is the
//! reading, on [`names`](crate::names)'s rule — *read, never judged*.
//!
//! **The `cmap`, not the shaper.** Coverage is asked of the file's character map rather
//! than of a completed layout, and the two answer different questions: a shaper reports
//! what it substituted, which depends on the features a script turns on, while `cmap`
//! reports what the file claims to map. The claim is the one ADR-0007 makes — *"renders
//! `.notdef`"* is a `cmap` miss — and it is the one a report can state without the reader
//! having to know which shaping plan ran.
//!
//! **A collection is asked face by face.** A `.ttc` entry names its face with `index`
//! (ADR-0007), so coverage is asked of that face alone: two faces of one collection may
//! map different characters, and answering from the file as a whole would report a glyph
//! the chain entry cannot reach.

use skrifa::{FontRef, MetadataProvider};

/// One face's character map, as the set of characters it claims to map.
///
/// Named for the OpenType table it is, rather than for the question it answers: *coverage*
/// already names an unrelated thing in this project — ADR-0018's cross-track coverage, and
/// the check that asks it.
///
/// Built once per file and asked per character: a chain is re-asked for every run of every
/// element, and re-parsing the file per character is the one way to make an `O(text)` check
/// `O(text × fonts)`.
pub struct Charmap {
    /// The characters this face maps, as scalar values. A `BTreeSet` rather than the live
    /// `Charmap`, which borrows the font bytes and would tie every caller's lifetime to
    /// them.
    mapped: std::collections::BTreeSet<u32>,
}

impl Charmap {
    /// The face at `index` (or the only face, for a single-face file).
    ///
    /// The error is one sentence about the bytes, like [`crate::names::faces`]'s, because
    /// the caller is a report: which file, and what was wrong with it.
    pub fn of(bytes: &[u8], index: Option<u32>) -> Result<Charmap, String> {
        let index = index.unwrap_or(0);
        let font = FontRef::from_index(bytes, index)
            .map_err(|e| format!("face {index} could not be read ({e})"))?;
        Ok(Charmap::read(&font))
    }

    fn read(font: &FontRef<'_>) -> Charmap {
        Charmap {
            mapped: font
                .charmap()
                .mappings()
                // A `cmap` may map a character explicitly to glyph 0, which *is* `.notdef`
                // — the tofu ADR-0007 is about. Kept out of the set, so "mapped" means
                // "mapped to something drawable" rather than "has an entry".
                .filter(|(_, glyph)| glyph.to_u32() != 0)
                .map(|(c, _)| c)
                .collect(),
        }
    }

    /// Does this face map `c`?
    pub fn covers(&self, c: char) -> bool {
        self.mapped.contains(&(c as u32))
    }
}

/// Characters that are **not** tofu when no face maps them.
///
/// Unicode's `Default_Ignorable_Code_Point` property: a conforming renderer draws nothing
/// for these whether or not a font maps them, so asking a `cmap` about one and calling the
/// miss an `error` would fire on every project that spells an emoji with a ZWJ — which is
/// most of them, and none of them wrong.
///
/// Spelled as a list rather than taken from a table, because the set Montagent needs is the
/// one ADR-0007 names — ZWJ/ZWNJ, RLM/LRM, the variation selectors — plus the three
/// zero-width characters that reach a caption from the same paste (soft hyphen, zero-width
/// space, word joiner) and the byte-order mark. A hand-rolled approximation of the *whole*
/// property would be a rule with no provenance (ADR-0061); this is an explicit exemption
/// list, and a character outside it is reported.
pub fn is_default_ignorable(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'          // SOFT HYPHEN
            | '\u{200B}'    // ZERO WIDTH SPACE
            | '\u{200C}'    // ZERO WIDTH NON-JOINER
            | '\u{200D}'    // ZERO WIDTH JOINER
            | '\u{200E}'    // LEFT-TO-RIGHT MARK
            | '\u{200F}'    // RIGHT-TO-LEFT MARK
            | '\u{2060}'    // WORD JOINER
            | '\u{FEFF}'    // ZERO WIDTH NO-BREAK SPACE
            | '\u{FE00}'..='\u{FE0F}'      // VARIATION SELECTOR-1..16
            | '\u{E0100}'..='\u{E01EF}'    // VARIATION SELECTOR-17..256
    )
}

/// `U+0301`, in the notation the standard writes and a reader can paste into a search.
///
/// Here rather than in either check that prints one: `montagent-core` names a code point in a
/// glyph-coverage finding and again in an invisible-character census, and two spellings of
/// one notation is one more place for them to drift.
pub fn codepoint(c: char) -> String {
    format!("U+{:04X}", c as u32)
}

/// `U+200D ZERO WIDTH JOINER` — the code point and the name a reader can search for.
///
/// The names are spelled here rather than looked up. The set that needs one is exactly
/// [`is_default_ignorable`]'s, and a Unicode name table carrying 30,000 entries to print
/// eight strings is a dependency with nothing to recommend it. A character outside the set
/// has no name here and is spelled as its code point alone.
pub fn named_codepoint(c: char) -> String {
    let name = match c {
        '\u{00AD}' => "SOFT HYPHEN",
        '\u{200B}' => "ZERO WIDTH SPACE",
        '\u{200C}' => "ZERO WIDTH NON-JOINER",
        '\u{200D}' => "ZERO WIDTH JOINER",
        '\u{200E}' => "LEFT-TO-RIGHT MARK",
        '\u{200F}' => "RIGHT-TO-LEFT MARK",
        '\u{2060}' => "WORD JOINER",
        '\u{FEFF}' => "ZERO WIDTH NO-BREAK SPACE",
        '\u{FE00}'..='\u{FE0F}' | '\u{E0100}'..='\u{E01EF}' => "VARIATION SELECTOR",
        _ => return codepoint(c),
    };
    format!("{} {name}", codepoint(c))
}
