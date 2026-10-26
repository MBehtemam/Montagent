//! ADR-0201: Bounded nested sequences (type: nest) tests.

use montagent_core::report::ExitCode;
use montagent_core::validate;
use montagent_core::verbs::frame::{Ask, frame};

mod common;
use common::{tempdir, write_project};

#[test]
fn valid_nest_element_validates_cleanly() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "background": "#000000",
      "duration": 5000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "container",
              "type": "nest",
              "start": 0,
              "end": 4000,
              "x": 960,
              "y": 540,
              "width": 800,
              "height": 600,
              "elements": [
                {
                  "id": "child_rect",
                  "type": "rect",
                  "start": 500,
                  "end": 3500,
                  "width": 200,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "valid_nest.montagent.json", project);
    let report = validate(&path);

    assert_eq!(report.summary().error, 0, "findings: {:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn child_element_extending_before_nest_start_emits_temporal_bounds_error() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "duration": 5000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "container",
              "type": "nest",
              "start": 1000,
              "end": 4000,
              "width": 800,
              "height": 600,
              "elements": [
                {
                  "id": "early_child",
                  "type": "rect",
                  "start": 500,
                  "end": 3000,
                  "width": 200,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "early_child.montagent.json", project);
    let report = validate(&path);

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-NEST-TEMPORAL-BOUNDS"),
        "expected E-NEST-TEMPORAL-BOUNDS, got: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn child_element_extending_past_nest_end_emits_temporal_bounds_error() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "duration": 6000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "container",
              "type": "nest",
              "start": 1000,
              "end": 4000,
              "width": 800,
              "height": 600,
              "elements": [
                {
                  "id": "late_child",
                  "type": "rect",
                  "start": 2000,
                  "end": 4500,
                  "width": 200,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "late_child.montagent.json", project);
    let report = validate(&path);

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-NEST-TEMPORAL-BOUNDS"),
        "expected E-NEST-TEMPORAL-BOUNDS, got: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn nesting_depth_exceeding_four_emits_depth_error() {
    let dir = tempdir(line!());
    // 5 levels of nesting: nest1 -> nest2 -> nest3 -> nest4 -> nest5
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "duration": 5000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "nest1",
              "type": "nest",
              "start": 0,
              "end": 5000,
              "width": 1000,
              "height": 1000,
              "elements": [
                {
                  "id": "nest2",
                  "type": "nest",
                  "start": 0,
                  "end": 5000,
                  "width": 800,
                  "height": 800,
                  "elements": [
                    {
                      "id": "nest3",
                      "type": "nest",
                      "start": 0,
                      "end": 5000,
                      "width": 600,
                      "height": 600,
                      "elements": [
                        {
                          "id": "nest4",
                          "type": "nest",
                          "start": 0,
                          "end": 5000,
                          "width": 400,
                          "height": 400,
                          "elements": [
                            {
                              "id": "nest5",
                              "type": "nest",
                              "start": 0,
                              "end": 5000,
                              "width": 200,
                              "height": 200,
                              "elements": []
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "deep_nest.montagent.json", project);
    let report = validate(&path);

    assert!(
        report.findings.iter().any(|f| f.code == "E-NEST-DEPTH"),
        "expected E-NEST-DEPTH, got: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn nesting_depth_of_four_is_valid() {
    let dir = tempdir(line!());
    // 4 levels of nesting: nest1 -> nest2 -> nest3 -> nest4
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "duration": 5000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "nest1",
              "type": "nest",
              "start": 0,
              "end": 5000,
              "width": 1000,
              "height": 1000,
              "elements": [
                {
                  "id": "nest2",
                  "type": "nest",
                  "start": 0,
                  "end": 5000,
                  "width": 800,
                  "height": 800,
                  "elements": [
                    {
                      "id": "nest3",
                      "type": "nest",
                      "start": 0,
                      "end": 5000,
                      "width": 600,
                      "height": 600,
                      "elements": [
                        {
                          "id": "nest4",
                          "type": "nest",
                          "start": 0,
                          "end": 5000,
                          "width": 400,
                          "height": 400,
                          "elements": []
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "valid_depth_4.montagent.json", project);
    let report = validate(&path);

    assert_eq!(report.summary().error, 0, "findings: {:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn duplicate_element_id_emits_duplicate_error() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "duration": 5000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "shared_id",
              "type": "nest",
              "start": 0,
              "end": 4000,
              "width": 800,
              "height": 600,
              "elements": [
                {
                  "id": "shared_id",
                  "type": "rect",
                  "start": 0,
                  "end": 4000,
                  "width": 200,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "duplicate_id.montagent.json", project);
    let report = validate(&path);

    assert!(
        report.findings.iter().any(|f| f.code == "E-ID-DUPLICATE"),
        "expected E-ID-DUPLICATE, got: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn rendering_frame_with_nest_renders_child_elements() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 100, "height": 100},
      "fps": 25,
      "background": "#000000",
      "duration": 1000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "nest1",
              "type": "nest",
              "start": 0,
              "end": 1000,
              "width": 60,
              "height": 60,
              "x": 50,
              "y": 50,
              "elements": [
                {
                  "id": "child1",
                  "type": "rect",
                  "start": 0,
                  "end": 1000,
                  "width": 40,
                  "height": 40,
                  "x": 30,
                  "y": 30,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "render_nest.montagent.json", project);
    let ask = Ask {
        at: Some(100),
        png: true,
        full: true,
        ..Ask::default()
    };

    let answer = frame(&path, &ask);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let bytes = answer.image().expect("frame png bytes").bytes.clone();
    assert!(!bytes.is_empty());

    // Decode with image crate to inspect pixels
    let img = image::load_from_memory(&bytes).expect("valid png image");
    let rgba = img.to_rgba8();

    // Center pixel (50, 50) should be within the child red rect
    let center = rgba.get_pixel(50, 50);
    assert_eq!(center[0], 255, "red channel at center");
    assert_eq!(center[1], 0, "green channel at center");
    assert_eq!(center[2], 0, "blue channel at center");

    // Corner pixel (5, 5) should be the black background
    let corner = rgba.get_pixel(5, 5);
    assert_eq!(corner[0], 0, "red channel at corner");
    assert_eq!(corner[1], 0, "green channel at corner");
    assert_eq!(corner[2], 0, "blue channel at corner");
}

#[test]
fn circular_nest_reference_emits_circular_error() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "duration": 5000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "nest_a",
              "type": "nest",
              "start": 0,
              "end": 4000,
              "width": 800,
              "height": 600,
              "elements": [
                {
                  "id": "nest_a",
                  "type": "nest",
                  "start": 0,
                  "end": 4000,
                  "width": 400,
                  "height": 300,
                  "elements": []
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "circular_nest.montagent.json", project);
    let report = validate(&path);

    assert!(
        report.findings.iter().any(|f| f.code == "E-NEST-CIRCULAR"),
        "expected E-NEST-CIRCULAR, got: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn child_element_specifying_track_key_is_rejected_by_schema() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 1920, "height": 1080},
      "fps": 30,
      "duration": 5000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "nest1",
              "type": "nest",
              "start": 0,
              "end": 4000,
              "width": 800,
              "height": 600,
              "elements": [
                {
                  "id": "child1",
                  "type": "rect",
                  "track": "v1",
                  "start": 0,
                  "end": 4000,
                  "width": 200,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "child_with_track.montagent.json", project);
    let report = validate(&path);

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-SCHEMA-UNKNOWN-KEY"),
        "expected E-SCHEMA-UNKNOWN-KEY for track on child, got: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn rendering_frame_with_nest_opacity_composites_onto_background() {
    let dir = tempdir(line!());
    // Black background (0, 0, 0), white rect (255, 255, 255) inside nest with 50% opacity (0.5)
    // Composited result at center should blend to approx 128 (within +/- 3 tolerance)
    let project = r##"{
      "frame": {"width": 100, "height": 100},
      "fps": 25,
      "background": "#000000",
      "duration": 1000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "v1",
          "layer": 1,
          "elements": [
            {
              "id": "nest1",
              "type": "nest",
              "start": 0,
              "end": 1000,
              "x": 50,
              "y": 50,
              "width": 60,
              "height": 60,
              "opacity": 0.5,
              "elements": [
                {
                  "id": "child1",
                  "type": "rect",
                  "start": 0,
                  "end": 1000,
                  "x": 30,
                  "y": 30,
                  "width": 40,
                  "height": 40,
                  "fill": "#FFFFFF"
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "render_nest_opacity.montagent.json", project);
    let ask = Ask {
        at: Some(100),
        png: true,
        full: true,
        ..Ask::default()
    };

    let answer = frame(&path, &ask);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let bytes = answer.image().expect("frame png bytes").bytes.clone();
    let img = image::load_from_memory(&bytes).expect("valid png image");
    let rgba = img.to_rgba8();

    let center = rgba.get_pixel(50, 50);
    // Skia premultiplied alpha composite of white 50% on black gives 128 (±2)
    assert!(
        (center[0] as i32 - 128).abs() <= 3,
        "expected ~128 red, got {}",
        center[0]
    );
    assert!(
        (center[1] as i32 - 128).abs() <= 3,
        "expected ~128 green, got {}",
        center[1]
    );
    assert!(
        (center[2] as i32 - 128).abs() <= 3,
        "expected ~128 blue, got {}",
        center[2]
    );
}

#[test]
fn native_transitions_between_nest_scenes_validate_cleanly() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 400, "height": 200},
      "fps": 25,
      "background": "#000000",
      "duration": 3000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "scene_a",
          "layer": 0,
          "elements": [
            {
              "id": "scene1",
              "type": "nest",
              "start": 0,
              "end": 2000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg1",
                  "type": "rect",
                  "start": 0,
                  "end": 2000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        },
        {
          "name": "scene_b",
          "layer": 1,
          "elements": [
            {
              "id": "scene2",
              "type": "nest",
              "start": 1000,
              "end": 3000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg2",
                  "type": "rect",
                  "start": 1000,
                  "end": 3000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#0000FF"
                }
              ]
            }
          ]
        },
        {
          "name": "bridge",
          "layer": 2,
          "elements": [
            {
              "id": "trans",
              "type": "transition",
              "start": 1000,
              "end": 2000,
              "kind": "push",
              "direction": "left",
              "from": "scene1",
              "to": "scene2"
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "nest_transition.montagent.json", project);
    let report = validate(&path);

    assert_eq!(report.summary().error, 0, "findings: {:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn native_push_transition_between_nest_scenes_renders_expected_halves() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 400, "height": 200},
      "fps": 25,
      "background": "#000000",
      "duration": 3000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "scene_a",
          "layer": 0,
          "elements": [
            {
              "id": "scene1",
              "type": "nest",
              "start": 0,
              "end": 2000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg1",
                  "type": "rect",
                  "start": 0,
                  "end": 2000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        },
        {
          "name": "scene_b",
          "layer": 1,
          "elements": [
            {
              "id": "scene2",
              "type": "nest",
              "start": 1000,
              "end": 3000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg2",
                  "type": "rect",
                  "start": 1000,
                  "end": 3000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#0000FF"
                }
              ]
            }
          ]
        },
        {
          "name": "bridge",
          "layer": 2,
          "elements": [
            {
              "id": "trans",
              "type": "transition",
              "start": 1000,
              "end": 2000,
              "kind": "push",
              "direction": "left",
              "from": "scene1",
              "to": "scene2"
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "nest_push.montagent.json", project);
    let ask = Ask {
        at: Some(1500),
        png: true,
        full: true,
        ..Ask::default()
    };

    let answer = frame(&path, &ask);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let bytes = answer.image().expect("frame png bytes").bytes.clone();
    let img = image::load_from_memory(&bytes).expect("valid png image");
    let rgba = img.to_rgba8();

    // At t=1500 (halfway through push left):
    // Left half (x=100, y=100) shows scene1 (red #FF0000)
    let left_pixel = rgba.get_pixel(100, 100);
    assert_eq!(left_pixel[0], 255, "left pixel red");
    assert_eq!(left_pixel[1], 0, "left pixel green");
    assert_eq!(left_pixel[2], 0, "left pixel blue");

    // Right half (x=300, y=100) shows scene2 (blue #0000FF)
    let right_pixel = rgba.get_pixel(300, 100);
    assert_eq!(right_pixel[0], 0, "right pixel red");
    assert_eq!(right_pixel[1], 0, "right pixel green");
    assert_eq!(right_pixel[2], 255, "right pixel blue");
}

#[test]
fn native_crossfade_transition_between_nest_scenes_blends_pixels() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 400, "height": 200},
      "fps": 25,
      "background": "#000000",
      "duration": 3000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "scene_a",
          "layer": 0,
          "elements": [
            {
              "id": "scene1",
              "type": "nest",
              "start": 0,
              "end": 2000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg1",
                  "type": "rect",
                  "start": 0,
                  "end": 2000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        },
        {
          "name": "scene_b",
          "layer": 1,
          "elements": [
            {
              "id": "scene2",
              "type": "nest",
              "start": 1000,
              "end": 3000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg2",
                  "type": "rect",
                  "start": 1000,
                  "end": 3000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#0000FF"
                }
              ]
            }
          ]
        },
        {
          "name": "bridge",
          "layer": 2,
          "elements": [
            {
              "id": "trans",
              "type": "transition",
              "start": 1000,
              "end": 2000,
              "kind": "crossfade",
              "from": "scene1",
              "to": "scene2"
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "nest_crossfade.montagent.json", project);
    let ask = Ask {
        at: Some(1500),
        png: true,
        full: true,
        ..Ask::default()
    };

    let answer = frame(&path, &ask);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let bytes = answer.image().expect("frame png bytes").bytes.clone();
    let img = image::load_from_memory(&bytes).expect("valid png image");
    let rgba = img.to_rgba8();

    // At t=1500 (halfway through crossfade):
    // Scene 1 opacity 0.5 (red) + Scene 2 opacity 0.5 (blue) on black.
    // In Porter-Duff SrcOver: scene 1 paints red at 128 over black, then scene 2 composites
    // blue at 128 over scene 1, attenuating the red underneath by (1 - 0.5) -> ~64 red.
    let center = rgba.get_pixel(200, 100);
    assert!(
        (center[0] as i32 - 64).abs() <= 3,
        "expected ~64 red, got {}",
        center[0]
    );
    assert_eq!(center[1], 0, "green is 0");
    assert!(
        (center[2] as i32 - 128).abs() <= 3,
        "expected ~128 blue, got {}",
        center[2]
    );
}

#[test]
fn native_wipe_transition_between_nest_scenes_renders_sharp_edge() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 400, "height": 200},
      "fps": 25,
      "background": "#000000",
      "duration": 3000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "scene_a",
          "layer": 0,
          "elements": [
            {
              "id": "scene1",
              "type": "nest",
              "start": 0,
              "end": 2000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg1",
                  "type": "rect",
                  "start": 0,
                  "end": 2000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        },
        {
          "name": "scene_b",
          "layer": 1,
          "elements": [
            {
              "id": "scene2",
              "type": "nest",
              "start": 1000,
              "end": 3000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg2",
                  "type": "rect",
                  "start": 1000,
                  "end": 3000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#0000FF"
                }
              ]
            }
          ]
        },
        {
          "name": "bridge",
          "layer": 2,
          "elements": [
            {
              "id": "trans",
              "type": "transition",
              "start": 1000,
              "end": 2000,
              "kind": "wipe",
              "direction": "left",
              "from": "scene1",
              "to": "scene2"
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "nest_wipe.montagent.json", project);
    let ask = Ask {
        at: Some(1500),
        png: true,
        full: true,
        ..Ask::default()
    };

    let answer = frame(&path, &ask);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let bytes = answer.image().expect("frame png bytes").bytes.clone();
    let img = image::load_from_memory(&bytes).expect("valid png image");
    let rgba = img.to_rgba8();

    // At t=1500: wipe left travels leftward, edge is at midpoint (x=200).
    // The wipe travels towards the left, so:
    // Left half (x=100, y=100) is not yet reached: outgoing scene1 (red #FF0000)
    // Right half (x=300, y=100) is passed: incoming scene2 (blue #0000FF)
    let left_pixel = rgba.get_pixel(100, 100);
    assert_eq!(left_pixel[0], 255, "left pixel red");
    assert_eq!(left_pixel[1], 0, "left pixel green");
    assert_eq!(left_pixel[2], 0, "left pixel blue");

    let right_pixel = rgba.get_pixel(300, 100);
    assert_eq!(right_pixel[0], 0, "right pixel red");
    assert_eq!(right_pixel[1], 0, "right pixel green");
    assert_eq!(right_pixel[2], 255, "right pixel blue");
}

#[test]
fn native_slide_transition_between_nest_scenes_slides_incoming_over_outgoing() {
    let dir = tempdir(line!());
    let project = r##"{
      "frame": {"width": 400, "height": 200},
      "fps": 25,
      "background": "#000000",
      "duration": 3000,
      "output": "out.mp4",
      "tracks": [
        {
          "name": "scene_a",
          "layer": 0,
          "elements": [
            {
              "id": "scene1",
              "type": "nest",
              "start": 0,
              "end": 2000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg1",
                  "type": "rect",
                  "start": 0,
                  "end": 2000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#FF0000"
                }
              ]
            }
          ]
        },
        {
          "name": "scene_b",
          "layer": 1,
          "elements": [
            {
              "id": "scene2",
              "type": "nest",
              "start": 1000,
              "end": 3000,
              "x": 200,
              "y": 100,
              "width": 400,
              "height": 200,
              "elements": [
                {
                  "id": "bg2",
                  "type": "rect",
                  "start": 1000,
                  "end": 3000,
                  "x": 200,
                  "y": 100,
                  "width": 400,
                  "height": 200,
                  "fill": "#0000FF"
                }
              ]
            }
          ]
        },
        {
          "name": "bridge",
          "layer": 2,
          "elements": [
            {
              "id": "trans",
              "type": "transition",
              "start": 1000,
              "end": 2000,
              "kind": "slide",
              "direction": "left",
              "from": "scene1",
              "to": "scene2"
            }
          ]
        }
      ]
    }"##;

    let path = write_project(&dir, "nest_slide.montagent.json", project);
    let ask = Ask {
        at: Some(1500),
        png: true,
        full: true,
        ..Ask::default()
    };

    let answer = frame(&path, &ask);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let bytes = answer.image().expect("frame png bytes").bytes.clone();
    let img = image::load_from_memory(&bytes).expect("valid png image");
    let rgba = img.to_rgba8();

    // At t=1500 (halfway through slide left):
    // scene1 (red) stays in place on layer 0.
    // scene2 (blue) on layer 1 is halfway in from the right, covering [200, 400].
    // Left half (x=100, y=100) still shows outgoing scene1 (red #FF0000).
    let left_pixel = rgba.get_pixel(100, 100);
    assert_eq!(left_pixel[0], 255, "left pixel red");
    assert_eq!(left_pixel[1], 0, "left pixel green");
    assert_eq!(left_pixel[2], 0, "left pixel blue");

    // Right half (x=300, y=100) shows incoming scene2 (blue #0000FF).
    let right_pixel = rgba.get_pixel(300, 100);
    assert_eq!(right_pixel[0], 0, "right pixel red");
    assert_eq!(right_pixel[1], 0, "right pixel green");
    assert_eq!(right_pixel[2], 255, "right pixel blue");
}



