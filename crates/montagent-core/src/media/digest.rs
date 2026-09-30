//! ADR-0117's staleness digest: *is the file at the output path the rendering of this
//! document, as it and its files stand now?*
//!
//! `render` stamps it into the deliverable ([`super::attest`]) and `verify` recomputes it. Two
//! equal digests mean the document and every file it resolves are what they were when the
//! file was rendered; two unequal ones mean one of them changed, and every measurement
//! mismatch downstream would descend from that one fact — which is why `verify` stops there.
//!
//! **What is hashed, and why each part:**
//!
//! - **The document in canonical (`fmt`) form**, so a whitespace or key-order edit — which
//!   renders identically — does not make a deliverable stale.
//! - **Every local source's [`sidecar::fingerprint`]** (size plus the first and last 64 KiB),
//!   under the spelling the document uses. A source replaced on disk after the render breaks
//!   *"this is the rendering of this project"* exactly as a document edit does — and
//!   MONTAGENT-1's renumbering shuffle is the case: a `mv` keeps mtime to the nanosecond, so
//!   **size and mtime are never an identity here.**
//! - **Every `fonts`-table file's SHA-256**, which is [`FontEntry::sha256`]'s definition.
//!
//! The spelling rather than the resolved path goes into the hash, so moving a whole project
//! tree does not make its deliverable stale by this measure. (ADR-0104's identity still does
//! make it `Foreign`; that is a different question, answered first.)
//!
//! **Not hashed:** the engine version (see [`super::attest`]), and a remote source's bytes.
//! ADR-0056 gives remote sources no persistent identity, so a URL whose content changes
//! upstream is outside what this digest can see; its spelling is in the document and is
//! covered by the first part. Stated as a limit in `verify`'s `NOT CHECKED`.
//!
//! **A fingerprint that could not be read is never a mismatch.** ADR-0069's rule for this
//! module's inputs is that such a failure is silence: the answer is [`Digest::Unread`], which
//! `render` stamps as `digest=none` and `verify` reports as *staleness unknown*.
//!
//! [`FontEntry::sha256`]: super::sidecar::FontEntry::sha256

use std::collections::BTreeSet;

use serde_json::Value;

use super::{Source, display_local, sidecar};
use crate::fonts::sha256_hex;
use crate::permissive::Loose;

/// The version of what is hashed. In the input rather than beside it, so a later change to
/// what the digest covers cannot collide with a digest this grammar wrote.
const GRAMMAR: &str = "montagent-digest/1";

/// A digest, or the inputs that kept one from being computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Digest {
    /// Lowercase hex SHA-256 over every part above.
    Known(String),
    /// Every input that could not be read, as a person would type its path. Never empty.
    Unread(Vec<String>),
}

impl Digest {
    pub fn known(&self) -> Option<&str> {
        match self {
            Digest::Known(hex) => Some(hex),
            Digest::Unread(_) => None,
        }
    }
}

/// The digest of `document` and every file it resolves, now.
pub fn of(document: &Loose) -> Digest {
    let base = crate::checks::project_dir(document);
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    let mut feed = |text: &str| sha2::Digest::update(&mut hasher, text.as_bytes());
    let mut unread = Vec::new();

    feed(GRAMMAR);
    feed("\n");
    feed(&crate::checks::layout::canonical(document));

    // Document order, first appearance only: the order is a property of the document, so two
    // runs over one document feed the same sequence.
    let mut seen = BTreeSet::new();
    for element in document.elements() {
        let Some(spelled) = element.get("source").and_then(Value::as_str) else {
            continue;
        };
        if !seen.insert(spelled.to_string()) {
            continue;
        }
        let Source::Local(path) = Source::resolve(spelled, &base) else {
            continue;
        };
        match sidecar::fingerprint(&path) {
            Some(fingerprint) => feed(&format!("source\t{spelled}\t{fingerprint}\n")),
            None => unread.push(display_local(&path)),
        }
    }

    if let Some(table) = document.value().get("fonts").and_then(Value::as_object) {
        for (key, chain) in table {
            for entry in chain.as_array().into_iter().flatten() {
                let Some(file) = entry.get("file").and_then(Value::as_str) else {
                    continue;
                };
                let path = base.join(file);
                match std::fs::read(&path) {
                    Ok(bytes) => feed(&format!("font\t{key}\t{file}\t{}\n", sha256_hex(&bytes))),
                    Err(_) => unread.push(display_local(&path)),
                }
            }
        }
    }

    if !unread.is_empty() {
        return Digest::Unread(unread);
    }
    Digest::Known(format!("{:x}", sha2::Digest::finalize(hasher)))
}
