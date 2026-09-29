//! One face of a font collection, as a font file in its own right.
//!
//! ADR-0007 gives a chain entry an **`index`** *"for `.ttc` collections, defaulting to 0"*,
//! on a counted reason: *"49 of the fonts in a stock macOS `/System/Library/Fonts` are
//! collections; this is the normal case."* Two bullets earlier the same ADR rules that **a
//! field the renderer cannot honour is worse than no field** — *"it reads as declarative
//! and is silently ignored, which is the failure this format exists to prevent."* This
//! module is what stops `index` from being that field.
//!
//! **Why the face has to be cut out rather than picked.** `fontique::Collection::register_fonts`
//! registers *every* face of a collection into the one family, and `parley` then chooses
//! between them by a CSS attribute query — width, style and weight — which ADR-0007's format
//! does not carry (*"No `weight`, no `bold`. A different weight is a different file."*). So
//! the query runs at its defaults and the face that wins is a property of **what else is in
//! the file**, not of anything in the document. Measured on a stock `Avenir Next.ttc`, all
//! twelve declared indices shaped `Handgloves`/100 to **546.90** — which is face **7**
//! (Regular), not face 0 (Bold, 582.70). The declared index reached nothing.
//!
//! **And it cannot be narrowed after the fact.** `Collection::unregister_font` addresses a
//! face by `(width, style, weight)` and not by index, and on that same stock collection the
//! triple is not unique: faces 0 and 1 (Bold, Bold Italic) both register as
//! `(1.0, Normal, 700)`, and faces 10 and 11 (Ultra Light, Ultra Light Italic) both as
//! `(1.0, Normal, 275)` — `fontique` reads neither italic's slope. Unregistering the other
//! eleven is therefore not an operation that can name the twelfth.
//!
//! What is left is to hand `fontique` a file that contains one face, so the attribute query
//! has nothing to choose between. A collection is a header pointing at N table directories
//! over one pool of tables, so the face already *is* a complete table directory; [`face`]
//! copies that directory and the tables it names into a standalone sfnt. A single-face file
//! is passed through untouched and unallocated, which is every font in this repository.

use std::borrow::Cow;

use skrifa::raw::{FileRef, FontRef, ReadError, TableDirectory};

/// Why a declared `index` did not name a face.
///
/// Two variants rather than a sentence, because the caller ([`crate::fonts`]) already owns
/// the prose: it holds the path, and its [`crate::FontError`] is the one place a font
/// failure is worded for a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceError {
    /// The bytes are not an sfnt this build can parse — not a font, or not one of its
    /// flavours.
    NotAFont,
    /// The file parsed, and carries no face at the declared index. Carries how many it
    /// does have, so the caller can say so.
    NoSuchFace { faces: u32 },
}

/// The bytes of face `index`, as a font file in its own right.
///
/// **Borrowed for a single-face file**, which is the case every font in this repository and
/// most of any project takes: the returned `Cow` is the caller's own slice, and nothing is
/// copied or parsed beyond the table directory. Only a collection allocates.
///
/// `index` is never optional here. ADR-0007 defaults it to 0, and the default is the
/// caller's to apply — but it has to be *applied*, not skipped: an omitted `index` on a
/// collection means face 0, and leaving the whole file registered would resolve it by the
/// same attribute query that loses a declared index (this module's header), so on
/// `Avenir Next.ttc` an omitted `index` lands on face 7 exactly as `"index": 0` does.
pub fn face(bytes: &[u8], index: u32) -> Result<Cow<'_, [u8]>, FaceError> {
    match FileRef::new(bytes) {
        Ok(FileRef::Font(_)) if index == 0 => Ok(Cow::Borrowed(bytes)),
        // A single-face file carries exactly face 0, so any other index names nothing —
        // reported with the same "it carries N" clause a collection gets rather than a
        // second wording for the same mistake.
        Ok(FileRef::Font(_)) => Err(FaceError::NoSuchFace { faces: 1 }),
        Ok(FileRef::Collection(collection)) => {
            let faces = collection.len();
            if index >= faces {
                return Err(FaceError::NoSuchFace { faces });
            }
            // In range and still unreadable is a malformed collection rather than a bad
            // index, so it is the file that is reported, not the number the author wrote.
            let font = collection.get(index).map_err(|_| FaceError::NotAFont)?;
            Ok(Cow::Owned(standalone(bytes, &font)))
        }
        Err(_) => Err(FaceError::NotAFont),
    }
}

/// How many faces this file carries — 1 for a single-face file.
///
/// For `fonts list` and for a report that wants to say what an out-of-range index missed.
pub fn faces(bytes: &[u8]) -> Result<u32, ReadError> {
    match FileRef::new(bytes)? {
        FileRef::Font(_) => Ok(1),
        FileRef::Collection(collection) => Ok(collection.len()),
    }
}

/// Copy one face's table directory and the tables it names into a standalone sfnt.
///
/// The tables are copied **verbatim and so are their checksums**: a table's checksum covers
/// its own bytes, which do not change, so recomputing them could only introduce a
/// disagreement. `head.checkSumAdjustment` is the one field that is genuinely about the
/// *file*, and it is recomputed below — leaving the collection's value would ship a number
/// that describes a file this one is not, which is the defect this module exists to remove,
/// in miniature.
fn standalone(bytes: &[u8], font: &FontRef<'_>) -> Vec<u8> {
    let directory: &TableDirectory = font.table_directory();
    let records = directory.table_records();
    let count = records.len();

    let mut out = Vec::new();
    out.extend_from_slice(&directory.sfnt_version().to_be_bytes());
    out.extend_from_slice(&(count as u16).to_be_bytes());
    // The three redundant binary-search hints, by the formula in the sfnt spec. Derived
    // rather than copied: `count` is this file's table count, and a collection's face
    // directory is free to disagree with its own.
    //
    // Computed in `u32` and truncated. The fields are `u16` and the formula overflows one
    // above 4096 tables — which no real font has and a malformed one may claim, and these
    // three are advisory hints that neither `skrifa` nor `fontique` reads. Deliberate
    // truncation of a degenerate input beats an arithmetic panic on somebody's font file.
    let entry_selector = count.max(1).ilog2();
    let search_range = (1u32 << entry_selector) * 16;
    out.extend_from_slice(&(search_range as u16).to_be_bytes());
    out.extend_from_slice(&(entry_selector as u16).to_be_bytes());
    out.extend_from_slice(&((count as u32 * 16).wrapping_sub(search_range) as u16).to_be_bytes());

    // The records are written after the tables they point at are placed, so the directory
    // is reserved now and filled in as each table lands.
    let directory_end = out.len() + count * 16;
    out.resize(directory_end, 0);

    let mut head_at = None;
    for (position, record) in records.iter().enumerate() {
        let from = record.offset() as usize;
        let length = record.length() as usize;
        // `checked_add` because the record is untrusted: a length near `usize::MAX` would
        // panic the addition in a debug build before `get` ever saw the range.
        let Some(table) = from.checked_add(length).and_then(|to| bytes.get(from..to)) else {
            // A record pointing outside the file: copy nothing and leave the record's
            // length as it was. `fontique` refuses the result, which is the same answer a
            // caller would get from the collection itself.
            continue;
        };

        let at = out.len() as u32;
        if record.tag() == HEAD {
            head_at = Some(at as usize);
        }
        out.extend_from_slice(table);
        // Every table begins on a four-byte boundary, and the pad is not its length.
        while !out.len().is_multiple_of(4) {
            out.push(0);
        }

        let slot = directory_end - count * 16 + position * 16;
        out[slot..slot + 4].copy_from_slice(record.tag().to_be_bytes().as_slice());
        out[slot + 4..slot + 8].copy_from_slice(&record.checksum().to_be_bytes());
        out[slot + 8..slot + 12].copy_from_slice(&at.to_be_bytes());
        out[slot + 12..slot + 16].copy_from_slice(&(length as u32).to_be_bytes());
    }

    if let Some(head) = head_at {
        set_checksum_adjustment(&mut out, head);
    }
    out
}

const HEAD: skrifa::Tag = skrifa::Tag::new(b"head");

/// `head.checkSumAdjustment`: `0xB1B0AFBA` less the whole file's checksum, computed with the
/// field itself zeroed (sfnt spec, *Calculating checksums*).
fn set_checksum_adjustment(out: &mut [u8], head: usize) {
    let field = head + 8;
    let Some(slot) = out.get_mut(field..field + 4) else {
        return;
    };
    slot.copy_from_slice(&0u32.to_be_bytes());

    let mut sum = 0u32;
    for word in out.chunks(4) {
        // The final chunk is short only if the file is not four-byte aligned, which the
        // padding above prevents; treating a short tail as zero-padded is the spec's rule
        // and costs nothing here.
        let mut bytes = [0u8; 4];
        bytes[..word.len()].copy_from_slice(word);
        sum = sum.wrapping_add(u32::from_be_bytes(bytes));
    }

    let adjustment = 0xB1B0_AFBAu32.wrapping_sub(sum);
    out[field..field + 4].copy_from_slice(&adjustment.to_be_bytes());
}
