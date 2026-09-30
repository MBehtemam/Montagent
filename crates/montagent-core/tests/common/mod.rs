//! Scratch-directory helpers shared by the integration tests.

#![allow(dead_code)]

pub mod compare;
pub mod media;

use std::path::{Path, PathBuf};

/// A scratch directory created fresh on every run, unique to the line that asked for it
/// so that tests running in parallel never share one.
pub fn tempdir(caller_line: u32) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "montagent-core-tests/{}-line-{caller_line}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Write `body` to `name` inside `dir` and hand back the path.
pub fn write_project(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write project file");
    path
}

/// The same project, written in the canonical convention.
///
/// Test projects are usually composed by interpolation, which produces one long line — and
/// `validate` reports that, correctly and unconditionally, as `L-LAYOUT` (ADR-0041). A test
/// about some *other* question routes its project through here, so that the convention is
/// not a thing every such test has to hand-maintain in a string literal. A test about layout
/// itself writes its bytes directly; that is what `tests/fmt.rs` does.
///
/// It goes through the product's own writer rather than a hand-formatted literal, so a test
/// project cannot drift from the convention the product enforces.
pub fn canonical(body: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(body).expect("a test writes valid JSON");
    montagent_core::write::canonical(&montagent_core::layout::canonicalise(&value))
}

/// Whether this machine has the `ffmpeg`/`ffprobe` the disk half of `validate` needs.
///
/// ADR-0009 ships Montagent as *"a binary, plus an `ffmpeg` the user supplies"*, so not
/// having one is a legitimate state rather than a broken checkout — and one the tool
/// answers with exit 70 rather than with a verdict. Tests that need real media say so
/// through this, which asks the same question `validate` asks; inferring it from an exit
/// code afterwards would green-light every *other* internal failure too.
pub fn has_ffprobe() -> bool {
    match montagent_core::media::tools::resolve() {
        Ok(_) => true,
        Err(missing) => {
            eprintln!("skipping: {}", missing.reason());
            false
        }
    }
}

/// A path rendered with `/` separators, so a tail can be compared to one.
///
/// The project format writes `audio/05-cobweb.mp3`. A *resolved* path is the
/// platform's, and on Windows that is backslashes plus, after `canonicalize`, a
/// `\\?\` verbatim prefix — so `ends_with("audio/05-cobweb.mp3")` is false for
/// a path that is entirely correct. Normalise before comparing a tail; the
/// separator is the platform's, never the document's.
pub fn with_forward_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

/// The committed fixture's directory.
///
/// One definition, because four test binaries want it and a path spelled four times is
/// four things to fix when the fixture moves.
pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

/// The fixture's project file.
pub fn fixture_project() -> PathBuf {
    fixture_dir().join("en-halloween-decorating.montagent.json")
}

/// The constructed document with one unpainted visual state (#437): at 25 fps, `a` ends at
/// 1010 and `b` starts at 1030 on different tracks, over a whole-span `bg`.
pub fn unpainted_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/unpainted-visual-state/unpainted-visual-state.montagent.json")
}

/// A project file written into the fixture's own directory and removed again, even when
/// the assertion between the two panics.
///
/// The fixture's assets resolve against the project file's own directory (ADR-0053), so a
/// variant of it — one element removed, one value broken — has to sit beside them rather
/// than in a scratch directory of its own. Shared by `reference_frames.rs` and
/// `effects.rs`: a bare write-then-remove leaves the copy behind whenever the assertion
/// between them fails, which is exactly when someone is reading `git status`.
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn beside_the_fixture(name: &str, body: &str) -> Scratch {
        // One fixed name **per caller**, rather than a unique one per write: a caller may
        // write several in sequence inside a single test and can share one file — but the
        // harness runs the *tests* on parallel threads, and two of them writing one path
        // would race. Naming it at the call site is what keeps that visible. None of these
        // is a project any other test reads and nothing globs this directory for
        // `*.montagent.json`, so the only cost of a copy that a hard abort leaves behind is
        // a line of `git status` — and the next run overwrites it.
        let path = fixture_dir().join(format!("{name}.montagent.json"));
        std::fs::write(&path, body).expect("write the scratch project beside the fixture");
        Scratch(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// A project file as plain JSON.
///
/// Tests that assert *about the fixture* read it as data rather than restating it: a table
/// of hand-copied instants is a second statement of the project, and the first one to
/// drift would be the copy. Shared, because three test binaries want the same two lines.
pub fn document(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("the project")).expect("json")
}

/// Every element of a project, across all of its tracks — a track supplies stacking, never
/// timing, so for a question about the clock the partition into tracks carries nothing.
pub fn elements(document: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> {
    document["tracks"]
        .as_array()
        .expect("tracks")
        .iter()
        .flat_map(|track| track["elements"].as_array().expect("elements"))
}
