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
//! **A `.ttc` entry is cut down to its face before it is registered.** ADR-0007's `index`
//! is honoured by handing `fontique` a file that holds one face — see [`crate::sfnt`] for
//! why it cannot be honoured by selecting one afterwards. This module's part is only to
//! apply the default: ADR-0007 says `index` *"defaults to 0"*, and an omitted one is
//! **not** the same as leaving the collection whole, which resolves by an attribute query
//! the format does not carry. A single-face file is passed through untouched.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parley::FontContext;
use parley::fontique::{Blob, Collection, CollectionOptions, FontInfoOverride, SourceCache};
use parley::style::{FontFamily, FontFamilyName};

use crate::sfnt;

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
    /// The file, where one is named. `None` for a key the project does not declare, which
    /// is about the table rather than about a file, and for a face registered from bytes
    /// ([`Fonts::register_bytes`]), which came from no file.
    pub path: Option<PathBuf>,
    /// What was wrong, as one clause: it is printed after *"<path> could not be read: "*
    /// by the caller, so it neither repeats the path nor ends in a full stop. With no
    /// `path`, [`Display`](std::fmt::Display) prints it alone.
    pub reason: String,
}

impl FontError {
    fn unreadable(path: &Path, e: &std::io::Error) -> FontError {
        FontError {
            path: Some(path.to_path_buf()),
            reason: e.to_string(),
        }
    }

    fn not_a_font(path: Option<&Path>) -> FontError {
        FontError {
            path: path.map(Path::to_path_buf),
            reason: "it registered no font family — not a font, or not one this build can \
                     parse"
                .to_string(),
        }
    }

    fn no_such_face(path: Option<&Path>, index: u32, faces: u32) -> FontError {
        FontError {
            path: path.map(Path::to_path_buf),
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

    /// Register one key as a single face whose bytes are already in memory.
    ///
    /// For a face the **binary** carries rather than one a project declares: Montagent's own
    /// chrome face (#421) is `include_bytes!`'d, because a face read off a path is a face a
    /// clean machine may not have, and the chrome must not depend on the document or the
    /// machine it runs on. So there is no path here, and nothing is added to
    /// [`Fonts::opened`] — that list is every file the registry read, and this reads none.
    ///
    /// Otherwise the same registration as a chain entry: a collection is cut down to its
    /// `index` (face 0 when omitted, ADR-0102), the face goes in under a synthetic family
    /// name, and a refusal carries the same sentence with no path in front of it.
    pub fn register_bytes(
        &mut self,
        key: &str,
        bytes: &[u8],
        index: Option<u32>,
    ) -> Result<(), FontError> {
        let family = self.register_face(key, 0, bytes, index, None)?;
        self.chains.insert(key.to_string(), vec![family]);
        Ok(())
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
        self.register_face(key, position, &bytes, file.index, Some(&file.path))
    }

    /// Register the face at `index` of `bytes` as one chain position, naming `path` in any
    /// refusal when the bytes came from one.
    fn register_face(
        &mut self,
        key: &str,
        position: usize,
        bytes: &[u8],
        index: Option<u32>,
        path: Option<&Path>,
    ) -> Result<String, FontError> {
        // ADR-0007 defaults `index` to 0, and the default is applied here rather than left
        // to the registry: an omitted index on a collection means face 0, and a whole
        // collection resolves to whichever face a default attribute query wins — face 7 on
        // a stock `Avenir Next.ttc`, never face 0. Omitted and `0` are the same request,
        // and both have to be cut out.
        let index = index.unwrap_or(0);
        let face = match sfnt::face(bytes, index) {
            Ok(face) => face,
            Err(sfnt::FaceError::NotAFont) => return Err(FontError::not_a_font(path)),
            Err(sfnt::FaceError::NoSuchFace { faces }) => {
                return Err(FontError::no_such_face(path, index, faces));
            }
        };

        let family = synthetic_family(key, position);
        let registered = self.context.collection.register_fonts(
            Blob::new(Arc::new(face.into_owned())),
            Some(FontInfoOverride {
                family_name: Some(&family),
                ..Default::default()
            }),
        );

        // One face went in, so this is no longer a question about *which* face came back —
        // only whether the face registered at all. A parsable sfnt that registers nothing
        // is one this build cannot draw with, which is the same answer as not a font.
        if registered.iter().all(|(_, fonts)| fonts.is_empty()) {
            return Err(FontError::not_a_font(path));
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
