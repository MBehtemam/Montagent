use montagent_core::model::{Marker, MarkerKind, Project};
use serde_json::json;

#[test]
fn project_and_element_markers_roundtrip_and_preserve_presence() {
    let raw = json!({
        "frame": { "width": 1920, "height": 1080 },
        "fps": 30,
        "duration": 10000,
        "markers": [
            { "at": 0, "label": "Intro", "kind": "chapter" },
            { "at": 1500, "duration": 500, "label": "SFX cue", "kind": "cue" },
            { "at": 2000, "kind": "beat" },
            { "at": 3000, "label": "Implicit comment" },
            { "at": 4000, "kind": "comment" }
        ],
        "tracks": [
            {
                "name": "v1",
                "layer": 1,
                "elements": [
                    {
                        "id": "v1-1",
                        "type": "video",
                        "start": 0,
                        "end": 5000,
                        "source": "assets/clip.mp4",
                        "source_start": 0,
                        "source_end": 5000,
                        "width": 1920,
                        "height": 1080,
                        "fit": "literal",
                        "markers": [
                            { "at": 400, "label": "Action beat", "kind": "beat" },
                            { "at": 1200, "duration": 300, "label": "Dialog problem", "kind": "comment" }
                        ]
                    },
                    {
                        "id": "v1-2",
                        "type": "rect",
                        "start": 5000,
                        "end": 8000,
                        "width": 100,
                        "height": 100,
                        "fill": "#FF0000",
                        "markers": []
                    },
                    {
                        "id": "v1-3",
                        "type": "text",
                        "start": 8000,
                        "end": 10000,
                        "width": 400,
                        "height": 100,
                        "font": "brand",
                        "size": 32,
                        "runs": [{ "text": "Hello" }]
                    }
                ]
            }
        ]
    });

    let project: Project = serde_json::from_value(raw.clone()).expect("valid project with markers");
    let root_markers = project.markers.as_ref().expect("project markers present");
    assert_eq!(root_markers.len(), 5);
    assert_eq!(
        root_markers[0],
        Marker {
            at: 0,
            duration: None,
            label: Some("Intro".into()),
            kind: Some(MarkerKind::Chapter),
        }
    );
    assert_eq!(root_markers[0].effective_kind(), MarkerKind::Chapter);
    assert_eq!(root_markers[3].effective_kind(), MarkerKind::Comment);

    let reserialized = serde_json::to_value(&project).expect("serialize project");
    assert_eq!(reserialized["markers"], raw["markers"]);
    assert_eq!(
        reserialized["tracks"][0]["elements"][0]["markers"],
        raw["tracks"][0]["elements"][0]["markers"]
    );
    assert_eq!(
        reserialized["tracks"][0]["elements"][1]["markers"],
        json!([])
    );
    assert!(reserialized["tracks"][0]["elements"][2]
        .get("markers")
        .is_none());
}

#[test]
fn marker_rejects_negative_at_negative_duration_unknown_kind_and_unknown_field() {
    for bad_marker in [
        json!({ "at": -1 }),
        json!({ "at": 100, "duration": -5 }),
        json!({ "at": 100, "kind": "todo" }),
        json!({ "at": 100, "extra": true }),
    ] {
        let doc = json!({
            "frame": { "width": 1920, "height": 1080 },
            "fps": 30,
            "markers": [bad_marker],
            "tracks": []
        });
        assert!(
            serde_json::from_value::<Project>(doc).is_err(),
            "expected error for marker: {bad_marker}"
        );
    }
}

mod common;

#[test]
fn validate_rejects_invalid_project_and_element_markers() {
    use montagent_core::report::ExitCode;

    for (line, bad_marker) in [
        (1001, json!({ "at": -1 })),
        (1002, json!({ "at": 100, "duration": -5 })),
        (1003, json!({ "at": 100, "kind": "todo" })),
        (1004, json!({ "at": 100, "extra": true })),
    ] {
        let dir = common::tempdir(line);
        let root_doc = common::canonical(
            &json!({
                "frame": { "width": 1080, "height": 1920 },
                "fps": 25,
                "markers": [bad_marker],
                "tracks": []
            })
            .to_string(),
        );
        let root_path = common::write_project(&dir, "root.montagent.json", &root_doc);
        let root_report = montagent_core::validate(&root_path);
        assert_eq!(
            root_report.exit_code(),
            ExitCode::Errors,
            "expected validate error on project marker {bad_marker}"
        );

        let elem_doc = common::canonical(
            &json!({
                "frame": { "width": 1080, "height": 1920 },
                "fps": 25,
                "tracks": [{
                    "name": "v1",
                    "layer": 1,
                    "elements": [{
                        "id": "r1",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "x": 0,
                        "y": 0,
                        "width": 100,
                        "height": 100,
                        "markers": [bad_marker]
                    }]
                }]
            })
            .to_string(),
        );
        let elem_path = common::write_project(&dir, "elem.montagent.json", &elem_doc);
        let elem_report = montagent_core::validate(&elem_path);
        assert_eq!(
            elem_report.exit_code(),
            ExitCode::Errors,
            "expected validate error on element marker {bad_marker}"
        );
    }
}

#[test]
fn validate_rejects_project_marker_exceeding_declared_duration() {
    use montagent_core::finding::Class;
    use montagent_core::report::ExitCode;

    let dir = common::tempdir(1010);

    // Marker at exactly project.duration (5000) is legal; marker at 5001 exceeds project.duration.
    let doc = common::canonical(
        &json!({
            "frame": { "width": 1080, "height": 1920 },
            "fps": 25,
            "duration": 5000,
            "markers": [
                { "at": 5000, "label": "End" },
                { "at": 5001, "label": "Past end" }
            ],
            "tracks": [{
                "name": "v1",
                "layer": 1,
                "elements": [{
                    "id": "r1",
                    "type": "rect",
                    "start": 0,
                    "end": 5000,
                    "x": 0,
                    "y": 0,
                    "width": 1080,
                    "height": 1920,
                    "fill": "#000000"
                }]
            }]
        })
        .to_string(),
    );
    let path = common::write_project(&dir, "past.montagent.json", &doc);
    let report = montagent_core::validate(&path);
    assert_eq!(report.exit_code(), ExitCode::Errors);

    let past: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-MARKER-PAST-DURATION")
        .collect();
    assert_eq!(past.len(), 1, "expected 1 E-MARKER-PAST-DURATION finding, got {past:?}");
    assert_eq!(past[0].class, Class::Error);
    assert_eq!(past[0].fields["index"], 1);
    assert_eq!(past[0].fields["at"], 5001);
    assert_eq!(past[0].fields["duration"], 5000);
}

#[test]
fn shift_positive_global_shifts_project_markers_and_preserves_element_markers() {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::shift::{Ask, shift};

    let dir = common::tempdir(1020);
    let doc = common::canonical(
        &json!({
            "frame": { "width": 1080, "height": 1920 },
            "fps": 25,
            "markers": [
                { "at": 500, "label": "Before" },
                { "at": 1000, "label": "At cut", "kind": "chapter" },
                { "at": 2000, "label": "After cut", "kind": "beat" }
            ],
            "tracks": [{
                "name": "v1",
                "layer": 1,
                "elements": [
                    {
                        "id": "r0",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "x": 0,
                        "y": 0,
                        "width": 1080,
                        "height": 1920,
                        "fill": "#000000"
                    },
                    {
                        "id": "r1",
                        "type": "rect",
                        "start": 1000,
                        "end": 2000,
                        "x": 0,
                        "y": 0,
                        "width": 1080,
                        "height": 1920,
                        "fill": "#FFFFFF",
                        "markers": [
                            { "at": 200, "label": "Local beat", "kind": "beat" },
                            { "at": 800, "label": "Local cue", "kind": "cue" }
                        ]
                    }
                ]
            }]
        })
        .to_string(),
    );
    let path = common::write_project(&dir, "shift_pos.montagent.json", &doc);
    let ask = Ask::new(1000, 500);
    let ans = shift(&path, &ask);
    assert_eq!(
        ans.report().exit_code(),
        ExitCode::Ok,
        "unexpected findings: {:?}",
        ans.report().findings
    );

    let updated: Project =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).expect("valid project");
    let root_markers = updated.markers.expect("project markers");
    assert_eq!(root_markers.len(), 3);
    assert_eq!(root_markers[0].at, 500);
    assert_eq!(root_markers[1].at, 1500);
    assert_eq!(root_markers[2].at, 2500);

    let elem_markers = updated.tracks[0].elements[1]
        .markers()
        .expect("element markers");
    assert_eq!(elem_markers.len(), 2);
    assert_eq!(elem_markers[0].at, 200);
    assert_eq!(elem_markers[1].at, 800);
}

#[test]
fn shift_scoped_to_track_leaves_project_markers_untouched() {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::shift::{Ask, shift};

    let dir = common::tempdir(1030);
    let doc = common::canonical(
        &json!({
            "frame": { "width": 1080, "height": 1920 },
            "fps": 25,
            "markers": [
                { "at": 500, "label": "Before" },
                { "at": 1000, "label": "At cut" }
            ],
            "tracks": [{
                "name": "v1",
                "layer": 1,
                "elements": [{
                    "id": "r1",
                    "type": "rect",
                    "start": 1000,
                    "end": 2000,
                    "x": 0,
                    "y": 0,
                    "width": 1080,
                    "height": 1920,
                    "fill": "#FFFFFF"
                }]
            }]
        })
        .to_string(),
    );
    let path = common::write_project(&dir, "shift_scoped.montagent.json", &doc);
    let mut ask = Ask::new(1000, 500);
    ask.scope = Some("v1".into());
    let ans = shift(&path, &ask);
    assert_eq!(
        ans.report().exit_code(),
        ExitCode::Ok,
        "unexpected findings: {:?}",
        ans.report().findings
    );

    let updated: Project =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).expect("valid project");
    let root_markers = updated.markers.expect("project markers");
    assert_eq!(root_markers[0].at, 500);
    assert_eq!(root_markers[1].at, 1000);
}

#[test]
fn shift_negative_global_shifts_downstream_project_markers_and_excises_cut_span() {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::shift::{Ask, shift};

    let dir = common::tempdir(1040);
    let doc = common::canonical(
        &json!({
            "frame": { "width": 1080, "height": 1920 },
            "fps": 25,
            "markers": [
                { "at": 500, "label": "Before" },
                { "at": 1200, "label": "Inside cut" },
                { "at": 2000, "label": "At cut end" },
                { "at": 3000, "label": "After cut" }
            ],
            "tracks": [{
                "name": "v1",
                "layer": 1,
                "elements": [
                    {
                        "id": "r0",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "x": 0,
                        "y": 0,
                        "width": 1080,
                        "height": 1920,
                        "fill": "#000000"
                    },
                    {
                        "id": "r1",
                        "type": "rect",
                        "start": 2000,
                        "end": 3000,
                        "x": 0,
                        "y": 0,
                        "width": 1080,
                        "height": 1920,
                        "fill": "#FFFFFF",
                        "markers": [
                            { "at": 150, "label": "Local" }
                        ]
                    }
                ]
            }]
        })
        .to_string(),
    );
    let path = common::write_project(&dir, "shift_neg.montagent.json", &doc);
    let mut ask = Ask::delete(1000, -1000);
    ask.release = vec![(1000, 2000)];
    let ans = shift(&path, &ask);
    assert_eq!(
        ans.report().exit_code(),
        ExitCode::Ok,
        "unexpected findings: {:?}",
        ans.report().findings
    );

    let updated: Project =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).expect("valid project");
    let root_markers = updated.markers.expect("project markers");
    assert_eq!(root_markers.len(), 3);
    assert_eq!(root_markers[0].label.as_deref(), Some("Before"));
    assert_eq!(root_markers[0].at, 500);
    assert_eq!(root_markers[1].label.as_deref(), Some("At cut end"));
    assert_eq!(root_markers[1].at, 1000);
    assert_eq!(root_markers[2].label.as_deref(), Some("After cut"));
    assert_eq!(root_markers[2].at, 2000);

    let elem_markers = updated.tracks[0].elements[1]
        .markers()
        .expect("element markers");
    assert_eq!(elem_markers.len(), 1);
    assert_eq!(elem_markers[0].at, 150);
}

#[test]
fn markers_are_render_inert_for_frame_and_render() {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::frame::{Ask as FrameAsk, frame};

    let dir = common::tempdir(1050);
    let without_markers = common::canonical(
        &json!({
            "frame": { "width": 200, "height": 200 },
            "fps": 25,
            "background": "#000000",
            "tracks": [{
                "name": "v1",
                "layer": 1,
                "elements": [{
                    "id": "r0",
                    "type": "rect",
                    "start": 0,
                    "end": 1000,
                    "x": 20,
                    "y": 20,
                    "width": 160,
                    "height": 160,
                    "fill": "#FF5500"
                }]
            }]
        })
        .to_string(),
    );
    let with_markers = common::canonical(
        &json!({
            "frame": { "width": 200, "height": 200 },
            "fps": 25,
            "background": "#000000",
            "markers": [
                { "at": 0, "label": "Intro", "kind": "chapter" },
                { "at": 500, "label": "Beat", "kind": "beat" }
            ],
            "tracks": [{
                "name": "v1",
                "layer": 1,
                "elements": [{
                    "id": "r0",
                    "type": "rect",
                    "start": 0,
                    "end": 1000,
                    "x": 20,
                    "y": 20,
                    "width": 160,
                    "height": 160,
                    "fill": "#FF5500",
                    "markers": [
                        { "at": 250, "label": "Local cue", "kind": "cue" }
                    ]
                }]
            }]
        })
        .to_string(),
    );

    let path_without = common::write_project(&dir, "no_markers.montagent.json", &without_markers);
    let path_with = common::write_project(&dir, "with_markers.montagent.json", &with_markers);

    let ask = FrameAsk {
        at: Some(500),
        png: true,
        full: true,
        ..FrameAsk::default()
    };
    let ans_without = frame(&path_without, &ask);
    let ans_with = frame(&path_with, &ask);
    assert_eq!(ans_without.report().exit_code(), ExitCode::Ok);
    assert_eq!(ans_with.report().exit_code(), ExitCode::Ok);

    let bytes_without = &ans_without.image().expect("image without markers").bytes;
    let bytes_with = &ans_with.image().expect("image with markers").bytes;
    assert_eq!(
        bytes_without, bytes_with,
        "frame raster output must be byte-identical with and without markers"
    );
}

#[test]
fn render_exports_project_chapter_markers_to_container_metadata_and_excludes_others() {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::render::{Ask as RenderAsk, render};

    if !common::has_ffprobe() {
        eprintln!("skipping: no ffprobe on PATH");
        return;
    }

    let dir = common::tempdir(1060);
    let out_mp4 = dir.join("chapters.mp4");
    let doc = common::canonical(
        &json!({
            "frame": { "width": 200, "height": 200 },
            "fps": 25,
            "duration": 2000,
            "background": "#000000",
            "output": out_mp4.display().to_string(),
            "markers": [
                { "at": 1000, "label": "Main Segment", "duration": 0, "kind": "chapter" },
                { "at": 0, "label": "Intro", "kind": "chapter" },
                { "at": 1600, "label": "Outro", "duration": 300, "kind": "chapter" },
                { "at": 500, "label": "Director note", "kind": "comment" },
                { "at": 700, "label": "Downbeat", "kind": "beat" },
                { "at": 900, "label": "SFX cue", "kind": "cue" },
                { "at": 300, "label": "Unclassified" }
            ],
            "tracks": [{
                "name": "v1",
                "layer": 1,
                "elements": [{
                    "id": "r0",
                    "type": "rect",
                    "start": 0,
                    "end": 2000,
                    "x": 0,
                    "y": 0,
                    "width": 200,
                    "height": 200,
                    "fill": "#224466",
                    "markers": [
                        { "at": 100, "label": "Element chapter", "kind": "chapter" },
                        { "at": 400, "label": "Element beat", "kind": "beat" }
                    ]
                }]
            }]
        })
        .to_string(),
    );

    let path = common::write_project(&dir, "chapters.montagent.json", &doc);
    let ans = render(&path, &RenderAsk::default(), &mut |_| {});
    assert_eq!(
        ans.report().exit_code(),
        ExitCode::Ok,
        "unexpected findings: {:?}",
        ans.report().findings
    );
    assert!(out_mp4.exists(), "rendered MP4 should exist");

    let probe_out = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_chapters",
            "-show_format",
        ])
        .arg(&out_mp4)
        .output()
        .expect("ffprobe ran");
    assert!(probe_out.status.success(), "ffprobe succeeded");

    let probed: serde_json::Value =
        serde_json::from_slice(&probe_out.stdout).expect("valid ffprobe json");
    let chapters = probed["chapters"].as_array().expect("chapters array");
    assert_eq!(
        chapters.len(),
        3,
        "only project-level chapter markers should be exported: {chapters:?}"
    );

    // Chapter 0: Intro (at=0, no duration -> end=next chapter at=1000)
    assert_eq!(chapters[0]["start"], 0);
    assert_eq!(chapters[0]["end"], 1000);
    assert_eq!(chapters[0]["tags"]["title"], "Intro");

    // Chapter 1: Main Segment (at=1000, duration=0 -> end=next chapter at=1600)
    assert_eq!(chapters[1]["start"], 1000);
    assert_eq!(chapters[1]["end"], 1600);
    assert_eq!(chapters[1]["tags"]["title"], "Main Segment");

    // Chapter 2: Outro (at=1600, duration=300 -> start=1600, end=1900)
    assert_eq!(chapters[2]["start"], 1600);
    assert_eq!(chapters[2]["end"], 1900);
    assert_eq!(chapters[2]["tags"]["title"], "Outro");
}



