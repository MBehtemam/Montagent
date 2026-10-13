//! Output deliverable validation (ADR-0202, ADR-0203, ADR-0204).

use serde_json::json;
use std::collections::HashSet;

use crate::finding::Finding;
use crate::model::output::infer_container;
use crate::permissive::Loose;
use crate::report::Report;

/// Validates deliverable output configuration, container matching, codec/profile compatibility,
/// alpha transparency requirements, and audio stem definitions.
pub fn check(document: &Loose, report: &mut Report) {
    let at = |code: &str| Finding::new(code).at_file(document.path());

    let Some(raw_output) = document.value().get("output") else {
        return;
    };

    let (path_str, container_opt, video_obj, stems_arr) = match raw_output {
        serde_json::Value::String(s) => (Some(s.as_str()), None, None, None),
        serde_json::Value::Object(map) => {
            let path = map.get("path").and_then(|v| v.as_str());
            let container = map.get("container").and_then(|v| v.as_str());
            let video = map.get("video").and_then(|v| v.as_object());
            let stems = map.get("stems").and_then(|v| v.as_array());
            (path, container, video, stems)
        }
        _ => return, // non-string/non-object is caught by schema checks
    };

    let Some(path) = path_str else {
        report.push(at("E-OUTPUT-EMPTY"));
        return;
    };

    if path.trim().is_empty() || path.contains("..") {
        report.push(at("E-OUTPUT-EMPTY"));
        return;
    }

    let inferred = infer_container(path);

    if let Some(container) = container_opt {
        if let Some(inferred) = inferred
            && container.to_ascii_lowercase() != inferred
        {
            report.push(
                at("E-OUTPUT-EXTENSION-MISMATCH")
                    .field("container", json!(container))
                    .field("path", json!(path)),
            );
        }
    } else if inferred.is_none() {
        report.push(at("E-OUTPUT-EXTENSION-UNKNOWN").field("path", json!(path)));
    }

    let effective_container = container_opt
        .map(|c| c.to_ascii_lowercase())
        .or_else(|| inferred.map(|s| s.to_string()))
        .unwrap_or_else(|| "mp4".to_string());

    if let Some(video) = video_obj {
        let codec = video
            .get("codec")
            .and_then(|v| v.as_str())
            .unwrap_or("h264");
        let profile = video.get("profile").and_then(|v| v.as_str());
        let alpha = video
            .get("alpha")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check container / codec compatibility
        if effective_container == "mp4" && (codec == "prores" || codec == "dnxhr") {
            report.push(
                at("E-CONTAINER-CODEC-MISMATCH")
                    .field("container", json!(effective_container))
                    .field("codec", json!(codec)),
            );
        }

        // Check profile validity
        if let Some(prof) = profile {
            match codec {
                "prores" => {
                    const PRORES_PROFILES: &[&str] =
                        &["proxy", "lt", "standard", "hq", "4444", "4444xq"];
                    if !PRORES_PROFILES.contains(&prof) {
                        report.push(
                            at("E-CODEC-PROFILE-INVALID")
                                .field("codec", json!(codec))
                                .field("profile", json!(prof)),
                        );
                    }
                }
                "dnxhr" => {
                    const DNXHR_PROFILES: &[&str] = &[
                        "dnxhr_lb",
                        "lb",
                        "dnxhr_sq",
                        "sq",
                        "dnxhr_hq",
                        "hq",
                        "dnxhr_hqx",
                        "hqx",
                        "dnxhr_444",
                        "444",
                    ];
                    if !DNXHR_PROFILES.contains(&prof) {
                        report.push(
                            at("E-CODEC-PROFILE-INVALID")
                                .field("codec", json!(codec))
                                .field("profile", json!(prof)),
                        );
                    }
                }
                "h264" => {
                    const H264_PROFILES: &[&str] = &["baseline", "main", "high"];
                    if !H264_PROFILES.contains(&prof) {
                        report.push(
                            at("E-CODEC-PROFILE-INVALID")
                                .field("codec", json!(codec))
                                .field("profile", json!(prof)),
                        );
                    }
                }
                _ => {}
            }
        }

        // Check alpha support
        if alpha {
            if codec != "prores" {
                report.push(at("E-CODEC-ALPHA-UNSUPPORTED").field("codec", json!(codec)));
            } else if let Some(prof) = profile {
                if prof != "4444" && prof != "4444xq" {
                    report.push(
                        at("E-CODEC-ALPHA-UNSUPPORTED")
                            .field("codec", json!(format!("prores {prof}"))),
                    );
                }
            } else {
                // ProRes with no profile defaults to standard (422), which does not support alpha
                report.push(
                    at("E-CODEC-ALPHA-UNSUPPORTED")
                        .field("codec", json!("prores (profile unstated)")),
                );
            }
        }
    }

    // Check stems
    if let Some(stems) = stems_arr {
        let mut paths_seen = HashSet::new();
        paths_seen.insert(path.to_string());

        let audible_elements: Vec<(&str, &serde_json::Value)> = document
            .elements_with_track()
            .filter_map(|(element, track)| {
                let kind = element.get("type").and_then(|v| v.as_str())?;
                if kind != "audio" && kind != "video" {
                    return None;
                }
                if let Some(vol) = element.get("volume").and_then(|v| v.as_f64())
                    && vol == 0.0
                {
                    return None;
                }
                let role = element
                    .get("role")
                    .and_then(|v| v.as_str())
                    .or_else(|| track.get("role").and_then(|v| v.as_str()))
                    .unwrap_or("sfx");
                Some((role, element))
            })
            .collect();

        for stem in stems {
            let Some(stem_obj) = stem.as_object() else {
                continue;
            };
            let stem_role = stem_obj
                .get("role")
                .or_else(|| stem_obj.get("stem"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let stem_path = stem_obj.get("path").and_then(|v| v.as_str()).unwrap_or("");

            if !stem_path.ends_with(".wav") {
                report.push(at("E-STEM-FORMAT-INVALID").field("path", json!(stem_path)));
            }

            if !paths_seen.insert(stem_path.to_string()) {
                report.push(at("E-STEM-PATH-COLLISION").field("path", json!(stem_path)));
            }

            let count = match stem_role {
                "dialogue" | "voiceover" | "music" | "sfx" | "foley" => audible_elements
                    .iter()
                    .filter(|(r, _)| *r == stem_role)
                    .count(),
                "m_and_e" => audible_elements
                    .iter()
                    .filter(|(r, _)| *r != "dialogue" && *r != "voiceover")
                    .count(),
                "mix" => audible_elements.len(),
                _ => 0,
            };

            if count == 0 {
                report.push(
                    at("E-STEM-ROLE-EMPTY")
                        .field("path", json!(stem_path))
                        .field("role", json!(stem_role)),
                );
            }
        }
    }
}
