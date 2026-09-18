// scene.json -> structs. Shared by both backends. Throwaway.
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Span {
    pub image: String,
    pub start: f64,
    pub end: f64,
    #[serde(rename = "cropW")]
    pub crop_w: f64,
    #[serde(rename = "cropH")]
    pub crop_h: f64,
    #[serde(rename = "zoomFrom")]
    pub zoom_from: f64,
    #[serde(rename = "zoomTo")]
    pub zoom_to: f64,
    #[serde(rename = "zoomStep")]
    pub zoom_step: f64,
    /// #159: optional destination rect, so more than one Ken Burns still can
    /// be simultaneously live (e.g. a PiP inset) instead of always filling
    /// the whole card. Absent => the original full-card placement.
    #[serde(default)]
    pub dx: Option<f64>,
    #[serde(default)]
    pub dy: Option<f64>,
    #[serde(default)]
    pub dw: Option<f64>,
    #[serde(default)]
    pub dh: Option<f64>,
}

#[derive(Deserialize, Clone)]
pub struct Badge {
    pub src: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// A real video clip on the timeline. Absent from #6's scene; the whole point of #34.
#[derive(Deserialize, Clone)]
pub struct Clip {
    pub src: String,
    /// timeline seconds
    pub start: f64,
    pub end: f64,
    /// seconds into the source file that `start` maps to
    #[serde(rename = "srcStart")]
    pub src_start: f64,
    pub dx: f64,
    pub dy: f64,
    pub dw: f64,
    pub dh: f64,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum Event {
    #[serde(rename = "rect")]
    Rect {
        layer: i32,
        start: f64,
        end: f64,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        colour: String,
    },
    #[serde(rename = "text")]
    Text {
        layer: i32,
        start: f64,
        end: f64,
        x: f64,
        y: f64,
        align: u8,
        colour: String,
        size: f64,
        font: String,
        #[allow(dead_code)]
        bold: bool,
        lines: Vec<String>,
    },
}

impl Event {
    pub fn layer(&self) -> i32 {
        match self {
            Event::Rect { layer, .. } | Event::Text { layer, .. } => *layer,
        }
    }
    pub fn window(&self) -> (f64, f64) {
        match self {
            Event::Rect { start, end, .. } | Event::Text { start, end, .. } => (*start, *end),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Audio {
    pub src: String,
    pub at: f64,
}

#[derive(Deserialize, Clone)]
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
    #[serde(rename = "cardH")]
    pub card_h: f64,
    pub background: String,
    pub spans: Vec<Span>,
    pub badge: Badge,
    pub events: Vec<Event>,
    pub audio: Vec<Audio>,
    #[serde(default)]
    pub clips: Vec<Clip>,
}

pub fn colour(hex: &str) -> [u8; 4] {
    let h = hex.trim_start_matches('#');
    let v = u32::from_str_radix(h, 16).unwrap_or(0);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8, 255]
}
