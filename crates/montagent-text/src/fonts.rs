//! The declared font chain, registered from files and nothing else.
//!
//! ADR-0007: *"Always a file path, relative to the project. Never a system family name."*
//! The reason is measured rather than stylistic — the original fixture's `SF Pro Rounded`
//! *"resolves on its author's machine only because it was hand-installed"*, so the one real
//! project ever authored names a font a clean machine cannot find.
//!
//! **Nothing outside the chain can be opened, structurally.** `fontique`'s system-font
//! discovery is not compiled in (the workspace takes `parley` without its `system`
//! feature), so there is no code in this binary that could resolve a system family name.
//! Spec #168's story 53 — *"a project that renders on my machine renders on a clean
//! one"* — is therefore a property of the build rather than of this module's discipline.
//!
//! **Each chain entry registers under a synthetic family name**, `"<key>#<position>"`,
//! rather than under whatever name the file gives itself. Three things follow, and all
//! three are the point: two entries whose files claim the same family name stay two
//! distinct entries; the chain's *order* is the order the author wrote, not an order a
//! font's own metadata chose; and a name in the document can never collide with a name a
//! font file happens to carry.
//!
//! **Recorded residual — `.ttc` face selection.** [`FontFile::index`] is checked against
//! the faces the file actually registered, so an out-of-range index is refused rather than
//! silently ignored. It does not yet *select* among them: `fontique` registers a
//! collection's faces into one family and picks between them on width/style/weight
//! attributes, which the format does not carry (ADR-0007: *"No `weight`, no `bold`. A
//! different weight is a different file."*). Every font in the committed fixture is a
//! single-face file, so this is untested territory rather than a known-wrong answer, and
//! it is named here so the next reader does not have to discover it.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parley::FontContext;
use parley::fontique::{Blob, Collection, CollectionOptions, FontInfoOverride, SourceCache};
use parley::style::{FontFamily, FontFamilyName};

/// One entry in a font's ordered fallback chain, as the project's `fonts` table writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFile {
    /// The file itself, already resolved against the project (ADR-0053).
    pub path: PathBuf,
    /// For `.ttc` collections.
    pub index: Option<u32>,
}

/// Why a declared chain did not reach the layout.
///
/// A path and a sentence, rather than a variant per cause. The caller is a report, and what
/// a report needs is exactly these two: *which file*, and *what was wrong with it*. A
/// project declaring four chains over six files needs the first of those or the reader pays
/// for a bisect; an enum it would have to re-word into prose anyway is a second place the
/// sentence could drift from the one the file itself would give.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontError {
    /// The file, where one is named. `None` only for a key the project does not declare,
    /// which is about the table rather than about a file.
    pub path: Option<PathBuf>,
    /// What was wrong, as one clause: it is printed after *"<path> could not be read: "*
    /// by the caller, so it neither repeats the path nor ends in a full stop.
    pub reason: String,
}

impl FontError {
    fn unreadable(path: &Path, e: &std::io::Error) -> FontError {
        FontError {
            path: Some(path.to_path_buf()),
            reason: e.to_string(),
        }
    }

    fn not_a_font(path: &Path) -> FontError {
        FontError {
            path: Some(path.to_path_buf()),
            reason: "it registered no font family — not a font, or not one this build can \
                     parse"
                .to_string(),
        }
    }

    fn no_such_face(path: &Path, index: u32, faces: usize) -> FontError {
        FontError {
            path: Some(path.to_path_buf()),
            reason: format!("it has no face at `index` {index} (it carries {faces})"),
        }
    }

    /// The project's `fonts` table declares no chain under this key.
    pub fn undeclared(key: &str) -> FontError {
        FontError {
            path: None,
            reason: format!(
                "the project's `fonts` table declares no `{key}` (ADR-0007: a text \
                 element's `font` is a key into that table, never a system family name)"
            ),
        }
    }
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(path) => write!(f, "{} could not be read: {}", path.display(), self.reason),
            None => f.write_str(&self.reason),
        }
    }
}

impl std::error::Error for FontError {}

/// Every font chain one measurement may shape in, and the parley context holding their
/// bytes.
pub struct Fonts {
    context: FontContext,
    /// Key → the synthetic family names of its chain, in the order the author wrote them.
    chains: BTreeMap<String, Vec<String>>,
    /// Every font file this registry has actually read, in the order it read them.
    ///
    /// Kept so a caller can *report* it. ADR-0007's "the renderer opens nothing outside
    /// the declared chain" is structural — `fontique`'s system-font discovery is not
    /// compiled in — but a structural property is invisible to a test, and #212 asks for
    /// one asserted with a decoy font installed. This list is what that test reads: the
    /// answer says which files were opened, and a decoy that was never opened cannot be
    /// in it.
    opened: Vec<PathBuf>,
}

impl Default for Fonts {
    fn default() -> Self {
        Fonts::new()
    }
}

impl Fonts {
    /// An empty registry that will never consult a system font book.
    pub fn new() -> Fonts {
        Fonts {
            context: FontContext {
                collection: Collection::new(CollectionOptions {
                    // One measurement, one thread: the shared store is pure overhead.
                    shared: false,
                    // Belt and braces. The feature that would implement this is not
                    // compiled in, so the flag changes nothing — it is set so that a build
                    // which *did* compile it in would still open nothing (ADR-0007).
                    system_fonts: false,
                }),
                source_cache: SourceCache::default(),
            },
            chains: BTreeMap::new(),
            opened: Vec::new(),
        }
    }

    /// Every font file this registry has read, in the order it read them.
    ///
    /// The whole set, which is the point: it is not "the files from the chain" filtered
    /// after the fact but every path [`Fonts::register`] passed to the filesystem, so a
    /// file from anywhere else would appear here if one were ever opened.
    pub fn opened(&self) -> &[PathBuf] {
        &self.opened
    }

    /// Register one key's whole chain, in order.
    ///
    /// An empty chain is registered as an empty chain rather than refused: what a
    /// zero-entry `fonts` entry means is `validate`'s question, and ADR-0024 keeps every
    /// judgment about the document out of the measuring path.
    pub fn register(&mut self, key: &str, chain: &[FontFile]) -> Result<(), FontError> {
        let mut families = Vec::with_capacity(chain.len());
        for (position, file) in chain.iter().enumerate() {
            families.push(self.register_one(key, position, file)?);
        }
        self.chains.insert(key.to_string(), families);
        Ok(())
    }

    /// The families one chain resolves to, as parley's font stack.
    ///
    /// Owned rather than borrowed from the registry: laying out needs the stack and
    /// [`Fonts::context`] at the same time, and a borrowed stack would hold the registry
    /// immutably across the mutable borrow the layout takes.
    pub fn chain(&self, key: &str) -> Result<FontFamily<'static>, FontError> {
        let families = self
            .chains
            .get(key)
            .ok_or_else(|| FontError::undeclared(key))?;
        Ok(FontFamily::List(
            families
                .iter()
                .map(|name| FontFamilyName::Named(Cow::Owned(name.clone())))
                .collect(),
        ))
    }

    /// Is this key declared?
    pub fn declares(&self, key: &str) -> bool {
        self.chains.contains_key(key)
    }

    /// The parley context the layout shapes against.
    pub(crate) fn context(&mut self) -> &mut FontContext {
        &mut self.context
    }

    fn register_one(
        &mut self,
        key: &str,
        position: usize,
        file: &FontFile,
    ) -> Result<String, FontError> {
        let bytes = std::fs::read(&file.path).map_err(|e| FontError::unreadable(&file.path, &e))?;
        // Recorded on the read rather than on success: a file that was opened and then
        // rejected was still opened, and a list that quietly dropped it would be a weaker
        // claim than the one ADR-0007 makes.
        if !self.opened.contains(&file.path) {
            self.opened.push(file.path.clone());
        }

        let family = synthetic_family(key, position);
        let registered = self.context.collection.register_fonts(
            Blob::new(Arc::new(bytes)),
            Some(FontInfoOverride {
                family_name: Some(&family),
                ..Default::default()
            }),
        );

        let faces: Vec<u32> = registered
            .iter()
            .flat_map(|(_, fonts)| fonts.iter().map(|font| font.index()))
            .collect();
        if faces.is_empty() {
            return Err(FontError::not_a_font(&file.path));
        }
        if let Some(index) = file.index
            && !faces.contains(&index)
        {
            return Err(FontError::no_such_face(&file.path, index, faces.len()));
        }
        Ok(family)
    }
}

/// The name one chain entry is registered under.
///
/// `#` rather than a character an author could type into a `fonts` key: the two namespaces
/// must not be able to meet, and a key containing `#` would otherwise be able to name
/// another key's chain entry.
fn synthetic_family(key: &str, position: usize) -> String {
    format!("{key}#{position}")
}

/// Read one chain entry's path, resolved against the project's own directory (ADR-0053).
pub fn resolve(file: &str, index: Option<u32>, base: &Path) -> FontFile {
    FontFile {
        path: base.join(file),
        index,
    }
}
