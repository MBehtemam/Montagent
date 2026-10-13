//! Format interchange engine for Montagent (ADR-0209).
//!
//! Provides decoupled, zero-renderer format conversion between Montagent project documents
//! and external industry formats:
//! - SubRip (`.srt`) and WebVTT (`.vtt`) subtitles (ADR-0212).
//! - OpenTimelineIO (`.otio`) timelines (ADR-0210).
//! - CMX 3600 EDL and Final Cut Pro XML (`.fcpxml`) sequences (ADR-0211).

pub mod subtitles;

pub use subtitles::{
    SubtitleCue, SubtitleError, SubtitleFormat, export_subtitles, import_subtitles, parse_srt,
    parse_vtt, serialize_srt, serialize_vtt,
};
