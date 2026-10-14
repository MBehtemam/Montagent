//! SMPTE Timecode calculation and formatting under ADR-0211 and ADR-0199.

use montagent_core::model::FrameRate;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum TimecodeError {
    InvalidFormat(String),
    InvalidComponent(String),
}

impl fmt::Display for TimecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimecodeError::InvalidFormat(s) => write!(f, "invalid SMPTE timecode format: '{s}'"),
            TimecodeError::InvalidComponent(s) => {
                write!(f, "invalid SMPTE timecode component: '{s}'")
            }
        }
    }
}

impl std::error::Error for TimecodeError {}

/// Parse a SMPTE timecode string (`HH:MM:SS:FF` or `HH:MM:SS;FF`) into integer milliseconds.
pub fn parse_smpte_timecode(tc: &str, fps: &FrameRate) -> Result<i64, TimecodeError> {
    let tc = tc.trim();
    let is_drop_frame = tc.contains(';');

    let parts: Vec<&str> = tc.split([':', ';']).collect();
    if parts.len() != 4 {
        return Err(TimecodeError::InvalidFormat(tc.to_string()));
    }

    let hh: i64 = parts[0]
        .parse()
        .map_err(|_| TimecodeError::InvalidComponent(parts[0].to_string()))?;
    let mm: i64 = parts[1]
        .parse()
        .map_err(|_| TimecodeError::InvalidComponent(parts[1].to_string()))?;
    let ss: i64 = parts[2]
        .parse()
        .map_err(|_| TimecodeError::InvalidComponent(parts[2].to_string()))?;
    let ff: i64 = parts[3]
        .parse()
        .map_err(|_| TimecodeError::InvalidComponent(parts[3].to_string()))?;

    let fps_nominal = (fps.num() as f64 / fps.den() as f64).round() as i64;
    let fps_nominal = fps_nominal.max(1);

    let total_frames = if is_drop_frame {
        let total_minutes = hh * 60 + mm;
        let drop_frames = 2 * (total_minutes - (total_minutes / 10));
        ((hh * 60 + mm) * 60 + ss) * fps_nominal + ff - drop_frames
    } else {
        ((hh * 60 + mm) * 60 + ss) * fps_nominal + ff
    };

    let ms = ((total_frames as f64 * 1000.0 * fps.den() as f64) / fps.num() as f64).round() as i64;
    Ok(ms)
}

/// Format milliseconds into a SMPTE timecode string (`HH:MM:SS:FF` or `HH:MM:SS;FF`).
pub fn format_smpte_timecode(ms: i64, fps: &FrameRate, drop_frame: bool) -> String {
    let fps_nominal = (fps.num() as f64 / fps.den() as f64).round() as i64;
    let fps_nominal = fps_nominal.max(1);
    let total_frames =
        ((ms as f64 * fps.num() as f64) / (1000.0 * fps.den() as f64)).round() as i64;

    if drop_frame && fps_nominal == 30 {
        let ten_min_blocks = total_frames / 17982;
        let rem = total_frames % 17982;
        let extra_min = if rem > 2 { (rem - 2) / 1798 } else { 0 };
        let total_minutes = ten_min_blocks * 10 + extra_min;
        let frame_in_min = (total_frames + 2 * (total_minutes - (total_minutes / 10))) % 1800;
        let ss = frame_in_min / 30;
        let ff = frame_in_min % 30;
        let mm = total_minutes % 60;
        let hh = total_minutes / 60;
        format!("{:02}:{:02}:{:02};{:02}", hh, mm, ss, ff)
    } else {
        let ff = total_frames % fps_nominal;
        let total_seconds = total_frames / fps_nominal;
        let ss = total_seconds % 60;
        let total_minutes = total_seconds / 60;
        let mm = total_minutes % 60;
        let hh = total_minutes / 60;
        format!("{:02}:{:02}:{:02}:{:02}", hh, mm, ss, ff)
    }
}
