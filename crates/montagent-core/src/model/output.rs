//! Output deliverable model and container inference (ADR-0202, ADR-0203, ADR-0204).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The deliverable output definition.
///
/// Untagged union under ADR-0202:
/// - A shorthand relative path string (`"out/video.mp4"`), maintaining 100% backward compatibility.
/// - A structured `OutputConfig` object declaring container, video, audio, and stems configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Output {
    Path(String),
    Config(OutputConfig),
}

impl Output {
    pub fn path(&self) -> &str {
        match self {
            Output::Path(p) => p.as_str(),
            Output::Config(c) => c.path.as_str(),
        }
    }

    pub fn container(&self) -> Option<&str> {
        match self {
            Output::Path(p) => infer_container(p),
            Output::Config(c) => c.container.as_deref().or_else(|| infer_container(&c.path)),
        }
    }

    pub fn video(&self) -> Option<&VideoExportConfig> {
        match self {
            Output::Path(_) => None,
            Output::Config(c) => c.video.as_ref(),
        }
    }

    pub fn audio(&self) -> Option<&AudioExportConfig> {
        match self {
            Output::Path(_) => None,
            Output::Config(c) => c.audio.as_ref(),
        }
    }

    pub fn stems(&self) -> &[StemExportConfig] {
        match self {
            Output::Path(_) => &[],
            Output::Config(c) => c.stems.as_deref().unwrap_or(&[]),
        }
    }
}

/// Infers the container format from a destination file extension.
pub fn infer_container(path: &str) -> Option<&'static str> {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".mp4") {
        Some("mp4")
    } else if lower.ends_with(".mov") {
        Some("mov")
    } else if lower.ends_with(".mxf") {
        Some("mxf")
    } else {
        None
    }
}

/// Detailed deliverable output configuration (ADR-0202).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    /// Relative filesystem destination for the deliverable.
    pub path: String,
    /// Container format override (`"mp4"`, `"mov"`, `"mxf"`). Inferred from path extension if omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    /// Video encoding configuration (codec, profile, crf, alpha).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoExportConfig>,
    /// Audio encoding configuration (codec, sample rate, bitrate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioExportConfig>,
    /// Auxiliary audio stems to export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stems: Option<Vec<StemExportConfig>>,
}

/// Video encoding parameters (ADR-0202, ADR-0203).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VideoExportConfig {
    /// Video codec (`"h264"`, `"prores"`, `"dnxhr"`). Default `"h264"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    /// Codec profile (e.g. `"standard"`, `"4444"` for ProRes; `"dnxhr_hq"` for DNxHR).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Constant Rate Factor for H.264 (default 20).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crf: Option<u8>,
    /// Whether to preserve the alpha channel transparency (requires ProRes 4444).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha: Option<bool>,
}

/// Audio encoding parameters (ADR-0202, ADR-0204).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioExportConfig {
    /// Audio codec (`"aac"`, `"pcm"`). Default `"aac"` for MP4, `"pcm"` for MOV/MXF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    /// Sample rate in Hz (default 48000).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    /// Target bitrate for compressed audio (e.g. `"160k"` for AAC).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<String>,
}

/// Auxiliary audio stem deliverable (ADR-0204).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StemExportConfig {
    /// Semantic audio role to isolate (`"dialogue"`, `"voiceover"`, `"music"`, `"sfx"`, `"foley"`).
    pub role: String,
    /// Destination path ending in `.wav`.
    pub path: String,
}
