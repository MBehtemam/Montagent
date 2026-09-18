// Where the repository is, so the harness runs from anywhere.
//
// #34 hardcoded absolute paths, which was fine for a throwaway and is not fine
// now that ADR-0010 keeps this in-tree as the oracle and #189 runs it on six
// targets in CI. Scene asset paths are repository-relative and resolved here.
use std::path::{Path, PathBuf};

/// The repository root: four levels above `docs/research/prototypes/rust-rasterizer`.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .expect("the repository root is four levels above this crate")
        .to_path_buf()
}

/// This crate's own directory, which is where the scene files live.
pub fn harness_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A harness-relative path (a scene file, a golden frame), made absolute.
pub fn in_harness(p: &str) -> String {
    let path = Path::new(p);
    if path.is_absolute() {
        return p.to_string();
    }
    harness_dir().join(path).to_string_lossy().into_owned()
}

/// A repository-relative path, made absolute. An already-absolute path is left
/// alone, so a scene may still point at something outside the tree.
pub fn resolve(p: &str) -> String {
    let path = Path::new(p);
    if path.is_absolute() {
        return p.to_string();
    }
    root().join(path).to_string_lossy().into_owned()
}

/// The font the fixture vendors, and the only one this harness opens.
///
/// #143 re-vendored Open Runde because the reference video's SF Pro Rounded was
/// never in the repository and is not redistributable. That matters here beyond
/// licensing: #34 shaped against a **system** family, which exists on one of the
/// six tier-1 targets and would make the oracle unrunnable on the other five.
/// Opening nothing outside a declared font file is also what ADR-0010 requires
/// of the real renderer, so the harness now models the thing it guards.
pub fn vendored_font() -> PathBuf {
    root().join("fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
}
