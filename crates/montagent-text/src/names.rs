//! What a font file calls itself, and what it says about its own licence.
//!
//! ADR-0057's `fonts list` enumerates faces *"with paths, face indices and detected licence
//! status"*, and `fonts vendor`'s gate reads *"the font's `name` table records for IDs 13
//! (License Description) and 14 (License Info URL)"* before a byte is copied. Both need the
//! same handful of strings off the same table, and this module is the one place they are
//! read.
//!
//! It **reads and derives, and judges nothing**: which family names are blocklisted and
//! which licence texts are recognised are rules about the project, and rules live in the
//! core (ADR-0011). What is here is the OpenType `name` table, as `skrifa` exposes it, for
//! every face in a file — one face for a `.ttf`/`.otf`, several for a collection, and the
//! index beside each so a `.ttc` face can be named the way the `fonts` table names one.
//!
//! No `fsType`. ADR-0057's research established that the OS/2 embedding bits answer a
//! different legal question — *"may an application embed this font inside a document"* —
//! from *"may this file be copied into a repository"*, and that the two do not correlate. A
//! field that looks like the answer and is not would be worse than its absence.

use skrifa::string::StringId;
use skrifa::{FontRef, MetadataProvider};

/// One face's own account of itself.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FaceNames {
    /// Its position in the file: `0` for a single-face file, the collection index otherwise.
    pub index: u32,
    /// Name ID 1, the family name.
    pub family: Option<String>,
    /// Name ID 16, the typographic family name — where a family is split into "SF Pro Text"
    /// and "SF Pro Display" at ID 1, this is the name the vendor actually markets.
    pub typographic_family: Option<String>,
    /// Name ID 4, the full name.
    pub full_name: Option<String>,
    /// Name ID 6, the PostScript name — `SFProText-Regular`.
    pub postscript: Option<String>,
    /// Name ID 13, the licence description.
    pub licence_description: Option<String>,
    /// Name ID 14, the licence info URL.
    pub licence_url: Option<String>,
}

/// Every face in a font file, in index order.
///
/// The error is one sentence about the bytes — *"not a font, or not one this build can
/// parse"* — because the caller is a report, and what a report needs is which file and
/// what was wrong with it.
pub fn faces(bytes: &[u8]) -> Result<Vec<FaceNames>, String> {
    let file = skrifa::raw::FileRef::new(bytes)
        .map_err(|e| format!("not a font this build can parse ({e})"))?;
    let mut faces = Vec::new();
    match file {
        skrifa::raw::FileRef::Font(font) => faces.push(read(&font, 0)),
        skrifa::raw::FileRef::Collection(collection) => {
            for index in 0..collection.len() {
                let font = collection.get(index).map_err(|e| {
                    format!("face {index} of the collection could not be read ({e})")
                })?;
                faces.push(read(&font, index));
            }
        }
    }
    if faces.is_empty() {
        return Err("it carries no faces".to_string());
    }
    Ok(faces)
}

fn read(font: &FontRef<'_>, index: u32) -> FaceNames {
    let string = |id: StringId| {
        font.localized_strings(id)
            .english_or_first()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
    };
    FaceNames {
        index,
        family: string(StringId::FAMILY_NAME),
        typographic_family: string(StringId::TYPOGRAPHIC_FAMILY_NAME),
        full_name: string(StringId::FULL_NAME),
        postscript: string(StringId::POSTSCRIPT_NAME),
        licence_description: string(StringId::LICENSE_DESCRIPTION),
        licence_url: string(StringId::LICENSE_URL),
    }
}
