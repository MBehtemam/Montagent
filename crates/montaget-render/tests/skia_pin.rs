//! The Skia pin is in exactly one place, and the prebuilt key is *derived* from
//! it rather than stated twice.
//!
//! ADR-0010 accepted a C++ dependency on one premise: the required prebuilt is
//! published, so nobody compiles Skia from source. `skia-safe` keys its download
//! on the **exact sorted feature set**, which makes the feature list a load-bearing
//! value and not a preference — `svg` and `skottie` both imply `textlayout` and
//! would move the key. #36 therefore asks for the version and feature set to be
//! pinned in exactly one place, and #189 asks the canary to assert the **resolved
//! key**, not merely that a build succeeded.
//!
//! These tests hold up the half of that a scheduled job cannot: that the workspace
//! has one pin, that no other manifest quietly grows a second one, and that the
//! key `ci/assert_prebuilt_key.py` looks for in a cold build's log is the key this
//! feature list actually resolves to. The canary holds up the other half — whether
//! upstream still publishes it.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// The version the feature graph below was read from. Bumping the pin without
/// re-reading the graph is the mistake this guards: a feature that gains an
/// implication upstream moves the key silently.
const GRAPH_READ_FROM: &str = "=0.153.2";

/// `skia-safe`'s `[features]`, verbatim for the version above, restricted to what
/// can reach a `skia-bindings` feature. Cargo resolves this graph for real; the
/// copy exists so the test can resolve it without a build.
const SKIA_SAFE_FEATURES: &[(&str, &[&str])] = &[
    ("egl", &["gl", "skia-bindings/egl"]),
    ("embed-freetype", &["skia-bindings/embed-freetype"]),
    ("embed-icudtl", &["skia-bindings/embed-icudtl"]),
    ("freetype-woff2", &["skia-bindings/freetype-woff2"]),
    ("ganesh", &["skia-bindings/ganesh"]),
    ("gl", &["ganesh", "skia-bindings/gl"]),
    ("graphite", &["skia-bindings/graphite"]),
    ("jpeg", &["skia-bindings/jpeg"]),
    ("metal", &["skia-bindings/metal"]),
    ("no-compile", &["skia-bindings/no-compile"]),
    ("pdf", &["skia-bindings/pdf", "jpeg"]),
    ("skottie", &["skia-bindings/skottie", "textlayout"]),
    ("svg", &["skia-bindings/svg"]),
    ("textlayout", &["skia-bindings/textlayout"]),
    ("vulkan", &["skia-bindings/vulkan"]),
    ("wayland", &["egl", "skia-bindings/wayland"]),
    ("webp", &["webp-encode", "webp-decode"]),
    ("webp-decode", &["skia-bindings/webp-decode"]),
    ("webp-encode", &["skia-bindings/webp-encode"]),
    ("binary-cache", &["skia-bindings/binary-cache"]),
    ("d3d", &["skia-bindings/d3d"]),
    ("x11", &["gl", "skia-bindings/x11"]),
];

/// `skia-bindings`' `[features]`, same version, same restriction.
const SKIA_BINDINGS_FEATURES: &[(&str, &[&str])] = &[
    ("jpeg", &["jpeg-encode", "jpeg-decode"]),
    ("pdf", &["jpeg"]),
    ("skottie", &["textlayout"]),
    ("svg", &["textlayout"]),
    ("webp", &["webp-encode", "webp-decode"]),
];

/// The `skia-bindings` features `Features::from_cargo_env` actually observes —
/// the only ones that reach the key. `binary-cache`, `embed-icudtl` and
/// `no-compile` are build-machinery switches and are deliberately absent, which
/// is why `no-skia-source-build` can be turned on in CI without moving the key.
const KEYED_FEATURES: &[&str] = &[
    "pdf",
    "gl",
    "egl",
    "wayland",
    "x11",
    "vulkan",
    "metal",
    "d3d",
    "ganesh",
    "graphite",
    "textlayout",
    "svg",
    "skottie",
    "webp-encode",
    "webp-decode",
    "embed-freetype",
    "freetype-woff2",
    "jpeg-encode",
    "jpeg-decode",
];

/// `Features::to_key`'s substitutions, before sorting and joining with `-`.
const KEY_REPLACEMENTS: &[(&str, &str)] = &[
    ("webp-encode", "webpe"),
    ("webp-decode", "webpd"),
    ("embed-freetype", "ftembed"),
    ("freetype-woff2", "ftwoff2"),
    ("jpeg-encode", "jpege"),
    ("jpeg-decode", "jpegd"),
];

/// Anything here in the resolved set means a GPU build. ADR-0010 pins the
/// CPU-only key and ADR-0021 records GPU as an open, unmeasured lever for a
/// future ticket — one that moves the key and is therefore that ticket's to take
/// through all six targets.
const GPU_FEATURES: &[&str] = &[
    "gl", "egl", "wayland", "x11", "vulkan", "metal", "d3d", "ganesh", "graphite",
];

fn repo_root() -> PathBuf {
    // crates/montaget-render -> crates -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

fn workspace_manifest() -> toml::Value {
    let path = repo_root().join("Cargo.toml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path:?}: {e}"));
    toml::from_str(&text).unwrap_or_else(|e| panic!("parsing {path:?}: {e}"))
}

/// The one pin: `[workspace.dependencies] skia-safe`.
fn pin() -> toml::Value {
    workspace_manifest()["workspace"]["dependencies"]["skia-safe"].clone()
}

fn declared_features() -> Vec<String> {
    pin()["features"]
        .as_array()
        .expect("the pin states its feature list explicitly")
        .iter()
        .map(|v| v.as_str().expect("a feature is a string").to_string())
        .collect()
}

/// Resolve `skia-safe`'s declared features down to the `skia-bindings` features
/// the build script keys on, then spell them the way `Features::to_key` does.
fn resolved_key(declared: &[String]) -> String {
    let safe: HashMap<_, _> = SKIA_SAFE_FEATURES.iter().copied().collect();
    let bindings: HashMap<_, _> = SKIA_BINDINGS_FEATURES.iter().copied().collect();

    let mut safe_on: BTreeSet<String> = BTreeSet::new();
    let mut bindings_on: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = declared.to_vec();

    while let Some(feature) = queue.pop() {
        if let Some(rest) = feature.strip_prefix("skia-bindings/") {
            if !bindings_on.insert(rest.to_string()) {
                continue;
            }
            for implied in bindings.get(rest).copied().unwrap_or(&[]) {
                queue.push(format!("skia-bindings/{implied}"));
            }
            continue;
        }
        if !safe_on.insert(feature.clone()) {
            continue;
        }
        for implied in safe.get(feature.as_str()).copied().unwrap_or(&[]) {
            queue.push(implied.to_string());
        }
    }

    let mut key: Vec<String> = bindings_on
        .iter()
        .filter(|f| KEYED_FEATURES.contains(&f.as_str()))
        .map(|f| {
            KEY_REPLACEMENTS
                .iter()
                .find(|(from, _)| from == f)
                .map(|(_, to)| (*to).to_string())
                .unwrap_or_else(|| f.clone())
        })
        .collect();
    key.sort();
    key.join("-")
}

/// The key's segments, as a set. Note these are *key spellings*: `jpeg-encode`
/// appears here as `jpege`. Every feature checked against this set below is one
/// with no key replacement, so the two spellings coincide.
fn resolved_key_segments(declared: &[String]) -> BTreeSet<String> {
    resolved_key(declared)
        .split('-')
        .map(|s| s.to_string())
        .collect()
}

/// The declared feature list resolves to the key the rest of the repo expects.
///
/// This is what makes `prebuilt-key` a derivation rather than a second
/// statement of the same fact: change the feature list and this test fails until
/// the key is updated too, which is the ticket #189 asks for.
#[test]
fn the_declared_feature_set_resolves_to_the_recorded_key() {
    let manifest = workspace_manifest();
    let expected = manifest["workspace"]["metadata"]["skia"]["prebuilt-key"]
        .as_str()
        .expect("workspace.metadata.skia.prebuilt-key is recorded");

    assert_eq!(
        resolved_key(&declared_features()),
        expected,
        "the pinned feature set no longer resolves to the recorded prebuilt key. \
         Changing the feature set is its own ticket that re-verifies all six targets (#189)."
    );
    assert_eq!(
        expected, "jpegd-jpege-pdf",
        "ADR-0010 names the required key, and moving off it is an ADR-level decision"
    );
}

/// ADR-0010: CPU-only, no `ganesh`, no `gl`.
#[test]
fn the_pinned_build_is_cpu_only() {
    let resolved = resolved_key_segments(&declared_features());
    for gpu in GPU_FEATURES {
        assert!(
            !resolved.contains(*gpu),
            "`{gpu}` is on: the pinned key is meant to be CPU-only (ADR-0010). \
             A GPU prebuilt is a different published key and its own ticket (ADR-0021)."
        );
    }
    // `textlayout` is excluded for a second, independent reason: ADR-0010 rules
    // out SkParagraph, because ADR-0008 needs break opportunities and a named
    // segmenter that ICU-sealed-inside cannot expose. Taking it would ship two
    // shapers, and the one `measure` reports would not be the one that draws.
    assert!(
        !resolved.contains("textlayout"),
        "`textlayout` is on, which both moves the key and re-seals ICU (ADR-0008, ADR-0010)"
    );
}

/// The graph above is a copy of one version's manifests. If the version moves,
/// the copy must be re-read rather than trusted.
#[test]
fn the_pinned_version_is_the_one_the_feature_graph_was_read_from() {
    let pin = pin();
    let version = pin["version"].as_str().expect("the pin states a version");
    assert_eq!(
        version, GRAPH_READ_FROM,
        "skia-safe was bumped. Re-read `[features]` from the new skia-safe and \
         skia-bindings manifests into this test, then re-verify all six targets (#36)."
    );
    assert!(
        version.starts_with('='),
        "the version is pinned exactly, so a patch release cannot move the key unseen"
    );
}

/// `default-features = false` plus an explicit list means an upstream change to
/// `skia-safe`'s defaults cannot move Montaget's key without a diff here.
#[test]
fn the_pin_does_not_inherit_upstream_defaults() {
    assert_eq!(
        pin()["default-features"].as_bool(),
        Some(false),
        "the feature set must be stated, not inherited"
    );
}

fn manifests(dir: &Path, found: &mut Vec<PathBuf>) {
    // Hidden directories are skipped wholesale: `.git`, a `.venv`, and in
    // particular a stale `.claude/worktrees/` checkout, which is another copy of
    // this repository rather than a second declaration in it.
    let skip = ["target", "node_modules"];
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if !skip.contains(&name.as_ref()) && !name.starts_with('.') {
                manifests(&path, found);
            }
        } else if name == "Cargo.toml" {
            found.push(path);
        }
    }
}

/// Every dependency table in a manifest, including the platform-gated ones.
///
/// `[target.'cfg(windows)'.dependencies]` is checked because it is the shape a
/// second pin would realistically take: the reason to add one is a
/// platform-specific backend, and `d3d` or `metal` moves the prebuilt key on
/// that target while the root pin and the canary still describe the old one.
fn dependency_tables(manifest: &toml::Value) -> Vec<(String, &toml::Value)> {
    const TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    let mut found = Vec::new();

    for table in TABLES {
        if let Some(t) = manifest.get(table) {
            found.push((format!("[{table}]"), t));
        }
    }

    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for (cfg, spec) in targets {
            for table in TABLES {
                if let Some(t) = spec.get(table) {
                    found.push((format!("[target.'{cfg}'.{table}]"), t));
                }
            }
        }
    }

    found
}

/// Exactly one place. Every other manifest that wants Skia takes the workspace
/// pin; a second literal version anywhere in the tree is the drift #36 asks to
/// be made impossible.
#[test]
fn no_manifest_declares_skia_independently() {
    let root = repo_root();
    let mut found = Vec::new();
    manifests(&root, &mut found);
    assert!(
        found.len() > 1,
        "the manifest sweep found nothing to check, which means it is broken"
    );

    for path in found {
        if path == root.join("Cargo.toml") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_default();
        let Ok(value) = toml::from_str::<toml::Value>(&text) else {
            continue;
        };
        let relative = path.strip_prefix(&root).unwrap_or(&path).display();
        for (table, deps) in dependency_tables(&value) {
            let Some(dep) = deps.get("skia-safe") else {
                continue;
            };
            assert_eq!(
                dep.get("workspace").and_then(toml::Value::as_bool),
                Some(true),
                "{relative} declares `skia-safe` itself in {table}. The version and \
                 feature set are pinned in exactly one place (#36); use \
                 `skia-safe.workspace = true`."
            );
            assert!(
                dep.get("features").is_none() && dep.get("version").is_none(),
                "{relative} adds to the workspace `skia-safe` pin in {table}. \
                 Any addition moves the prebuilt key."
            );
        }
    }
}

/// The sweep above is only worth anything if it can see a platform-gated table.
#[test]
fn the_single_pin_guard_looks_inside_target_specific_tables() {
    let manifest: toml::Value = toml::from_str(
        r#"
        [dependencies]
        serde = "1"

        [target.'cfg(windows)'.dependencies]
        skia-safe = { version = "=0.153.2", features = ["d3d"] }
        "#,
    )
    .expect("the fixture parses");

    let tables = dependency_tables(&manifest);
    assert!(
        tables
            .iter()
            .any(|(name, deps)| name.contains("cfg(windows)") && deps.get("skia-safe").is_some()),
        "a platform-gated second pin would have gone unseen: {:?}",
        tables.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
}

/// ADR-0064 commits to all six desktop tier-1 targets, and #189 requires the
/// suite to run on every one of them. The list is recorded beside the pin
/// because the canary and the suite must agree about what "all six" means.
#[test]
fn the_target_matrix_is_adr_0064s_six() {
    let manifest = workspace_manifest();
    let declared: BTreeSet<String> = manifest["workspace"]["metadata"]["skia"]["targets"]
        .as_array()
        .expect("workspace.metadata.skia.targets is recorded")
        .iter()
        .map(|v| v.as_str().expect("a target is a string").to_string())
        .collect();

    let expected: BTreeSet<String> = ["apple-darwin", "unknown-linux-gnu", "pc-windows-msvc"]
        .iter()
        .flat_map(|sys| {
            ["aarch64", "x86_64"]
                .iter()
                .map(move |arch| format!("{arch}-{sys}"))
        })
        .collect();

    assert_eq!(
        declared, expected,
        "ADR-0064's matrix is {{aarch64, x86_64}} x {{apple-darwin, unknown-linux-gnu, pc-windows-msvc}}"
    );
}

/// Every `- target: <triple>` / `os: <label>` pair a workflow declares, in
/// order. Parsed by shape rather than as YAML so the test needs no dependency
/// for something this regular.
fn matrix_entries(text: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut pending: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(target) = line.strip_prefix("- target:") {
            pending = Some(target.trim().to_string());
        } else if let Some(os) = line.strip_prefix("os:")
            && let Some(target) = pending.take()
        {
            entries.push((target, os.trim().to_string()));
        }
    }
    entries
}

/// Both workflows cover every target, on the same runner. A target that is in
/// the pin but in neither matrix is a target nobody is checking; a target the
/// two workflows run on *different* runners is a suite and a canary that are no
/// longer answering about the same machine.
///
/// What this cannot check is whether a runner label still exists — GitHub
/// retires images, and a retired label fails to schedule rather than failing a
/// test. Keeping the two files in lockstep at least means a retirement is one
/// decision rather than two, and cannot be half-applied.
#[test]
fn both_workflows_cover_every_target_on_the_same_runner() {
    let root = repo_root();
    let manifest = workspace_manifest();
    let targets: BTreeSet<String> = manifest["workspace"]["metadata"]["skia"]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let mut seen: Vec<(&str, Vec<(String, String)>)> = Vec::new();
    for workflow in ["ci.yml", "skia-canary.yml"] {
        let path = root.join(".github/workflows").join(workflow);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{workflow} must exist for #189 to be done: {e}"));
        let entries = matrix_entries(&text);

        let covered: BTreeSet<String> = entries.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            covered, targets,
            "{workflow}'s matrix is not ADR-0064's six targets; the suite and the canary \
             both run on all six (#36)"
        );
        for (target, os) in &entries {
            assert!(
                !os.is_empty() && !os.contains("${{"),
                "{workflow}: {target} has no literal runner label"
            );
        }
        seen.push((workflow, entries));
    }

    let (first_name, first) = &seen[0];
    let (second_name, second) = &seen[1];
    for (target, os) in first {
        let other = second
            .iter()
            .find(|(t, _)| t == target)
            .map(|(_, os)| os.as_str())
            .unwrap_or("<missing>");
        assert_eq!(
            os, other,
            "{first_name} runs {target} on {os} but {second_name} runs it on {other}; \
             the suite and the canary must answer about the same machine"
        );
    }
}

/// The canary's assertion script reads the key from the pin rather than carrying
/// its own copy.
#[test]
fn the_canary_script_reads_the_key_from_the_pin() {
    let script = repo_root().join("ci/assert_prebuilt_key.py");
    let text = fs::read_to_string(&script)
        .unwrap_or_else(|e| panic!("the canary's assertion script must exist: {e}"));
    assert!(
        text.contains("prebuilt-key"),
        "the script must read `workspace.metadata.skia.prebuilt-key` rather than hardcode a key"
    );
    assert!(
        !text.contains("\"jpegd-jpege-pdf\"") && !text.contains("'jpegd-jpege-pdf'"),
        "the script carries a literal copy of the key, which is a second place to change it"
    );
}
