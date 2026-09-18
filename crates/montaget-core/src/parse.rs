//! Reading a project file, and the one failure that is not a finding about a document.
//!
//! ADR-0011: *"Every tool fails identically on a malformed file, with line, column, byte
//! offset, the offending line verbatim, and a caret. Nothing may partially process a
//! malformed file: read, parse, check, write atomically, or do nothing."*

use serde_json::Value;
use std::path::Path;

use crate::document::Document;
use crate::finding::Finding;

/// Read and parse a project file, or produce the one finding that means "fix the file".
///
/// The error is boxed because a `Finding` is a large value and every caller on the happy
/// path would otherwise carry its width.
pub fn read(path: &Path) -> Result<Document, Box<Finding>> {
    let display = path.display().to_string();

    let source = std::fs::read_to_string(path).map_err(|e| {
        // Its own code, not `E-PARSE`. Both are exit 2 — "the file could not be read or
        // parsed" — but a file that was never opened has no line, column or offending
        // line, and `E-PARSE`'s template is built around having all three. One code,
        // one field set (ADR-0006), so an absent file gets the code whose field set it
        // can actually fill.
        Box::new(
            Finding::new("E-READ")
                .at_file(&display)
                .field("reason", Value::String(e.to_string()))
                .advise_class(serde_json::json!({"value": "check the path and its permissions"})),
        )
    })?;

    match serde_json::from_str::<Value>(&source) {
        Ok(value) => Ok(Document::from_value(display, value)),
        Err(e) => {
            let line = e.line() as u32;
            let column = e.column() as u32;
            let byte_offset = byte_offset_of(&source, line, column);
            Err(Box::new(
                Finding::new("E-PARSE")
                    .at_file(&display)
                    .at_offset(line, column, byte_offset)
                    .field("reason", Value::String(reason_of(&e)))
                    .field("line", Value::from(line))
                    .field("column", Value::from(column))
                    .field("byte_offset", Value::from(byte_offset))
                    .field("excerpt", Value::String(excerpt(&source, line, column)))
                    .refuse_class(),
            ))
        }
    }
}

/// `serde_json`'s `Display` appends " at line L column C", which the template already
/// states from the located fields. Keep the cause, drop the duplicate.
fn reason_of(e: &serde_json::Error) -> String {
    let text = e.to_string();
    match text.find(" at line ") {
        Some(i) => text[..i].to_string(),
        None => text,
    }
}

/// The offset, in bytes from the start of the file, that a 1-based line and column
/// resolve to. `serde_json` counts columns in bytes, so this is an addition rather than
/// a re-scan, and it degrades to the start of the line rather than panicking if the
/// column runs past the line's end.
fn byte_offset_of(source: &str, line: u32, column: u32) -> u64 {
    let start = line_start(source, line);
    let line_len = source[start..].find('\n').unwrap_or(source.len() - start);
    let within = (column.saturating_sub(1) as usize).min(line_len);
    (start + within) as u64
}

fn line_start(source: &str, line: u32) -> usize {
    let mut start = 0;
    for _ in 1..line.max(1) {
        match source[start..].find('\n') {
            Some(i) => start += i + 1,
            None => break,
        }
    }
    start
}

/// The offending line verbatim, and a caret under the column.
///
/// ```text
///   3 |   "fps": ,
///     |          ^
/// ```
fn excerpt(source: &str, line: u32, column: u32) -> String {
    let start = line_start(source, line);
    let text = source[start..]
        .split('\n')
        .next()
        .unwrap_or_default()
        .trim_end_matches('\r');

    let number = line.to_string();
    let gutter = " ".repeat(number.len());
    // Count in characters, not bytes, so the caret lands under the glyph on a line
    // carrying multi-byte text. The column may land mid-character, so walk back to a
    // boundary rather than slicing blindly.
    let mut byte_column = (column.saturating_sub(1) as usize).min(text.len());
    while byte_column > 0 && !text.is_char_boundary(byte_column) {
        byte_column -= 1;
    }
    let pad = " ".repeat(text[..byte_column].chars().count());

    format!("{number} | {text}\n{gutter} | {pad}^")
}
