//! Apple Final Cut Pro XML (FCPXML) sequence translator under ADR-0211.

use std::collections::HashMap;
use std::fmt;

use montagent_core::model::{
    Audio, Body, Colour, Element, Fit, Frame, FrameRate, Project, Track, Video,
};

#[derive(Debug)]
pub enum FcpxmlError {
    InvalidXml(String),
    InvalidTime(String),
    MissingSpine,
}

impl fmt::Display for FcpxmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FcpxmlError::InvalidXml(s) => write!(f, "invalid FCPXML: {s}"),
            FcpxmlError::InvalidTime(s) => write!(f, "invalid FCPXML rational time: {s}"),
            FcpxmlError::MissingSpine => write!(f, "FCPXML sequence contains no spine"),
        }
    }
}

impl std::error::Error for FcpxmlError {}

/// Parse a rational FCPXML time string (e.g. `"1001/30000s"`, `"10s"`, `"2.5s"`, `"0s"`) into milliseconds.
pub fn parse_fcpxml_time(s: &str) -> Result<i64, FcpxmlError> {
    let s = s.trim().trim_end_matches('s');
    if s.is_empty() {
        return Ok(0);
    }
    if let Some((num_s, den_s)) = s.split_once('/') {
        let num: f64 = num_s.trim().parse().map_err(|_| FcpxmlError::InvalidTime(s.to_string()))?;
        let den: f64 = den_s.trim().parse().map_err(|_| FcpxmlError::InvalidTime(s.to_string()))?;
        if den == 0.0 {
            return Ok(0);
        }
        Ok(((num * 1000.0) / den).round() as i64)
    } else {
        let sec: f64 = s.trim().parse().map_err(|_| FcpxmlError::InvalidTime(s.to_string()))?;
        Ok((sec * 1000.0).round() as i64)
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

/// Parse FCPXML `frameDuration` string into a Montagent `FrameRate`.
pub fn parse_frame_duration(s: &str) -> FrameRate {
    let s = s.trim().trim_end_matches('s');
    let Some((num_s, den_s)) = s.split_once('/') else {
        return FrameRate::integer(24);
    };
    let Ok(num) = num_s.trim().parse::<i64>() else {
        return FrameRate::integer(24);
    };
    let Ok(den) = den_s.trim().parse::<i64>() else {
        return FrameRate::integer(24);
    };
    if num <= 0 || den <= 0 {
        return FrameRate::integer(24);
    }
    if den == 30000 && num == 1001 {
        return FrameRate::new(30000, 1001);
    }
    if den == 24000 && num == 1001 {
        return FrameRate::new(24000, 1001);
    }
    if den == 60000 && num == 1001 {
        return FrameRate::new(60000, 1001);
    }
    let g = gcd(den, num);
    FrameRate::new(den / g, num / g)
}

/// A parsed XML element node.
#[derive(Debug, Clone)]
pub struct XmlNode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<XmlNode>,
}

impl XmlNode {
    pub fn get_attr<'a>(&'a self, name: &str) -> Option<&'a str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    pub fn find_child<'a>(&'a self, tag: &str) -> Option<&'a XmlNode> {
        self.children.iter().find(|c| c.tag == tag)
    }

    pub fn find_all_children<'a>(&'a self, tag: &str) -> impl Iterator<Item = &'a XmlNode> {
        self.children.iter().filter(move |c| c.tag == tag)
    }
}

/// Parse an XML string into an `XmlNode` hierarchy.
pub fn parse_xml(xml: &str) -> Result<XmlNode, FcpxmlError> {
    let mut chars = xml.char_indices().peekable();

    fn parse_element<'a>(
        chars: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
        xml: &'a str,
    ) -> Result<Option<XmlNode>, FcpxmlError> {
        while let Some(&(_, c)) = chars.peek() {
            if c != '<' {
                chars.next();
                continue;
            }

            chars.next(); // consume '<'

            // Check if comment, doctype, or xml header
            if let Some(&(_, next_c)) = chars.peek() {
                if next_c == '?' || next_c == '!' {
                    // Skip until '>'
                    for (_, end_c) in chars.by_ref() {
                        if end_c == '>' {
                            break;
                        }
                    }
                    continue;
                }
                if next_c == '/' {
                    // Closing tag reached
                    return Ok(None);
                }
            }

            // Parse tag name
            let tag_start = match chars.peek() {
                Some(&(idx, _)) => idx,
                None => return Err(FcpxmlError::InvalidXml("unexpected end of input".to_string())),
            };

            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() || c == '/' || c == '>' {
                    break;
                }
                chars.next();
            }

            let tag_end = match chars.peek() {
                Some(&(idx, _)) => idx,
                None => xml.len(),
            };

            let tag = xml[tag_start..tag_end].to_string();

            // Parse attributes
            let mut attributes = HashMap::new();
            let mut self_closing = false;

            loop {
                // Skip whitespace
                while let Some(&(_, c)) = chars.peek() {
                    if !c.is_whitespace() {
                        break;
                    }
                    chars.next();
                }

                match chars.peek() {
                    Some(&(_, '/')) => {
                        chars.next();
                        if let Some(&(_, '>')) = chars.peek() {
                            chars.next();
                            self_closing = true;
                            break;
                        }
                    }
                    Some(&(_, '>')) => {
                        chars.next();
                        break;
                    }
                    Some(&(attr_start, c)) if c.is_alphanumeric() || c == '_' || c == '-' || c == ':' => {
                        // Scan attribute name
                        while let Some(&(_, ac)) = chars.peek() {
                            if ac == '=' || ac.is_whitespace() || ac == '>' || ac == '/' {
                                break;
                            }
                            chars.next();
                        }
                        let attr_name_end = match chars.peek() {
                            Some(&(idx, _)) => idx,
                            None => xml.len(),
                        };
                        let attr_name = xml[attr_start..attr_name_end].to_string();

                        // Expect '='
                        while let Some(&(_, eq_c)) = chars.peek() {
                            chars.next();
                            if eq_c == '=' {
                                break;
                            }
                        }

                        // Expect quote '"' or '\''
                        let quote = match chars.peek() {
                            Some(&(_, q)) if q == '"' || q == '\'' => {
                                chars.next();
                                q
                            }
                            _ => continue,
                        };

                        let val_start = match chars.peek() {
                            Some(&(idx, _)) => idx,
                            None => xml.len(),
                        };

                        while let Some(&(_, qc)) = chars.peek() {
                            if qc == quote {
                                break;
                            }
                            chars.next();
                        }

                        let val_end = match chars.peek() {
                            Some(&(idx, _)) => idx,
                            None => xml.len(),
                        };
                        chars.next(); // consume closing quote

                        let val = xml[val_start..val_end].to_string();
                        attributes.insert(attr_name, val);
                    }
                    Some(_) => {
                        chars.next();
                    }
                    None => break,
                }
            }

            if self_closing {
                return Ok(Some(XmlNode {
                    tag,
                    attributes,
                    children: Vec::new(),
                }));
            }

            // Parse children until closing tag
            let mut children = Vec::new();
            while let Some(child) = parse_element(chars, xml)? {
                children.push(child);
            }

            // Consume closing tag `</tag>`
            for (_, c) in chars.by_ref() {
                if c == '>' {
                    break;
                }
            }

            return Ok(Some(XmlNode {
                tag,
                attributes,
                children,
            }));
        }

        Ok(None)
    }

    match parse_element(&mut chars, xml)? {
        Some(node) => Ok(node),
        None => Err(FcpxmlError::InvalidXml("no root element found".to_string())),
    }
}

/// Ingest an Apple Final Cut Pro XML (FCPXML) string into a Montagent `Project`.
pub fn fcpxml_to_project(xml_text: &str) -> Result<Project, FcpxmlError> {
    let root = parse_xml(xml_text)?;

    // Scan <resources> for formats and assets
    let mut frame_width = 1920;
    let mut frame_height = 1080;
    let mut fps = FrameRate::integer(24);
    let mut asset_map: HashMap<String, String> = HashMap::new();

    if let Some(resources) = root.find_child("resources") {
        for fmt_node in resources.find_all_children("format") {
            if let Some(dur_str) = fmt_node.get_attr("frameDuration") {
                fps = parse_frame_duration(dur_str);
            }
            if let Some(w) = fmt_node.get_attr("width").and_then(|w| w.parse().ok()) {
                frame_width = w;
            }
            if let Some(h) = fmt_node.get_attr("height").and_then(|h| h.parse().ok()) {
                frame_height = h;
            }
        }

        for asset_node in resources.find_all_children("asset") {
            if let (Some(id), Some(src)) = (asset_node.get_attr("id"), asset_node.get_attr("src")) {
                let clean_src = src.strip_prefix("file://").unwrap_or(src);
                asset_map.insert(id.to_string(), clean_src.to_string());
            }
        }
    }

    // Find sequence spine
    fn find_spine(node: &XmlNode) -> Option<&XmlNode> {
        if node.tag == "spine" {
            return Some(node);
        }
        for child in &node.children {
            if let Some(spine) = find_spine(child) {
                return Some(spine);
            }
        }
        None
    }

    let spine = find_spine(&root).ok_or(FcpxmlError::MissingSpine)?;

    // Track lane maps: lane (i64) -> Vec<Element>
    let mut lane_map: HashMap<i64, Vec<Element>> = HashMap::new();
    let mut primary_cursor_ms: i64 = 0;
    let mut item_counter: usize = 1;

    for child in &spine.children {
        if child.tag == "gap" {
            let dur_ms = child
                .get_attr("duration")
                .map(parse_fcpxml_time)
                .transpose()?
                .unwrap_or(0);
            primary_cursor_ms += dur_ms;
            continue;
        }

        // Could be <clip>, <asset-clip>, <video>, <audio>
        let is_clip = matches!(child.tag.as_str(), "clip" | "asset-clip" | "video" | "audio");
        if !is_clip {
            continue;
        }

        let lane: i64 = child
            .get_attr("lane")
            .and_then(|l| l.parse().ok())
            .unwrap_or(0);

        let dur_ms = child
            .get_attr("duration")
            .map(parse_fcpxml_time)
            .transpose()?
            .unwrap_or(1000);

        let start_offset_ms = if let Some(off_str) = child.get_attr("offset") {
            parse_fcpxml_time(off_str)?
        } else if lane == 0 {
            primary_cursor_ms
        } else {
            0
        };

        let src_start_ms = child
            .get_attr("start")
            .map(parse_fcpxml_time)
            .transpose()?
            .unwrap_or(0);

        let src_end_ms = src_start_ms + dur_ms;

        // Resolve asset source
        let asset_ref = child
            .get_attr("ref")
            .or_else(|| {
                child.find_child("video").and_then(|v| v.get_attr("ref"))
                    .or_else(|| child.find_child("audio").and_then(|a| a.get_attr("ref")))
            });

        let source = asset_ref
            .and_then(|r| asset_map.get(r).cloned())
            .unwrap_or_else(|| format!("media/clip_{}.mp4", item_counter));

        let elem_id = child
            .get_attr("name")
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("clip_{}", item_counter));

        let is_audio = lane < 0 || child.tag == "audio";

        let body = if is_audio {
            Body::Audio(Audio {
                source,
                source_start: src_start_ms,
                source_end: src_end_ms,
                role: None,
                speed: None,
                overrun: None,
                volume: None,
                audio_effects: None,
            })
        } else {
            Body::Video(Video {
                source,
                source_start: Some(src_start_ms),
                source_end: Some(src_end_ms),
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
                speed: None,
                overrun: None,
                volume: None,
                effects: None,
                audio_effects: None,
            })
        };

        lane_map.entry(lane).or_default().push(Element {
            id: elem_id,
            group: None,
            start: start_offset_ms,
            end: start_offset_ms + dur_ms,
            layer: None,
            body,
        });

        if lane == 0 {
            primary_cursor_ms = start_offset_ms + dur_ms;
        }

        item_counter += 1;
    }

    // Assemble tracks from lane map
    let mut tracks = Vec::new();
    let mut sorted_lanes: Vec<i64> = lane_map.keys().copied().collect();
    sorted_lanes.sort();

    for lane in sorted_lanes {
        let mut elements = lane_map.remove(&lane).unwrap();
        elements.sort_by_key(|e| e.start);

        let (track_name, layer) = if lane == 0 {
            ("video_spine".to_string(), 0)
        } else if lane > 0 {
            (format!("video_lane_{lane}"), lane)
        } else {
            (format!("audio_lane_{}", -lane), lane)
        };

        tracks.push(Track {
            name: track_name,
            layer,
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
        frame: Frame { width: frame_width, height: frame_height },
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

/// Export a Montagent `Project` to Apple Final Cut Pro XML (FCPXML 1.9) string.
pub fn project_to_fcpxml(project: &Project) -> Result<String, FcpxmlError> {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE fcpxml>\n");
    xml.push_str("<fcpxml version=\"1.9\">\n");

    // Resources
    xml.push_str("    <resources>\n");
    let frame_dur = format!("{}/{}s", project.fps.den(), project.fps.num());
    xml.push_str(&format!(
        "        <format id=\"r1\" name=\"MontagentFormat\" frameDuration=\"{}\" width=\"{}\" height=\"{}\"/>\n",
        frame_dur, project.frame.width, project.frame.height
    ));

    let mut asset_ids: HashMap<String, String> = HashMap::new();
    let mut asset_counter = 2;

    for track in &project.tracks {
        for el in &track.elements {
            let src = match &el.body {
                Body::Video(v) => Some(v.source.as_str()),
                Body::Audio(a) => Some(a.source.as_str()),
                _ => None,
            };
            if let Some(source) = src.filter(|s| !asset_ids.contains_key(*s)) {
                let id = format!("r{asset_counter}");
                asset_ids.insert(source.to_string(), id.clone());
                xml.push_str(&format!(
                    "        <asset id=\"{}\" name=\"{}\" src=\"{}\" start=\"0s\" hasVideo=\"1\" hasAudio=\"1\"/>\n",
                    id, el.id, source
                ));
                asset_counter += 1;
            }
        }
    }
    xml.push_str("    </resources>\n");

    // Sequence & Spine
    xml.push_str("    <library>\n");
    xml.push_str("        <event name=\"Montagent Event\">\n");
    xml.push_str("            <project name=\"Montagent Project\">\n");
    xml.push_str("                <sequence format=\"r1\">\n");
    xml.push_str("                    <spine>\n");

    for track in &project.tracks {
        let is_audio = track.elements.iter().all(|e| matches!(e.body, Body::Audio(_)));
        let lane = if is_audio { -1 } else { track.layer };

        let mut sorted = track.elements.clone();
        sorted.sort_by_key(|e| e.start);

        for el in sorted {
            let (source, src_start) = match &el.body {
                Body::Video(v) => (v.source.clone(), v.source_start.unwrap_or(0)),
                Body::Audio(a) => (a.source.clone(), a.source_start),
                _ => continue,
            };

            let asset_ref = asset_ids.get(&source).cloned().unwrap_or_else(|| "r2".to_string());
            let dur_ms = el.end - el.start;

            let offset_s = format!("{}/1000s", el.start);
            let dur_s = format!("{}/1000s", dur_ms);
            let start_s = format!("{}/1000s", src_start);

            if lane == 0 {
                xml.push_str(&format!(
                    "                        <asset-clip ref=\"{}\" offset=\"{}\" duration=\"{}\" start=\"{}\" name=\"{}\"/>\n",
                    asset_ref, offset_s, dur_s, start_s, el.id
                ));
            } else {
                xml.push_str(&format!(
                    "                        <asset-clip ref=\"{}\" offset=\"{}\" duration=\"{}\" start=\"{}\" lane=\"{}\" name=\"{}\"/>\n",
                    asset_ref, offset_s, dur_s, start_s, lane, el.id
                ));
            }
        }
    }

    xml.push_str("                    </spine>\n");
    xml.push_str("                </sequence>\n");
    xml.push_str("            </project>\n");
    xml.push_str("        </event>\n");
    xml.push_str("    </library>\n");
    xml.push_str("</fcpxml>\n");

    Ok(xml)
}
