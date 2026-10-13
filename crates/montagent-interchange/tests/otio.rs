use montagent_core::model::{Body, FrameRate, Speed};
use montagent_interchange::{otio_to_project, project_to_otio, rate_to_frame_rate};

#[test]
fn rate_mapping_ntsc_and_integer() {
    let rate_23976 = rate_to_frame_rate(23.976023976);
    assert_eq!(rate_23976.num(), 24000);
    assert_eq!(rate_23976.den(), 1001);

    let rate_2997 = rate_to_frame_rate(29.97);
    assert_eq!(rate_2997.num(), 30000);
    assert_eq!(rate_2997.den(), 1001);

    let rate_60 = rate_to_frame_rate(60.0);
    assert_eq!(rate_60.num(), 60);
    assert_eq!(rate_60.den(), 1);
}

#[test]
fn otio_sample_ingestion_with_gaps_and_speed() {
    let otio_json = r#"{
  "OTIO_SCHEMA": "Timeline.1",
  "name": "Test_Timeline",
  "global_start_time": {
    "OTIO_SCHEMA": "RationalTime.1",
    "value": 0.0,
    "rate": 24.0
  },
  "tracks": {
    "OTIO_SCHEMA": "Stack.1",
    "name": "tracks",
    "children": [
      {
        "OTIO_SCHEMA": "Track.1",
        "name": "V1",
        "kind": "Video",
        "children": [
          {
            "OTIO_SCHEMA": "Gap.1",
            "name": "gap1",
            "source_range": {
              "OTIO_SCHEMA": "TimeRange.1",
              "start_time": {
                "OTIO_SCHEMA": "RationalTime.1",
                "value": 0.0,
                "rate": 24.0
              },
              "duration": {
                "OTIO_SCHEMA": "RationalTime.1",
                "value": 24.0,
                "rate": 24.0
              }
            }
          },
          {
            "OTIO_SCHEMA": "Clip.2",
            "name": "interview_clip",
            "source_range": {
              "OTIO_SCHEMA": "TimeRange.1",
              "start_time": {
                "OTIO_SCHEMA": "RationalTime.1",
                "value": 48.0,
                "rate": 24.0
              },
              "duration": {
                "OTIO_SCHEMA": "RationalTime.1",
                "value": 72.0,
                "rate": 24.0
              }
            },
            "media_reference": {
              "OTIO_SCHEMA": "ExternalReference.1",
              "target_url": "media/interview.mp4"
            },
            "effects": [
              {
                "OTIO_SCHEMA": "LinearTimeWarp.1",
                "name": "SpeedWarp",
                "time_scalar": 1.25
              }
            ]
          }
        ]
      },
      {
        "OTIO_SCHEMA": "Track.1",
        "name": "A1",
        "kind": "Audio",
        "children": [
          {
            "OTIO_SCHEMA": "Clip.2",
            "name": "voiceover",
            "source_range": {
              "OTIO_SCHEMA": "TimeRange.1",
              "start_time": {
                "OTIO_SCHEMA": "RationalTime.1",
                "value": 0.0,
                "rate": 24.0
              },
              "duration": {
                "OTIO_SCHEMA": "RationalTime.1",
                "value": 96.0,
                "rate": 24.0
              }
            },
            "media_reference": {
              "OTIO_SCHEMA": "ExternalReference.1",
              "target_url": "media/vo.wav"
            }
          }
        ]
      }
    ]
  }
}"#;

    let project = otio_to_project(otio_json).expect("otio ingestion failed");

    assert_eq!(project.fps, FrameRate::integer(24));
    assert_eq!(project.tracks.len(), 2);

    // Track 1: V1
    let v1 = &project.tracks[0];
    assert_eq!(v1.name, "V1");
    assert_eq!(v1.elements.len(), 1);
    let v_elem = &v1.elements[0];
    // Gap was 24 frames @ 24fps = 1000ms. Clip starts at 1000ms.
    assert_eq!(v_elem.start, 1000);
    // Duration was 72 frames @ 24fps = 3000ms. End is 4000ms.
    assert_eq!(v_elem.end, 4000);

    if let Body::Video(v) = &v_elem.body {
        assert_eq!(v.source, "media/interview.mp4");
        assert_eq!(v.source_start, Some(2000)); // 48 frames = 2000ms
        assert_eq!(v.source_end, Some(5000));   // 2000 + 3000
        assert_eq!(v.speed, Some(Speed(1.25)));
    } else {
        panic!("expected video body");
    }

    // Track 2: A1
    let a1 = &project.tracks[1];
    assert_eq!(a1.name, "A1");
    assert_eq!(a1.elements.len(), 1);
    let a_elem = &a1.elements[0];
    assert_eq!(a_elem.start, 0);
    assert_eq!(a_elem.end, 4000); // 96 frames = 4000ms

    if let Body::Audio(a) = &a_elem.body {
        assert_eq!(a.source, "media/vo.wav");
        assert_eq!(a.source_start, 0);
        assert_eq!(a.source_end, 4000);
    } else {
        panic!("expected audio body");
    }
}

#[test]
fn otio_round_trip_gap_synthesis() {
    let otio_json = r#"{
  "OTIO_SCHEMA": "Timeline.1",
  "name": "RoundTrip",
  "global_start_time": {
    "OTIO_SCHEMA": "RationalTime.1",
    "value": 0.0,
    "rate": 30.0
  },
  "tracks": {
    "OTIO_SCHEMA": "Stack.1",
    "name": "tracks",
    "children": [
      {
        "OTIO_SCHEMA": "Track.1",
        "name": "V1",
        "kind": "Video",
        "children": [
          {
            "OTIO_SCHEMA": "Gap.1",
            "source_range": {
              "OTIO_SCHEMA": "TimeRange.1",
              "start_time": { "OTIO_SCHEMA": "RationalTime.1", "value": 0.0, "rate": 30.0 },
              "duration": { "OTIO_SCHEMA": "RationalTime.1", "value": 30.0, "rate": 30.0 }
            }
          },
          {
            "OTIO_SCHEMA": "Clip.2",
            "name": "clipA",
            "source_range": {
              "OTIO_SCHEMA": "TimeRange.1",
              "start_time": { "OTIO_SCHEMA": "RationalTime.1", "value": 0.0, "rate": 30.0 },
              "duration": { "OTIO_SCHEMA": "RationalTime.1", "value": 60.0, "rate": 30.0 }
            },
            "media_reference": {
              "OTIO_SCHEMA": "ExternalReference.1",
              "target_url": "media/a.mp4"
            }
          },
          {
            "OTIO_SCHEMA": "Gap.1",
            "source_range": {
              "OTIO_SCHEMA": "TimeRange.1",
              "start_time": { "OTIO_SCHEMA": "RationalTime.1", "value": 90.0, "rate": 30.0 },
              "duration": { "OTIO_SCHEMA": "RationalTime.1", "value": 30.0, "rate": 30.0 }
            }
          },
          {
            "OTIO_SCHEMA": "Clip.2",
            "name": "clipB",
            "source_range": {
              "OTIO_SCHEMA": "TimeRange.1",
              "start_time": { "OTIO_SCHEMA": "RationalTime.1", "value": 0.0, "rate": 30.0 },
              "duration": { "OTIO_SCHEMA": "RationalTime.1", "value": 60.0, "rate": 30.0 }
            },
            "media_reference": {
              "OTIO_SCHEMA": "ExternalReference.1",
              "target_url": "media/b.mp4"
            }
          }
        ]
      }
    ]
  }
}"#;

    // Import -> Project
    let project = otio_to_project(otio_json).expect("import");
    assert_eq!(project.tracks[0].elements.len(), 2);
    assert_eq!(project.tracks[0].elements[0].start, 1000);
    assert_eq!(project.tracks[0].elements[0].end, 3000);
    assert_eq!(project.tracks[0].elements[1].start, 4000);
    assert_eq!(project.tracks[0].elements[1].end, 6000);

    // Export -> OTIO
    let exported_otio = project_to_otio(&project).expect("export");
    assert!(exported_otio.contains("Gap.1"));
    assert!(exported_otio.contains("media/a.mp4"));
    assert!(exported_otio.contains("media/b.mp4"));

    // Re-import exported OTIO -> Project
    let reimported = otio_to_project(&exported_otio).expect("reimport");
    assert_eq!(reimported.tracks[0].elements.len(), 2);
    assert_eq!(reimported.tracks[0].elements[0].start, 1000);
    assert_eq!(reimported.tracks[0].elements[0].end, 3000);
    assert_eq!(reimported.tracks[0].elements[1].start, 4000);
    assert_eq!(reimported.tracks[0].elements[1].end, 6000);
}
