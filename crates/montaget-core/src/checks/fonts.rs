//! Everything `validate` asks about the `fonts` table: ADR-0057's attestation, and
//! ADR-0007's glyph coverage and font census (#206).
//!
//! *"`validate`'s check becomes 'every path any chain references resolves to a `fontVendor`
//! entry whose hash matches the bytes on disk,' a purely local, deterministic comparison."*
//! It re-checks **integrity, never licence law**: once vendored, an entry is evidence of a
//! decision made at a point in time, and the two shapes a re-verification could take — a
//! hash-to-known-font whitelist, or a live licence oracle — are both worse than not
//! checking. So nothing here reads a licence string; it reads bytes and compares a hash.
//!
//! Three findings on the referenced side and one on the attested side:
//!
//! | what was established | finding |
//! | --- | --- |
//! | the `fonts`-table path has no file behind it | `E-FONT-MISSING` |
//! | the file is there and no `fontVendor` entry names it | `E-FONT-UNATTESTED` |
//! | the entry is there and its `sha256` is not the file's | `E-FONT-HASH-MISMATCH` |
//! | a `fontVendor` entry names a path no chain references | `N-FONT-ATTESTATION-ORPHANED` |
//!
//! The orphan is a `note` and is **never auto-pruned**: `fmt` adds and removes no key by
//! construction, and whether an attestation for a file no chain uses is still wanted is
//! the author's decision.
//!
//! ADR-0007 adds three more questions about the same table. Two of them need the text as
//! well as the files, and the third needs what a *previous run* saw:
//!
//! | what was established | finding |
//! | --- | --- |
//! | nothing in a chain maps a character the text uses | `E-FONT-NO-GLYPH` |
//! | the project's text is set in more than one declared font | `N-FONT-CENSUS` |
//! | a chain's bytes are not the ones this project was last validated against | `R-FONT-SWAP` |
//!
//! **One gap, named rather than left to be discovered.** An element whose `font` names a
//! key the `fonts` table does not declare gets no finding here. It is a dangling reference
//! like an anchor's, ADR-0007 does not name it among `validate`'s font checks, and #206's
//! acceptance criteria do not ask for it — so inventing a seventh code for it belongs to
//! the ticket that decides its class, not to this one. What it does mean is that coverage
//! is not *claimed* for such an element: no chain, no claim.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use montaget_text::Charmap;
use montaget_text::glyphs::{codepoint, is_default_ignorable};
use serde_json::{Value, json};

use crate::checks::{StyledText, pluralised};
use crate::finding::{Census, Finding};
use crate::fonts::sha256_hex;
use crate::media::sidecar::{FontEntry, ProjectFonts, Sidecar};
use crate::permissive::Loose;
use crate::report::Report;

/// Every question `validate` asks about the `fonts` table.
///
/// `cache` is the sidecar this run reads its font chains from and writes them back to, or
/// `None` for a run with no cache at all — `MONTAGET_CACHE_DIR=""`, or a machine that will
/// not say where a cache belongs. With no cache there is nothing to compare against, so
/// `R-FONT-SWAP` cannot fire; every other check here is unaffected, because every other
/// check reads only the document and the files it names.
pub fn check(document: &Loose, cache: Option<&Path>, report: &mut Report) {
    attestation(document, report);

    let elements = crate::checks::styled_text(document);
    let usage = font_usage(&elements);
    glyph_coverage(document, &usage, report);
    font_census(document, &usage, report);
    font_swap(document, cache, &usage, report);
}

/// ADR-0057's three referenced-side findings and one attested-side note.
fn attestation(document: &Loose, report: &mut Report) {
    let referenced = referenced(document);
    let attested = attested(document);
    let base = crate::checks::project_dir(document);

    for file in &referenced {
        let resolved = base.join(file);
        let bytes = match std::fs::read(&resolved) {
            Ok(bytes) => bytes,
            Err(e) => {
                report.push(
                    Finding::new("E-FONT-MISSING")
                        .at_file(document.path())
                        .field("file", json!(file))
                        .field("resolved", json!(resolved.display().to_string()))
                        .field("detail", json!(e.to_string()))
                        .repair_value(json!({
                            "value": "put the file where the document says, or correct the path"
                        })),
                );
                // A hash cannot be compared against bytes that are not there; the entry's
                // presence is still a fact worth stating in the same run.
                if !attested.contains_key(file) {
                    report.push(unattested(document, file));
                }
                continue;
            }
        };

        match attested.get(file) {
            None => report.push(unattested(document, file)),
            // A `sha256` that is not a string is the schema check's to report; a second
            // finding here would be the same fault said twice.
            Some(None) => {}
            Some(Some(recorded)) => {
                let actual = sha256_hex(&bytes);
                if &actual != recorded {
                    report.push(
                        Finding::new("E-FONT-HASH-MISMATCH")
                            .at_file(document.path())
                            .field("file", json!(file))
                            .field("recorded", json!(recorded))
                            .field("actual", json!(actual)),
                    );
                }
            }
        }
    }

    for file in attested.keys() {
        if !referenced.contains(file) {
            report.push(
                Finding::new("N-FONT-ATTESTATION-ORPHANED")
                    .at_file(document.path())
                    .field("file", json!(file)),
            );
        }
    }
}

fn unattested(document: &Loose, file: &str) -> Finding {
    Finding::new("E-FONT-UNATTESTED")
        .at_file(document.path())
        .field("file", json!(file))
        .repair_value(json!({
            "value": format!("montaget fonts vendor {} <path to the font> [--licence <identifier>]", document.path())
        }))
}

/// Every `file` any `fonts` chain names, as the document writes it, once each.
///
/// Read permissively: a chain entry that is not an object, or a `file` that is not a
/// string, is the schema check's to report and is simply not a path here.
fn referenced(document: &Loose) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    let Some(fonts) = document.value()["fonts"].as_object() else {
        return files;
    };
    for chain in fonts.values() {
        let Some(entries) = chain.as_array() else {
            continue;
        };
        for entry in entries {
            if let Some(file) = entry["file"].as_str() {
                files.insert(file.to_string());
            }
        }
    }
    files
}

/// Every `fontVendor` entry, by path, with its recorded `sha256` where that is a string.
fn attested(document: &Loose) -> BTreeMap<String, Option<String>> {
    let mut entries = BTreeMap::new();
    let Some(table) = document.value()["fontVendor"].as_object() else {
        return entries;
    };
    for (file, entry) in table {
        entries.insert(
            file.clone(),
            entry
                .get("sha256")
                .and_then(Value::as_str)
                .map(String::from),
        );
    }
    entries
}

// ---- ADR-0007's glyph coverage and font census (#206) ----------------------------------

/// What one `text` element sets in one declared font: the key, and every character it asks
/// that key to draw.
///
/// One entry per (element, key) rather than per run, because both questions below are asked
/// per element: coverage reports *"this element needs a glyph this chain does not have"*,
/// and the census counts elements. An element whose base style is `brand` and whose one
/// emphasised run is `brand+fa` appears under both keys, which is the truth about it — the
/// alternative, crediting it to its base style alone, would leave the emphasised run's font
/// out of a census whose whole job is to find the odd one out.
struct Usage {
    subject: String,
    track: Option<String>,
    key: String,
    /// Every character this element asks of this key, once each, in first-appearance order
    /// — the order they read in the document, which is the order a reader finds them.
    characters: Vec<char>,
    /// The same set, for the membership test. A caption is short and a `Vec` scan would do;
    /// a `runs` array holding a transcript is not, and `O(text²)` on one element is the one
    /// way this check becomes something an author notices.
    seen: BTreeSet<char>,
}

/// Every (element, key) pair the project's text declares, in document order.
///
/// A run's `font` is a style delta over the element's base (ADR-0007), so a run carrying
/// none inherits the element's key. An element with no readable `font` at all contributes
/// nothing: which key it meant is not in the document.
fn font_usage(elements: &[StyledText]) -> Vec<Usage> {
    let mut usage: Vec<Usage> = Vec::new();
    for element in elements {
        let element_start = usage.len();
        for run in &element.runs {
            let Some(key) = run.font.as_deref().or(element.font.as_deref()) else {
                continue;
            };
            // Searched from this element's own first entry rather than from the start of
            // the list: the entries for one element are contiguous by construction, and
            // rescanning every earlier element's would be quadratic in the project.
            let at = match usage[element_start..].iter().position(|u| u.key == key) {
                Some(at) => element_start + at,
                None => {
                    usage.push(Usage {
                        subject: element.subject.clone(),
                        track: element.track.clone(),
                        key: key.to_string(),
                        characters: Vec::new(),
                        seen: BTreeSet::new(),
                    });
                    usage.len() - 1
                }
            };
            for c in run.text.chars() {
                if usage[at].seen.insert(c) {
                    usage[at].characters.push(c);
                }
            }
        }
    }
    usage
}

/// ADR-0007: *"A character with no glyph in any chain entry renders `.notdef` and is a
/// `validate` **error**."*
///
/// **Asked of the whole chain, not of its first entry.** That is what a fallback chain is
/// for: ADR-0007's own `brand+fa` declares Inter and Vazirmatn precisely so that the Persian
/// the first file cannot draw is drawn by the second.
///
/// **A chain no file of which could be read produces no finding.** The attestation check
/// above has already said so under `E-FONT-MISSING`, and *"this font has no glyph for ك"*
/// derived from a font nobody could open is the false confidence ADR-0006 exists to prevent.
fn glyph_coverage(document: &Loose, usage: &[Usage], report: &mut Report) {
    let mut chains = Chains::of(document);

    for element in usage {
        let Some(chain) = chains.get(&element.key) else {
            continue;
        };
        let missing: Vec<char> = element
            .characters
            .iter()
            .copied()
            .filter(|&c| !drawn_by_no_font(c) && !chain.covers(c))
            .collect();
        if missing.is_empty() {
            continue;
        }

        let mut finding = Finding::new("E-FONT-NO-GLYPH")
            .at_file(document.path())
            .at_element(element.subject.clone())
            .field("element", json!(element.subject))
            .field("font", json!(element.key))
            .field("characters", json!(listed(&missing)))
            .field("chain", json!(chain.files.join(", ")));
        if let Some(track) = &element.track {
            finding = finding.at_track(track.clone());
        }
        if let Some(census) = elsewhere(&mut chains, &element.key, &missing) {
            finding = finding.census(census);
        }
        report.push(finding);
    }
}

/// ADR-0043's sibling census for a refuse-class finding: *"a census narrows"* without
/// deciding anything.
///
/// The group that exists here is **the project's own other declared chains that do map
/// every one of these characters**. It is inert and document-derived — ADR-0007's `brand+fa`
/// is in the ADR's worked table precisely because `brand` cannot draw Persian — and it
/// carries the fix without proposing it: whether the right move is to add that file to this
/// chain, to move the run to that key, or to change the text is the author's, and the census
/// says none of them.
///
/// Omitted rather than stated as a negative when no other chain maps them (ADR-0058's rule):
/// *"no other font in this project has them either"* implies a hypothesis was tested that
/// the reader never asked about.
fn elsewhere(chains: &mut Chains<'_>, key: &str, missing: &[char]) -> Option<Census> {
    let others: Vec<String> = chains
        .declared()
        .into_iter()
        .filter(|other| other != key)
        .filter(|other| {
            chains
                .get(other)
                .is_some_and(|chain| missing.iter().all(|&c| chain.covers(c)))
        })
        .collect();

    let mut census = Census::on("font");
    let members: Vec<String> = missing
        .iter()
        .map(|&c| format!("{} {c}", codepoint(c)))
        .collect();
    for other in &others {
        census = census.group(json!(other), members.clone());
    }
    (!others.is_empty()).then_some(census)
}

/// Characters a conforming renderer draws nothing for whether or not a font maps them, so
/// that a `cmap` miss on one is not tofu.
///
/// Two sets: Unicode's default-ignorable code points
/// ([`montaget_text::coverage::is_default_ignorable`]), and the two line-break characters —
/// a `\n` is the format's only line break (ADR-0008) and a `\r` is the half of a CRLF a
/// paste left behind. Neither reaches a shaper as a glyph, and a font is not short of
/// anything for not mapping them.
fn drawn_by_no_font(c: char) -> bool {
    c == '\n' || c == '\r' || is_default_ignorable(c)
}

/// `U+0E01 ก, U+0E02 ข` — every missing character, with the code point beside the character
/// itself so the finding survives a terminal that cannot draw it either.
///
/// Cut at eight, because the finding's job is to name the condition and a chain missing a
/// whole script would otherwise print a caption's worth of code points into a report whose
/// noise budget ADR-0006 fixes.
fn listed(missing: &[char]) -> String {
    const LIMIT: usize = 8;
    let shown: Vec<String> = missing
        .iter()
        .take(LIMIT)
        .map(|&c| format!("{} {c}", codepoint(c)))
        .collect();
    match missing.len() > LIMIT {
        true => format!("{} and {} more", shown.join(", "), missing.len() - LIMIT),
        false => shown.join(", "),
    }
}

/// ADR-0007's font census — *"23 elements use `brand`; 1 uses `brand-old`"*.
///
/// **Fires when the project's text is set in more than one declared font, and not
/// otherwise.** A project with one font has a census of one group, which states a fact
/// nobody can act on and which ADR-0006's noise budget would spend a line on every run. The
/// finding a reader wants is the one the ADR wrote down, and its shape is two groups of
/// very different sizes.
///
/// **Never sorted, never scored.** ADR-0043: a census *"must not be worded in a way that
/// implies the larger group is the correct one"*, and the fixture's own `word-08-target` is
/// a census outlier that is correct. Groups keep first-appearance order.
fn font_census(document: &Loose, usage: &[Usage], report: &mut Report) {
    let mut keys: Vec<(String, Vec<String>)> = Vec::new();
    for element in usage {
        match keys.iter_mut().find(|(key, _)| key == &element.key) {
            Some((_, members)) => members.push(element.subject.clone()),
            None => keys.push((element.key.clone(), vec![element.subject.clone()])),
        }
    }
    if keys.len() < 2 {
        return;
    }

    let distribution = keys
        .iter()
        .map(|(key, members)| match members.len() {
            1 => format!("1 element uses `{key}`"),
            count => format!("{count} elements use `{key}`"),
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut census = Census::on("font");
    for (key, members) in &keys {
        census = census.group(json!(key), members.clone());
    }

    report.push(
        Finding::new("N-FONT-CENSUS")
            .at_file(document.path())
            .field("distribution", json!(distribution))
            .census(census),
    );
}

/// The declared chains, read from disk once each and only when something asks.
///
/// Lazy because a project declaring five chains and setting its text in one should open one
/// font, and cached because a chain is re-asked for every element that uses it — the fixture
/// asks `brand` twenty-two times, and re-parsing a `cmap` twenty-two times would make an
/// `O(text)` check `O(text × elements)`.
struct Chains<'a> {
    document: &'a Loose,
    base: PathBuf,
    loaded: BTreeMap<String, Option<Chain>>,
}

/// One key's whole fallback chain: the files as the document spells them, and a coverage
/// set per face that could be read.
struct Chain {
    files: Vec<String>,
    faces: Vec<Charmap>,
}

impl Chain {
    /// ADR-0007: *"Fallback walks the chain and stops."*
    fn covers(&self, c: char) -> bool {
        self.faces.iter().any(|face| face.covers(c))
    }
}

impl<'a> Chains<'a> {
    fn of(document: &'a Loose) -> Chains<'a> {
        Chains {
            document,
            base: crate::checks::project_dir(document),
            loaded: BTreeMap::new(),
        }
    }

    /// This key's chain, or `None` where no claim about coverage can be made: the key is
    /// undeclared, or not one file of it could be read as a font.
    fn get(&mut self, key: &str) -> Option<&Chain> {
        if !self.loaded.contains_key(key) {
            let chain = self.read(key);
            self.loaded.insert(key.to_string(), chain);
        }
        self.loaded.get(key)?.as_ref()
    }

    /// Every key the `fonts` table declares, whether or not it has been read yet.
    fn declared(&self) -> Vec<String> {
        self.document
            .value()
            .get("fonts")
            .and_then(Value::as_object)
            .map(|table| table.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn read(&self, key: &str) -> Option<Chain> {
        let entries = self
            .document
            .value()
            .get("fonts")
            .and_then(Value::as_object)?
            .get(key)?
            .as_array()?;

        let mut files = Vec::new();
        let mut faces = Vec::new();
        for entry in entries {
            let Some(file) = entry.get("file").and_then(Value::as_str) else {
                continue;
            };
            files.push(file.to_string());
            // Every failure here is silence: a file that is not there, or that is not a
            // font, is already `E-FONT-MISSING` or `fonts list`'s "unreadable", and saying
            // it a second time in the vocabulary of glyph coverage would be one fault
            // reported as two.
            let Ok(bytes) = std::fs::read(self.base.join(file)) else {
                continue;
            };
            let index = entry.get("index").and_then(Value::as_u64).map(|i| i as u32);
            if let Ok(charmap) = Charmap::of(&bytes, index) {
                faces.push(charmap);
            }
        }

        (!faces.is_empty()).then_some(Chain { files, faces })
    }
}

// ---- ADR-0007's font swap, and the cache that makes one visible (#206) ------------------

/// ADR-0007: *"Font files join ADR-0006's `(path, size, mtime)` probe cache — a font
/// swapped in place is a silent whole-project render change that no census sees."* And spec
/// #168's story 54: *"a font-swap census when the `fonts` table changes, so that I learn
/// that 22 elements' hand-tuned sizes are now unverified."*
///
/// **Both halves are one comparison**, because a chain's identity is the ordered *content
/// hash* of its entries: the table repointed at different bytes and the same table over
/// rewritten bytes are the same difference. ADR-0007's `(path, size, mtime)` is the cache
/// *key* over that — it decides whether a file has to be read and hashed again, exactly as
/// ADR-0069's triple decides whether a source has to be probed again — and never the
/// comparison itself, because an mtime moves for reasons that are not edits.
///
/// ADR-0007 is why either matters — its own use exercise found that the font swap,
/// *"one line under a fonts table"*, **silently invalidated every hand-tuned size and
/// hand-placed break in the document**, and adopted the table *"with a census and a
/// font-swap finding, not as a bare win"*.
///
/// **A key seen for the first time is silent**, which is [`crate::media::session::MissKind`]
/// `First` against `Changed`: a project validated on a fresh machine has not changed, it has
/// merely never been looked at, and a finding on every first run would be an announcement
/// that Montaget has no memory rather than a fact about the project.
///
/// **A key nothing sets its text in is silent too.** The finding names *"which measured
/// layouts are now unverified"*, and a chain no element uses has none.
///
/// **A key the table no longer declares is the same finding**, not a different one. It is
/// the largest version of spec #168's story 54 — every element still naming it has lost the
/// font it was measured in — and it reaches this check rather than the coverage one because
/// the coverage check has nothing to open. The *dangling reference* it also is has no code
/// yet; see this module's header.
fn font_swap(document: &Loose, cache: Option<&Path>, usage: &[Usage], report: &mut Report) {
    let Some(cache) = cache else {
        return;
    };
    let sidecar = Sidecar::load(cache.to_path_buf());
    let project = project_key(document);

    let previous = sidecar.fonts().get(&project);
    let observed = observed_chains(document, previous);

    if let Some(previous) = previous {
        let declared = declared_keys(document);
        for (key, before) in &previous.keys {
            let detail = match observed.get(key) {
                Some(now) if same_bytes(before, now) => continue,
                Some(now) => what_changed(before, now),
                // Absent from `observed` for one of two reasons. A key the table still
                // declares is one whose files would not open — already `E-FONT-MISSING`,
                // and nothing here is entitled to call that a swap. A key the table no
                // longer declares at all is a swap of the largest kind.
                None if declared.contains(key) => continue,
                None => "the `fonts` table no longer declares it".to_string(),
            };
            let members: Vec<String> = usage
                .iter()
                .filter(|u| &u.key == key)
                .map(|u| u.subject.clone())
                .collect();
            if members.is_empty() {
                continue;
            }
            report.push(
                Finding::new("R-FONT-SWAP")
                    .at_file(document.path())
                    .field("font", json!(key))
                    .field("detail", json!(detail))
                    // The count, which the sentence needs; the census below carries the
                    // names, which a sentence cannot.
                    .field("elements", json!(pluralised(members.len(), "element")))
                    .census(Census::on("font").group(json!(key), members)),
            );
        }
    }

    // Written back whether or not anything fired: what this run observed is what the next
    // one compares against.
    sidecar.record_fonts(project, observed);
}

/// Every key the `fonts` table declares right now, whatever state its files are in.
fn declared_keys(document: &Loose) -> BTreeSet<String> {
    document
        .value()
        .get("fonts")
        .and_then(Value::as_object)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

/// The project's canonical path — the key a later run must arrive at from any directory.
///
/// A path that will not canonicalise is keyed under the spelling it arrived in, which is
/// [`crate::media::probe::LocalKey`]'s rule and its reason: a partly-observed key still
/// answers, it just answers for one spelling.
fn project_key(document: &Loose) -> String {
    let path = Path::new(document.path());
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

/// Every declared chain whose files Montaget could observe in full, right now.
///
/// **A chain with one unreadable entry is omitted entirely, not recorded short.** A chain
/// recorded without the file that would not open would compare unequal the moment that file
/// came back, and announce a swap that never happened — and `E-FONT-MISSING` has already
/// said the one true thing there was to say.
fn observed_chains(
    document: &Loose,
    previous: Option<&ProjectFonts>,
) -> BTreeMap<String, Vec<FontEntry>> {
    let base = crate::checks::project_dir(document);
    let mut chains = BTreeMap::new();
    let Some(table) = document.value().get("fonts").and_then(Value::as_object) else {
        return chains;
    };

    for (key, chain) in table {
        let Some(entries) = chain.as_array() else {
            continue;
        };
        let recorded = previous.and_then(|previous| previous.keys.get(key));
        let observed: Option<Vec<FontEntry>> = entries
            .iter()
            .enumerate()
            .map(|(position, entry)| {
                let file = entry.get("file").and_then(Value::as_str)?;
                let metadata = std::fs::metadata(base.join(file)).ok()?;
                let size = metadata.len();
                let mtime_ns = metadata.modified().ok().and_then(|time| {
                    time.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_nanos() as i128)
                });
                // The cache key doing its one job: an entry whose `(file, size, mtime)` is
                // what was recorded is the file that was recorded, so its hash is known and
                // the bytes are not read. Everything else is read and hashed.
                let sha256 = match recorded.and_then(|chain| chain.get(position)) {
                    Some(before)
                        if before.file == file
                            && before.size == size
                            && before.mtime_ns == mtime_ns
                            && mtime_ns.is_some() =>
                    {
                        before.sha256.clone()
                    }
                    _ => sha256_hex(&std::fs::read(base.join(file)).ok()?),
                };
                Some(FontEntry {
                    file: file.to_string(),
                    size,
                    mtime_ns,
                    sha256,
                })
            })
            .collect();
        if let Some(observed) = observed {
            chains.insert(key.clone(), observed);
        }
    }
    chains
}

/// Whether two recordings of one chain are the same *fonts*, whatever the filesystem says
/// about them.
///
/// Hashes, in order, and nothing else: a chain repointed at a byte-identical copy of the
/// same file renders identically, and every size and break measured against it still holds.
fn same_bytes(before: &[FontEntry], now: &[FontEntry]) -> bool {
    before.len() == now.len()
        && before
            .iter()
            .zip(now)
            .all(|(before, now)| before.sha256 == now.sha256)
}

/// What changed, in one clause, said as the difference rather than as two lists.
///
/// The table half first, because it is the half an author can see in their own diff; the
/// in-place half second, because it is the one ADR-0007 calls silent.
fn what_changed(before: &[FontEntry], now: &[FontEntry]) -> String {
    let spelled_before: Vec<&str> = before.iter().map(|e| e.file.as_str()).collect();
    let spelled_now: Vec<&str> = now.iter().map(|e| e.file.as_str()).collect();
    if spelled_before != spelled_now {
        return format!(
            "it names {} where it named {}",
            quoted_list(&spelled_now),
            quoted_list(&spelled_before)
        );
    }

    let rewritten: Vec<String> = now
        .iter()
        .zip(before)
        .filter(|(now, before)| now.sha256 != before.sha256)
        .map(|(now, before)| match now.size == before.size {
            true => format!(
                "`{}` was rewritten in place (the same {} bytes, a later modification time)",
                now.file, now.size
            ),
            false => format!(
                "`{}` was rewritten in place ({} bytes, and {} when it was last seen)",
                now.file, now.size, before.size
            ),
        })
        .collect();
    english_list(&rewritten)
}

fn quoted_list(files: &[&str]) -> String {
    english_list(
        &files
            .iter()
            .map(|file| format!("`{file}`"))
            .collect::<Vec<_>>(),
    )
}

/// `a`, `a and b`, `a, b and c` — an English list, because the finding is a sentence.
fn english_list(items: &[String]) -> String {
    match items {
        [] => "nothing Montaget can name".to_string(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
