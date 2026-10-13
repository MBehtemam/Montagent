//! Project-level master effects validation (ADR-0205).
//!
//! - **`E-PROJECT-EFFECT-MASK`**: A `mask` effect is an element-local shape mask
//!   and cannot be applied at the project root level.
//! - **`E-PROJECT-EFFECT-KEYFRAME`**: Master effects execute at the device boundary
//!   and do not admit keyframes or animations; all parameters must be static literals.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    for finding in findings(document) {
        report.push(finding);
    }
}

pub(crate) fn findings(document: &Loose) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(effects) = document.value().get("effects").and_then(Value::as_array) else {
        return out;
    };

    for (index, effect) in effects.iter().enumerate() {
        let name = effect.get("name").and_then(Value::as_str).unwrap_or("");

        if name == "mask" {
            out.push(
                Finding::new("E-PROJECT-EFFECT-MASK")
                    .at_file(document.path())
                    .field("index", json!(index)),
            );
        }

        if let Some(obj) = effect.as_object() {
            for (key, val) in obj {
                if key == "name" {
                    continue;
                }
                if is_keyframed(val) {
                    out.push(
                        Finding::new("E-PROJECT-EFFECT-KEYFRAME")
                            .at_file(document.path())
                            .field("index", json!(index))
                            .field("property", json!(key)),
                    );
                }
            }
        }
    }

    out
}

fn is_keyframed(val: &Value) -> bool {
    match val {
        Value::Array(items) => items.iter().any(|item| {
            item.as_object()
                .is_some_and(|obj| obj.contains_key("t") && obj.contains_key("v"))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mask_in_project_effects_is_refused() {
        let doc = Loose::new(
            "test.json",
            json!({
                "frame": {"width": 1920, "height": 1080},
                "fps": 30,
                "effects": [
                    {"name": "mask", "shape": "rect"}
                ],
                "tracks": []
            }),
        );
        let f = findings(&doc);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code, "E-PROJECT-EFFECT-MASK");
    }

    #[test]
    fn keyframes_in_project_effects_are_refused() {
        let doc = Loose::new(
            "test.json",
            json!({
                "frame": {"width": 1920, "height": 1080},
                "fps": 30,
                "effects": [
                    {
                        "name": "blur",
                        "radius": [
                            {"t": 0, "v": 0.0},
                            {"t": 1000, "v": 10.0, "ease": "linear"}
                        ]
                    }
                ],
                "tracks": []
            }),
        );
        let f = findings(&doc);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code, "E-PROJECT-EFFECT-KEYFRAME");
    }
}
