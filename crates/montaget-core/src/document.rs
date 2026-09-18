//! The project document — the spine every later ticket hangs a check off.
//!
//! **Deliberately lenient.** The only failure this layer can produce is "these bytes are
//! not JSON" (`E-PARSE`, exit 2). A missing `tracks`, an `fps` that is a string, an
//! unknown key, a retired spelling, an element with no `id` — every one of those is a
//! *finding* about a readable document, and ADR-0011 keeps exit 1 and exit 2 apart
//! precisely so the two cannot be confused. Typing them into `serde` here would turn
//! each one into a parse failure and collapse the distinction.
//!
//! So each field is `Option`, and the whole raw [`Value`] is kept beside the typed view.
//! The schema layer (ADR-0016's unknown-key error, ADR-0017's closed schema, the
//! per-type element shapes) is a later ticket, and it reads from [`Document::value`].

use serde_json::Value;

/// A parsed project file: the raw JSON, plus the handful of fields the spine needs.
#[derive(Debug, Clone)]
pub struct Document {
    /// The path the document was read from, as given.
    pub path: String,
    /// The document as parsed, byte-faithful in content if not in whitespace. The
    /// authority for every check; the typed fields below are a convenience over it.
    pub value: Value,
    pub frame: Option<Frame>,
    pub fps: Option<i64>,
    pub duration: Option<i64>,
    pub background: Option<String>,
    pub output: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub name: Option<String>,
    pub layer: Option<i64>,
    pub elements: Vec<Element>,
    pub value: Value,
}

/// One element, in the shape every element shares whatever its type (`CONTEXT.md`).
/// The per-type fields stay in [`Element::value`] until the schema ticket types them.
#[derive(Debug, Clone)]
pub struct Element {
    pub id: Option<String>,
    pub kind: Option<String>,
    pub group: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub value: Value,
}

impl Document {
    /// Build the typed view over an already-parsed JSON value.
    pub fn from_value(path: impl Into<String>, value: Value) -> Self {
        let tracks = value["tracks"]
            .as_array()
            .map(|tracks| tracks.iter().map(Track::from_value).collect())
            .unwrap_or_default();

        Document {
            path: path.into(),
            frame: Frame::from_value(&value["frame"]),
            fps: value["fps"].as_i64(),
            duration: value["duration"].as_i64(),
            background: value["background"].as_str().map(str::to_owned),
            output: value["output"].as_str().map(str::to_owned),
            tracks,
            value,
        }
    }

    /// Every element in the document, in document order. Array order carries no meaning
    /// for timing or stacking (ADR-0060); this is a traversal, not a ranking.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.tracks.iter().flat_map(|track| track.elements.iter())
    }
}

impl Frame {
    fn from_value(value: &Value) -> Option<Self> {
        Some(Frame {
            width: value["width"].as_i64()?,
            height: value["height"].as_i64()?,
        })
    }
}

impl Track {
    fn from_value(value: &Value) -> Self {
        Track {
            name: value["name"].as_str().map(str::to_owned),
            layer: value["layer"].as_i64(),
            elements: value["elements"]
                .as_array()
                .map(|els| els.iter().map(Element::from_value).collect())
                .unwrap_or_default(),
            value: value.clone(),
        }
    }
}

impl Element {
    fn from_value(value: &Value) -> Self {
        Element {
            id: value["id"].as_str().map(str::to_owned),
            kind: value["type"].as_str().map(str::to_owned),
            group: value["group"].as_str().map(str::to_owned),
            start: value["start"].as_i64(),
            end: value["end"].as_i64(),
            value: value.clone(),
        }
    }
}
