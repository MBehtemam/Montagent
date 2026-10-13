//! Conversion between OpenTimelineIO documents and Montagent Project models under ADR-0210.

use std::fmt;

use montagent_core::model::{
    Audio, Body, Colour, Element, Fit, Frame, FrameRate, Project, Speed, Track, Video,
};

use super::schema::{
    OtioChild, OtioClip, OtioEffect, OtioGap, OtioMediaReference, OtioRationalTime, OtioStack,
    OtioTimeRange, OtioTimeline, OtioTrack,
};

/// Errors encountered during OTIO conversion.
#[derive(Debug)]
pub enum OtioError {
    Json(serde_json::Error),
    MissingTracks,
    InvalidRate(f64),
    InvalidClip(String),
}

impl fmt::Display for OtioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OtioError::Json(e) => write!(f, "OTIO JSON error: {e}"),
            OtioError::MissingTracks => write!(f, "OTIO timeline has no tracks stack"),
            OtioError::InvalidRate(r) => write!(f, "invalid OTIO frame rate: {r}"),
            OtioError::InvalidClip(msg) => write!(f, "invalid OTIO clip: {msg}"),
        }
    }
}

impl std::error::Error for OtioError {}

impl From<serde_json::Error> for OtioError {
    fn from(err: serde_json::Error) -> Self {
        OtioError::Json(err)
    }
}

/// Convert an OTIO float rate to Montagent's exact `FrameRate` representation (ADR-0199).
pub fn rate_to_frame_rate(rate: f64) -> FrameRate {
    if (rate - 23.976023976).abs() < 0.01 || (rate - 23.976).abs() < 0.01 || (rate - 23.98).abs() < 0.01 {
        FrameRate::new(24000, 1001)
    } else if (rate - 29.97002997).abs() < 0.01 || (rate - 29.97).abs() < 0.01 {
        FrameRate::new(30000, 1001)
    } else if (rate - 59.94005994).abs() < 0.01 || (rate - 59.94).abs() < 0.01 {
        FrameRate::new(60000, 1001)
    } else {
        let int_rate = (rate.round() as i64).max(1);
        FrameRate::integer(int_rate)
    }
}

/// Convert a Montagent `FrameRate` to an OTIO float rate.
pub fn frame_rate_to_rate(fps: &FrameRate) -> f64 {
    fps.num() as f64 / fps.den() as f64
}

/// Normalize media reference URL (strip leading `file://` if present).
fn normalize_url(url: &str) -> String {
    if let Some(stripped) = url.strip_prefix("file://") {
        stripped.to_string()
    } else {
        url.to_string()
    }
}

/// Ingest an OpenTimelineIO JSON document into a Montagent `Project`.
pub fn otio_to_project(otio_json: &str) -> Result<Project, OtioError> {
    let timeline: OtioTimeline = serde_json::from_str(otio_json)?;

    // Determine sequence frame rate
    let rate = if let Some(gst) = &timeline.global_start_time {
        if gst.rate > 0.0 { gst.rate } else { 24.0 }
    } else {
        // Inspect first child track
        timeline
            .tracks
            .children
            .iter()
            .find_map(|t| {
                t.children.iter().find_map(|c| match c {
                    OtioChild::Clip(clip) => {
                        clip.source_range.as_ref().map(|sr| sr.duration.rate)
                    }
                    OtioChild::Gap(gap) => {
                        gap.source_range.as_ref().map(|sr| sr.duration.rate)
                    }
                    _ => None,
                })
            })
            .unwrap_or(24.0)
    };

    let fps = rate_to_frame_rate(rate);
    let frame_width = 1920;
    let frame_height = 1080;

    let mut tracks = Vec::new();

    for (layer_idx, otio_track) in timeline.tracks.children.into_iter().enumerate() {
        let is_audio = otio_track.kind.to_lowercase() == "audio";
        let mut elements = Vec::new();
        let mut cursor_ms: i64 = 0;

        for (item_idx, child) in otio_track.children.into_iter().enumerate() {
            match child {
                OtioChild::Gap(gap) => {
                    let dur_ms = gap
                        .source_range
                        .as_ref()
                        .map(|sr| sr.duration.to_ms())
                        .unwrap_or(0);
                    cursor_ms += dur_ms;
                }
                OtioChild::Clip(clip) => {
                    let dur_ms = clip
                        .source_range
                        .as_ref()
                        .map(|sr| sr.duration.to_ms())
                        .unwrap_or(0);

                    let source_start = clip
                        .source_range
                        .as_ref()
                        .map(|sr| sr.start_time.to_ms())
                        .unwrap_or(0);

                    let source_end = source_start + dur_ms;

                    let source_path = clip
                        .media_reference
                        .as_ref()
                        .and_then(|mr| mr.target_url.as_ref())
                        .map(|url| normalize_url(url))
                        .unwrap_or_else(|| format!("media/clip_{}.mp4", item_idx + 1));

                    let speed = clip.effects.iter().find_map(|e| {
                        if e.otio_schema.starts_with("LinearTimeWarp") {
                            e.time_scalar.map(Speed)
                        } else {
                            None
                        }
                    });

                    let element_id = if !clip.name.is_empty() {
                        format!("{}_{}", clip.name, item_idx + 1)
                    } else {
                        format!("clip_{}_{}", layer_idx + 1, item_idx + 1)
                    };

                    let body = if is_audio {
                        Body::Audio(Audio {
                            source: source_path,
                            source_start,
                            source_end,
                            role: None,
                            speed,
                            overrun: None,
                            volume: None,
                            audio_effects: None,
                        })
                    } else {
                        Body::Video(Video {
                            source: source_path,
                            source_start: Some(source_start),
                            source_end: Some(source_end),
                            source_time: None,
                            x: None,
                            y: None,
                            origin: None,
                            width: frame_width,
                            height: frame_height,
                            fit: Fit::Contain,
                            clip: None,
                            scale: None,
                            rotation: None,
                            swivel: None,
                            tilt: None,
                            perspective: None,
                            opacity: None,
                            blend: None,
                            motion_blur: None,
                            role: None,
                            speed,
                            overrun: None,
                            volume: None,
                            effects: None,
                            audio_effects: None,
                        })
                    };

                    elements.push(Element {
                        id: element_id,
                        group: None,
                        start: cursor_ms,
                        end: cursor_ms + dur_ms,
                        layer: None,
                        body,
                    });

                    cursor_ms += dur_ms;
                }
                OtioChild::Transition(_) | OtioChild::Unknown(_) => {
                    // Handled as pass-through
                }
            }
        }

        let track_name = if !otio_track.name.is_empty() {
            otio_track.name
        } else if is_audio {
            format!("A{}", layer_idx + 1)
        } else {
            format!("V{}", layer_idx + 1)
        };

        tracks.push(Track {
            name: track_name,
            layer: layer_idx as i64,
            role: None,
            elements,
        });
    }

    let duration = tracks
        .iter()
        .flat_map(|t| t.elements.iter().map(|e| e.end))
        .max()
        .unwrap_or(0);

    Ok(Project {
        frame: Frame {
            width: frame_width,
            height: frame_height,
        },
        fps,
        background: Some(Colour::from_bytes([0, 0, 0, 255])),
        duration: Some(duration),
        looping: None,
        output: None,
        master: None,
        effects: None,
        fonts: None,
        font_vendor: None,
        tracks,
    })
}

/// Export a Montagent `Project` to an OpenTimelineIO JSON string.
pub fn project_to_otio(project: &Project) -> Result<String, OtioError> {
    let rate = frame_rate_to_rate(&project.fps);
    let mut otio_tracks = Vec::new();

    for track in &project.tracks {
        let is_audio = track.elements.iter().all(|e| matches!(e.body, Body::Audio(_)));
        let kind = if is_audio { "Audio" } else { "Video" };

        let mut sorted_elements = track.elements.clone();
        sorted_elements.sort_by_key(|e| e.start);

        let mut children = Vec::new();
        let mut cursor_ms: i64 = 0;

        for el in sorted_elements {
            // Gap before element
            if el.start > cursor_ms {
                let gap_duration_ms = el.start - cursor_ms;
                let gap = OtioGap {
                    otio_schema: "Gap.1".to_string(),
                    name: "gap".to_string(),
                    source_range: Some(OtioTimeRange::new(
                        OtioRationalTime::from_ms(cursor_ms, rate),
                        OtioRationalTime::from_ms(gap_duration_ms, rate),
                    )),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                };
                children.push(OtioChild::Gap(gap));
            }

            let dur_ms = el.end - el.start;
            let (source, source_start, speed) = match &el.body {
                Body::Video(v) => (
                    v.source.clone(),
                    v.source_start.unwrap_or(0),
                    v.speed.as_ref().map(|s| s.0),
                ),
                Body::Audio(a) => (a.source.clone(), a.source_start, a.speed.as_ref().map(|s| s.0)),
                Body::Image(img) => (img.source.clone(), 0, None),
                _ => (String::new(), 0, None),
            };

            let mut effects = Vec::new();
            if let Some(s) = speed {
                effects.push(OtioEffect {
                    otio_schema: "LinearTimeWarp.1".to_string(),
                    name: "LinearTimeWarp".to_string(),
                    effect_name: Some("LinearTimeWarp".to_string()),
                    time_scalar: Some(s),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                });
            }

            let clip = OtioClip {
                otio_schema: "Clip.2".to_string(),
                name: el.id.clone(),
                source_range: Some(OtioTimeRange::new(
                    OtioRationalTime::from_ms(source_start, rate),
                    OtioRationalTime::from_ms(dur_ms, rate),
                )),
                media_reference: Some(OtioMediaReference::from_url(&source)),
                effects,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            };

            children.push(OtioChild::Clip(Box::new(clip)));
            cursor_ms = el.end;
        }

        otio_tracks.push(OtioTrack {
            otio_schema: "Track.1".to_string(),
            name: track.name.clone(),
            kind: kind.to_string(),
            children,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        });
    }

    let timeline = OtioTimeline {
        otio_schema: "Timeline.1".to_string(),
        name: "Montagent Timeline".to_string(),
        tracks: OtioStack {
            otio_schema: "Stack.1".to_string(),
            name: "tracks".to_string(),
            children: otio_tracks,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        },
        global_start_time: Some(OtioRationalTime::new(0.0, rate)),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    };

    Ok(serde_json::to_string_pretty(&timeline)?)
}
