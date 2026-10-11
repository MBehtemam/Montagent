//! Explicit authoring-time acquisition. Never called by validation or rendering.
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, serde::Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    Url {
        url: String,
    },
    GoogleFonts {
        family: Option<String>,
        file: Option<String>,
        revision: Option<String>,
        weight: Option<u16>,
        style: Option<String>,
    },
}
#[derive(Debug, Clone, Deserialize, serde::Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Acquire {
    pub source: Source,
    /// New bundle directory; existing directories are never overwritten.
    pub destination: PathBuf,
    pub face_index: Option<u32>,
    pub expected_sha256: Option<String>,
}
type Failure = (&'static str, String);
const MAX_BYTES: u64 = 32 * 1024 * 1024;
fn failure(code: &'static str, detail: impl ToString) -> Failure {
    (code, detail.to_string())
}
fn outcome(error: Failure) -> Value {
    json!({"status":"failed","execution":{"scope":"local"},"files":[],"receipt":null,
        "findings":[{"code":error.0,"detail":error.1}],"next_step":"Correct the reported input or explicitly acquire native font files externally, then retry."})
}
pub fn acquire(ask: &Acquire) -> Value {
    acquire_using(ask, &download)
}
fn acquire_using(
    ask: &Acquire,
    fetch: &dyn Fn(&str) -> Result<(Vec<u8>, String), Failure>,
) -> Value {
    match prepare(ask, fetch) {
        Ok(value) => value,
        Err(error) => outcome(error),
    }
}
fn public_address(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v) => {
            let o = v.octets();
            !v.is_private()
                && !v.is_loopback()
                && !v.is_link_local()
                && !v.is_broadcast()
                && !v.is_documentation()
                && !v.is_unspecified()
                && !v.is_multicast()
                && o[0] != 0
                && o[0] < 224
                && !(o[0] == 100 && (64..=127).contains(&o[1]))
                && !(o[0] == 198 && (o[1] == 18 || o[1] == 19))
        }
        std::net::IpAddr::V6(v) => {
            if let Some(v4) = v.to_ipv4_mapped() {
                return public_address(v4.into());
            }
            let s = v.segments();
            s[0] & 0xe000 == 0x2000
                && s[0] != 0x2001
                && s[0] != 0x2002
                && s[0] != 0x3fff
                && !v.is_loopback()
                && !v.is_unspecified()
                && !v.is_multicast()
                && s[0] & 0xfe00 != 0xfc00
                && s[0] & 0xffc0 != 0xfe80
                && !(s[0] == 0x2001 && s[1] == 0xdb8)
        }
    }
}
fn download(locator: &str) -> Result<(Vec<u8>, String), Failure> {
    use std::io::Read;
    use std::net::ToSocketAddrs;
    let mut url = reqwest::Url::parse(locator).map_err(|e| failure("unsupported_source", e))?;
    let started = std::time::Instant::now();
    for _ in 0..=5 {
        if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
            return Err(failure(
                "network_denied",
                "Only HTTPS URLs without credentials are supported",
            ));
        }
        let host = url
            .host_str()
            .ok_or_else(|| failure("network_denied", "URL has no host"))?;
        let owned_host = host.to_string();
        let port = url.port_or_known_default().unwrap_or(443);
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let result = (owned_host.as_str(), port)
                .to_socket_addrs()
                .map(|a| a.collect::<Vec<_>>());
            let _ = send.send(result);
        });
        let addresses = receive
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| failure("limit_exceeded", "DNS resolution timed out"))?
            .map_err(|e| failure("fetch_failure", e))?;
        if addresses.is_empty() || addresses.iter().any(|a| !public_address(a.ip())) {
            return Err(failure(
                "network_denied",
                "Private, loopback and reserved addresses are refused",
            ));
        }
        let remaining = std::time::Duration::from_secs(30)
            .checked_sub(started.elapsed())
            .ok_or_else(|| failure("limit_exceeded", "Download duration exceeded"))?;
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(host, &addresses)
            .timeout(remaining)
            .user_agent("Montagent-font-acquisition")
            .build()
            .map_err(|e| failure("fetch_failure", e))?;
        let response = client
            .get(url.clone())
            .send()
            .map_err(|e| failure("fetch_failure", e))?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| failure("fetch_failure", "Redirect missing Location"))?;
            url = url
                .join(location)
                .map_err(|e| failure("fetch_failure", e))?;
            continue;
        }
        if !response.status().is_success() {
            return Err(failure(
                "fetch_failure",
                format!("HTTP {}", response.status()),
            ));
        }
        if response.content_length().is_some_and(|n| n > MAX_BYTES) {
            return Err(failure("limit_exceeded", "Payload too large"));
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| failure("fetch_failure", e))?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(failure("limit_exceeded", "Payload too large"));
        }
        return Ok((bytes, url.to_string()));
    }
    Err(failure("limit_exceeded", "Too many redirects"))
}
fn prepare(
    ask: &Acquire,
    fetch: &dyn Fn(&str) -> Result<(Vec<u8>, String), Failure>,
) -> Result<Value, Failure> {
    let mut materials: Vec<(String, Vec<u8>, String, String)> = Vec::new();
    let mut source = json!({"kind":"url"});
    let (requested, bytes, final_url) = match &ask.source {
        Source::Url { url } => {
            let (bytes, final_url) = fetch(url)?;
            (url.clone(), bytes, final_url)
        }
        Source::GoogleFonts {
            family,
            file,
            revision,
            weight,
            style,
        } => {
            if family.as_ref().is_none_or(|f| f.trim().is_empty()) && file.is_none() {
                return Err(failure(
                    "unsupported_selection",
                    "Provide family or repository file",
                ));
            }
            let revision = revision.as_deref().unwrap_or("main");
            if !revision
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c))
            {
                return Err(failure(
                    "unsupported_selection",
                    "Invalid repository revision",
                ));
            }
            let (commit_bytes, _) = fetch(&format!(
                "https://api.github.com/repos/google/fonts/commits/{revision}"
            ))?;
            let commit_value: Value =
                serde_json::from_slice(&commit_bytes).map_err(|e| failure("fetch_failure", e))?;
            let commit = commit_value["sha"]
                .as_str()
                .filter(|v| v.len() == 40 && v.chars().all(|c| c.is_ascii_hexdigit()))
                .ok_or_else(|| failure("fetch_failure", "Missing pinned commit"))?;
            let (tree_bytes, _) = fetch(&format!(
                "https://api.github.com/repos/google/fonts/git/trees/{commit}?recursive=1"
            ))?;
            let tree: Value =
                serde_json::from_slice(&tree_bytes).map_err(|e| failure("fetch_failure", e))?;
            if tree["truncated"].as_bool() != Some(false) {
                return Err(failure("limit_exceeded", "Repository tree was truncated"));
            }
            let entries = tree["tree"]
                .as_array()
                .ok_or_else(|| failure("fetch_failure", "Missing repository tree"))?;
            let normal = |v: &str| {
                v.chars()
                    .filter(|c| c.is_alphanumeric())
                    .flat_map(char::to_lowercase)
                    .collect::<String>()
            };
            let family_key = family.as_deref().map(normal);
            let paths: Vec<_> = entries
                .iter()
                .filter_map(|e| e["path"].as_str())
                .filter(|path| {
                    let parts: Vec<_> = path.split('/').collect();
                    (parts.len() == 3 || (parts.len() == 4 && parts[2] == "static"))
                        && parts
                            .iter()
                            .all(|p| !p.is_empty() && *p != "." && *p != ".." && !p.contains('\\'))
                        && matches!(parts[0], "ofl" | "apache" | "ufl")
                        && (path.ends_with(".ttf")
                            || path.ends_with(".otf")
                            || path.ends_with(".ttc")
                            || path.ends_with(".otc"))
                        && file.as_ref().is_none_or(|f| f == path)
                        && family_key.as_ref().is_none_or(|f| normal(parts[1]) == *f)
                })
                .collect();
            if paths.len() > 64 {
                return Err(failure(
                    "limit_exceeded",
                    "Too many candidate files; select an exact repository file",
                ));
            }
            let mut candidates = Vec::new();
            let mut suitable = Vec::new();
            for path in paths {
                let url = format!("https://raw.githubusercontent.com/google/fonts/{commit}/{path}");
                let (data, final_url) = fetch(&url)?;
                let names = montagent_text::names::faces(&data)
                    .map_err(|e| failure("invalid_payload", e))?;
                let metadata = metadata(&data)?;
                let matches = metadata.iter().any(|m| {
                    ask.face_index.is_none_or(|i| m["index"] == i)
                        && family_key.as_ref().is_none_or(|requested| {
                            names.iter().any(|face| {
                                face.index == m["index"].as_u64().unwrap_or(u64::MAX) as u32
                                    && face
                                        .typographic_family
                                        .as_ref()
                                        .or(face.family.as_ref())
                                        .is_some_and(|f| normal(f) == *requested)
                            })
                        })
                        && weight.is_none_or(|w| m["weight"] == w)
                        && style.as_ref().is_none_or(|s| m["style"] == *s)
                        && (!(weight.is_some() || style.is_some()) || m["variable"] == false)
                });
                candidates.push(json!({"repository_path":path,"commit":commit,"faces":names,"metadata":metadata,"sha256":crate::fonts::sha256_hex(&data)}));
                if matches {
                    suitable.push((path.to_string(), url, data, final_url));
                }
            }
            if suitable.len() != 1 {
                return Ok(
                    json!({"status":"selection_required","execution":{"scope":"local"},"files":[],"receipt":null,"candidates":candidates,"findings":[{"code":if suitable.is_empty(){"unsupported_selection"}else{"ambiguous_selection"}}],"next_step":"Select an exact suitable repository file and retry; exact weight/style requires static bytes."}),
                );
            }
            let (path, url, data, final_url) = suitable.remove(0);
            let directory = path.split('/').take(2).collect::<Vec<_>>().join("/");
            for material in entries.iter().filter_map(|e| e["path"].as_str()) {
                let Some((dir, name)) = material.rsplit_once('/') else {
                    continue;
                };
                let upper = name.to_ascii_uppercase();
                if dir == directory
                    && !name.contains('\\')
                    && name != "."
                    && name != ".."
                    && (upper.contains("LICENSE")
                        || upper.contains("LICENCE")
                        || upper.contains("COPYRIGHT")
                        || upper.contains("NOTICE")
                        || upper == "OFL.TXT"
                        || upper.contains("ATTRIBUTION"))
                {
                    let material_url = format!(
                        "https://raw.githubusercontent.com/google/fonts/{commit}/{material}"
                    );
                    let (material_bytes, _) = fetch(&material_url)?;
                    materials.push((name.into(), material_bytes, material_url, material.into()));
                }
            }
            source = json!({"kind":"google_fonts","commit":commit,"repository_path":path});
            (url, data, final_url)
        }
    };
    source["requested_url"] = json!(requested);
    source["final_url"] = json!(final_url);
    let hash = crate::fonts::sha256_hex(&bytes);
    if ask
        .expected_sha256
        .as_ref()
        .is_some_and(|h| !h.eq_ignore_ascii_case(&hash))
    {
        return Err(failure(
            "expected_hash_mismatch",
            "Downloaded font does not match expected SHA-256",
        ));
    }
    if !matches!(
        bytes.get(..4),
        Some(b"OTTO" | b"ttcf" | b"true" | b"\x00\x01\x00\x00")
    ) {
        return Err(failure(
            "invalid_payload",
            "Expected native TTF/OTF/TTC/OTC bytes; archives, HTML and web fonts are unsupported",
        ));
    }
    let faces = montagent_text::names::faces(&bytes).map_err(|e| failure("invalid_payload", e))?;
    if (faces.len() > 1 && ask.face_index.is_none())
        || ask.face_index.is_some_and(|i| i as usize >= faces.len())
    {
        return Ok(
            json!({"status":"selection_required","execution":{"scope":"local"},"files":[],"receipt":null,
            "candidates":faces,"findings":[{"code":if ask.face_index.is_none(){"required_face"}else{"invalid_face"}}],"next_step":"Select an explicit valid face_index and retry."}),
        );
    }
    let extension = if bytes.starts_with(b"ttcf") {
        "ttc"
    } else if bytes.starts_with(b"OTTO") {
        "otf"
    } else {
        "ttf"
    };
    let name = format!("font.{extension}");
    let mut findings = Vec::new();
    match crate::fonts::licence::bucket(&faces) {
        crate::fonts::licence::Bucket::Recognised{identifier} => findings.push(json!({"code":"recognized_identifier","identifier":identifier,"basis":"embedded_metadata"})),
        crate::fonts::licence::Bucket::Blocklisted{matched,..} => findings.push(json!({"code":"blocklisted","matched":matched})),
        crate::fonts::licence::Bucket::Unknown => findings.push(json!({"code":"unknown_licence"})),
    }

    if materials.is_empty() {
        findings.push(
            json!({"code":"missing_evidence","detail":"No accompanying materials available"}),
        );
    }
    let material_records:Vec<_>=materials.iter().map(|(name,data,url,path)|json!({"file":name,"sha256":crate::fonts::sha256_hex(data),"source_url":url,"repository_path":path,"font_sha256":hash})).collect();
    let font = json!({"file":name,"sha256":hash,"faces":faces,"metadata":metadata(&bytes)?,"face_index":ask.face_index.unwrap_or(0)});
    let receipt = json!({"schema_version":1,"source":source,"retrieved_at":chrono::Utc::now().to_rfc3339(),
        "selection":{"source":ask.source,"face_index":ask.face_index.unwrap_or(0)},"font":font,"materials":material_records,"licence_findings":findings});
    let receipt_bytes =
        serde_json::to_vec_pretty(&receipt).map_err(|e| failure("invalid_payload", e))?;
    let mut files = vec![(name.clone(), bytes)];
    files.extend(materials.into_iter().map(|(name, data, _, _)| (name, data)));
    files.push(("receipt.json".into(), receipt_bytes.clone()));
    publish(&ask.destination, &files)?;
    Ok(
        json!({"status":"acquired","execution":{"scope":"local"},"files":[{"path":ask.destination.join(&name),"sha256":hash,"faces":faces}],
        "receipt":{"path":ask.destination.join("receipt.json"),"sha256":crate::fonts::sha256_hex(&receipt_bytes)},
        "findings":std::mem::take(&mut findings),"next_step":"Invoke fonts vendor with the font path and receipt; acquisition does not grant redistribution permission."}),
    )
}
fn metadata(bytes: &[u8]) -> Result<Vec<Value>, Failure> {
    use skrifa::raw::TableProvider;
    let count = montagent_text::sfnt::faces(bytes).map_err(|e| failure("invalid_payload", e))?;
    (0..count).map(|index|{
        let selected=montagent_text::sfnt::face(bytes,index).map_err(|e|failure("invalid_payload",format!("{e:?}")))?;
        let font=skrifa::FontRef::new(&selected).map_err(|e|failure("invalid_payload",e))?;
        let os2=font.os2().ok();
        let weight=os2.as_ref().map(|t|t.us_weight_class());
        let italic=os2.as_ref().is_some_and(|t|t.fs_selection().contains(skrifa::raw::tables::os2::SelectionFlags::ITALIC));
        Ok(json!({"index":index,"weight":weight,"style":if italic{"italic"}else{"normal"},"variable":font.fvar().is_ok()}))
    }).collect()
}
fn publish(destination: &std::path::Path, files: &[(String, Vec<u8>)]) -> Result<(), Failure> {
    use std::io::Write;
    if destination.exists() {
        return Err(failure(
            "collision",
            "Acquisition destination already exists; choose a new bundle directory",
        ));
    }
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| failure("invalid_destination", e))?;
    let stage = parent.join(format!(
        ".montagent-acquire-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir(&stage).map_err(|e| failure("invalid_destination", e))?;
    let mut reserved = false;
    let result = (|| {
        for (name, bytes) in files {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(stage.join(name))
                .map_err(|e| failure("invalid_destination", e))?;
            file.write_all(bytes)
                .and_then(|_| file.sync_all())
                .map_err(|e| failure("invalid_destination", e))?;
        }
        // Reserve the destination without replacing an existing directory, then move each
        // staged file. The receipt moves last, so a failed publication is never usable.
        std::fs::create_dir(destination).map_err(|e| failure("collision", e))?;
        reserved = true;
        for (name, _) in files {
            std::fs::rename(stage.join(name), destination.join(name))
                .map_err(|e| failure("invalid_destination", e))?;
        }
        Ok(())
    })();
    if result.is_err() && reserved {
        let _ = std::fs::remove_dir_all(destination);
    }
    let _ = std::fs::remove_dir_all(&stage);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_google_family_retains_original_licence_and_inspected_candidates() {
        let destination =
            std::env::temp_dir().join(format!("montagent-google-fixture-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&destination);
        let commit = "0123456789012345678901234567890123456789";
        let font = include_bytes!("../../../../fixtures/skills/fonts/Inter-Bold.ttf").to_vec();
        let fetch = |url: &str| {
            let bytes = if url.contains("/commits/") {
                serde_json::to_vec(&json!({"sha":commit})).unwrap()
            } else if url.contains("/git/trees/") {
                serde_json::to_vec(&json!({"truncated":false,"tree":[{"path":"ofl/inter/Inter-Bold.ttf"},{"path":"ofl/inter/OFL.txt"}]})).unwrap()
            } else if url.ends_with("OFL.txt") {
                b"SIL OPEN FONT LICENSE Version 1.1".to_vec()
            } else {
                font.clone()
            };
            Ok((bytes, url.into()))
        };
        let out = acquire_using(
            &Acquire {
                source: Source::GoogleFonts {
                    family: Some("Inter".into()),
                    file: None,
                    revision: None,
                    weight: Some(700),
                    style: None,
                },
                destination: destination.clone(),
                face_index: None,
                expected_sha256: None,
            },
            &fetch,
        );
        assert_eq!(out["status"], "acquired", "{out}");
        let receipt: Value =
            serde_json::from_slice(&std::fs::read(destination.join("receipt.json")).unwrap())
                .unwrap();
        assert_eq!(receipt["source"]["commit"], commit);
        assert_eq!(
            receipt["materials"][0]["repository_path"],
            "ofl/inter/OFL.txt"
        );
        assert_eq!(
            std::fs::read(destination.join("OFL.txt")).unwrap(),
            b"SIL OPEN FONT LICENSE Version 1.1"
        );
        std::fs::remove_dir_all(destination).unwrap();
    }
    #[test]
    fn hash_mismatch_leaves_existing_destination_untouched() {
        let destination =
            std::env::temp_dir().join(format!("montagent-hash-fixture-{}", std::process::id()));
        std::fs::create_dir_all(&destination).unwrap();
        std::fs::write(destination.join("font.ttf"), b"existing").unwrap();
        let out = acquire_using(
            &Acquire {
                source: Source::Url {
                    url: "https://example.com/font.ttf".into(),
                },
                destination: destination.clone(),
                face_index: None,
                expected_sha256: Some("wrong".into()),
            },
            &|url| {
                Ok((
                    include_bytes!("../../../../fixtures/skills/fonts/Inter-Bold.ttf").to_vec(),
                    url.into(),
                ))
            },
        );
        assert_eq!(out["findings"][0]["code"], "expected_hash_mismatch");
        assert_eq!(
            std::fs::read(destination.join("font.ttf")).unwrap(),
            b"existing"
        );
        assert!(!destination.join("receipt.json").exists());
        std::fs::remove_dir_all(destination).unwrap();
    }
    #[test]
    fn successful_native_download_preserves_bytes_and_receipt() {
        let destination =
            std::env::temp_dir().join(format!("montagent-acquire-fixture-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&destination);
        let bytes = include_bytes!("../../../../fixtures/skills/fonts/Inter-Bold.ttf").to_vec();
        let out = acquire_using(
            &Acquire {
                source: Source::Url {
                    url: "https://example.com/font.ttf".into(),
                },
                destination: destination.clone(),
                face_index: None,
                expected_sha256: None,
            },
            &|url| Ok((bytes.clone(), url.into())),
        );
        assert_eq!(out["status"], "acquired");
        assert_eq!(std::fs::read(destination.join("font.ttf")).unwrap(), bytes);
        let receipt: Value =
            serde_json::from_slice(&std::fs::read(destination.join("receipt.json")).unwrap())
                .unwrap();
        assert_eq!(receipt["font"]["file"], "font.ttf");
        assert_eq!(receipt["schema_version"], 1);
        std::fs::remove_dir_all(destination).unwrap();
    }
    #[test]
    fn private_network_url_is_denied_before_any_destination_is_created() {
        let destination = std::env::temp_dir().join("montagent-acquire-denied");
        let out = acquire(&Acquire {
            source: Source::Url {
                url: "https://127.0.0.1/font.ttf".into(),
            },
            destination: destination.clone(),
            face_index: None,
            expected_sha256: None,
        });
        assert_eq!(out["findings"][0]["code"], "network_denied");
        assert!(!destination.exists());
    }
}
