//! The canonical writing convention: the one place a project becomes bytes.
//!
//! ADR-0005 made the convention load-bearing rather than cosmetic — *"agents edit by
//! exact-string replace, and unstable formatting means a replace can silently hit two
//! elements"* — and ADR-0041 turned it from folklore into a checked MUST after it broke
//! anyway, because nothing checked it. ADR-0007 fixes the one rule that looks like a
//! library default and is not: **the whole element, runs included, serialises on one
//! line**, with no length threshold, because a conditional serializer makes formatting
//! non-idempotent — add three characters, cross the threshold, and the next write reflows
//! the element into eight lines with a diff that has nothing to do with the edit.
//!
//! The layout is a property of *where* a value sits, not of what it contains, so this
//! module takes a [`serde_json::Value`] rather than a [`crate::model::Project`]. That is
//! what lets the permissive path (ADR-0042) — a document carrying an unknown key, which
//! the strict model by construction cannot hold — print through the identical writer.
//!
//! [`atomically`] is the other half: the whole-file write every write tool reuses, which
//! replaces a project's bytes or leaves the file exactly as it was. What is *canonical* and
//! what is *durable* are two questions, and they are two functions here so that neither can
//! change the other by accident.

use std::io::Write as _;
use std::path::Path;

use serde_json::Value;

/// Render a project as its canonical bytes, ending in a newline.
pub fn canonical(project: &Value) -> String {
    let Value::Object(root) = project else {
        // Not project-shaped. Nothing here can improve on the compact form, and inventing
        // a layout for a document this writer does not understand is how a formatter
        // damages a file it was pointed at by mistake (ADR-0042).
        return format!("{}\n", inline(project, Spacing::Spaced));
    };

    let mut out = String::from("{\n");
    for (i, (key, value)) in root.iter().enumerate() {
        out.push_str("  ");
        out.push_str(&quoted(key));
        out.push_str(": ");
        if key == "tracks" {
            out.push_str(&tracks(value, 2));
        } else {
            out.push_str(&inline(value, Spacing::Spaced));
        }
        if i + 1 < root.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

/// The tracks array: one track per entry, each expanded, with its elements one per line.
fn tracks(value: &Value, indent: usize) -> String {
    let Value::Array(tracks) = value else {
        return inline(value, Spacing::Spaced);
    };
    if tracks.is_empty() {
        // A header-only project writes `"tracks": []`, not an empty pair of lines.
        return "[]".to_string();
    }

    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    let mut out = String::from("[\n");
    for (i, track) in tracks.iter().enumerate() {
        out.push_str(&inner);
        out.push_str(&object(track, indent + 2));
        if i + 1 < tracks.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&pad);
    out.push(']');
    out
}

/// One track: its own keys on their own lines, its elements one per line.
fn object(value: &Value, indent: usize) -> String {
    let Value::Object(track) = value else {
        return inline(value, Spacing::Spaced);
    };

    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    let mut out = String::from("{\n");
    for (i, (key, value)) in track.iter().enumerate() {
        out.push_str(&inner);
        out.push_str(&quoted(key));
        out.push_str(": ");
        if key == "elements" {
            out.push_str(&elements(value, indent + 2));
        } else {
            out.push_str(&inline(value, Spacing::Spaced));
        }
        if i + 1 < track.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&pad);
    out.push('}');
    out
}

/// The elements array: one element per line, each tight.
///
/// Tight rather than spaced is what makes an element's text a unique matchable substring:
/// the same fields written with spaces would still be legal JSON and would still round-trip
/// through every parser, and every exact-string replace an agent had written against the
/// old spelling would find nothing.
fn elements(value: &Value, indent: usize) -> String {
    let Value::Array(elements) = value else {
        return inline(value, Spacing::Spaced);
    };
    if elements.is_empty() {
        return "[]".to_string();
    }

    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    let mut out = String::from("[\n");
    for (i, element) in elements.iter().enumerate() {
        out.push_str(&inner);
        out.push_str(&inline(element, Spacing::Tight));
        if i + 1 < elements.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&pad);
    out.push(']');
    out
}

/// Whether a one-line value breathes. The header does; an element does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spacing {
    Spaced,
    Tight,
}

impl Spacing {
    fn colon(self) -> &'static str {
        match self {
            Spacing::Spaced => ": ",
            Spacing::Tight => ":",
        }
    }

    fn comma(self) -> &'static str {
        match self {
            Spacing::Spaced => ", ",
            Spacing::Tight => ",",
        }
    }
}

/// One value on one line.
fn inline(value: &Value, spacing: Spacing) -> String {
    match value {
        Value::Object(map) => {
            let body: Vec<String> = map
                .iter()
                .map(|(key, value)| {
                    format!(
                        "{}{}{}",
                        quoted(key),
                        spacing.colon(),
                        inline(value, spacing)
                    )
                })
                .collect();
            format!("{{{}}}", body.join(spacing.comma()))
        }
        Value::Array(items) => {
            let body: Vec<String> = items.iter().map(|item| inline(item, spacing)).collect();
            format!("[{}]", body.join(spacing.comma()))
        }
        // Scalars go through `serde_json` itself: string escaping is exacting, and numbers
        // must print in the shortest form that round-trips — `1.0` stays `1.0` and does not
        // become `1`, which would change what the next exact-string replace matches.
        // Non-ASCII stays raw, which ADR-0007 requires of every writer.
        scalar => scalar.to_string(),
    }
}

fn quoted(key: &str) -> String {
    Value::String(key.to_string()).to_string()
}

/// Replace `path`'s contents with `bytes`, atomically — or leave the file exactly as it
/// was.
///
/// ADR-0011: *"Nothing may partially process a malformed file: read, parse, check, write
/// atomically, or do nothing."* The hazard is not theoretical. The file this writes is the
/// one the agent is mid-edit on and the one git is tracking, and a half-written project is
/// worse than an unformatted one: it is unparseable, so the next run of every verb answers
/// `E-PARSE` and the work is gone.
///
/// So the destination is never opened for writing. The bytes go to a sibling temp file,
/// are flushed and synced, and then a single `rename` puts them in place — which is atomic
/// on every filesystem Montaget's six targets run on. A failure anywhere before the rename
/// leaves the original untouched and takes the temp file with it.
///
/// The temp file is a *sibling* rather than a file in the system temp directory, because
/// `rename` across filesystems fails — and a project on an external disk or a network share
/// is the normal case for video work, not an exotic one.
///
/// Every later write tool reuses this. It takes bytes rather than a document on purpose:
/// what is canonical is [`canonical`]'s business, and what is durable is this function's,
/// and neither should be able to change the other by accident.
pub fn atomically(path: &Path, bytes: &str) -> std::io::Result<()> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let temp = directory.join(temp_name(path));

    // Scoped so the handle is closed before the rename. Windows refuses to rename a file
    // that is still open, and a write that works on Unix and fails on Windows is exactly
    // the class of defect the six-target suite exists to catch (#189).
    let written = (|| {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(bytes.as_bytes())?;
        // `write_all` returning `Ok` only means the bytes reached the OS. Without this, a
        // machine that loses power between the rename and the flush has a file that is
        // present, named correctly, and empty.
        file.sync_all()
    })();

    if let Err(e) = written.and_then(|()| std::fs::rename(&temp, path)) {
        // Best-effort: the write already failed, and failing to tidy up after it is not a
        // second thing to report at the caller. What matters — the original file is
        // untouched — is true either way.
        let _ = std::fs::remove_file(&temp);
        return Err(e);
    }
    Ok(())
}

/// A sibling name nothing else will pick.
///
/// The process id and a clock reading, so two Montagets formatting one file — a `fmt` in a
/// terminal and an MCP server in the same second — cannot land on the same temp path and
/// interleave their bytes.
fn temp_name(path: &Path) -> String {
    let stem = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!(".{stem}.montaget-tmp-{}-{nanos}", std::process::id())
}
