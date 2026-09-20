//! The attestation half of ADR-0057, run by `validate` on every project.
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

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::fonts::sha256_hex;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
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
