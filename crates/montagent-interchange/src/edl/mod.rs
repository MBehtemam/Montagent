//! CMX 3600 EDL sequence translator under ADR-0211.

use std::fmt;

use montagent_core::model::{
    Audio, Body, Colour, Element, Fit, Frame, FrameRate, Project, Track, Video,
};

use crate::timecode::{format_smpte_timecode, parse_smpte_timecode};

#[derive(Debug)]
pub enum EdlError {
    InvalidFormat(String),
    Timecode(String),
}

impl fmt::Display for EdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdlError::InvalidFormat(s) => write!(f, "invalid EDL line: {s}"),
            EdlError::Timecode(s) => write!(f, "EDL timecode error: {s}"),
        }
    }
}

impl std::error::Error for EdlError {}

/// A parsed CMX 3600 EDL event record.
#[derive(Debug, Clone)]
pub struct EdlEvent {
    pub event_number: usize,
    pub reel: String,
    pub channel: String,
    pub trans_type: String,
    pub trans_duration: Option<usize>,
    pub src_in: i64,
    pub src_out: i64,
    pub rec_in: i64,
    pub rec_out: i64,
    pub clip_name: Option<String>,
}

/// Ingest a CMX 3600 EDL document into a Montagent `Project`.
pub fn edl_to_project(edl_text: &str, fps: FrameRate) -> Result<Project, EdlError> {
    let mut events: Vec<EdlEvent> = Vec::new();

    for line in edl_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for clip name comments
        if trimmed.starts_with('*') {
            if let Some(rest) = trimmed.strip_prefix("* FROM CLIP NAME:")
                .or_else(|| trimmed.strip_prefix("* FROM CLIP:"))
            {
                let name = rest.trim();
                if let Some(last_event) = events.last_mut() {
                    last_event.clip_name = Some(name.to_string());
                }
            }
            continue;
        }

        // Check if header
        if trimmed.starts_with("TITLE:") || trimmed.starts_with("FCM:") {
            continue;
        }

        // Parse event line
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() < 8 {
            continue;
        }

        let event_number: usize = tokens[0]
            .parse()
            .map_err(|_| EdlError::InvalidFormat(trimmed.to_string()))?;
        let reel = tokens[1].to_string();
        let channel = tokens[2].to_uppercase();
        let trans_type = tokens[3].to_uppercase();

        let (trans_duration, tc_tokens) = if tokens[4].contains(':') || tokens[4].contains(';') {
            (None, &tokens[4..])
        } else {
            let dur: usize = tokens[4].parse().unwrap_or(0);
            (Some(dur), &tokens[5..])
        };

        if tc_tokens.len() < 4 {
            continue;
        }

        let src_in = parse_smpte_timecode(tc_tokens[0], &fps)
            .map_err(|e| EdlError::Timecode(e.to_string()))?;
        let src_out = parse_smpte_timecode(tc_tokens[1], &fps)
            .map_err(|e| EdlError::Timecode(e.to_string()))?;
        let rec_in = parse_smpte_timecode(tc_tokens[2], &fps)
            .map_err(|e| EdlError::Timecode(e.to_string()))?;
        let rec_out = parse_smpte_timecode(tc_tokens[3], &fps)
            .map_err(|e| EdlError::Timecode(e.to_string()))?;

        events.push(EdlEvent {
            event_number,
            reel,
            channel,
            trans_type,
            trans_duration,
            src_in,
            src_out,
            rec_in,
            rec_out,
            clip_name: None,
        });
    }

    if events.is_empty() {
        return Ok(Project {
            frame: Frame { width: 1920, height: 1080 },
            fps,
            background: Some(Colour::from_bytes([0, 0, 0, 255])),
            duration: Some(0),
            looping: None,
            output: None,
            master: None,
            effects: None,
            fonts: None,
            font_vendor: None,
            tracks: Vec::new(),
        });
    }

    // Determine timeline offset (standard CMX 3600 EDLs start at 01:00:00:00 = 1 hour = 3,600,000ms)
    let min_rec_in = events.iter().map(|e| e.rec_in).min().unwrap_or(0);
    let timeline_offset = if min_rec_in >= 3_600_000 {
        min_rec_in
    } else {
        0
    };

    let mut video_elements = Vec::new();
    let mut audio_elements = Vec::new();

    for ev in &events {
        let start = ev.rec_in.saturating_sub(timeline_offset);
        let end = ev.rec_out.saturating_sub(timeline_offset);
        let source_path = ev
            .clip_name
            .clone()
            .unwrap_or_else(|| format!("media/{}_{}.mp4", ev.reel, ev.event_number));

        let is_video = ev.channel.contains('V') || ev.channel == "B";
        let is_audio = ev.channel.contains('A') || ev.channel == "B";

        if is_video {
            video_elements.push(Element {
                id: format!("video_{:03}", ev.event_number),
                group: None,
                start,
                end,
                layer: None,
                body: Body::Video(Video {
                    source: source_path.clone(),
                    source_start: Some(ev.src_in),
                    source_end: Some(ev.src_out),
                    source_time: None,
                    x: None,
                    y: None,
                    origin: None,
                    width: 1920,
                    height: 1080,
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
                    speed: None,
                    overrun: None,
                    volume: None,
                    effects: None,
                    audio_effects: None,
                }),
            });
        }

        if is_audio {
            audio_elements.push(Element {
                id: format!("audio_{:03}", ev.event_number),
                group: None,
                start,
                end,
                layer: None,
                body: Body::Audio(Audio {
                    source: source_path,
                    source_start: ev.src_in,
                    source_end: ev.src_out,
                    role: None,
                    speed: None,
                    overrun: None,
                    volume: None,
                    audio_effects: None,
                }),
            });
        }
    }

    let mut tracks = Vec::new();
    if !video_elements.is_empty() {
        video_elements.sort_by_key(|e| e.start);
        tracks.push(Track {
            name: "video".to_string(),
            layer: 1,
            role: None,
            elements: video_elements,
        });
    }

    if !audio_elements.is_empty() {
        audio_elements.sort_by_key(|e| e.start);
        tracks.push(Track {
            name: "audio_1".to_string(),
            layer: -1,
            role: None,
            elements: audio_elements,
        });
    }

    let duration = tracks
        .iter()
        .flat_map(|t| t.elements.iter().map(|e| e.end))
        .max()
        .unwrap_or(0);

    Ok(Project {
        frame: Frame { width: 1920, height: 1080 },
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

/// Export a Montagent `Project` to CMX 3600 EDL string.
pub fn project_to_edl(project: &Project, title: &str) -> Result<String, EdlError> {
    let mut out = String::new();
    out.push_str(&format!("TITLE: {}\n", title));
    out.push_str("FCM: NON-DROP FRAME\n\n");

    let mut event_idx = 1;

    for track in &project.tracks {
        let is_audio = track.elements.iter().all(|e| matches!(e.body, Body::Audio(_)));
        let channel = if is_audio { "A    " } else { "V    " };

        let mut sorted = track.elements.clone();
        sorted.sort_by_key(|e| e.start);

        for el in sorted {
            let (source, src_start, src_end) = match &el.body {
                Body::Video(v) => (
                    v.source.clone(),
                    v.source_start.unwrap_or(0),
                    v.source_end.unwrap_or(el.end - el.start),
                ),
                Body::Audio(a) => (a.source.clone(), a.source_start, a.source_end),
                _ => continue,
            };

            let src_in_tc = format_smpte_timecode(src_start, &project.fps, false);
            let src_out_tc = format_smpte_timecode(src_end, &project.fps, false);
            let rec_in_tc = format_smpte_timecode(el.start, &project.fps, false);
            let rec_out_tc = format_smpte_timecode(el.end, &project.fps, false);

            out.push_str(&format!(
                "{:03}  AX       {} C        {} {} {} {}\n",
                event_idx, channel, src_in_tc, src_out_tc, rec_in_tc, rec_out_tc
            ));
            out.push_str(&format!("* FROM CLIP NAME: {}\n\n", source));
            event_idx += 1;
        }
    }

    Ok(out)
}
