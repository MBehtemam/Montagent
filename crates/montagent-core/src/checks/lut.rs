//! 3D LUT (.cube) validation check (ADR-0206).
//!
//! Validates:
//! 1. Every `lut3d` effect on `project.effects` and on every track element references a `.cube` file on disk.
//! 2. Emits `E-LUT-NOT-FOUND` if the file does not exist.
//! 3. Emits `E-LUT-INVALID` if the file cannot be parsed according to ADR-0206 (e.g. 1D LUT, wrong size, malformed entries).

use std::fs;

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// Check every `lut3d` effect for valid `.cube` files on disk (ADR-0206).
pub fn check(document: &Loose, report: &mut Report) {
    let base = crate::checks::project_dir(document);

    // Check project-level effects
    if let Some(effects) = document.value().get("effects").and_then(Value::as_array) {
        check_effects_luts("the project", effects, &base, document.path(), report);
    }

    // Check element-level effects across all tracks
    for element in document.elements() {
        let name = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        if let Some(effects) = element.get("effects").and_then(Value::as_array) {
            check_effects_luts(&name, effects, &base, document.path(), report);
        }
    }
}

fn check_effects_luts(
    subject: &str,
    effects: &[Value],
    base: &std::path::Path,
    doc_path: &str,
    report: &mut Report,
) {
    for (i, effect) in effects.iter().enumerate() {
        let Some(name) = effect.get("name").and_then(Value::as_str) else {
            continue;
        };
        if name != "lut3d" {
            continue;
        }

        let Some(source) = effect.get("source").and_then(Value::as_str) else {
            continue;
        };

        let path = base.join(source);
        if !path.exists() || !path.is_file() {
            report.push(
                Finding::new("E-LUT-NOT-FOUND")
                    .at_file(doc_path)
                    .field("element", json!(subject))
                    .field("index", json!(i))
                    .field("source", json!(source))
                    .field("path", json!(path.to_string_lossy()))
                    .repair_value(json!({
                        "value": "put the .cube file where the effect says, or correct the path"
                    })),
            );
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                report.push(
                    Finding::new("E-LUT-INVALID")
                        .at_file(doc_path)
                        .field("element", json!(subject))
                        .field("index", json!(i))
                        .field("source", json!(source))
                        .field("reason", json!(e.to_string())),
                );
                continue;
            }
        };

        if let Err(err) = montagent_render::canvas::parse_cube(&content) {
            report.push(
                Finding::new("E-LUT-INVALID")
                    .at_file(doc_path)
                    .field("element", json!(subject))
                    .field("index", json!(i))
                    .field("source", json!(source))
                    .field("reason", json!(err)),
            );
        }
    }
}
