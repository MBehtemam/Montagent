//! The renderer opens nothing outside the declared font chain, structurally.
//!
//! ADR-0007 makes a project's `fonts` an ordered chain of font **files**, never a
//! system family: *"a family name is an entry in a table you cannot read, cannot
//! commit, and that differs per machine."* ADR-0010 states the consequence for
//! the renderer — it opens nothing outside the chain — and spec #168's story 53
//! states it for the agent: *"a project that renders on my machine renders on a
//! clean one."*
//!
//! Left to runtime discipline that is a rule somebody remembers. `fontique`'s
//! `system` feature is the code that would resolve a family name against a
//! machine's font book, and with it not compiled in there is nothing to
//! remember: the lookup cannot happen because it does not exist.
//!
//! This is not hypothetical. #34's harness shaped against the system family
//! `SF Pro Rounded`, which is present on one of ADR-0064's six tier-1 targets —
//! and which #143 established is not in the repository and is not
//! redistributable. It looked right for a year on the machine that wrote it.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

fn parley_pin() -> toml::Value {
    let path = repo_root().join("Cargo.toml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path:?}: {e}"));
    let manifest: toml::Value =
        toml::from_str(&text).unwrap_or_else(|e| panic!("parsing {path:?}: {e}"));
    manifest["workspace"]["dependencies"]["parley"].clone()
}

#[test]
fn the_text_stack_is_built_without_system_font_discovery() {
    let pin = parley_pin();

    assert_eq!(
        pin["default-features"].as_bool(),
        Some(false),
        "`parley`'s default features include `system`, which compiles in system-font \
         discovery. ADR-0007 makes a font a file and never a family name, and ADR-0010 \
         requires the renderer to open nothing outside the declared chain."
    );

    let features: Vec<&str> = pin["features"]
        .as_array()
        .expect("the pin states its feature list explicitly")
        .iter()
        .map(|v| v.as_str().expect("a feature is a string"))
        .collect();

    assert!(
        !features.contains(&"system"),
        "`system` is back on: a family name could resolve against the machine's font \
         book, so a project that renders here would not render on a clean one (#168, \
         story 53)."
    );
}

/// No crate in the tree *queries* a machine's font book.
///
/// Note what this does and does not claim. It does **not** claim nothing links
/// `fontconfig`: Skia's own Linux link line ends in `-lfreetype -lfontconfig`
/// (`skia-bindings`' `platform/linux.rs`), so a Linux build needs those
/// libraries present whatever `parley` is built with, and CI installs them.
///
/// What it claims is narrower and is the part that matters here:
/// `yeslogic-fontconfig-sys` is the crate `fontique` uses to *enumerate and
/// resolve* system font families, and it is absent. Skia linking a library it
/// never asks Montaget's font questions through is incidental; a font-discovery
/// crate in the tree would be a route by which a family name could resolve
/// against the machine, which ADR-0007 and ADR-0010 rule out.
#[test]
fn no_crate_in_the_tree_resolves_a_system_font_family() {
    let lock = repo_root().join("Cargo.lock");
    let text = fs::read_to_string(&lock).unwrap_or_else(|e| panic!("reading {lock:?}: {e}"));
    assert!(
        !text.contains("yeslogic-fontconfig-sys"),
        "`yeslogic-fontconfig-sys` is back in the lock file. It arrives with \
         `fontique`'s `system` feature, and it is how a family name would come to \
         resolve against whatever fonts a machine happens to have."
    );
}
