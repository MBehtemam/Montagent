//! The segmenter `measure` **names** is the segmenter `measure` **ran**.
//!
//! ADR-0008 makes `measure` report *"the segmenter and data version alongside the
//! offsets"*, and the reason is that two versions of this data legitimately disagree — the
//! committed Thai evidence in `docs/research/prototypes/thai-line-breaking/results/` and
//! this build's own answer already differ by one offset. A version string is therefore a
//! load-bearing value rather than a label, and the way it goes wrong is the ordinary way: a
//! dependency bump nobody carries into the constant, after which every reported offset
//! cites a version that did not produce it.
//!
//! So the constant is checked against the workspace pin rather than trusted. The same shape
//! as `montagent-render/tests/skia_pin.rs`, for the same reason: a number stated in two
//! places is a number that can disagree with itself.

use std::fs;
use std::path::{Path, PathBuf};

use montagent_text::SEGMENTER;

fn workspace_manifest() -> toml::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    toml::from_str(&fs::read_to_string(&path).expect("the workspace manifest"))
        .expect("the workspace manifest is TOML")
}

fn lockfile() -> toml::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    toml::from_str(&fs::read_to_string(&path).expect("the workspace lockfile"))
        .expect("the lockfile is TOML")
}

/// Every version the lockfile resolved for one package name.
fn locked(name: &str) -> Vec<String> {
    lockfile()["package"]
        .as_array()
        .expect("a lockfile is a list of packages")
        .iter()
        .filter(|package| package["name"].as_str() == Some(name))
        .filter_map(|package| package["version"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn the_reported_version_is_the_pinned_one() {
    let pin = workspace_manifest()["workspace"]["dependencies"][SEGMENTER.name]["version"]
        .as_str()
        .expect("the segmenter is pinned in the workspace manifest")
        .to_string();

    assert_eq!(
        pin,
        format!("={}", SEGMENTER.version),
        "`{}` is pinned at `{pin}` and `measure` reports `{}`",
        SEGMENTER.name,
        SEGMENTER.version
    );
}

#[test]
fn the_reported_version_is_the_one_cargo_resolved() {
    // The pin is `=`, so exactly one version can be in the tree. Both halves are asserted
    // because they fail differently: a manifest edit without a `cargo update` leaves the
    // lockfile behind, and the binary links what the lockfile says.
    assert_eq!(
        locked(SEGMENTER.name),
        vec![SEGMENTER.version.to_string()],
        "the lockfile disagrees with the reported segmenter version"
    );
}

#[test]
fn the_reported_data_version_is_the_one_cargo_resolved() {
    let (name, version) = SEGMENTER
        .data
        .rsplit_once(' ')
        .expect("the data is reported as `<crate> <version>`");

    // The data crate is not pinned directly — it arrives through `icu_segmenter` — so the
    // lockfile is the only place its version is stated, and a bump moves it without
    // touching any manifest. This is the half a `=` pin does not cover.
    assert_eq!(
        locked(name),
        vec![version.to_string()],
        "the lockfile disagrees with the reported segmenter data version"
    );
}

#[test]
fn the_segmenter_is_declared_with_parleys_own_feature_set() {
    let manifest = workspace_manifest();
    let features = manifest["workspace"]["dependencies"][SEGMENTER.name]["features"]
        .as_array()
        .expect("the features are listed explicitly")
        .iter()
        .filter_map(|feature| feature.as_str())
        .collect::<Vec<_>>();

    // `compiled_data` alone is what `parley` already asks of this crate, which is what makes
    // ADR-0008's *"the same shared node parley links"* true rather than aspirational: the
    // declaration adds no data to the binary that was not already in it. It is also what
    // `LineSegmenter::new_dictionary` needs, and adding `auto`/`lstm` would link a second
    // model that answers Thai differently — the constant says `dictionary`, and this keeps
    // the cheaper-looking constructor from drifting in beside it.
    assert_eq!(features, ["compiled_data"]);
    assert_eq!(
        manifest["workspace"]["dependencies"][SEGMENTER.name]["default-features"].as_bool(),
        Some(false),
        "defaults would pull `auto`, and with it the LSTM model"
    );
}

#[test]
fn the_committed_thai_evidence_is_still_where_the_adr_says_it_is() {
    // ADR-0008 and #27 rest on this directory, and `docs/agents/domain.md` requires the
    // evidence an ADR cites to resolve to a path that exists. It is also this crate's
    // oracle: `parley-on-opportunities.txt` is what established that the *dictionary* model
    // — not `new_auto`'s LSTM — is the one whose offsets the project has actually seen.
    let evidence: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../docs/research/prototypes/thai-line-breaking/results/parley-on-opportunities.txt",
    );
    assert!(
        evidence.exists(),
        "{} is ADR-0008's committed evidence and is gone",
        evidence.display()
    );
}
