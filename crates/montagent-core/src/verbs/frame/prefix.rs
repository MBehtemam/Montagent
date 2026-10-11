//! The static-prefix cache (ADR-0195, #932): the bottom run of time-invariant layers of a
//! frame, composited once and put back instead of painted again.
//!
//! Painters that paint frames (`render`, `preview`) share one immutable
//! [`Cached`] — the last prefix published — and [`Painter::paint`](super::Painter::paint)
//! restores it while the frame's prefix is the same elements on the same canvas. The cache is
//! the pixels *and* everything the prefix's painting recorded (`painted`, `not_painted`,
//! `painted_partially`, `declined`), so a frame that restores reports what a frame that
//! painted reports (ADR-0144 §7).
//!
//! **Why it changes no byte.** The surface is 8-bit premultiplied RGBA, every draw already
//! rounds to 8 bits, and a restore is an exact copy, so the layers above the prefix see the
//! input they would have seen (probed byte for byte on the repro kit, #923). The only way
//! it could change one is by caching a layer whose pixels differ between two frames, which
//! is what [`invariant`] exists to refuse: it is a whitelist, and anything it does not know
//! ends the prefix.

use std::cell::Cell;
use std::sync::{Arc, Mutex};

use montagent_render::canvas::Raster;
use serde_json::Value;

use super::{NotPainted, animatable};
use crate::finding::Finding;

/// `off` turns the cache off for a measurement sweep that times the shipped binary as a
/// process, as [`crate::verbs::render::PAINTING_VAR`] forces K and C. `#[doc(hidden)]`
/// measurement plumbing, never a setting: nothing a project, a flag or an MCP call says
/// reaches it, and it changes no byte of the file.
#[doc(hidden)]
pub const PREFIX_CACHE_VAR: &str = "MONTAGENT_PREFIX_CACHE";

thread_local! {
    static FORCED: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Whether painters built on this thread cache their prefix: on, unless a test forced it off
/// or [`PREFIX_CACHE_VAR`] says `off`.
pub(crate) fn enabled() -> bool {
    FORCED
        .with(Cell::get)
        .unwrap_or_else(|| std::env::var(PREFIX_CACHE_VAR).map_or(true, |value| value != "off"))
}

/// Force the cache on or off for this thread's painters until the guard drops.
#[doc(hidden)]
#[must_use = "the override lasts as long as the guard"]
pub struct ForcedPrefixCache {
    before: Option<bool>,
}

#[doc(hidden)]
pub fn force_prefix_cache(on: bool) -> ForcedPrefixCache {
    let before = FORCED.with(|cell| cell.replace(Some(on)));
    ForcedPrefixCache { before }
}

impl Drop for ForcedPrefixCache {
    fn drop(&mut self) {
        FORCED.with(|cell| cell.set(self.before));
    }
}

/// The effects whose result depends on the canvas and their own parameters alone. Every
/// other name, `grain` (keyed on the local frame) among them, ends the prefix.
const STATIC_EFFECTS: [&str; 10] = [
    "blur",
    "shadow",
    "mask",
    "tint",
    "saturation",
    "brightness",
    "contrast",
    "chroma",
    "posterize",
    "glow",
];

/// Whether `element` paints the same pixels on every frame it is present in, as far as its
/// own document goes. (Whether a running transition bridges it is per frame, and asked of
/// the painter.)
///
/// A whitelist: a still-image, text or vector element, `normal` blend, no `motion_blur`, no
/// keyframe anywhere (every animatable property, effect parameters and gradient parameters
/// included, is read through [`animatable::declared`]), only effects that read nothing but
/// their parameters, and, on a `text`, no `units` stagger and no `highlight` window.
pub(crate) fn invariant(element: &Value) -> bool {
    let kind = element.get("type").and_then(Value::as_str);
    if !matches!(kind, Some("image" | "text" | "rect" | "ellipse" | "path")) {
        return false;
    }
    if element
        .get("blend")
        .and_then(Value::as_str)
        .is_some_and(|blend| blend != "normal")
        || element.get("motion_blur").is_some()
        || element.get("group").is_some()
        || element.get("units").is_some()
    {
        return false;
    }
    if animatable::declared(element)
        .iter()
        .any(|declared| declared.records().is_some())
    {
        return false;
    }
    let effects = element.get("effects").and_then(Value::as_array);
    if effects.into_iter().flatten().any(|effect| {
        !effect
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| STATIC_EFFECTS.contains(&name))
    }) {
        return false;
    }
    let highlighted = element
        .get("runs")
        .and_then(Value::as_array)
        .is_some_and(|runs| runs.iter().any(|run| run.get("highlight").is_some()));
    !highlighted
}

/// The latest immutable composite, shared across one render's painters. The mutex only
/// guards publication and lookup; no painting or pixel copying holds it.
pub(crate) type Shared = Arc<Mutex<Option<Arc<Cached>>>>;

/// What one painter has composited and what painting it recorded.
pub(crate) struct Cached {
    /// The prefix's elements, by their position in the painter's element list, and the
    /// canvas it was composited on.
    pub(crate) key: (Vec<usize>, (i32, i32, u32, u32)),
    pub(crate) pixels: Raster,
    pub(crate) record: Record,
}

/// The per-frame lists the prefix's elements appended, in the order they appended them.
#[derive(Default, Clone)]
pub(crate) struct Record {
    pub(crate) painted: Vec<String>,
    pub(crate) not_painted: Vec<NotPainted>,
    pub(crate) painted_partially: Vec<NotPainted>,
    pub(crate) declined: Vec<Finding>,
    pub(crate) sources: Vec<String>,
    pub(crate) fonts: Vec<String>,
}
