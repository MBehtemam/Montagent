//! **Montagent owns the line partition**, and it splits on UAX #14's mandatory breaks
//! rather than on `split('\n')` (ADR-0008).
//!
//! The mandatory-break set is **BK, CR, LF and NL** — U+000A, U+000D, **CRLF as one break
//! and not two**, U+0085, U+000B, U+000C, U+2028 and U+2029. ADR-0007 commits to *"UTF-8,
//! NFC, written as raw characters"* with *"No tidying pass, ever"*, so a CR an editor
//! introduced is content the format promises to preserve — and a naive `split('\n')`
//! leaves a stray `\r` inside the line, where it would be shaped as a glyph and measured
//! into the advance width.
//!
//! Three independent reasons the partition is Montagent's own, from ADR-0008: ADR-0007's
//! line-height rule needs to know which runs are on which line *before* anything is
//! placed; splitting at a mandatory break is semantics-preserving under no-auto-wrap; and
//! it takes a third party's mandatory-break handling off the critical path — which is not
//! hypothetical, since parley 0.11.1 silently discards a mandatory `\n` inside Thai, Khmer
//! and Lao runs in its default wrap mode.
//!
//! **These characters also delimit bidi paragraphs**, so each line is handed to the layout
//! stack as an independent paragraph and a mandatory break resets the embedding level.
//!
//! The invariant ADR-0008 makes part of the renderer contract — *"for every script in the
//! fixture matrix, `lines == mandatory breaks + 1`, split at exactly the authored
//! offsets"* — is a property of [`partition`] rather than a test's discipline: it emits one
//! line per terminator plus one, which is why text ending in a break ends in an empty line.

/// One line of a partitioned text: its own bytes, where they sat in the whole, and the
/// mandatory break that ended it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line<'a> {
    /// The line's content, with its terminator **not** included.
    pub text: &'a str,
    /// The byte offset [`Line::text`] starts at, in the text [`partition`] was given.
    pub start: usize,
    /// The mandatory break that ended this line, verbatim — `"\r\n"` where the author
    /// wrote one — or `""` on the last line, which the end of the text ends.
    ///
    /// Carried rather than discarded because it is the one thing distinguishing the two
    /// spellings of the same partition: a document whose lines are CRLF-terminated and one
    /// whose lines are LF-terminated partition identically, and only this says which was
    /// written.
    pub terminator: &'a str,
}

impl Line<'_> {
    /// The byte offset one past this line's content — before its terminator, not after it.
    pub fn end(&self) -> usize {
        self.start + self.text.len()
    }
}

/// Is this character a UAX #14 mandatory break — class BK, CR, LF or NL?
///
/// The whole set, spelled out rather than reached through a character-class table: it is
/// seven code points fixed by the standard, and a reader checking Montagent against UAX #14
/// should be able to see all seven without resolving a dependency. The eighth mandatory
/// break ADR-0008 lists — CRLF — is a *pair*, and is [`partition`]'s to recognise rather
/// than this predicate's.
pub fn is_mandatory_break(c: char) -> bool {
    matches!(
        c,
        '\u{000A}'   // LF  — class LF
        | '\u{000D}' // CR  — class CR
        | '\u{000B}' // VT  — class BK
        | '\u{000C}' // FF  — class BK
        | '\u{0085}' // NEL — class NL
        | '\u{2028}' // LINE SEPARATOR      — class BK
        | '\u{2029}' // PARAGRAPH SEPARATOR — class BK
    )
}

/// Split a text into its lines at every mandatory break.
///
/// Never empty: a text with no mandatory break is one line, and the empty text is one
/// empty line. Text ending in a mandatory break ends in an empty line, because
/// `lines == mandatory breaks + 1` is the invariant ADR-0008 states and an empty last line
/// is what a trailing break means.
pub fn partition(text: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut cursor = text.char_indices();

    while let Some((at, c)) = cursor.next() {
        if !is_mandatory_break(c) {
            continue;
        }
        // CRLF is **one** break and not two (ADR-0008). Consuming the LF here rather than
        // testing for it on the next iteration is what keeps that true: a CR that is not
        // followed by an LF is still its own break, so the two cases differ only in how
        // much of the text the terminator spans.
        let mut after = at + c.len_utf8();
        if c == '\r' && text[after..].starts_with('\n') {
            cursor.next();
            after += '\n'.len_utf8();
        }
        lines.push(Line {
            text: &text[start..at],
            start,
            terminator: &text[at..after],
        });
        start = after;
    }

    lines.push(Line {
        text: &text[start..],
        start,
        terminator: "",
    });
    lines
}
