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
