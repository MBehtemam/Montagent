//! Portable, hash-bound acquisition evidence. Reading evidence never fetches anything.
use crate::fonts::sha256_hex;
use crate::model::FontEvidence;
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};

pub struct Prepared {
    pub face_index: Option<u32>,
    pub writes: Vec<(String, Vec<u8>)>,
    pub reference: FontEvidence,
    pub recognised: Option<String>,
    pub alternatives: Vec<String>,
    pub conflict: bool,
    pub resolved_licence: Option<String>,
}

/// Reject traversal and symlink escapes both for reading and prospective writes.
pub fn confined(base: &Path, file: &str) -> Result<PathBuf, String> {
    let relative = Path::new(file);
    if file.is_empty()
        || file.contains('\\')
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("invalid evidence path: {file}"));
    }
    let base = if base.as_os_str().is_empty() {
        Path::new(".")
    } else {
        base
    };
    let root = base.canonicalize().map_err(|e| e.to_string())?;
    let target = root.join(relative);
    let mut existing = target.as_path();
    while std::fs::symlink_metadata(existing).is_err() {
        existing = existing.parent().ok_or("invalid destination")?;
    }
    if !existing
        .canonicalize()
        .map_err(|e| e.to_string())?
        .starts_with(&root)
    {
        return Err(format!("path escapes project: {file}"));
    }
    Ok(target)
}
fn checked(base: &Path, value: &Value) -> Result<Vec<u8>, String> {
    let file = value["file"].as_str().ok_or("missing evidence file")?;
    let hash = value["sha256"].as_str().ok_or("missing evidence hash")?;
    let bytes = std::fs::read(confined(base, file)?).map_err(|e| format!("{file}: {e}"))?;
    if sha256_hex(&bytes) != hash {
        return Err(format!("evidence hash mismatch: {file}"));
    }
    Ok(bytes)
}
fn recognised(text: &str) -> Option<String> {
    let text = text.to_ascii_lowercase();
    if text.contains("sil open font license version 1.1")
        && text.contains("permission is hereby granted")
        && text.contains("reserved font name")
    {
        Some("OFL-1.1".into())
    } else if text.contains("apache license")
        && text.contains("version 2.0, january 2004")
        && text.contains("terms and conditions for use, reproduction, and distribution")
    {
        Some("Apache-2.0".into())
    } else {
        None
    }
}
pub fn prepare(
    receipt_path: &Path,
    resolution: Option<&Path>,
    font: &[u8],
    destination: &str,
) -> Result<Prepared, String> {
    let raw = std::fs::read(receipt_path).map_err(|e| e.to_string())?;
    let receipt: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    if receipt["schema_version"] != 1 {
        return Err("unsupported receipt schema".into());
    }
    #[derive(serde::Deserialize)]
    struct SelectedFont {
        face_index: u32,
    }
    let selected: SelectedFont = serde_json::from_value(receipt["font"].clone())
        .map_err(|e| format!("invalid receipt selected Face: {e}"))?;
    let hash = sha256_hex(font);
    let base = match receipt_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    if receipt["font"]["sha256"] != hash || checked(base, &receipt["font"])? != font {
        return Err("receipt font does not match vendor input".into());
    }
    let directory = format!("{destination}.evidence");
    let mut writes = vec![(format!("{directory}/receipt.json"), raw.clone())];
    let mut artifacts = vec![json!({"file":"receipt.json","sha256":sha256_hex(&raw)})];
    let materials = receipt["materials"]
        .as_array()
        .ok_or("missing receipt materials")?;
    let source = &receipt["source"];
    let commit = source["commit"].as_str().unwrap_or("");
    let repository_font = source["repository_path"].as_str().unwrap_or("");
    let pinned = family(repository_font).is_some()
        && source["kind"] == "google_fonts"
        && commit.len() == 40
        && commit.bytes().all(|b| b.is_ascii_hexdigit())
        && source["final_url"].as_str().is_some_and(|url| {
            url == format!(
                "https://raw.githubusercontent.com/google/fonts/{commit}/{repository_font}"
            )
        });
    let mut alternatives = Vec::new();
    let conflict = receipt["licence_findings"].as_array().is_some_and(|fs| {
        fs.iter().any(|f| {
            f["code"]
                .as_str()
                .is_some_and(|c| c.to_ascii_uppercase().contains("CONFLICT"))
                || f["kind"] == "material_conflict"
        })
    });
    for (index, material) in materials.iter().enumerate() {
        if material["font_sha256"] != hash {
            return Err("material is not associated with font hash".into());
        }
        let bytes = checked(base, material)?;
        let file = format!("material-{index}.txt");
        artifacts.push(json!({"file":file,"sha256":sha256_hex(&bytes),"original_file":material["file"],"source_url":material["source_url"],"repository_path":material["repository_path"]}));
        if pinned {
            let repo = material["repository_path"].as_str().unwrap_or("");
            let associated = !repo.is_empty()
                && family(repo).is_some()
                && family(repo) == family(repository_font)
                && material["source_url"]
                    == format!("https://raw.githubusercontent.com/google/fonts/{commit}/{repo}");
            if associated
                && let Some(identifier) = recognised(&String::from_utf8_lossy(&bytes))
                && !alternatives.contains(&identifier)
            {
                alternatives.push(identifier);
            }
        }
        writes.push((format!("{directory}/{file}"), bytes));
    }
    let resolved_licence = if let Some(path) = resolution {
        let raw = std::fs::read(path).map_err(|e| e.to_string())?;
        let record: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        let selected = record["licence"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or("resolution needs licence")?;
        if record["font_sha256"] != hash
            || record["caller_adoption"] != true
            || record["rationale"]
                .as_str()
                .is_none_or(|s| s.trim().is_empty())
            || record["authorization_basis"]
                .as_str()
                .is_none_or(|s| s.trim().is_empty())
        {
            return Err("resolution requires exact font hash, rationale and explicit authorized caller adoption".into());
        }
        let refs = record["evidence"]
            .as_array()
            .filter(|r| !r.is_empty())
            .ok_or("resolution needs conflicting evidence references")?;
        for reference in refs {
            if !materials
                .iter()
                .any(|m| m["file"] == reference["file"] && m["sha256"] == reference["sha256"])
            {
                return Err("resolution references unverified evidence".into());
            }
        }
        let findings = receipt["licence_findings"].as_array();
        let relevant: Vec<&Value> = findings
            .into_iter()
            .flatten()
            .filter(|f| {
                f["kind"] == "material_conflict"
                    || f["code"]
                        .as_str()
                        .is_some_and(|c| c.to_ascii_uppercase().contains("CONFLICT"))
            })
            .flat_map(|f| f["evidence"].as_array().into_iter().flatten())
            .collect();
        if relevant.iter().any(|required| {
            !refs
                .iter()
                .any(|r| r["file"] == required["file"] && r["sha256"] == required["sha256"])
        }) {
            return Err(
                "resolution must cite the contradictory evidence named by the findings".into(),
            );
        }
        artifacts.push(json!({"file":"resolution.json","sha256":sha256_hex(&raw)}));
        writes.push((format!("{directory}/resolution.json"), raw));
        Some(selected.to_string())
    } else {
        None
    };
    let manifest = serde_json::to_vec_pretty(
        &json!({"schema_version":1,"font_sha256":hash,"artifacts":artifacts,"recognized_alternatives":alternatives}),
    )
    .map_err(|e| e.to_string())?;
    let reference = FontEvidence {
        file: format!("{directory}/manifest.json"),
        sha256: sha256_hex(&manifest),
    };
    writes.push((reference.file.clone(), manifest));
    Ok(Prepared {
        face_index: Some(selected.face_index),
        writes,
        reference,
        recognised: if alternatives.len() == 1 {
            alternatives.first().cloned()
        } else {
            None
        },
        alternatives,
        conflict,
        resolved_licence,
    })
}

fn family(path: &str) -> Option<String> {
    if path.contains('\\')
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() < 3 || !["ofl", "apache", "ufl"].contains(&parts[0]) {
        return None;
    }
    Some(format!("{}/{}", parts[0], parts[1]))
}
pub fn verify(base: &Path, reference: &FontEvidence, font_hash: &str) -> Result<(), String> {
    let value = json!({"file":reference.file,"sha256":reference.sha256});
    let raw = checked(base, &value)?;
    let manifest: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    if manifest["schema_version"] != 1 || manifest["font_sha256"] != font_hash {
        return Err("manifest does not match font".into());
    }
    let path = confined(base, &reference.file)?;
    let parent = path.parent().ok_or("manifest missing directory")?;
    for artifact in manifest["artifacts"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("missing manifest artifacts")?
    {
        checked(parent, artifact)?;
    }
    Ok(())
}
