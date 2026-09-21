//! What `validate` asks about the bytes **inside** a run — three of ADR-0007's five text
//! checks, and the two of those that need no font at all.
//!
//! ADR-0007's consequence list gives `validate` *"a **grapheme-cluster** check that no run
//! boundary splits a base from its combining mark, an **invisible-character** census
//! (ZWJ/ZWNJ, RLM/LRM, variation selectors), and a **mixed-normalization** finding"*. All
//! three read the concatenated `runs` text of a `text` element and nothing else: no font,
//! no disk, no clock.
//!
//! | what was established | finding |
//! | --- | --- |
//! | a run boundary falls inside a grapheme cluster | `E-RUN-SPLIT-CLUSTER` |
//! | the project's text carries characters that occupy no space | `N-TEXT-INVISIBLE` |
//! | one string is spelled two canonically-equivalent ways | `N-TEXT-MIXED-NORMALIZATION` |
//!
//! **Nothing here rewrites anything, and that is ADR-0007's rule rather than this
//! module's restraint**: *"No tidying pass, ever — whitespace inside a run is content"*,
//! and the fixture's `cobweb  -  cobweb` carries deliberate double spaces that every
//! tidier would eat. So the normalization finding *names* two spellings and never picks
//! one, which is also why it is a `note` and not a `LAYOUT` finding — a `LAYOUT` finding's
//! prose sends the reader to `fmt`, and `fmt` must refuse to make this change.

use montaget_text::glyphs::{codepoint, is_default_ignorable, named_codepoint};
use serde_json::json;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::checks::StyledText;
use crate::finding::{Census, Finding};
use crate::permissive::Loose;
use crate::report::Report;

/// All three checks, over every `text` element in document order.
pub fn check(document: &Loose, report: &mut Report) {
    let elements = crate::checks::styled_text(document);
    split_clusters(document, &elements, report);
    invisible(document, &elements, report);
    mixed_normalization(document, &elements, report);
}

// ---- `E-RUN-SPLIT-CLUSTER` -------------------------------------------------------------

/// ADR-0007: *"A run boundary is style-only."* A style boundary is also a **shaping**
/// boundary — each run is shaped in its own style, so a combining mark that begins a run
/// has no base in that run to attach to and renders on a dotted circle.
///
/// The offsets compared are byte offsets into the concatenated text: the cumulative length
/// of the runs before each boundary, against the cluster starts
/// [`UnicodeSegmentation::grapheme_indices`] reports over the same string. A boundary that
/// is not a cluster start is a boundary inside a cluster.
fn split_clusters(document: &Loose, elements: &[StyledText], report: &mut Report) {
    for element in elements {
        let joined = element.joined();
        let starts: Vec<usize> = joined.grapheme_indices(true).map(|(at, _)| at).collect();

        let mut at = 0usize;
        for (index, run) in element.runs.iter().enumerate() {
            at += run.text.len();
            // The boundary *after* the last run is the end of the string, which every
            // cluster walk agrees with and which no later run begins at.
            // `binary_search` rather than `contains`: `starts` is ascending by
            // construction, and a linear scan per boundary would make one long element's
            // check quadratic in its own length.
            if index + 1 == element.runs.len() || starts.binary_search(&at).is_ok() {
                continue;
            }
            let Some(cluster) = cluster_containing(&joined, &starts, at) else {
                continue;
            };
            // `index + 2`: `index` is zero-based and names the run *before* the boundary,
            // and the finding names the run the boundary falls before.
            report.push(split_finding(document, element, index + 2, cluster));
        }
    }
}

/// The whole cluster the byte offset `at` falls inside.
fn cluster_containing<'a>(joined: &'a str, starts: &[usize], at: usize) -> Option<&'a str> {
    let start = *starts.iter().rev().find(|&&start| start < at)?;
    let end = starts
        .iter()
        .find(|&&candidate| candidate > at)
        .copied()
        .unwrap_or(joined.len());
    joined.get(start..end)
}

fn split_finding(document: &Loose, element: &StyledText, run: usize, cluster: &str) -> Finding {
    let mut finding = Finding::new("E-RUN-SPLIT-CLUSTER")
        .at_file(document.path())
        .at_element(element.subject.clone())
        .field("element", json!(element.subject))
        // One-based, because it is the run an author counts to in the array they are
        // looking at, and because "run 0" reads as "no run". The run *after* the boundary:
        // it is the one holding the orphaned mark, and the one whose `text` an author edits.
        .field("run", json!(run))
        .field("cluster", json!(format!("`{cluster}`")))
        .field("codepoints", json!(codepoints(cluster)));
    if let Some(track) = &element.track {
        finding = finding.at_track(track.clone());
    }
    finding
}

// ---- `N-TEXT-INVISIBLE` -----------------------------------------------------------------

/// ADR-0007's invisible-character census, over the set that ADR names.
///
/// One finding for the project rather than one per element: it is a *census*, and its whole
/// value is the distribution — which characters, how many, and where. Split per element it
/// would be a list of single-member censuses, which is the shape ADR-0043 says a census is
/// not.
///
/// It never judges. ZWJ is how an emoji sequence is spelled, and RLM is how a bidi
/// ambiguity is settled without a `dir` override — ADR-0007 designs `dir` precisely because
/// the alternative is *"embedding invisible control characters in a string"*, which means
/// the ADR expects to find some. So the finding counts them and stops.
fn invisible(document: &Loose, elements: &[StyledText], report: &mut Report) {
    // In code-point order, so the summary and the census read the same way on every run
    // regardless of which element happened to come first.
    let mut found: std::collections::BTreeMap<char, (usize, Vec<String>)> = Default::default();
    for element in elements {
        for c in element.joined().chars() {
            if !is_default_ignorable(c) {
                continue;
            }
            let (count, members) = found.entry(c).or_default();
            *count += 1;
            if members.last() != Some(&element.subject) {
                members.push(element.subject.clone());
            }
        }
    }
    if found.is_empty() {
        return;
    }

    let occurrences: usize = found.values().map(|(count, _)| count).sum();
    let summary = found
        .iter()
        .map(|(c, (count, _))| format!("{} ×{count}", named_codepoint(*c)))
        .collect::<Vec<_>>()
        .join(", ");

    let mut census = Census::on("character");
    for (c, (_, members)) in &found {
        census = census.group(json!(named_codepoint(*c)), members.clone());
    }

    report.push(
        Finding::new("N-TEXT-INVISIBLE")
            .at_file(document.path())
            .field("occurrences", json!(occupy_no_space(occurrences)))
            .field("summary", json!(summary))
            .census(census),
    );
}

// ---- `N-TEXT-MIXED-NORMALIZATION` -------------------------------------------------------

/// Spec #168's story 105: *"two strings that look identical and compare unequal are
/// named"*.
///
/// Grouped by NFC form, which is the form ADR-0007 requires of every writer — so on a
/// document Montaget wrote this finds nothing, and what it is for is the one the format
/// does not control: a caption pasted from a source that decomposes its accents, beside the
/// same caption typed composed. The two render identically (canonical equivalence is a
/// rendering-neutrality guarantee, and HarfBuzz re-normalizes internally), and an
/// exact-string replace finds one of them.
///
/// **A lone non-NFC string is not this finding.** ADR-0007 requires NFC of every *writer*,
/// and a decomposed string with no composed twin is not two spellings of anything — nothing
/// compares unequal, and the census would have one group. Naming it would also be naming a
/// defect whose only repair is the tidying pass ADR-0007 forbids.
fn mixed_normalization(document: &Loose, elements: &[StyledText], report: &mut Report) {
    // A `Vec` for the order — first appearance, so the findings read down the document —
    // and a map for the lookup, so grouping a project's captions is linear rather than
    // quadratic in the number of distinct strings.
    let mut by_nfc: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let mut at: std::collections::HashMap<String, usize> = Default::default();
    for element in elements {
        let raw = element.joined();
        if raw.is_empty() {
            continue;
        }
        let nfc: String = raw.nfc().collect();
        match at.get(&nfc) {
            Some(&index) => by_nfc[index].1.push((element.subject.clone(), raw)),
            None => {
                at.insert(nfc.clone(), by_nfc.len());
                by_nfc.push((nfc, vec![(element.subject.clone(), raw)]));
            }
        }
    }

    for (nfc, members) in by_nfc {
        let spellings: Vec<&String> = {
            let mut spellings: Vec<&String> = Vec::new();
            for (_, raw) in &members {
                if !spellings.contains(&raw) {
                    spellings.push(raw);
                }
            }
            spellings
        };
        if spellings.len() < 2 {
            continue;
        }

        // Grouped by the *form* each spelling is in rather than by its bytes: the bytes are
        // by construction indistinguishable on screen, which is the whole complaint, so a
        // census printing them twice would print the same line twice.
        let mut census = Census::on("normalization");
        for spelling in &spellings {
            let form = form_of(spelling, &nfc);
            let group: Vec<String> = members
                .iter()
                .filter(|(_, raw)| &raw == spelling)
                .map(|(subject, _)| subject.clone())
                .collect();
            census = census.group(json!(form), group);
        }

        report.push(
            Finding::new("N-TEXT-MIXED-NORMALIZATION")
                .at_file(document.path())
                .field("spellings", json!(format!("{} spellings", spellings.len())))
                .field("text", json!(format!("`{}`", elided(&nfc))))
                .census(census),
        );
    }
}

/// Which normalization form a spelling is in, as a word for the census.
///
/// `NFC`, `NFD`, or `neither` — never `NFKC`/`NFKD`, which ADR-0007 forbids to every
/// writer and which this check must therefore not legitimise by naming as a form a document
/// could be in.
fn form_of(spelling: &str, nfc: &str) -> &'static str {
    if spelling == nfc {
        return "NFC";
    }
    if spelling.chars().eq(nfc.nfd()) {
        return "NFD";
    }
    "neither"
}

// ---- shared prose ----------------------------------------------------------------------

/// Every code point of a string, in that notation.
fn codepoints(s: &str) -> String {
    s.chars().map(codepoint).collect::<Vec<_>>().join(" ")
}

/// A caption's worth of text, cut to something a one-line finding can carry.
///
/// Cut by grapheme cluster rather than by byte or by `char`, because a finding about a
/// split cluster that split one itself to fit on the line would be unreadable in exactly
/// the case it is for.
fn elided(text: &str) -> String {
    const LIMIT: usize = 40;
    let clusters: Vec<&str> = text.graphemes(true).collect();
    match clusters.len() > LIMIT {
        true => format!("{}…", clusters[..LIMIT].concat()),
        false => text.to_string(),
    }
}

/// `3 characters in the project's text occupy no space` — the count and the verb that
/// agrees with it, because the template that prints this cannot inflect.
fn occupy_no_space(count: usize) -> String {
    match count {
        1 => "1 character in the project's text occupies no space".to_string(),
        count => format!("{count} characters in the project's text occupy no space"),
    }
}
