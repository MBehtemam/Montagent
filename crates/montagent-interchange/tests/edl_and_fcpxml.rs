use montagent_core::model::{Body, FrameRate};
use montagent_interchange::{
    edl_to_project, fcpxml_to_project, format_smpte_timecode, parse_fcpxml_time,
    parse_smpte_timecode, project_to_edl, project_to_fcpxml,
};

#[test]
fn timecode_parsing_and_formatting() {
    let fps_30 = FrameRate::integer(30);

    let ms_1h = parse_smpte_timecode("01:00:00:00", &fps_30).expect("parse 1 hour");
    assert_eq!(ms_1h, 3_600_000);

    let formatted = format_smpte_timecode(3_600_000, &fps_30, false);
    assert_eq!(formatted, "01:00:00:00");

    let ms_5s = parse_smpte_timecode("00:00:05:15", &fps_30).expect("parse 5s 15f");
    assert_eq!(ms_5s, 5_500);
}

#[test]
fn edl_import_export_round_trip() {
    let edl_sample = r#"TITLE: DEMO_REEL
FCM: NON-DROP FRAME

001  AX       V     C        00:00:00:00 00:00:04:00 01:00:00:00 01:00:04:00
* FROM CLIP NAME: interview_a.mp4

002  AX       V     C        00:00:02:00 00:00:06:00 01:00:04:00 01:00:08:00
* FROM CLIP NAME: broll_b.mp4

003  AX       A     C        00:00:00:00 00:00:08:00 01:00:00:00 01:00:08:00
* FROM CLIP NAME: dialogue.wav
"#;

    let fps = FrameRate::integer(24);
    let project = edl_to_project(edl_sample, fps).expect("parse edl");

    assert_eq!(project.tracks.len(), 2);

    // Track 0: video
    let v_track = &project.tracks[0];
    assert_eq!(v_track.name, "video");
    assert_eq!(v_track.elements.len(), 2);

    // Timeline offset 01:00:00:00 (1 hour) should be normalized to 0
    assert_eq!(v_track.elements[0].start, 0);
    // 4 seconds = 4000ms
    assert_eq!(v_track.elements[0].end, 4000);

    if let Body::Video(v) = &v_track.elements[0].body {
        assert_eq!(v.source, "interview_a.mp4");
        assert_eq!(v.source_start, Some(0));
        assert_eq!(v.source_end, Some(4000));
    } else {
        panic!("expected video");
    }

    assert_eq!(v_track.elements[1].start, 4000);
    assert_eq!(v_track.elements[1].end, 8000);

    // Track 1: audio_1
    let a_track = &project.tracks[1];
    assert_eq!(a_track.name, "audio_1");
    assert_eq!(a_track.elements.len(), 1);
    assert_eq!(a_track.elements[0].start, 0);
    assert_eq!(a_track.elements[0].end, 8000);

    // Export to EDL
    let exported = project_to_edl(&project, "DEMO_REEL").expect("export edl");
    assert!(exported.contains("TITLE: DEMO_REEL"));
    assert!(exported.contains("interview_a.mp4"));
    assert!(exported.contains("broll_b.mp4"));
    assert!(exported.contains("dialogue.wav"));
}

#[test]
fn fcpxml_import_export_round_trip() {
    let fcpxml_sample = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE fcpxml>
<fcpxml version="1.9">
    <resources>
        <format id="r1" name="FFVideoFormat1080p24" frameDuration="100/2400s" width="1920" height="1080"/>
        <asset id="r2" name="interview" src="file:///workspace/media/interview.mp4" start="0s" duration="100s" hasVideo="1" hasAudio="1"/>
        <asset id="r3" name="overlay" src="file:///workspace/media/logo.png" start="0s" duration="100s" hasVideo="1" hasAudio="0"/>
    </resources>
    <library>
        <event name="Event 1">
            <project name="Project 1">
                <sequence format="r1" duration="10s">
                    <spine>
                        <asset-clip ref="r2" offset="0s" duration="5s" start="2s" name="interview_clip"/>
                        <asset-clip ref="r3" offset="1s" duration="3s" start="0s" lane="1" name="logo_overlay"/>
                    </spine>
                </sequence>
            </project>
        </event>
    </library>
</fcpxml>"#;

    let project = fcpxml_to_project(fcpxml_sample).expect("parse fcpxml");

    assert_eq!(project.frame.width, 1920);
    assert_eq!(project.frame.height, 1080);
    assert_eq!(project.fps, FrameRate::integer(24));
    assert_eq!(project.tracks.len(), 2);

    // Spine track
    let spine = &project.tracks[0];
    assert_eq!(spine.layer, 0);
    assert_eq!(spine.elements.len(), 1);
    assert_eq!(spine.elements[0].start, 0);
    assert_eq!(spine.elements[0].end, 5000);
    if let Body::Video(v) = &spine.elements[0].body {
        assert_eq!(v.source, "/workspace/media/interview.mp4");
        assert_eq!(v.source_start, Some(2000));
        assert_eq!(v.source_end, Some(7000));
    }

    // Lane 1 track
    let lane1 = &project.tracks[1];
    assert_eq!(lane1.layer, 1);
    assert_eq!(lane1.elements.len(), 1);
    assert_eq!(lane1.elements[0].start, 1000);
    assert_eq!(lane1.elements[0].end, 4000);

    // Export to FCPXML
    let exported = project_to_fcpxml(&project).expect("export fcpxml");
    assert!(exported.contains("<fcpxml version=\"1.9\">"));
    assert!(exported.contains("/workspace/media/interview.mp4"));
    assert!(exported.contains("lane=\"1\""));

    // Re-import exported FCPXML
    let reimported = fcpxml_to_project(&exported).expect("reimport fcpxml");
    assert_eq!(reimported.tracks.len(), 2);
    assert_eq!(reimported.tracks[0].elements[0].start, 0);
    assert_eq!(reimported.tracks[0].elements[0].end, 5000);
}

#[test]
fn fcpxml_rational_time_parsing() {
    assert_eq!(parse_fcpxml_time("0s").unwrap(), 0);
    assert_eq!(parse_fcpxml_time("10s").unwrap(), 10000);
    assert_eq!(parse_fcpxml_time("1.5s").unwrap(), 1500);
    assert_eq!(parse_fcpxml_time("1001/30000s").unwrap(), 33);
    assert_eq!(parse_fcpxml_time("2400/2400s").unwrap(), 1000);
}
