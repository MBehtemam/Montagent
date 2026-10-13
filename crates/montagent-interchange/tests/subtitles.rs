use montagent_core::model::FrameRate;
use montagent_interchange::{
    SubtitleFormat, export_subtitles, import_subtitles, parse_srt, parse_vtt, serialize_srt,
    serialize_vtt,
};

#[test]
fn srt_parsing_and_serialization_round_trip() {
    let srt_sample = r#"1
00:00:01,000 --> 00:00:04,500
Hello world!
This is a test subtitle.

2
00:00:05,200 --> 00:00:08,123
Second caption line with <i>formatting</i>.
"#;

    let cues = parse_srt(srt_sample).expect("parse srt");
    assert_eq!(cues.len(), 2);

    assert_eq!(cues[0].start_ms, 1000);
    assert_eq!(cues[0].end_ms, 4500);
    assert_eq!(cues[0].text, "Hello world!\nThis is a test subtitle.");

    assert_eq!(cues[1].start_ms, 5200);
    assert_eq!(cues[1].end_ms, 8123);
    assert_eq!(cues[1].text, "Second caption line with formatting.");

    let serialized = serialize_srt(&cues);
    assert!(serialized.contains("00:00:01,000 --> 00:00:04,500"));
    assert!(serialized.contains("00:00:05,200 --> 00:00:08,123"));
    assert!(serialized.contains("Hello world!\nThis is a test subtitle."));
}

#[test]
fn vtt_parsing_and_serialization() {
    let vtt_sample = r#"WEBVTT

1
00:00:01.000 --> 00:00:04.500 position:50% line:80%
Welcome to Montagent WebVTT test.

2
00:00:06.000 --> 00:00:09.500
<v Speaker>Multi-line voice</v>
Second line text.
"#;

    let cues = parse_vtt(vtt_sample).expect("parse vtt");
    assert_eq!(cues.len(), 2);

    assert_eq!(cues[0].start_ms, 1000);
    assert_eq!(cues[0].end_ms, 4500);
    assert_eq!(cues[0].text, "Welcome to Montagent WebVTT test.");

    assert_eq!(cues[1].start_ms, 6000);
    assert_eq!(cues[1].end_ms, 9500);
    assert_eq!(cues[1].text, "Multi-line voice\nSecond line text.");

    let serialized = serialize_vtt(&cues);
    assert!(serialized.starts_with("WEBVTT\n\n"));
    assert!(serialized.contains("00:00:01.000 --> 00:00:04.500"));
    assert!(serialized.contains("00:00:06.000 --> 00:00:09.500"));
}

#[test]
fn project_import_export_round_trip() {
    let srt_sample = r#"1
00:00:02,000 --> 00:00:05,000
Caption Alpha

2
00:00:07,500 --> 00:00:10,000
Caption Beta
"#;

    let project = import_subtitles(
        srt_sample,
        SubtitleFormat::Srt,
        1920,
        1080,
        FrameRate::integer(30),
    )
    .expect("import subtitles to project");

    assert_eq!(project.frame.width, 1920);
    assert_eq!(project.frame.height, 1080);
    assert_eq!(project.tracks.len(), 1);
    assert_eq!(project.tracks[0].name, "captions");
    assert_eq!(project.tracks[0].elements.len(), 2);

    let exported_srt = export_subtitles(&project, SubtitleFormat::Srt).expect("export to srt");
    assert!(exported_srt.contains("Caption Alpha"));
    assert!(exported_srt.contains("Caption Beta"));
    assert!(exported_srt.contains("00:00:02,000 --> 00:00:05,000"));
    assert!(exported_srt.contains("00:00:07,500 --> 00:00:10,000"));

    let exported_vtt = export_subtitles(&project, SubtitleFormat::Vtt).expect("export to vtt");
    assert!(exported_vtt.starts_with("WEBVTT\n\n"));
    assert!(exported_vtt.contains("00:00:02.000 --> 00:00:05.000"));
    assert!(exported_vtt.contains("00:00:07.500 --> 00:00:10.000"));
}
