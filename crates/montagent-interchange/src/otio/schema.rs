//! OpenTimelineIO schema definitions and serialization structures.

use serde::{Deserialize, Serialize};

fn default_timeline_schema() -> String {
    "Timeline.1".to_string()
}
fn default_stack_schema() -> String {
    "Stack.1".to_string()
}
fn default_track_schema() -> String {
    "Track.1".to_string()
}
fn default_clip_schema() -> String {
    "Clip.2".to_string()
}
fn default_gap_schema() -> String {
    "Gap.1".to_string()
}
fn default_transition_schema() -> String {
    "Transition.1".to_string()
}
fn default_timerange_schema() -> String {
    "TimeRange.1".to_string()
}
fn default_rational_time_schema() -> String {
    "RationalTime.1".to_string()
}
fn default_media_ref_schema() -> String {
    "ExternalReference.1".to_string()
}

/// A rational time point or duration in OTIO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioRationalTime {
    #[serde(rename = "OTIO_SCHEMA", default = "default_rational_time_schema")]
    pub otio_schema: String,
    pub value: f64,
    pub rate: f64,
}

impl OtioRationalTime {
    pub fn new(value: f64, rate: f64) -> Self {
        Self {
            otio_schema: default_rational_time_schema(),
            value,
            rate,
        }
    }

    /// Converts to integer milliseconds using half-up rounding.
    pub fn to_ms(&self) -> i64 {
        if self.rate == 0.0 {
            0
        } else {
            ((self.value * 1000.0) / self.rate).round() as i64
        }
    }

    /// Creates an `OtioRationalTime` from milliseconds and a frame rate.
    pub fn from_ms(ms: i64, rate: f64) -> Self {
        let value = (ms as f64 * rate) / 1000.0;
        Self::new(value, rate)
    }
}

/// A half-open time range with a start time and duration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioTimeRange {
    #[serde(rename = "OTIO_SCHEMA", default = "default_timerange_schema")]
    pub otio_schema: String,
    pub start_time: OtioRationalTime,
    pub duration: OtioRationalTime,
}

impl OtioTimeRange {
    pub fn new(start_time: OtioRationalTime, duration: OtioRationalTime) -> Self {
        Self {
            otio_schema: default_timerange_schema(),
            start_time,
            duration,
        }
    }
}

/// An external asset media reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioMediaReference {
    #[serde(rename = "OTIO_SCHEMA", default = "default_media_ref_schema")]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub target_url: Option<String>,
    #[serde(default)]
    pub available_range: Option<OtioTimeRange>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl OtioMediaReference {
    pub fn from_url(url: &str) -> Self {
        Self {
            otio_schema: default_media_ref_schema(),
            name: String::new(),
            target_url: Some(url.to_string()),
            available_range: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// An effect modifying a clip (such as `LinearTimeWarp`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioEffect {
    #[serde(rename = "OTIO_SCHEMA", default)]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub effect_name: Option<String>,
    #[serde(default)]
    pub time_scalar: Option<f64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// A media clip child item in an OTIO track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioClip {
    #[serde(rename = "OTIO_SCHEMA", default = "default_clip_schema")]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub source_range: Option<OtioTimeRange>,
    #[serde(default)]
    pub media_reference: Option<OtioMediaReference>,
    #[serde(default)]
    pub effects: Vec<OtioEffect>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// A gap child item representing empty space between clips.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioGap {
    #[serde(rename = "OTIO_SCHEMA", default = "default_gap_schema")]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub source_range: Option<OtioTimeRange>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// A transition between adjacent items on an OTIO track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioTransition {
    #[serde(rename = "OTIO_SCHEMA", default = "default_transition_schema")]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub transition_type: Option<String>,
    #[serde(default)]
    pub in_offset: Option<OtioRationalTime>,
    #[serde(default)]
    pub out_offset: Option<OtioRationalTime>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// A child item in an OTIO track.
#[derive(Debug, Clone, PartialEq)]
pub enum OtioChild {
    Clip(Box<OtioClip>),
    Gap(OtioGap),
    Transition(OtioTransition),
    Unknown(serde_json::Value),
}

impl<'de> Deserialize<'de> for OtioChild {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = serde_json::Value::deserialize(deserializer)?;
        let schema = val
            .get("OTIO_SCHEMA")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if schema.starts_with("Clip") {
            serde_json::from_value(val)
                .map(|c| OtioChild::Clip(Box::new(c)))
                .map_err(serde::de::Error::custom)
        } else if schema.starts_with("Gap") {
            serde_json::from_value(val)
                .map(OtioChild::Gap)
                .map_err(serde::de::Error::custom)
        } else if schema.starts_with("Transition") {
            serde_json::from_value(val)
                .map(OtioChild::Transition)
                .map_err(serde::de::Error::custom)
        } else {
            Ok(OtioChild::Unknown(val))
        }
    }
}

impl Serialize for OtioChild {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            OtioChild::Clip(c) => c.serialize(serializer),
            OtioChild::Gap(g) => g.serialize(serializer),
            OtioChild::Transition(t) => t.serialize(serializer),
            OtioChild::Unknown(v) => v.serialize(serializer),
        }
    }
}

/// A sequential track in OTIO containing clips and gaps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioTrack {
    #[serde(rename = "OTIO_SCHEMA", default = "default_track_schema")]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_track_kind")]
    pub kind: String,
    #[serde(default)]
    pub children: Vec<OtioChild>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_track_kind() -> String {
    "Video".to_string()
}

/// A concurrent stack of tracks in OTIO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioStack {
    #[serde(rename = "OTIO_SCHEMA", default = "default_stack_schema")]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub children: Vec<OtioTrack>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Root OTIO timeline document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioTimeline {
    #[serde(rename = "OTIO_SCHEMA", default = "default_timeline_schema")]
    pub otio_schema: String,
    #[serde(default)]
    pub name: String,
    pub tracks: OtioStack,
    #[serde(default)]
    pub global_start_time: Option<OtioRationalTime>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}
