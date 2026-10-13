//! Standalone subtitle parsing and conversion (SRT, WebVTT) under ADR-0212.

use std::fmt;

use montagent_core::model::{
    Align, Animatable, Body, Colour, Element, Frame, FrameRate, Length, Paint, Project, Run,
    TextElement, Track,
};

/// Format of a subtitle file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleFormat {
    Srt,
    Vtt,
}

/// An error encountered while parsing or serializing subtitles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubtitleError {
    EmptyInput,
    InvalidTimestamp(String),
    MissingTimestamp(usize),
    InvalidFormat(String),
}

impl fmt::Display for SubtitleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubtitleError::EmptyInput => write!(f, "subtitle input is empty"),
            SubtitleError::InvalidTimestamp(s) => write!(f, "invalid subtitle timestamp: {s}"),
            SubtitleError::MissingTimestamp(line) => write!(f, "missing timestamp at line {line}"),
            SubtitleError::InvalidFormat(msg) => write!(f, "invalid subtitle format: {msg}"),
        }
    }
}

impl std::error::Error for SubtitleError {}

/// A single timed subtitle cue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleCue {
    /// 1-based index or cue identifier if present.
    pub index: Option<usize>,
    /// Start time in integer milliseconds.
    pub start_ms: i64,
    /// End time in integer milliseconds.
    pub end_ms: i64,
    /// Cleaned plain-text caption string.
    pub text: String,
}

/// Parse an SRT timestamp (`HH:MM:SS,mmm`) into integer milliseconds.
pub fn parse_srt_timestamp(s: &str) -> Result<i64, SubtitleError> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(SubtitleError::InvalidTimestamp(s.to_string()));
    }
    let hms: Vec<&str> = parts[0].split(':').collect();
    if hms.len() != 3 {
        return Err(SubtitleError::InvalidTimestamp(s.to_string()));
    }
    let h: i64 = hms[0]
        .parse()
        .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
    let m: i64 = hms[1]
        .parse()
        .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
    let sec: i64 = hms[2]
        .parse()
        .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
    let millis: i64 = parts[1]
        .parse()
        .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
    Ok((h * 3600 + m * 60 + sec) * 1000 + millis)
}

/// Format integer milliseconds to an SRT timestamp (`HH:MM:SS,mmm`).
pub fn format_srt_timestamp(ms: i64) -> String {
    let ms = ms.max(0);
    let total_seconds = ms / 1000;
    let millis = ms % 1000;
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    format!("{h:02}:{m:02}:{s:02},{millis:03}")
}

/// Parse a WebVTT timestamp (`HH:MM:SS.mmm` or `MM:SS.mmm`) into integer milliseconds.
pub fn parse_vtt_timestamp(s: &str) -> Result<i64, SubtitleError> {
    let s = s.trim();
    // Drop any trailing cue settings (e.g. "line:80% position:50%")
    let token = s.split_whitespace().next().unwrap_or("");
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return Err(SubtitleError::InvalidTimestamp(s.to_string()));
    }
    let millis: i64 = parts[1]
        .parse()
        .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
    let colons: Vec<&str> = parts[0].split(':').collect();
    let total_seconds = match colons.len() {
        2 => {
            let m: i64 = colons[0]
                .parse()
                .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
            let sec: i64 = colons[1]
                .parse()
                .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
            m * 60 + sec
        }
        3 => {
            let h: i64 = colons[0]
                .parse()
                .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
            let m: i64 = colons[1]
                .parse()
                .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
            let sec: i64 = colons[2]
                .parse()
                .map_err(|_| SubtitleError::InvalidTimestamp(s.to_string()))?;
            h * 3600 + m * 60 + sec
        }
        _ => return Err(SubtitleError::InvalidTimestamp(s.to_string())),
    };
    Ok(total_seconds * 1000 + millis)
}

/// Format integer milliseconds to a WebVTT timestamp (`HH:MM:SS.mmm`).
pub fn format_vtt_timestamp(ms: i64) -> String {
    let ms = ms.max(0);
    let total_seconds = ms / 1000;
    let millis = ms % 1000;
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

/// Strip HTML/VTT markup tags (e.g. `<b>`, `<i>`, `<c.color>`, `<v Speaker>`) from subtitle text.
pub fn strip_markup(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

/// Parse a SubRip (`.srt`) string into ordered cues.
pub fn parse_srt(input: &str) -> Result<Vec<SubtitleCue>, SubtitleError> {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut cues = Vec::new();
    let blocks = trimmed.split("\n\n");

    for block in blocks {
        let lines: Vec<&str> = block
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if lines.is_empty() {
            continue;
        }

        let (index, time_line_idx) = if lines[0].contains("-->") {
            (None, 0)
        } else {
            let idx = lines[0].parse::<usize>().ok();
            (idx, 1)
        };

        if time_line_idx >= lines.len() {
            return Err(SubtitleError::MissingTimestamp(time_line_idx));
        }

        let time_line = lines[time_line_idx];
        let arrow_parts: Vec<&str> = time_line.split("-->").collect();
        if arrow_parts.len() != 2 {
            return Err(SubtitleError::InvalidTimestamp(time_line.to_string()));
        }

        let start_ms = parse_srt_timestamp(arrow_parts[0])?;
        let end_ms = parse_srt_timestamp(arrow_parts[1])?;

        let text_lines: Vec<&str> = lines[time_line_idx + 1..].to_vec();
        let raw_text = text_lines.join("\n");
        let clean_text = strip_markup(&raw_text).trim().to_string();

        if !clean_text.is_empty() {
            cues.push(SubtitleCue {
                index,
                start_ms,
                end_ms,
                text: clean_text,
            });
        }
    }

    cues.sort_by_key(|c| c.start_ms);
    Ok(cues)
}

/// Parse a WebVTT (`.vtt`) string into ordered cues.
pub fn parse_vtt(input: &str) -> Result<Vec<SubtitleCue>, SubtitleError> {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if !trimmed.starts_with("WEBVTT") {
        return Err(SubtitleError::InvalidFormat(
            "file must start with WEBVTT header".to_string(),
        ));
    }

    let mut cues = Vec::new();
    let blocks = trimmed.split("\n\n");

    for (i, block) in blocks.enumerate() {
        // Skip header block
        if i == 0 && block.contains("WEBVTT") {
            continue;
        }

        let lines: Vec<&str> = block
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if lines.is_empty() {
            continue;
        }

        // Skip NOTE or STYLE blocks
        if lines[0].starts_with("NOTE") || lines[0].starts_with("STYLE") {
            continue;
        }

        let (index, time_line_idx) = if lines[0].contains("-->") {
            (None, 0)
        } else {
            let idx = lines[0].parse::<usize>().ok();
            (idx, 1)
        };

        if time_line_idx >= lines.len() {
            continue;
        }

        let time_line = lines[time_line_idx];
        let arrow_parts: Vec<&str> = time_line.split("-->").collect();
        if arrow_parts.len() != 2 {
            continue;
        }

        let start_ms = parse_vtt_timestamp(arrow_parts[0])?;
        let end_ms = parse_vtt_timestamp(arrow_parts[1])?;

        let text_lines: Vec<&str> = lines[time_line_idx + 1..].to_vec();
        let raw_text = text_lines.join("\n");
        let clean_text = strip_markup(&raw_text).trim().to_string();

        if !clean_text.is_empty() {
            cues.push(SubtitleCue {
                index,
                start_ms,
                end_ms,
                text: clean_text,
            });
        }
    }

    cues.sort_by_key(|c| c.start_ms);
    Ok(cues)
}

/// Serialize ordered cues to a SubRip (`.srt`) formatted string.
pub fn serialize_srt(cues: &[SubtitleCue]) -> String {
    let mut out = String::new();
    for (i, cue) in cues.iter().enumerate() {
        let index = i + 1;
        let start = format_srt_timestamp(cue.start_ms);
        let end = format_srt_timestamp(cue.end_ms);
        out.push_str(&format!("{index}\n{start} --> {end}\n{}\n\n", cue.text));
    }
    out
}

/// Serialize ordered cues to a WebVTT (`.vtt`) formatted string.
pub fn serialize_vtt(cues: &[SubtitleCue]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for (i, cue) in cues.iter().enumerate() {
        let index = i + 1;
        let start = format_vtt_timestamp(cue.start_ms);
        let end = format_vtt_timestamp(cue.end_ms);
        out.push_str(&format!("{index}\n{start} --> {end}\n{}\n\n", cue.text));
    }
    out
}

/// Convert a slice of subtitle cues into a Montagent `Track` containing `type: "text"` elements.
pub fn cues_to_track(
    cues: &[SubtitleCue],
    track_name: &str,
    layer: i64,
    frame_width: i64,
    frame_height: i64,
) -> Track {
    let mut elements = Vec::new();
    let text_height = (frame_height / 8).max(120);
    let text_y = frame_height - text_height - (frame_height / 20);

    for (i, cue) in cues.iter().enumerate() {
        let id = format!("caption_{}", i + 1);
        let text_el = TextElement {
            x: Some(Animatable::Static(0)),
            y: Some(Animatable::Static(text_y)),
            origin: None,
            width: frame_width,
            height: text_height,
            font: "default".to_string(),
            size: (frame_height / 24).max(36),
            line_height: Some(1.2),
            color: Some(Paint::Flat(Animatable::Static(Colour::from_bytes([
                255, 255, 255, 255,
            ])))),
            align: Some(Align::Center),
            runs: vec![Run {
                text: cue.text.clone(),
                font: None,
                size: None,
                color: None,
                stroke: None,
                stroke_width: None,
                dir: None,
                highlight: None,
                unit: None,
            }],
            stroke: Some(Paint::Flat(Animatable::Static(Colour::from_bytes([
                0, 0, 0, 255,
            ])))),
            stroke_width: Some(Animatable::Static(Length(3))),
            letter_spacing: None,
            units: None,
            path: None,
            path_offset: None,
            scale: None,
            rotation: None,
            swivel: None,
            tilt: None,
            perspective: None,
            opacity: None,
            blend: None,
            motion_blur: None,
            effects: None,
            caption: Some(true),
        };

        elements.push(Element {
            id,
            group: None,
            start: cue.start_ms,
            end: cue.end_ms,
            layer: None,
            body: Body::Text(text_el),
        });
    }

    Track {
        name: track_name.to_string(),
        layer,
        role: None,
        elements,
    }
}

/// Extract subtitle cues from a Montagent `Track` or `Project`.
pub fn cues_from_track(track: &Track) -> Vec<SubtitleCue> {
    let mut cues = Vec::new();
    for (i, el) in track.elements.iter().enumerate() {
        if let Body::Text(t) = &el.body {
            let full_text: String = t
                .runs
                .iter()
                .map(|r| r.text.as_str())
                .collect::<Vec<_>>()
                .join("");
            cues.push(SubtitleCue {
                index: Some(i + 1),
                start_ms: el.start,
                end_ms: el.end,
                text: full_text,
            });
        }
    }
    cues.sort_by_key(|c| c.start_ms);
    cues
}

/// Extract all subtitle cues from a Montagent `Project` by inspecting caption tracks or text elements.
pub fn cues_from_project(project: &Project) -> Vec<SubtitleCue> {
    // Prefer track named "captions" or "subtitles", otherwise scan all tracks for caption-flagged text elements
    let named_track = project
        .tracks
        .iter()
        .find(|t| t.name.to_lowercase() == "captions" || t.name.to_lowercase() == "subtitles");

    if let Some(t) = named_track {
        cues_from_track(t)
    } else {
        let mut cues = Vec::new();
        let mut idx = 1;
        for track in &project.tracks {
            for el in &track.elements {
                if let Body::Text(t) = &el.body
                    && t.caption.unwrap_or(true)
                {
                    let full_text: String = t
                        .runs
                        .iter()
                        .map(|r| r.text.as_str())
                        .collect::<Vec<_>>()
                        .join("");
                    cues.push(SubtitleCue {
                        index: Some(idx),
                        start_ms: el.start,
                        end_ms: el.end,
                        text: full_text,
                    });
                    idx += 1;
                }
            }
        }
        cues.sort_by_key(|c| c.start_ms);
        cues
    }
}

/// Import a subtitle string (`.srt` or `.vtt`) into a new standalone Montagent `Project`.
pub fn import_subtitles(
    input: &str,
    format: SubtitleFormat,
    width: i64,
    height: i64,
    fps: FrameRate,
) -> Result<Project, SubtitleError> {
    let cues = match format {
        SubtitleFormat::Srt => parse_srt(input)?,
        SubtitleFormat::Vtt => parse_vtt(input)?,
    };

    let total_duration = cues.iter().map(|c| c.end_ms).max().unwrap_or(0);
    let track = cues_to_track(&cues, "captions", 1, width, height);

    Ok(Project {
        frame: Frame { width, height },
        fps,
        background: Some(Colour::from_bytes([0, 0, 0, 255])),
        duration: Some(total_duration),
        looping: None,
        output: None,
        master: None,
        effects: None,
        fonts: None,
        font_vendor: None,
        tracks: vec![track],
    })
}

/// Export subtitles from a Montagent `Project` to formatted string (`.srt` or `.vtt`).
pub fn export_subtitles(
    project: &Project,
    format: SubtitleFormat,
) -> Result<String, SubtitleError> {
    let cues = cues_from_project(project);
    match format {
        SubtitleFormat::Srt => Ok(serialize_srt(&cues)),
        SubtitleFormat::Vtt => Ok(serialize_vtt(&cues)),
    }
}
