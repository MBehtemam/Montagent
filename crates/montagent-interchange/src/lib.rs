//! Format interchange engine for Montagent (ADR-0209).
//!
//! Provides decoupled, zero-renderer format conversion between Montagent project documents
//! and external industry formats:
//! - SubRip (`.srt`) and WebVTT (`.vtt`) subtitles (ADR-0212).
//! - OpenTimelineIO (`.otio`) timelines (ADR-0210).
//! - CMX 3600 EDL and Final Cut Pro XML (`.fcpxml`) sequences (ADR-0211).

pub mod edl;
pub mod fcpxml;
pub mod otio;
pub mod subtitles;
pub mod timecode;

pub use edl::{EdlError, EdlEvent, edl_to_project, project_to_edl};
pub use fcpxml::{FcpxmlError, fcpxml_to_project, parse_fcpxml_time, project_to_fcpxml};
pub use otio::{
    OtioChild, OtioClip, OtioEffect, OtioError, OtioGap, OtioMediaReference, OtioRationalTime,
    OtioStack, OtioTimeRange, OtioTimeline, OtioTrack, frame_rate_to_rate, otio_to_project,
    project_to_otio, rate_to_frame_rate,
};
pub use subtitles::{
    SubtitleCue, SubtitleError, SubtitleFormat, export_subtitles, import_subtitles, parse_srt,
    parse_vtt, serialize_srt, serialize_vtt,
};
pub use timecode::{TimecodeError, format_smpte_timecode, parse_smpte_timecode};
