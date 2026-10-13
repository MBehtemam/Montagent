//! OpenTimelineIO (OTIO) bidirectional timeline mapping under ADR-0210.

pub mod convert;
pub mod schema;

pub use convert::{
    OtioError, frame_rate_to_rate, otio_to_project, project_to_otio, rate_to_frame_rate,
};
pub use schema::{
    OtioChild, OtioClip, OtioEffect, OtioGap, OtioMediaReference, OtioRationalTime, OtioStack,
    OtioTimeRange, OtioTimeline, OtioTrack,
};
