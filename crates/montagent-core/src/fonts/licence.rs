//! ADR-0057's three-bucket licence check.
//!
//! Research done for that ADR established that there is **no reliable machine-readable
//! signal for "may this file be redistributed"** inside a font file — the OS/2 `fsType`
//! bits answer a different legal question — so the check cannot be a yes/no test. It is
//! three buckets, and it assumes some fonts are unknowable and treats that honestly:
//!
//! 1. **Known non-redistributable → hard refuse, no override.** A name on the blocklist
//!    below. *"An override affordance is itself the thing that makes the project a knowing
//!    party to an illegal copy."*
//! 2. **Known redistributable → copy, record the licence.** The `name` table's licence
//!    description (ID 13) or URL (ID 14) matches a recognisable open-licence pattern. A
//!    heuristic string match, not a legal determination, and it fails safe: anything it
//!    does not confidently recognise falls through.
//! 3. **Unknown or unparsable (the common case) → refuse until the caller declares.** Most
//!    fonts carry no usable licence metadata at all — the fixture's own Open Runde carries
//!    none — and `--licence <identifier>` turns detector ignorance into a recorded,
//!    attributed assertion rather than a silent guess.
//!
//! # The blocklist
//!
//! Small, hardcoded, shipped by Montagent, and **not user-removable** — a user-removable
//! hard gate is not a gate. It grows only by PR against a citable licence clause, never
//! from an external database or a runtime fetch. Seeded with the one confirmed case in
//! hand: Apple's system fonts, whose licence restricts them to UI mockups for software
//! running on Apple's own platforms and explicitly forbids embedding them in other
//! software products.
//!
//! Matching is on family name and PostScript name, case- and whitespace-insensitive, and
//! it has to be both: on a stock macOS the file that carries `.SFNS-Regular` calls its
//! *family* "System Font", which no list of marketed names would ever catch.
//!
//! Bucket 1 does not need to be comprehensive to be sound, because bucket 3 already
//! catches everything not explicitly known either way. Apple's script-specific system
//! faces (`.SF Arabic`, `.SF Armenian`, …) are not on the seed list and land in bucket 3,
//! where their Apple licence text recognises nothing and the caller is asked to declare —
//! a gap named here rather than closed with an entry no ADR has cited a clause for.

use montagent_text::FaceNames;

/// One of the three buckets, for one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bucket {
    /// A name on the blocklist. Refused, and no flag lifts it.
    Blocklisted {
        /// The blocklist entry that matched, as the list spells it.
        matched: &'static str,
        /// The name on the face that matched it, as the file spells it.
        name: String,
        /// The face that matched — for a collection, the first one that did.
        face: u32,
    },
    /// The licence strings match one recognised open licence, on every face.
    Recognised {
        /// The SPDX-style identifier the attestation records.
        identifier: &'static str,
    },
    /// Nothing recognisable either way.
    Unknown,
}

impl Bucket {
    /// The word `fonts list` prints: `blocklisted` / `recognised-open` / `unknown`.
    pub fn status(&self) -> &'static str {
        match self {
            Bucket::Blocklisted { .. } => "blocklisted",
            Bucket::Recognised { .. } => "recognised-open",
            Bucket::Unknown => "unknown",
        }
    }
}

/// Which bucket a file falls in, judged over **every** face it carries.
///
/// A file is copied whole, so one blocklisted face in a collection refuses the file, and a
/// collection is recognised only when every face recognises as the same licence. Anything
/// else — mixed, partial, or silent — is unknown, which is the fail-safe direction.
pub fn bucket(faces: &[FaceNames]) -> Bucket {
    for face in faces {
        if let Some((matched, name)) = blocklisted(face) {
            return Bucket::Blocklisted {
                matched,
                name: name.to_string(),
                face: face.index,
            };
        }
    }

    let mut identifier: Option<&'static str> = None;
    for face in faces {
        match (recognised(face), identifier) {
            (Some(found), None) => identifier = Some(found),
            (Some(found), Some(seen)) if found == seen => {}
            _ => return Bucket::Unknown,
        }
    }
    match identifier {
        Some(identifier) => Bucket::Recognised { identifier },
        None => Bucket::Unknown,
    }
}

/// The shipped blocklist (ADR-0057's seed list): Apple's system fonts, by their marketed
/// family names and their private on-disk names.
///
/// `.SF NS` is the ADR's `.SF NS *` — every private San Francisco face on disk carries
/// it as the head of its PostScript name (`.SFNS-Regular`, `.SFNSRounded-Bold`,
/// `.SFNSMono-Light`), and [`matches`] treats an entry as a head as well as a whole name.
pub const BLOCKLIST: &[&str] = &[
    "SF Pro",
    "SF Pro Text",
    "SF Pro Display",
    "SF Pro Rounded",
    "SF Compact",
    "SF Compact Text",
    "SF Compact Display",
    "SF Compact Rounded",
    "SF Mono",
    "New York",
    ".SF NS",
    ".SF Compact",
    ".New York",
    ".AppleSystemUIFont",
];

/// The names on one face that the blocklist is checked against: ADR-0057's "family name
/// and PostScript name" — both family names the table carries (IDs 1 and 16), and ID 6.
/// The full name (ID 4) is deliberately not one of them.
fn names_of(face: &FaceNames) -> impl Iterator<Item = &str> {
    [
        face.family.as_deref(),
        face.typographic_family.as_deref(),
        face.postscript.as_deref(),
    ]
    .into_iter()
    .flatten()
}

/// The first blocklist entry any of a face's names matches, and the name that matched it.
fn blocklisted(face: &FaceNames) -> Option<(&'static str, &str)> {
    for name in names_of(face) {
        for entry in BLOCKLIST {
            if matches(name, entry) {
                return Some((entry, name));
            }
        }
    }
    None
}

/// Case- and whitespace-insensitive: the name *is* the entry, or the entry is the head of
/// a PostScript-style name (`SFProText-Bold` against `SF Pro Text`), or the entry is one of
/// the private `.`-prefixed heads that Apple's on-disk names extend freely (`.SFNSRounded`
/// against `.SF NS`).
///
/// A bare marketed name is never a free prefix: `New York` must not match a hypothetical
/// "New York Times Modern", whose normalised form continues with a letter rather than a
/// `-`. The private names begin with `.`, which no third party's font does, so they may.
fn matches(name: &str, entry: &'static str) -> bool {
    let name = normalise(name);
    let entry = normalise(entry);
    if name == entry {
        return true;
    }
    let Some(rest) = name.strip_prefix(&entry) else {
        return false;
    };
    rest.starts_with('-') || entry.starts_with('.')
}

fn normalise(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The recognised open licences: an identifier and the phrases that unambiguously name it
/// in a licence description or URL.
///
/// Each phrase is a whole licence's name or its canonical URL, never a word like "open"
/// or "free" that a proprietary text could carry. Arial's Microsoft text, Apple's SF text
/// and the empty string all recognise nothing, which is the fail-safe direction.
const RECOGNISED: &[(&str, &[&str])] = &[
    (
        "OFL-1.1",
        &[
            "sil open font license",
            "sil open font licence",
            "scripts.sil.org/ofl",
            "openfontlicense.org",
        ],
    ),
    (
        "Apache-2.0",
        &[
            "apache license, version 2.0",
            "apache license version 2.0",
            "apache.org/licenses/license-2.0",
        ],
    ),
    (
        "UFL-1.0",
        &[
            "ubuntu font licence",
            "ubuntu font license",
            "font.ubuntu.com/ufl",
        ],
    ),
];

/// The one licence a face's strings name, or `None` when they name none — or more than
/// one, which is ambiguity and falls through rather than picking.
fn recognised(face: &FaceNames) -> Option<&'static str> {
    let text: String = [
        face.licence_description.as_deref(),
        face.licence_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n")
    .to_lowercase();
    if text.is_empty() {
        return None;
    }

    let mut found = None;
    for (identifier, phrases) in RECOGNISED {
        if phrases.iter().any(|phrase| text.contains(phrase)) {
            if found.is_some() {
                return None;
            }
            found = Some(*identifier);
        }
    }
    found
}

/// A font ADR-0057's research found to be open and broadly similar to a blocklisted one.
///
/// Advisory output only, never an action the tool takes: a refusal *may* name these, and
/// the label states the actual basis honestly. No font claims formal metric compatibility
/// with any Apple system face, so nothing here is phrased as if a swap were safe by
/// default — a swap under an unchanged `fonts` key would silently invalidate every
/// hand-tuned size and line break the original font's metrics produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Substitute {
    pub name: &'static str,
    pub licence: &'static str,
    pub source: &'static str,
    /// What the similarity rests on, in words — never "metric-compatible" unless it is.
    pub basis: &'static str,
}

/// Known open substitutes for a refused font — the blocklist entry that matched, and the
/// name on the face that matched it. Only what ADR-0057's research pass actually found; a
/// font with no researched substitute names none.
///
/// The name matters as well as the entry: `.SF NS` is the head of every private San
/// Francisco face, and Open Runde is a style-alike of the *Rounded* one only.
pub fn substitutes(matched: &str, name: &str) -> &'static [Substitute] {
    const OPEN_RUNDE: Substitute = Substitute {
        name: "Open Runde",
        licence: "OFL-1.1",
        source: "https://github.com/lauridskern/open-runde",
        basis: "visually similar to SF Pro Rounded; no metric compatibility is claimed, so \
                every measured size and break must be re-verified",
    };
    let rounded = matched == "SF Pro Rounded" || normalise(name).contains("rounded");
    if rounded { &[OPEN_RUNDE] } else { &[] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(family: Option<&str>, postscript: Option<&str>) -> FaceNames {
        FaceNames {
            index: 0,
            family: family.map(String::from),
            typographic_family: None,
            full_name: None,
            postscript: postscript.map(String::from),
            licence_description: None,
            licence_url: None,
        }
    }

    fn licensed(description: &str, url: Option<&str>) -> FaceNames {
        FaceNames {
            licence_description: Some(description.to_string()),
            licence_url: url.map(String::from),
            ..face(Some("Anything"), Some("Anything-Regular"))
        }
    }

    #[test]
    fn a_marketed_apple_name_is_blocklisted_case_and_whitespace_insensitively() {
        for name in [
            "SF Pro Rounded",
            "sf pro rounded",
            "SFProRounded",
            "SF  Pro Rounded",
        ] {
            assert!(
                matches!(
                    bucket(&[face(Some(name), None)]),
                    Bucket::Blocklisted { .. }
                ),
                "{name}"
            );
        }
    }

    #[test]
    fn a_postscript_name_is_matched_by_its_head() {
        // What the file on a stock macOS actually says: family "System Font", and the only
        // name that gives it away is the PostScript one.
        let stock = face(Some("System Font"), Some(".SFNS-Regular"));
        assert_eq!(
            bucket(&[stock]),
            Bucket::Blocklisted {
                matched: ".SF NS",
                name: ".SFNS-Regular".to_string(),
                face: 0
            }
        );
        assert!(matches!(
            bucket(&[face(Some("Anything"), Some("SFProText-Bold"))]),
            Bucket::Blocklisted {
                matched: "SF Pro Text",
                ..
            }
        ));
        assert!(matches!(
            bucket(&[face(Some(".SF NS Rounded"), Some(".SFNSRounded-Bold"))]),
            Bucket::Blocklisted {
                matched: ".SF NS",
                ..
            }
        ));
    }

    #[test]
    fn a_marketed_name_is_not_a_free_prefix() {
        // `New York` is on the list; a hypothetical third-party "New York Times Modern" is
        // not, and its normalised form continues with a letter rather than a `-`.
        assert_eq!(
            bucket(&[face(
                Some("New York Times Modern"),
                Some("NewYorkTimesModern")
            )]),
            Bucket::Unknown
        );
        assert_eq!(
            bucket(&[face(Some("Safari"), Some("Safari-Regular"))]),
            Bucket::Unknown
        );
    }

    #[test]
    fn one_blocklisted_face_refuses_the_whole_collection() {
        let faces = vec![
            face(Some("Helvetica"), Some("Helvetica")),
            FaceNames {
                index: 1,
                ..face(Some("SF Mono"), Some("SFMono-Regular"))
            },
        ];
        assert!(matches!(
            bucket(&faces),
            Bucket::Blocklisted { face: 1, .. }
        ));
    }

    #[test]
    fn the_three_named_open_licences_are_recognised_from_description_or_url() {
        assert_eq!(
            bucket(&[licensed(
                "This Font Software is licensed under the SIL Open Font License, Version 1.1.",
                None
            )]),
            Bucket::Recognised {
                identifier: "OFL-1.1"
            }
        );
        assert_eq!(
            bucket(&[licensed("", Some("https://scripts.sil.org/OFL"))]),
            Bucket::Recognised {
                identifier: "OFL-1.1"
            }
        );
        assert_eq!(
            bucket(&[licensed(
                "Licensed under the Apache License, Version 2.0",
                Some("http://www.apache.org/licenses/LICENSE-2.0")
            )]),
            Bucket::Recognised {
                identifier: "Apache-2.0"
            }
        );
        assert_eq!(
            bucket(&[licensed("Ubuntu Font Licence 1.0", None)]),
            Bucket::Recognised {
                identifier: "UFL-1.0"
            }
        );
    }

    #[test]
    fn a_proprietary_or_absent_licence_text_recognises_nothing() {
        // Arial's Microsoft text, as the file carries it.
        assert_eq!(
            bucket(&[licensed(
                "You may use this font to display and print content as permitted by the \
                 license terms for the product in which this font is included.",
                None
            )]),
            Bucket::Unknown
        );
        // The fixture's own Open Runde carries no licence strings at all.
        assert_eq!(
            bucket(&[face(Some("Open Runde"), Some("OpenRunde-Bold"))]),
            Bucket::Unknown
        );
    }

    #[test]
    fn a_text_naming_two_licences_is_ambiguous_and_falls_through() {
        assert_eq!(
            bucket(&[licensed(
                "Dual licensed under the SIL Open Font License and the Apache License, Version 2.0",
                None
            )]),
            Bucket::Unknown
        );
    }

    #[test]
    fn a_collection_is_recognised_only_when_every_face_agrees() {
        let ofl = licensed("SIL Open Font License, Version 1.1", None);
        let silent = face(Some("Quiet"), Some("Quiet-Regular"));
        assert_eq!(
            bucket(&[ofl.clone(), ofl.clone()]),
            Bucket::Recognised {
                identifier: "OFL-1.1"
            }
        );
        assert_eq!(bucket(&[ofl, silent]), Bucket::Unknown);
    }

    #[test]
    fn the_blocklist_is_the_seed_list_and_nothing_is_removable_at_runtime() {
        // `BLOCKLIST` is a `const`: there is no function in this module that takes an entry
        // away, which is the structural form of ADR-0057's "not user-removable".
        assert!(BLOCKLIST.contains(&"SF Pro Rounded"));
        assert!(BLOCKLIST.contains(&".AppleSystemUIFont"));
        assert!(BLOCKLIST.contains(&"New York"));
    }

    #[test]
    fn a_substitute_is_labelled_by_its_actual_basis_and_never_as_metric_compatible() {
        let open_runde = &substitutes("SF Pro Rounded", "SFProRounded-Bold")[0];
        assert_eq!(open_runde.name, "Open Runde");
        assert!(open_runde.basis.contains("no metric compatibility"));
        assert_eq!(substitutes(".SF NS", ".SFNSRounded-Regular").len(), 1);
        // `.SF NS` heads every private San Francisco face; the plain one is not rounded.
        assert!(substitutes(".SF NS", ".SFNS-Regular").is_empty());
        assert!(substitutes("SF Mono", "SFMono-Regular").is_empty());
    }
}
