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
    // crates/montagent-render -> crates -> repo root
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
/// `skia-safe`'s defaults cannot move Montagent's key without a diff here.
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

/// `[workspace.metadata.skia] <key>`, as a set of target triples.
fn metadata_targets(key: &str) -> BTreeSet<String> {
    workspace_manifest()["workspace"]["metadata"]["skia"][key]
        .as_array()
        .unwrap_or_else(|| panic!("workspace.metadata.skia.{key} is recorded"))
        .iter()
        .map(|v| v.as_str().expect("a target is a string").to_string())
        .collect()
}

/// ADR-0064's six shipped targets: every target a release builds an archive for
/// and the canary resolves the prebuilt on.
fn shipped_targets() -> BTreeSet<String> {
    metadata_targets("targets")
}

/// ADR-0188's build-only targets: shipped, built by `ci.yml`, never tested.
fn build_only_targets() -> BTreeSet<String> {
    metadata_targets("build-only-targets")
}

/// ADR-0064 ships all six desktop tier-1 targets, and the list is recorded beside
/// the pin because the release, the canary and `ci.yml` must agree about what
/// "all six" means. ADR-0188 stopped *testing* one of them, not shipping it, so
/// this list is still the six.
#[test]
fn the_target_matrix_is_adr_0064s_six() {
    let expected: BTreeSet<String> = ["apple-darwin", "unknown-linux-gnu", "pc-windows-msvc"]
        .iter()
        .flat_map(|sys| {
            ["aarch64", "x86_64"]
                .iter()
                .map(move |arch| format!("{arch}-{sys}"))
        })
        .collect();

    assert_eq!(
        shipped_targets(),
        expected,
        "ADR-0064's matrix is {{aarch64, x86_64}} x {{apple-darwin, unknown-linux-gnu, pc-windows-msvc}}"
    );
}

/// ADR-0188 takes exactly one target out of the suite: Intel macOS. The suite runs
/// on the other five. Stated as literal sets so that moving a second target to
/// build-only, or dropping one from the suite outright, fails here and has to be
/// its own decision.
#[test]
fn the_tested_targets_are_adr_0188s_five() {
    let build_only = build_only_targets();
    assert_eq!(
        build_only,
        BTreeSet::from(["x86_64-apple-darwin".to_string()]),
        "ADR-0188 takes exactly `x86_64-apple-darwin` out of the suite; another target \
         leaving it is another ADR"
    );
    assert!(
        build_only.is_subset(&shipped_targets()),
        "a build-only target is still a shipped target (ADR-0188)"
    );

    let tested: BTreeSet<String> = shipped_targets().difference(&build_only).cloned().collect();
    let expected: BTreeSet<String> = [
        "aarch64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
        "aarch64-pc-windows-msvc",
        "x86_64-pc-windows-msvc",
    ]
    .iter()
    .map(|t| t.to_string())
    .collect();
    assert_eq!(
        tested, expected,
        "ADR-0188's tier-1 test targets are these five"
    );
}

/// The jobs under a workflow's `jobs:`, each as `(id, its own lines)`. Parsed by
/// shape rather than as YAML, like [`matrix_entries`]: a job starts at a key
/// indented exactly two spaces and runs to the next one.
fn jobs(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut in_jobs = false;
    for line in text.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() && !line.starts_with('#') {
            in_jobs = line.trim_end() == "jobs:";
            continue;
        }
        if !in_jobs {
            continue;
        }
        let is_job_key = line.starts_with("  ")
            && !line[2..].starts_with([' ', '#'])
            && line.trim_end().ends_with(':');
        if is_job_key {
            let id = line.trim().trim_end_matches(':').to_string();
            found.push((id, String::new()));
        } else if let Some((_, body)) = found.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    found
}

/// One workflow's jobs, read from disk.
fn workflow_jobs(workflow: &str) -> Vec<(String, String)> {
    let path = repo_root().join(".github/workflows").join(workflow);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{workflow} must exist for #189 to be done: {e}"));
    let found = jobs(&text);
    assert!(
        !found.is_empty(),
        "{workflow}: the job sweep found nothing, which means it is broken"
    );
    found
}

/// Every `- target: <triple>` / `os: <label>` pair a job declares, in
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

/// The job's own `name:`, the one indented four spaces — not a step's or an
/// action input's, which sit deeper.
fn job_name(body: &str) -> Option<String> {
    body.lines().find_map(|line| {
        line.strip_prefix("    name:")
            .map(|rest| rest.trim().to_string())
    })
}

/// The job sweep sees every job, including one whose key follows a comment, and
/// stops at the next top-level key.
#[test]
fn the_job_sweep_splits_a_workflow_into_its_jobs() {
    let text = "\
name: x
jobs:
  first:
    name: ${{ matrix.target }}
    strategy:
      matrix:
        include:
          - target: a
            os: one
    steps:
      - name: a step ${{ matrix.target }}
  # a comment
  second-job:
    name: other
env:
  K: v
";
    let found = jobs(text);
    let ids: Vec<&str> = found.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["first", "second-job"]);
    assert_eq!(
        matrix_entries(&found[0].1),
        [("a".to_string(), "one".to_string())]
    );
    assert_eq!(
        job_name(&found[0].1).as_deref(),
        Some("${{ matrix.target }}")
    );
    assert!(!found[1].1.contains("K: v"), "the sweep ran past `jobs:`");
}

/// Each workflow covers exactly the targets it is meant to, and every workflow
/// runs a given target on the same runner.
///
/// - `ci.yml`'s `suite` runs on the five test targets and **only** those
///   (ADR-0188); its `build-only` job builds the build-only targets and **only**
///   those. No other `ci.yml` job carries a target matrix.
/// - `skia-canary.yml` resolves the prebuilt on all six shipped targets.
/// - `release.yml` builds all six: shipping is not testing (ADR-0188).
///
/// A target that is in the pin but in no matrix is a target nobody is checking; a
/// target two workflows run on *different* runners is a suite, a canary and a
/// release that are no longer answering about the same machine.
///
/// What this cannot check is whether a runner label still exists — GitHub
/// retires images, and a retired label fails to schedule rather than failing a
/// test. Keeping the files in lockstep at least means a retirement is one
/// decision rather than three, and cannot be half-applied.
#[test]
fn both_workflows_cover_every_target_on_the_same_runner() {
    let shipped = shipped_targets();
    let build_only = build_only_targets();
    let tested: BTreeSet<String> = shipped.difference(&build_only).cloned().collect();

    // (workflow, job, the targets that job must cover, exactly, and why)
    let expected: [(&str, &str, &BTreeSet<String>, &str); 4] = [
        (
            "ci.yml",
            "suite",
            &tested,
            "the suite runs on ADR-0188's five test targets, no more and no fewer",
        ),
        (
            "ci.yml",
            "build-only",
            &build_only,
            "the build-only job builds exactly ADR-0188's build-only targets",
        ),
        (
            "skia-canary.yml",
            "prebuilt",
            &shipped,
            "the canary resolves the prebuilt on all six shipped targets (#36)",
        ),
        (
            "release.yml",
            "build",
            &shipped,
            "a release ships all six of ADR-0064's targets",
        ),
    ];

    for workflow in ["ci.yml", "skia-canary.yml", "release.yml"] {
        let with_matrix: BTreeSet<String> = workflow_jobs(workflow)
            .iter()
            .filter(|(_, body)| !matrix_entries(body).is_empty())
            .map(|(id, _)| id.clone())
            .collect();
        let meant: BTreeSet<String> = expected
            .iter()
            .filter(|(w, ..)| *w == workflow)
            .map(|(_, job, ..)| job.to_string())
            .collect();
        assert_eq!(
            with_matrix, meant,
            "{workflow}: the jobs with a target matrix are not the ones this test holds \
             to a target set"
        );
    }

    let mut runner_of: HashMap<String, (String, String)> = HashMap::new();
    for (workflow, job, targets, why) in expected {
        let jobs = workflow_jobs(workflow);
        let body = &jobs
            .iter()
            .find(|(id, _)| id == job)
            .unwrap_or_else(|| panic!("{workflow} has no `{job}` job"))
            .1;
        let entries = matrix_entries(body);

        let covered: BTreeSet<String> = entries.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            entries.len(),
            covered.len(),
            "{workflow} `{job}` lists a target twice"
        );
        assert_eq!(&covered, targets, "{workflow} `{job}`: {why}");

        for (target, os) in &entries {
            assert!(
                !os.is_empty() && !os.contains("${{"),
                "{workflow} `{job}`: {target} has no literal runner label"
            );
            let here = format!("{workflow} `{job}`");
            match runner_of.get(target) {
                Some((there, other)) => assert_eq!(
                    os, other,
                    "{here} runs {target} on {os} but {there} runs it on {other}; the suite, \
                     the canary and the release must answer about the same machine"
                ),
                None => {
                    runner_of.insert(target.clone(), (here, os.clone()));
                }
            }
        }
    }
}

/// Every per-target check run `ci.yml` and the canary report has a name of its
/// own.
///
/// GitHub matches a required status check **by name**, so while both workflows
/// named their matrix jobs `${{ matrix.target }}`, a branch-protection rule
/// could not tell "the suite passed on `aarch64-apple-darwin`" from "the
/// prebuilt resolved on `aarch64-apple-darwin`" — six names, two producers each,
/// and the canary's success able to stand in for the suite's. Only `ci.yml`'s
/// suite legs are required, so the canary's are prefixed. ADR-0188's build-only
/// leg is the same hazard a third way: "`x86_64-apple-darwin` built" must never
/// be able to satisfy a requirement meant for "`x86_64-apple-darwin` passed the
/// suite".
///
/// So each job's name is expanded per target and every resulting name must have
/// exactly one producer. The suite keeps the bare `<target>` names, because those
/// are what branch protection requires, and no other job may report a bare
/// triple, even one the suite has stopped running.
///
/// This is a property of the *names*, which is why a test can hold it: the
/// collision was invisible in both files read separately and obvious the moment
/// the check runs on one commit were listed together.
#[test]
fn every_per_target_check_name_is_unique() {
    let shipped = shipped_targets();
    let mut producer_of: HashMap<String, String> = HashMap::new();
    let mut jobs_seen = 0;
    for workflow in ["ci.yml", "skia-canary.yml"] {
        for (job, body) in workflow_jobs(workflow) {
            let entries = matrix_entries(&body);
            if entries.is_empty() {
                continue;
            }
            jobs_seen += 1;
            let template =
                job_name(&body).unwrap_or_else(|| panic!("{workflow} `{job}` has no job name"));
            assert!(
                template.contains("${{ matrix.target }}"),
                "{workflow} `{job}`'s name {template:?} does not name its target, so its \
                 legs would share one check name"
            );
            if workflow == "ci.yml" && job == "suite" {
                assert_eq!(
                    template, "${{ matrix.target }}",
                    "the suite's check names are branch protection's required checks; \
                     renaming them is a change to that contract"
                );
            }
            for (target, _) in entries {
                let name = template.replace("${{ matrix.target }}", &target);
                let here = format!("{workflow} `{job}`");
                // A bare triple is the suite's name even for a target the suite no
                // longer runs: branch protection that still requires
                // `x86_64-apple-darwin` must not be satisfied by a build-only leg.
                assert!(
                    (workflow == "ci.yml" && job == "suite") || !shipped.contains(&name),
                    "{here} reports a check run named {name:?}, a bare target triple, which \
                     is the suite's name; a required check by that name would be satisfied \
                     by a job that is not the suite"
                );
                if let Some(there) = producer_of.insert(name.clone(), here.clone()) {
                    panic!(
                        "{here} and {there} both report a check run named {name:?}, and a \
                         branch-protection rule matches a required check by name: one's \
                         result could satisfy a requirement meant for the other."
                    );
                }
            }
        }
    }
    assert_eq!(
        jobs_seen, 3,
        "expected three per-target jobs (the suite, the build-only job, the canary)"
    );
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
