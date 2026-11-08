//! The coarse `nest` scene cache (ADR-0245, #1144): offscreen child composites (`width × height`
//! 8-bit premultiplied RGBA plus child paint records) keyed by relative sub-graph hash,
//! shared across frames within a render and persisted under `<cache_dir>/nest-cache/` across
//! re-renders.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use montagent_render::canvas::{Extent, Raster};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::prefix::Record;
use crate::media::Source;

/// `off` turns the `nest` scene cache off for a measurement sweep or byte-identity comparison,
/// parallel to [`super::prefix::PREFIX_CACHE_VAR`].
#[doc(hidden)]
pub const NEST_CACHE_VAR: &str = "MONTAGENT_NEST_CACHE";

const MAGIC: &[u8; 4] = b"MNST";
const MAX_SCENES: usize = 64;
const MAX_BYTES: u64 = 256 * 1024 * 1024;

thread_local! {
    static FORCED: Cell<Option<bool>> = const { Cell::new(None) };
    static FORCED_DIR: RefCell<Option<Option<PathBuf>>> = const { RefCell::new(None) };
}

/// Whether painters built on this thread cache `nest` offscreen child composites.
pub(crate) fn enabled() -> bool {
    FORCED
        .with(Cell::get)
        .unwrap_or_else(|| std::env::var(NEST_CACHE_VAR).map_or(true, |value| value != "off"))
}

/// Force the `nest` scene cache on or off for this thread's painters until the guard drops.
#[doc(hidden)]
#[must_use = "the override lasts as long as the guard"]
pub struct ForcedNestCache {
    before: Option<bool>,
}

#[doc(hidden)]
pub fn force_nest_cache(on: bool) -> ForcedNestCache {
    let before = FORCED.with(|cell| cell.replace(Some(on)));
    ForcedNestCache { before }
}

impl Drop for ForcedNestCache {
    fn drop(&mut self) {
        FORCED.with(|cell| cell.set(self.before));
    }
}

/// Override the base cache directory for this thread's painters until the guard drops.
#[doc(hidden)]
#[must_use = "the override lasts as long as the guard"]
pub struct ForcedNestCacheDir {
    before: Option<Option<PathBuf>>,
}

#[doc(hidden)]
pub fn force_cache_dir(dir: Option<PathBuf>) -> ForcedNestCacheDir {
    let before = FORCED_DIR.with(|cell| cell.replace(Some(dir)));
    ForcedNestCacheDir { before }
}

impl Drop for ForcedNestCacheDir {
    fn drop(&mut self) {
        FORCED_DIR.with(|cell| {
            *cell.borrow_mut() = self.before.take();
        });
    }
}

pub(crate) fn current_dir_override() -> Option<Option<PathBuf>> {
    FORCED_DIR.with(|cell| cell.borrow().clone())
}

/// Where `<cache_dir>/nest-cache` lives, or `None` when disk persistence is off.
pub(crate) fn cache_dir() -> Option<PathBuf> {
    if let Some(overridden) = current_dir_override() {
        return overridden.map(|base| base.join("nest-cache"));
    }
    crate::media::sidecar::Sidecar::default_path()
        .and_then(|sidecar| sidecar.parent().map(|base| base.join("nest-cache")))
}

/// Cache slot within a `nest` sub-graph (ADR-0245 §2).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Slot {
    /// Active child indices in `nest.elements` when all active children at `instant` are
    /// time-invariant (`prefix::invariant`).
    State(Vec<usize>),
    /// Relative millisecond offset `instant - nest.start` when any active child varies with time.
    Offset(i64),
}

impl Slot {
    fn filename(&self) -> String {
        match self {
            Slot::State(indices) if indices.is_empty() => "state-empty.cache".to_string(),
            Slot::State(indices) => {
                let joined = indices
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join("_");
                format!("state-{joined}.cache")
            }
            Slot::Offset(ms) => format!("offset-{ms}.cache"),
        }
    }
}

/// In-memory map of `(subgraph_hash, slot) -> Arc<Cached>` shared across a render's painters.
pub(crate) type Shared = Arc<Mutex<BTreeMap<(String, Slot), Arc<Cached>>>>;

pub(crate) struct Cached {
    pub(crate) pixels: Raster,
    pub(crate) record: Record,
}

#[derive(Serialize, Deserialize)]
struct DiskRecord {
    painted: Vec<String>,
    sources: Vec<String>,
    fonts: Vec<String>,
}

/// Compute `(subgraph_hash_hex, can_persist_to_disk)` for a `nest` container's child composite.
pub(crate) fn subgraph_key(
    element: &Value,
    extent: Extent,
    fps: impl Into<crate::model::FrameRate>,
    document: &Value,
    project_dir: &Path,
) -> Option<(String, bool)> {
    let fps = fps.into();
    let nest_start = element.get("start").and_then(Value::as_i64)?;
    let children = element.get("elements")?;
    let normalized = normalize_children(children, nest_start);

    let mut asset_paths = BTreeSet::new();
    let mut font_keys = BTreeSet::new();
    collect_child_assets(children, &mut asset_paths, &mut font_keys);
    let mut referenced_fonts = serde_json::Map::new();
    if let Some(fonts) = document.get("fonts").and_then(Value::as_object) {
        for key in &font_keys {
            if let Some(entry) = fonts.get(key) {
                referenced_fonts.insert(key.clone(), entry.clone());
                collect_font_files(entry, &mut asset_paths);
            }
        }
    }

    let mut persist = true;
    let mut file_identities = Vec::with_capacity(asset_paths.len());
    for rel in asset_paths {
        match Source::resolve(&rel, project_dir) {
            Source::Local(path) => match file_identity(&path) {
                Some(identity) => file_identities.push((rel, identity)),
                None => persist = false,
            },
            Source::Remote(_) => persist = false,
        }
    }

    let payload = json!({
        "v": "montagent-nest-cache-v1",
        "fps": fps,
        "width": extent.width,
        "height": extent.height,
        "children": normalized,
        "fonts": Value::Object(referenced_fonts),
        "files": file_identities,
    });
    let serialized = serde_json::to_vec(&payload).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    let digest = format!("{:x}", hasher.finalize());
    Some((digest, persist))
}

/// Compute the cache slot within a `nest` sub-graph:
/// - `Slot::State(indices)` when all active children at `instant` are time-invariant (`prefix::invariant`),
/// - `Slot::Offset(ms)` when any active child varies with time and `!sampling`.
pub(crate) fn slot_key(
    nest_start: i64,
    instant: i64,
    sampling: bool,
    active: &[(usize, &Value)],
) -> Option<Slot> {
    if active
        .iter()
        .all(|(_, child)| super::prefix::invariant(child))
    {
        return Some(Slot::State(active.iter().map(|(idx, _)| *idx).collect()));
    }
    if sampling {
        return None;
    }
    Some(Slot::Offset(instant.saturating_sub(nest_start)))
}

/// Look up `(subgraph, slot)` in memory or on disk.
pub(crate) fn load(
    shared: &Shared,
    subgraph: &str,
    slot: &Slot,
    persist: bool,
) -> Option<Arc<Cached>> {
    if let Some(hit) = shared
        .lock()
        .unwrap()
        .get(&(subgraph.to_string(), slot.clone()))
        .cloned()
    {
        return Some(hit);
    }
    if !persist {
        return None;
    }
    let dir = cache_dir()?;
    let path = dir.join(subgraph).join(slot.filename());
    let cached = read_disk_entry(&path)?;
    let arc = Arc::new(cached);
    shared
        .lock()
        .unwrap()
        .insert((subgraph.to_string(), slot.clone()), arc.clone());
    Some(arc)
}

/// Publish `(subgraph, slot)` to memory and (when clean and `persist` is true) to disk.
pub(crate) fn store(
    shared: &Shared,
    subgraph: String,
    slot: Slot,
    persist: bool,
    cached: Arc<Cached>,
) {
    if persist
        && cached.record.not_painted.is_empty()
        && cached.record.painted_partially.is_empty()
        && cached.record.declined.is_empty()
        && let Some(dir) = cache_dir()
    {
        let scene_dir = dir.join(&subgraph);
        let path = scene_dir.join(slot.filename());
        if !path.exists() {
            let _ = write_disk_entry(&path, &cached);
            evict_if_needed(&dir);
        }
    }
    shared.lock().unwrap().insert((subgraph, slot), cached);
}

fn read_disk_entry(path: &Path) -> Option<Cached> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 16 || &bytes[0..4] != MAGIC {
        return None;
    }
    let width = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    let height = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    let json_len = u32::from_le_bytes(bytes[12..16].try_into().ok()?) as usize;
    let pixels_start = 16usize.checked_add(json_len)?;
    let pixels_len = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)?;
    if bytes.len() != pixels_start.checked_add(pixels_len)? {
        return None;
    }
    let disk_record: DiskRecord = serde_json::from_slice(&bytes[16..pixels_start]).ok()?;
    let pixels = Raster::from_premul_rgba(&bytes[pixels_start..], width, height)?;
    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) {
        let _ = file.set_modified(std::time::SystemTime::now());
    }
    Some(Cached {
        pixels,
        record: Record {
            painted: disk_record.painted,
            not_painted: Vec::new(),
            painted_partially: Vec::new(),
            declined: Vec::new(),
            sources: disk_record.sources,
            fonts: disk_record.fonts,
        },
    })
}

fn write_disk_entry(path: &Path, cached: &Cached) -> Option<()> {
    let (width, height, rgba) = cached.pixels.to_premul_rgba()?;
    let disk_record = DiskRecord {
        painted: cached.record.painted.clone(),
        sources: cached.record.sources.clone(),
        fonts: cached.record.fonts.clone(),
    };
    let meta = serde_json::to_vec(&disk_record).ok()?;
    let json_len = u32::try_from(meta.len()).ok()?;

    let parent = path.parent()?;
    std::fs::create_dir_all(parent).ok()?;
    let tmp = parent.join(format!(
        "{}.{}.tmp",
        path.file_name()?.to_string_lossy(),
        std::process::id()
    ));

    let mut buf = Vec::with_capacity(16 + meta.len() + rgba.len());
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&width.to_le_bytes());
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(&json_len.to_le_bytes());
    buf.extend_from_slice(&meta);
    buf.extend_from_slice(&rgba);

    if std::fs::write(&tmp, &buf).is_ok() {
        if std::fs::rename(&tmp, path).is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
    } else {
        let _ = std::fs::remove_file(&tmp);
    }
    Some(())
}

fn evict_if_needed(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut scenes: Vec<(u128, u64, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let (mtime, bytes) = scene_stats(&path);
        scenes.push((mtime, bytes, path));
    }
    let mut total_bytes: u64 = scenes.iter().map(|(_, b, _)| *b).sum();
    if scenes.len() <= MAX_SCENES && total_bytes <= MAX_BYTES {
        return;
    }
    scenes.sort_by_key(|(mtime, _, _)| *mtime);
    let mut remaining = scenes.len();
    for (_, bytes, path) in scenes {
        if remaining <= MAX_SCENES && total_bytes <= MAX_BYTES {
            break;
        }
        let _ = std::fs::remove_dir_all(&path);
        remaining = remaining.saturating_sub(1);
        total_bytes = total_bytes.saturating_sub(bytes);
    }
}

fn scene_stats(scene_dir: &Path) -> (u128, u64) {
    let mut newest = 0u128;
    let mut bytes = 0u64;
    if let Ok(entries) = std::fs::read_dir(scene_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                bytes = bytes.saturating_add(meta.len());
                if let Ok(modified) = meta.modified()
                    && let Ok(dur) = modified.duration_since(UNIX_EPOCH)
                {
                    newest = newest.max(dur.as_nanos());
                }
            }
        }
    }
    (newest, bytes)
}

fn file_identity(path: &Path) -> Option<(String, u64, i128, String)> {
    let canonical = path.canonicalize().ok()?;
    let meta = std::fs::metadata(&canonical).ok()?;
    let mtime_ns = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i128::try_from(d.as_nanos()).ok())?;
    let content = crate::media::sidecar::fingerprint(&canonical)?;
    Some((
        canonical.to_str()?.to_string(),
        meta.len(),
        mtime_ns,
        content,
    ))
}

fn collect_child_assets(
    value: &Value,
    assets: &mut BTreeSet<String>,
    fonts: &mut BTreeSet<String>,
) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_child_assets(item, assets, fonts);
            }
        }
        Value::Object(map) => {
            if let Some(source) = map.get("source").and_then(Value::as_str) {
                assets.insert(source.to_string());
            }
            if let Some(lut) = map.get("lut").and_then(Value::as_str) {
                assets.insert(lut.to_string());
            }
            if let Some(font) = map.get("font").and_then(Value::as_str) {
                fonts.insert(font.to_string());
            }
            for child in map.values() {
                collect_child_assets(child, assets, fonts);
            }
        }
        _ => {}
    }
}

fn collect_font_files(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::String(s) => {
            out.insert(s.clone());
        }
        Value::Array(items) => {
            for item in items {
                collect_font_files(item, out);
            }
        }
        Value::Object(map) => {
            if let Some(file) = map.get("file").and_then(Value::as_str) {
                out.insert(file.to_string());
            }
        }
        _ => {}
    }
}

/// Normalize child elements so every timeline timestamp (`start`, `end`, keyframe `t`, and
/// text-run `highlight` bounds) is relative to `nest_start`.
fn normalize_children(value: &Value, nest_start: i64) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| normalize_child_value(item, nest_start))
                .collect(),
        ),
        other => normalize_child_value(other, nest_start),
    }
}

fn normalize_child_value(value: &Value, nest_start: i64) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| normalize_child_value(item, nest_start))
                .collect(),
        ),
        Value::Object(map) => {
            let mut out = serde_json::Map::with_capacity(map.len());
            let is_keyframe = map.contains_key("t") && map.contains_key("v");
            for (k, v) in map {
                let normalized = match (k.as_str(), v) {
                    ("start" | "end", Value::Number(n)) if !is_keyframe => n
                        .as_i64()
                        .map(|ms| json!(ms.saturating_sub(nest_start)))
                        .unwrap_or_else(|| v.clone()),
                    ("t", Value::Number(n)) if is_keyframe => n
                        .as_i64()
                        .map(|ms| json!(ms.saturating_sub(nest_start)))
                        .unwrap_or_else(|| v.clone()),
                    _ => normalize_child_value(v, nest_start),
                };
                out.insert(k.clone(), normalized);
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}
