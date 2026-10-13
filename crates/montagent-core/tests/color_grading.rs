//! ASC CDL and 3D LUT (.cube) tetrahedral color grading tests (ADR-0206).

use std::fs;
use std::path::Path;

use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};
use montagent_core::verbs::validate::validate;
use serde_json::json;

mod common;
use common::{canonical, tempdir, write_project};

fn drawn(project: &Path, ask: &Ask) -> (serde_json::Value, Vec<u8>) {
    let answer = frame(project, ask);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    let bytes = answer
        .image()
        .expect("an answer with exit 0 carries a picture")
        .bytes
        .clone();
    (answer.to_json(), bytes)
}

fn pixels(bytes: &[u8]) -> image::RgbaImage {
    image::load_from_memory(bytes)
        .expect("the bytes decode as a picture")
        .to_rgba8()
}

const INVERT_2X2X2_CUBE: &str = r#"TITLE "Invert"
LUT_3D_SIZE 2
1.0 1.0 1.0
0.0 1.0 1.0
1.0 0.0 1.0
0.0 0.0 1.0
1.0 1.0 0.0
0.0 1.0 0.0
1.0 0.0 0.0
0.0 0.0 0.0
"#;

#[test]
fn cdl_color_grading_on_element() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#000000",
        "tracks": [
            {
                "id": "v1",
                "elements": [
                    {
                        "id": "white_box",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "width": 100,
                        "height": 100,
                        "fill": "#FFFFFF",
                        "effects": [
                            {
                                "name": "cdl",
                                "slope": [1.0, 0.5, 0.25],
                                "offset": [0.0, 0.0, 0.0],
                                "power": [1.0, 1.0, 1.0],
                                "saturation": 1.0
                            }
                        ]
                    }
                ]
            }
        ]
    });
    let project = write_project(
        &dir,
        "cdl_element.montagent.json",
        &canonical(&doc.to_string()),
    );

    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            at: Some(0),
            ..Default::default()
        },
    );
    let pic = pixels(&bytes);
    let p = pic.get_pixel(50, 50).0;
    // White [255, 255, 255] with slope [1.0, 0.5, 0.25]:
    // Red: 255
    // Green: ~128
    // Blue: ~64
    assert_eq!(p[0], 255);
    assert!((p[1] as i32 - 128).abs() <= 2);
    assert!((p[2] as i32 - 64).abs() <= 2);
    assert_eq!(p[3], 255);
}

#[test]
fn cdl_color_grading_on_project_master() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#FFFFFF",
        "effects": [
            {
                "name": "cdl",
                "slope": [0.5, 0.5, 0.5],
                "offset": [0.0, 0.0, 0.0],
                "power": [1.0, 1.0, 1.0]
            }
        ],
        "tracks": []
    });
    let project = write_project(
        &dir,
        "cdl_master.montagent.json",
        &canonical(&doc.to_string()),
    );

    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            at: Some(0),
            ..Default::default()
        },
    );
    let pic = pixels(&bytes);
    let p = pic.get_pixel(50, 50).0;
    // White background [255, 255, 255] scaled by 0.5 -> [128, 128, 128]
    assert!((p[0] as i32 - 128).abs() <= 2);
    assert!((p[1] as i32 - 128).abs() <= 2);
    assert!((p[2] as i32 - 128).abs() <= 2);
    assert_eq!(p[3], 255);
}

#[test]
fn cdl_validation_refuses_invalid_values() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#000000",
        "tracks": [
            {
                "id": "v1",
                "elements": [
                    {
                        "id": "bad_cdl",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "width": 100,
                        "height": 100,
                        "fill": "#FFFFFF",
                        "effects": [
                            {
                                "name": "cdl",
                                "slope": [-0.5, 1.0, 1.0],
                                "offset": [0.0, 0.0, 0.0],
                                "power": [1.0, 1.0, 1.0]
                            }
                        ]
                    }
                ]
            }
        ]
    });
    let project = write_project(&dir, "bad_cdl.montagent.json", &canonical(&doc.to_string()));

    let report = validate(&project);
    assert!(
        report.findings.iter().any(|f| f.code == "E-SCHEMA"),
        "expected E-SCHEMA for negative slope, got {:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}

#[test]
fn lut3d_color_grading_on_element() {
    let dir = tempdir(line!());
    let cube_path = dir.join("invert.cube");
    fs::write(&cube_path, INVERT_2X2X2_CUBE).expect("write cube file");

    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#000000",
        "tracks": [
            {
                "id": "v1",
                "elements": [
                    {
                        "id": "box",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "width": 100,
                        "height": 100,
                        "fill": "#FF0000",
                        "effects": [
                            {
                                "name": "lut3d",
                                "source": "invert.cube",
                                "intensity": 1.0
                            }
                        ]
                    }
                ]
            }
        ]
    });
    let project = write_project(
        &dir,
        "lut_element.montagent.json",
        &canonical(&doc.to_string()),
    );

    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            at: Some(0),
            ..Default::default()
        },
    );
    let pic = pixels(&bytes);
    let p = pic.get_pixel(50, 50).0;
    // Red [255, 0, 0] inverted by LUT becomes Cyan [0, 255, 255]
    assert_eq!(p[0], 0);
    assert_eq!(p[1], 255);
    assert_eq!(p[2], 255);
    assert_eq!(p[3], 255);
}

#[test]
fn lut3d_color_grading_on_project_master() {
    let dir = tempdir(line!());
    let cube_path = dir.join("invert.cube");
    fs::write(&cube_path, INVERT_2X2X2_CUBE).expect("write cube file");

    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#FF0000",
        "effects": [
            {
                "name": "lut3d",
                "source": "invert.cube"
            }
        ],
        "tracks": []
    });
    let project = write_project(
        &dir,
        "lut_master.montagent.json",
        &canonical(&doc.to_string()),
    );

    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            at: Some(0),
            ..Default::default()
        },
    );
    let pic = pixels(&bytes);
    let p = pic.get_pixel(50, 50).0;
    // Background Red [255, 0, 0] inverted by master LUT becomes Cyan [0, 255, 255]
    assert_eq!(p[0], 0);
    assert_eq!(p[1], 255);
    assert_eq!(p[2], 255);
    assert_eq!(p[3], 255);
}

#[test]
fn lut3d_validation_not_found() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#000000",
        "tracks": [
            {
                "id": "v1",
                "elements": [
                    {
                        "id": "box",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "width": 100,
                        "height": 100,
                        "fill": "#FF0000",
                        "effects": [
                            {
                                "name": "lut3d",
                                "source": "missing.cube"
                            }
                        ]
                    }
                ]
            }
        ]
    });
    let project = write_project(
        &dir,
        "missing_lut.montagent.json",
        &canonical(&doc.to_string()),
    );

    let report = validate(&project);
    assert!(
        report.findings.iter().any(|f| f.code == "E-LUT-NOT-FOUND"),
        "expected E-LUT-NOT-FOUND, got {:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}

#[test]
fn lut3d_validation_invalid_cube() {
    let dir = tempdir(line!());
    let cube_path = dir.join("bad.cube");
    fs::write(&cube_path, "LUT_1D_SIZE 2\n0.0 0.0 0.0\n1.0 1.0 1.0\n").expect("write bad cube");

    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#000000",
        "tracks": [
            {
                "id": "v1",
                "elements": [
                    {
                        "id": "box",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "width": 100,
                        "height": 100,
                        "fill": "#FF0000",
                        "effects": [
                            {
                                "name": "lut3d",
                                "source": "bad.cube"
                            }
                        ]
                    }
                ]
            }
        ]
    });
    let project = write_project(&dir, "bad_lut.montagent.json", &canonical(&doc.to_string()));

    let report = validate(&project);
    assert!(
        report.findings.iter().any(|f| f.code == "E-LUT-INVALID"),
        "expected E-LUT-INVALID, got {:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}
