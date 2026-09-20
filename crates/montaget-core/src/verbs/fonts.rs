//! `fonts list` and `fonts vendor` — the authoring-time tool ADR-0007 named and ADR-0057
//! specified.
//!
//! ADR-0007's framing is the one to keep: *"wherever this format refuses a convenience, the
//! convenience belongs in an authoring-time tool whose output is inert."* The format
//! refuses system family names; this is where the author finds a font on their machine
//! and freezes its bytes into the repo, so the project that renders here renders on a
//! clean machine.
//!
//! CLI-only, by ADR-0011's cost model. Vendoring a font is a once-per-project act, not a
//! step in the edit loop, and *"a CLI subcommand costs nothing until invoked"* — the agent
//! already carries a shell.
//!
//! # `list`: the enumeration, with the verdict an attempt would get
//!
//! Every font file under the system font directories — or under the roots the caller
//! names — with its path, each face's index (the `index` a `.ttc` entry in the `fonts`
//! table takes) and its **licence status**: `blocklisted`, `recognised-open` or `unknown`,
//! from the same three-bucket check `vendor` runs, *"so an author sees a doomed vendor
//! attempt before making it"*.
//!
//! # `vendor`: a local-only copy behind a hard gate
//!
//! The source is always a local file. There is no URL argument and no fetch-by-name;
//! acquiring a font the author does not have is a separate concern from vendoring one
//! they do, and a pure filesystem copy stays trivially auditable and deterministic.
//!
//! The order of operations is the decision: **the gate runs before any bytes are copied.**
//! Bucket 1 refuses and no flag lifts it; bucket 2 copies and records the recognised
//! identifier; bucket 3 refuses until the caller declares one with `--licence`. Only then
//! is the file written — atomically — under `fonts/` beside the project, and the
//! `fontVendor` entry keyed by that relative path written into the document.
//!
//! It is a write tool, so it answers with the new state's findings, never `ok`
//! (ADR-0011). A freshly vendored file that no `fonts` chain references yet is exactly
//! what that report will say — `N-FONT-ATTESTATION-ORPHANED` — which is the author's cue
//! to write the chain entry, and the reason the answer echoes the entry to write.
//!
//! # What it does not do
//!
//! - **It never overwrites a different font under the same path.** ADR-0007: a font
//!   swapped in place *"silently invalidated every hand-tuned size and hand-placed break in
//!   the document"*. A destination that already holds the identical bytes is re-attested;
//!   one holding different bytes is refused, and the author picks another name or removes
//!   the old file first.
//! - **It never edits the `fonts` table.** The attestation is keyed by file path and
//!   independent of how many chains reference it (ADR-0057); which chain the font joins
//!   is the author's declared change, made with ordinary file tools.
//! - **It never auto-vendors a substitute.** A refusal may name one, labelled by its
//!   actual basis; the author makes the call.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

use crate::finding::Finding;
use crate::fonts::licence::{self, Bucket};
use crate::fonts::sha256_hex;
use crate::layout;
use crate::model::FontAttestation;
use crate::parse;
use crate::report::Report;
use crate::write;

const LIST: &str = "fonts list";
const VENDOR: &str = "fonts vendor";

/// The directory a vendored font lands in, relative to the project — the convention
/// ADR-0007's own worked example uses (`fonts/Inter-SemiBold.ttf`).
const FONTS_DIR: &str = "fonts";

/// The extensions `list` treats as font files. `.dfont` and the Type 1 formats are not
/// here: `skrifa` does not parse them, and a face this build cannot open is not one a
/// project could declare.
const EXTENSIONS: &[&str] = &["ttf", "otf", "ttc", "otc"];

// ---- `fonts list` ---------------------------------------------------------------------

/// One face `list` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Listed {
    /// The file, as the platform spells it.
    pub path: String,
    /// The face's index in the file — what a `.ttc` entry's `index` takes.
    pub index: u32,
    pub family: Option<String>,
    pub postscript: Option<String>,
    /// `blocklisted` / `recognised-open` / `unknown`, judged over the whole file.
    pub status: &'static str,
    /// The recognised identifier, or the blocklist entry that matched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<String>,
}

/// A file with a font extension that could not be read as one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unreadable {
    pub path: String,
    pub reason: String,
}

/// What `list` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inventory {
    /// The roots that were walked — only those that exist, so the answer states where it
    /// actually looked.
    pub roots: Vec<String>,
    pub fonts: Vec<Listed>,
    pub unreadable: Vec<Unreadable>,
}

/// One `fonts list` invocation's answer.
pub struct Listing {
    inventory: Inventory,
    report: Report,
}

impl Listing {
    pub fn report(&self) -> &Report {
        &self.report
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    /// The canonical JSON: the report's own object, plus the inventory under `fonts`.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "fonts",
            serde_json::to_value(&self.inventory).unwrap_or(Value::Null),
        )
    }
}

/// The directories the platform installs fonts into, for the user and for the system.
///
/// A list rather than a discovery API: `fontique`'s system-font discovery is deliberately
/// not compiled into this binary (see `montaget_text::fonts`), and a fixed list of well-known
/// directories is both what that API would walk and something a reader can check.
pub fn system_roots() -> Vec<PathBuf> {
    let home = std::env::home_dir();
    let mut roots = Vec::new();
    if cfg!(target_os = "macos") {
        roots.push(PathBuf::from("/System/Library/Fonts"));
        roots.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = &home {
            roots.push(home.join("Library/Fonts"));
        }
    } else if cfg!(target_os = "windows") {
        if let Some(windir) = std::env::var_os("WINDIR") {
            roots.push(PathBuf::from(windir).join("Fonts"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            roots.push(PathBuf::from(local).join("Microsoft/Windows/Fonts"));
        }
    } else {
        roots.push(PathBuf::from("/usr/share/fonts"));
        roots.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = &home {
            roots.push(home.join(".local/share/fonts"));
            roots.push(home.join(".fonts"));
        }
    }
    roots
}

/// Enumerate every font file under `roots` — or, given none, under the platform's own font
/// directories — with each face's licence status.
///
/// That an empty list means "the system's" is decided here rather than in an adapter,
/// because which directories those are is a rule (ADR-0011: an adapter contains none).
/// Roots that do not exist are skipped rather than reported — `~/.fonts` is absent on most
/// machines and that is not a finding about anything. The walk is sorted, so two runs on
/// one machine answer in one order.
pub fn list(roots: &[PathBuf]) -> Listing {
    let mut inventory = Inventory {
        roots: Vec::new(),
        fonts: Vec::new(),
        unreadable: Vec::new(),
    };
    let system;
    let roots = if roots.is_empty() {
        system = system_roots();
        &system
    } else {
        roots
    };
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        inventory.roots.push(root.display().to_string());
        let mut files = Vec::new();
        walk(root, &mut files);
        files.sort();
        for path in files {
            describe(&path, &mut inventory);
        }
    }
    Listing {
        inventory,
        report: Report::new(LIST, None),
    }
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, files);
        } else if is_font_file(&path) {
            files.push(path);
        }
    }
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn describe(path: &Path, inventory: &mut Inventory) {
    let spelled = path.display().to_string();
    let faces = match std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| montaget_text::names::faces(&bytes))
    {
        Ok(faces) => faces,
        Err(reason) => {
            inventory.unreadable.push(Unreadable {
                path: spelled,
                reason,
            });
            return;
        }
    };
    let bucket = licence::bucket(&faces);
    let matched = match &bucket {
        Bucket::Blocklisted { matched, .. } => Some(matched.to_string()),
        Bucket::Recognised { identifier } => Some(identifier.to_string()),
        Bucket::Unknown => None,
    };
    for face in faces {
        inventory.fonts.push(Listed {
            path: spelled.clone(),
            index: face.index,
            family: face.family,
            postscript: face.postscript,
            status: bucket.status(),
            matched: matched.clone(),
        });
    }
}

// ---- `fonts vendor` -------------------------------------------------------------------

/// What one `fonts vendor` invocation asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vendor {
    /// The local font file to copy.
    pub font: PathBuf,
    /// Bucket 3's declaration: the licence identifier a human has verified. Ignored in
    /// bucket 1 — nothing lifts a blocklist refusal — and checked against the recognised
    /// one in bucket 2.
    pub licence: Option<String>,
    /// Where the file came from, in the author's words, for the attestation's `source`.
    /// Defaults to the absolute path it was copied from.
    pub source: Option<String>,
    /// The path to vendor it under, relative to the project, with `/` separators. Defaults
    /// to `fonts/<file name>`.
    pub destination: Option<String>,
}

/// What `vendor` wrote: the `fontVendor` entry, its key, and the chain entry the author
/// writes next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VendorView {
    /// The attestation's key: the vendored file's path relative to the project.
    pub file: String,
    pub attestation: FontAttestation,
    /// `declared` for bucket 3, `recognised` for bucket 2 — how the licence was established.
    pub licence_basis: &'static str,
    /// The `fonts`-table chain entry that references this file, ready to paste.
    pub chain_entry: Value,
    /// Whether the destination already held these exact bytes.
    pub already_present: bool,
}

/// One `fonts vendor` invocation's answer.
pub struct Vendored {
    view: Option<VendorView>,
    report: Report,
}

impl Vendored {
    pub fn report(&self) -> &Report {
        &self.report
    }

    pub fn view(&self) -> Option<&VendorView> {
        self.view.as_ref()
    }

    /// The canonical JSON: the report's own object, plus what was written under `vendor` —
    /// present and `null` where nothing was, on `query`'s rule.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "vendor",
            match &self.view {
                Some(view) => serde_json::to_value(view).unwrap_or(Value::Null),
                None => Value::Null,
            },
        )
    }
}

/// Vendor a font into the project at `path`, behind the licence gate.
pub fn vendor(path: &Path, ask: &Vendor) -> Vendored {
    let project = Some(path.display().to_string());
    let refused = |report| Vendored { view: None, report };

    let document = match parse::read(path) {
        Ok(document) => document,
        // ADR-0011: nothing may partially process a malformed file — and this verb would
        // write into it.
        Err(finding) => return refused(Report::unparseable(VENDOR, project, *finding)),
    };
    if let Err(not_a_project) = document.shape() {
        let mut report = Report::new(VENDOR, project);
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point fonts vendor at the project file",
        ));
        return refused(report);
    }

    let destination = match destination(ask) {
        Ok(destination) => destination,
        Err(reason) => return refused(Report::rejected(VENDOR, project, reason)),
    };

    // The source's bytes and its own account of itself, before anything is decided.
    let font = ask.font.display().to_string();
    let bytes = match std::fs::read(&ask.font) {
        Ok(bytes) => bytes,
        Err(e) => {
            return refused(Report::rejected(
                VENDOR,
                project,
                format!("`fonts vendor`: {font} could not be read: {e}"),
            ));
        }
    };
    let faces = match montaget_text::names::faces(&bytes) {
        Ok(faces) => faces,
        Err(reason) => {
            return refused(Report::rejected(
                VENDOR,
                project,
                format!("`fonts vendor`: {font} is {reason}"),
            ));
        }
    };

    // ---- The gate. Nothing below this line runs unless it passes. -------------------
    let (identifier, basis) = match licence::bucket(&faces) {
        Bucket::Blocklisted {
            matched,
            name,
            face,
        } => {
            let mut report = Report::new(VENDOR, project);
            report.push(
                Finding::new("E-FONT-BLOCKLISTED")
                    .at_file(document.path())
                    .field("font", json!(font))
                    .field("name", json!(name))
                    .field("face", json!(face))
                    .field("matched", json!(matched))
                    .field("substitutes", json!(substitutes(matched, &name))),
            );
            return refused(report);
        }
        Bucket::Recognised { identifier } => {
            if let Some(declared) = &ask.licence
                && declared != identifier
            {
                // Not a refusal about the font: the file says one thing and the command
                // another, and the command is the one to fix.
                return refused(Report::rejected(
                    VENDOR,
                    project,
                    format!(
                        "`fonts vendor`: {font}'s own licence text recognises as {identifier}, \
                         and `--licence` declares {declared}; drop the flag, or vendor the \
                         file you meant"
                    ),
                ));
            }
            (identifier.to_string(), "recognised")
        }
        Bucket::Unknown => match &ask.licence {
            Some(declared) if !declared.trim().is_empty() => {
                (declared.trim().to_string(), "declared")
            }
            _ => {
                let mut report = Report::new(VENDOR, project);
                report.push(
                    Finding::new("E-FONT-LICENCE-UNKNOWN")
                        .at_file(document.path())
                        .field("font", json!(font))
                        .field("name", json!(display_name(&faces)))
                        .field("detail", json!(licence_detail(&faces)))
                        .repair_value(json!({
                            "value": "have a human verify the file's licence, then re-run \
                                      with `--licence <identifier>`"
                        })),
                );
                return refused(report);
            }
        },
    };

    // ---- The copy. ------------------------------------------------------------------
    let base = crate::checks::project_dir(&document);
    let target = base.join(&destination);
    let already_present = match std::fs::read(&target) {
        Ok(existing) if existing == bytes => true,
        Ok(_) => {
            return refused(Report::rejected(
                VENDOR,
                project,
                format!(
                    "`fonts vendor`: {} already exists and holds a different font; a font \
                     swapped in place silently invalidates every measured size and break \
                     (ADR-0007). Vendor under another name with `--as`, or remove the old \
                     file first",
                    target.display()
                ),
            ));
        }
        Err(_) => false,
    };
    if !already_present {
        if let Some(parent) = target.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            let mut report = Report::new(VENDOR, project);
            report.could_not_write(parent.display(), &e);
            return refused(report);
        }
        if let Err(e) = write::atomically_bytes(&target, &bytes) {
            let mut report = Report::new(VENDOR, project);
            report.could_not_write(target.display(), &e);
            return refused(report);
        }
    }

    // ---- The attestation. -----------------------------------------------------------
    let attestation = FontAttestation {
        licence: identifier,
        source: match &ask.source {
            Some(source) => source.clone(),
            None => std::path::absolute(&ask.font)
                .unwrap_or_else(|_| ask.font.clone())
                .display()
                .to_string(),
        },
        sha256: sha256_hex(&bytes),
    };
    let mut value = document.value().clone();
    if let Some(root) = value.as_object_mut() {
        let table = root
            .entry("fontVendor")
            .or_insert_with(|| Value::Object(Default::default()));
        if !table.is_object() {
            *table = Value::Object(Default::default());
        }
        if let Some(table) = table.as_object_mut() {
            table.insert(
                destination.clone(),
                serde_json::to_value(&attestation).unwrap_or(Value::Null),
            );
            // Keyed by path and sorted by it, so two vendors in either order write one
            // table.
            let mut sorted: Vec<(String, Value)> =
                table.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            table.clear();
            for (k, v) in sorted {
                table.insert(k, v);
            }
        }
    }
    // The new key lands where ADR-0041's schema order puts it — after `fonts`, before
    // `tracks` — through the one predicate that knows that order, rather than an insertion
    // this verb would have to keep in step with the model.
    let canonical = write::canonical(&layout::canonicalise(&value));
    if let Err(e) = write::atomically(path, &canonical) {
        let mut report = Report::new(VENDOR, project);
        report.could_not_write(document.path(), &e);
        return refused(report);
    }

    // The write-tool invariant: the new state's findings, never `ok`.
    let mut report = crate::verbs::validate::validate(path);
    report.tool = VENDOR.to_string();
    Vendored {
        view: Some(VendorView {
            file: destination.clone(),
            attestation,
            licence_basis: basis,
            chain_entry: json!({"file": destination}),
            already_present,
        }),
        report,
    }
}

/// The relative path the font is vendored under, or why the one asked for is not one.
///
/// Inside the project, with `/` separators, because it is the string a `fonts` chain
/// entry will carry and the string `validate` resolves against the project's directory
/// (ADR-0053). A `..` or an absolute path would attest a file outside the repo, which is
/// the opposite of vendoring.
fn destination(ask: &Vendor) -> Result<String, String> {
    match &ask.destination {
        None => {
            let name = ask
                .font
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    format!("`fonts vendor`: {} has no file name", ask.font.display())
                })?;
            Ok(format!("{FONTS_DIR}/{name}"))
        }
        Some(asked) => {
            let asked = asked.replace('\\', "/");
            let escapes = asked.starts_with('/')
                || asked.split('/').any(|part| part == "..")
                || Path::new(&asked).is_absolute();
            if asked.trim().is_empty() || escapes {
                return Err(format!(
                    "`fonts vendor`: `--as {asked}` must be a path inside the project, \
                     relative to it, like `fonts/Inter-SemiBold.ttf`"
                ));
            }
            Ok(asked)
        }
    }
}

/// What a refusal calls the font: its family name where it states one, else its
/// PostScript name, else its file.
fn display_name(faces: &[montaget_text::FaceNames]) -> String {
    faces
        .first()
        .and_then(|f| f.family.clone().or_else(|| f.postscript.clone()))
        .unwrap_or_else(|| "a face with no name".to_string())
}

/// Why bucket 3 was reached, in words: no licence strings, or strings that named nothing
/// recognised — quoted, so the reader can judge them without opening the file.
fn licence_detail(faces: &[montaget_text::FaceNames]) -> String {
    let first = faces.iter().find_map(|f| {
        f.licence_description
            .clone()
            .or_else(|| f.licence_url.clone())
    });
    match first {
        None => "its `name` table carries no licence description (ID 13) and no licence URL \
                 (ID 14)"
            .to_string(),
        Some(text) => {
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            let shown: String = text.chars().take(160).collect();
            let ellipsis = if text.chars().count() > 160 {
                "…"
            } else {
                ""
            };
            format!(
                "its licence text reads \"{shown}{ellipsis}\", which names no open licence Montaget recognises"
            )
        }
    }
}

/// The substitutes a refusal names, as one sentence, or `none known`.
fn substitutes(matched: &str, name: &str) -> String {
    let known = licence::substitutes(matched, name);
    if known.is_empty() {
        return "none known".to_string();
    }
    known
        .iter()
        .map(|s| format!("{} ({}, {}) — {}", s.name, s.licence, s.source, s.basis))
        .collect::<Vec<_>>()
        .join("; ")
}
